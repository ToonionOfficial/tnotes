use axum::{
    Extension, Json,
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tnotes_core::{
    Ulid,
    db::{
        devices::{get_device_by_id, touch_device},
        sessions::get_session,
    },
    models::current_time_ms,
};

use crate::{
    middleware::{AuthenticatedDevice, SESSION_COOKIE_NAME},
    state::{AppState, WsTicket},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct WsTicketResponse {
    pub ticket: String,
    pub expires_in: u64,
}

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub ticket: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsServerMessage {
    #[serde(rename = "sync_notification")]
    SyncNotification {
        sender_device_id: String,
        count: usize,
    },
    #[serde(rename = "sync_required")]
    SyncRequired { reason: String, skipped: u64 },
    #[serde(rename = "pong")]
    Pong,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsClientMessage {
    #[serde(rename = "ping")]
    Ping,
}

pub async fn issue_ws_ticket_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedDevice>,
) -> Result<Json<WsTicketResponse>, (StatusCode, String)> {
    let now = current_time_ms();
    let ticket = format!("tkt_{}", Ulid::generate());
    let expires_at = now + 60_000; // valid for 60 seconds

    let mut tickets = state.pending_ws_tickets.lock().await;
    tickets.retain(|_, t| t.expires_at > now);

    tickets.insert(
        ticket.clone(),
        WsTicket {
            user_id: auth.user_id,
            device_id: auth.device_id,
            expires_at,
        },
    );

    Ok(Json(WsTicketResponse {
        ticket,
        expires_in: 60,
    }))
}

pub async fn ws_sync_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let now = current_time_ms();

    let (device_id, user_id) = if let Some(ticket_str) = query.ticket {
        let mut tickets = state.pending_ws_tickets.lock().await;
        let ticket = tickets.remove(ticket_str.trim()).ok_or((
            StatusCode::UNAUTHORIZED,
            "Invalid or already consumed ticket",
        ))?;

        if ticket.expires_at <= now {
            return Err((StatusCode::UNAUTHORIZED, "Ticket expired"));
        }

        let conn = state.db.lock().await;
        let _ = touch_device(&conn, &ticket.device_id, now);
        (ticket.device_id, ticket.user_id)
    } else if let Some(cookie) =
        axum_extra::extract::cookie::CookieJar::from_headers(&headers).get(SESSION_COOKIE_NAME)
    {
        let token = cookie.value().trim();
        let conn = state.db.lock().await;
        let session = get_session(&conn, token)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
            .ok_or((StatusCode::UNAUTHORIZED, "Invalid session cookie"))?;

        if session.is_expired() {
            return Err((StatusCode::UNAUTHORIZED, "Session expired"));
        }

        let device = get_device_by_id(&conn, &session.device_id)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
            .ok_or((StatusCode::UNAUTHORIZED, "Associated device not found"))?;

        let _ = touch_device(&conn, &session.device_id, now);
        (session.device_id, device.user_id)
    } else {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Missing WebSocket ticket or session cookie",
        ));
    };

    tracing::debug!(
        user_id = %user_id,
        device_id = %device_id,
        "WebSocket connection established"
    );

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
                        tracing::debug!(device_id = %device_id, "WebSocket closed");
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
                                let send_err = sender.send(Message::Text(json.into())).await.is_err();
                                if send_err {
                                    break;
                                }
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!(device_id = %device_id, skipped = %skipped, "WebSocket receiver lagged");
                        let notification = WsServerMessage::SyncRequired {
                            reason: "buffer_lagged".to_string(),
                            skipped,
                        };
                        if let Ok(json) = serde_json::to_string(&notification) {
                            let send_err = sender.send(Message::Text(json.into())).await.is_err();
                            if send_err {
                                break;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
        }
    }
}
