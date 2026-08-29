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
    tracing::debug!(
        user_id = %auth.user_id,
        device_id = %auth.device_id,
        changes_count = envelope.changes.len(),
        "Sync request received"
    );

    if envelope.device_id != auth.device_id {
        tracing::warn!(
            envelope_device_id = %envelope.device_id,
            auth_device_id = %auth.device_id,
            "Device mismatch on sync envelope"
        );
        return Err((
            StatusCode::FORBIDDEN,
            "Sync envelope device does not match authenticated device".to_string(),
        ));
    }

    let mut conn = state.db.lock().await;

    let response = process_sync_envelope(&mut conn, &envelope, &auth.user_id).map_err(|e| {
        tracing::error!("Error processing sync envelope: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    let broadcast_msg = WsBroadcastMessage {
        sender_device_id: auth.device_id,
        user_id: auth.user_id,
        changes: envelope.changes,
    };
    let _ = state.ws_sender.send(broadcast_msg);

    Ok(Json(response))
}
