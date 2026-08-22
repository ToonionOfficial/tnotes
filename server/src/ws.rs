use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use notat_core::{
    db::{devices::{get_device_by_id, touch_device}, sessions::get_session},
    models::current_time_ms,
};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsServerMessage {
    #[serde(rename = "sync_notification")]
    SyncNotification {
        sender_device_id: String,
        count: usize,
    },
    #[serde(rename = "pong")]
    Pong,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsClientMessage {
    #[serde(rename = "ping")]
    Ping,
}

pub async fn ws_sync_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let token = if let Some(auth_val) = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()) {
        auth_val.strip_prefix("Bearer ").unwrap_or(auth_val).trim().to_string()
    } else if let Some(q_token) = query.token {
        q_token.trim().to_string()
    } else if let Some(cookie) = axum_extra::extract::cookie::CookieJar::from_headers(&headers).get(crate::middleware::SESSION_COOKIE_NAME) {
        cookie.value().trim().to_string()
    } else {
        return Err((StatusCode::UNAUTHORIZED, "Missing authentication token or cookie"));
    };

    if token.is_empty() {
        return Err((StatusCode::UNAUTHORIZED, "Empty authentication token"));
    }

    let (device_id, user_id) = {
        let conn = state.db.lock().await;
        let session = get_session(&conn, &token)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
            .ok_or((StatusCode::UNAUTHORIZED, "Invalid session token"))?;

        if session.is_expired() {
            return Err((StatusCode::UNAUTHORIZED, "Session expired"));
        }

        let device = get_device_by_id(&conn, &session.device_id)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
            .ok_or((StatusCode::UNAUTHORIZED, "Associated device not found"))?;

        let now = current_time_ms();
        let _ = touch_device(&conn, &session.device_id, now);
        (session.device_id, device.user_id)
    };

    tracing::info!("WebSocket connected for user: {}, device: {}", user_id, device_id);

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, device_id, user_id)))
}

async fn handle_socket(socket: WebSocket, state: AppState, device_id: String, user_id: String) {
    let (mut sender, mut receiver) = socket.split();
    let mut broadcast_rx = state.ws_sender.subscribe();

    loop {
        tokio::select! {
            inbound = receiver.next() => {
                match inbound {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(client_msg) = serde_json::from_str::<WsClientMessage>(&text) {
                            match client_msg {
                                WsClientMessage::Ping => {
                                    let pong = WsServerMessage::Pong;
                                    if let Ok(json) = serde_json::to_string(&pong) {
                                        let _ = sender.send(Message::Text(json.into())).await;
                                    }
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        let _ = sender.send(Message::Pong(payload)).await;
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        tracing::info!("WebSocket closed for device: {}", device_id);
                        break;
                    }
                    _ => {}
                }
            }

            broadcast_msg = broadcast_rx.recv() => {
                match broadcast_msg {
                    Ok(msg) => {
                        if msg.user_id == user_id && msg.sender_device_id != device_id {
                            let notification = WsServerMessage::SyncNotification {
                                sender_device_id: msg.sender_device_id,
                                count: msg.changes.len(),
                            };
                            if let Ok(json) = serde_json::to_string(&notification) {
                                if sender.send(Message::Text(json.into())).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!("WebSocket receiver for device {} lagged by {} messages", device_id, skipped);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
        }
    }
}
