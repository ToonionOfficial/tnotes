use axum::{
    Json,
    extract::{Extension, State},
    http::StatusCode,
};
use tnotes_core::sync::{
    engine::process_sync_envelope,
    envelope::{SyncEnvelope, SyncResponse},
};

use crate::{
    middleware::AuthenticatedDevice,
    state::{AppState, WsBroadcastMessage},
};

pub async fn sync_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedDevice>,
    Json(envelope): Json<SyncEnvelope>,
) -> Result<Json<SyncResponse>, (StatusCode, String)> {
    tracing::info!(
        "[SYNC_ROUTE] Received sync from user_id='{}', device_id='{}', incoming_changes={}",
        auth.user_id,
        auth.device_id,
        envelope.changes.len()
    );

    if envelope.device_id != auth.device_id {
        tracing::warn!(
            "[SYNC_ROUTE] Device mismatch: envelope.device_id='{}' vs auth.device_id='{}'",
            envelope.device_id,
            auth.device_id
        );
        return Err((
            StatusCode::FORBIDDEN,
            "Sync envelope device does not match authenticated device".to_string(),
        ));
    }

    let mut conn = state.db.lock().await;

    let response = process_sync_envelope(&mut conn, &envelope, &auth.user_id).map_err(|e| {
        tracing::error!("[SYNC_ROUTE] Error processing sync envelope: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    let rx_count = state.ws_sender.receiver_count();
    let broadcast_msg = WsBroadcastMessage {
        sender_device_id: auth.device_id.clone(),
        user_id: auth.user_id.clone(),
        changes: envelope.changes,
    };
    let send_res = state.ws_sender.send(broadcast_msg);
    tracing::info!(
        "[SYNC_ROUTE] Broadcasted sync event (active WS receivers: {}, send_res: {:?})",
        rx_count,
        send_res.is_ok()
    );

    Ok(Json(response))
}
