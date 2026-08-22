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
    let mut conn = state.db.lock().await;

    let response = process_sync_envelope(&mut conn, &envelope, &auth.user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !envelope.changes.is_empty() {
        let broadcast_msg = WsBroadcastMessage {
            sender_device_id: auth.device_id,
            user_id: auth.user_id,
            changes: envelope.changes,
        };
        let _ = state.ws_sender.send(broadcast_msg);
    }

    Ok(Json(response))
}
