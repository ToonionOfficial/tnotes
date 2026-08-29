use axum::{
    Json,
    extract::{Extension, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tnotes_core::{
    db::{
        devices::list_devices_by_user, folders::count_active_folders_for_user,
        notes::count_active_notes_for_user,
    },
    sync::{
        engine::process_sync_envelope,
        envelope::{SyncEnvelope, SyncResponse},
    },
};

use crate::{
    middleware::AuthenticatedDevice,
    state::{AppState, WsBroadcastMessage},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct StatsResponse {
    pub notes_count: usize,
    pub folders_count: usize,
    pub devices_count: usize,
}

pub async fn stats_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedDevice>,
) -> Result<Json<StatsResponse>, (StatusCode, String)> {
    let conn = state.db.lock().await;

    let notes_count = count_active_notes_for_user(&conn, &auth.user_id).unwrap_or(0);
    let folders_count = count_active_folders_for_user(&conn, &auth.user_id).unwrap_or(0);
    let devices = list_devices_by_user(&conn, &auth.user_id).unwrap_or_default();

    Ok(Json(StatsResponse {
        notes_count,
        folders_count,
        devices_count: devices.len(),
    }))
}

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
