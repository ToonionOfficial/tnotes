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
    // Clean up expired tickets
    tickets.retain(|_, t| t.expires_at > now);

    tickets.insert(
        ticket.clone(),
        WsTicket {
            user_id: auth.user_id.clone(),
            device_id: auth.device_id.clone(),
            expires_at,
        },
    );

    tracing::info!(
        "[WS_TICKET] Issued ticket '{}' for user_id='{}', device_id='{}'",
        ticket,
        auth.user_id,
        auth.device_id
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
        let ticket = tickets.remove(ticket_str.trim()).ok_or_else(|| {
            tracing::warn!(
                "[WS_UPGRADE] Rejecting WS connect: ticket='{}' not found or already consumed",
                ticket_str
            );
            (
                StatusCode::UNAUTHORIZED,
                "Invalid or already consumed ticket",
            )
        })?;

        if ticket.expires_at <= now {
            tracing::warn!(
                "[WS_UPGRADE] Rejecting WS connect: ticket='{}' expired",
                ticket_str
            );
            return Err((StatusCode::UNAUTHORIZED, "Ticket expired"));
        }

        let conn = state.db.lock().await;
        let _ = touch_device(&conn, &ticket.device_id, now);
        tracing::info!(
            "[WS_UPGRADE] Successfully validated ticket for user_id='{}', device_id='{}'",
            ticket.user_id,
            ticket.device_id
        );
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
        tracing::info!(
            "[WS_UPGRADE] Successfully validated cookie for user_id='{}', device_id='{}'",
            device.user_id,
            session.device_id
        );
        (session.device_id, device.user_id)
    } else {
        tracing::warn!("[WS_UPGRADE] Rejecting WS connect: Missing ticket and session cookie");
        return Err((
            StatusCode::UNAUTHORIZED,
            "Missing WebSocket ticket or session cookie",
        ));
    };

    tracing::info!(
        "[WS_UPGRADE] WebSocket upgrading for user='{}', device='{}'",
        user_id,
        device_id
    );

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, device_id, user_id)))
}

async fn handle_socket(socket: WebSocket, state: AppState, device_id: String, user_id: String) {
    tracing::info!(
        "[WS_SOCKET] Active connection opened for user='{}', device='{}'",
        user_id,
        device_id
    );
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
                        tracing::info!("[WS_SOCKET] WebSocket closed for device='{}'", device_id);
                        break;
                    }
                    _ => {}
                }
            }

            broadcast_msg = broadcast_rx.recv() => {
                match broadcast_msg {
                    Ok(msg) => {
                        tracing::info!(
                            "[WS_SOCKET] broadcast_rx received: sender='{}', msg_user='{}', my_device='{}', my_user='{}'",
                            msg.sender_device_id,
                            msg.user_id,
                            device_id,
                            user_id
                        );
                        if msg.user_id == user_id && msg.sender_device_id != device_id {
                            let notification = WsServerMessage::SyncNotification {
                                sender_device_id: msg.sender_device_id.clone(),
                                count: msg.changes.len(),
                            };
                            if let Ok(json) = serde_json::to_string(&notification) {
                                tracing::info!(
                                    "[WS_SOCKET] Sending SyncNotification to device='{}': {}",
                                    device_id,
                                    json
                                );
                                let send_err = sender.send(Message::Text(json.into())).await.is_err();
                                if send_err {
                                    tracing::warn!("[WS_SOCKET] Send failed for device='{}', closing", device_id);
                                    break;
                                }
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!("[WS_SOCKET] Receiver for device='{}' lagged by {} messages", device_id, skipped);
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
