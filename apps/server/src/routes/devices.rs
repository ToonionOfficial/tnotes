use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tnotes_core::db::devices::{delete_device, get_device_by_id, list_devices_by_user};

use crate::{middleware::AuthenticatedDevice, state::AppState};

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceItemResponse {
    pub id: String,
    pub name: String,
    pub platform: String,
    pub last_seen_at: i64,
    pub created_at: i64,
    pub is_current: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteDeviceResponse {
    pub ok: bool,
}

pub async fn list_devices_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedDevice>,
) -> Result<Json<Vec<DeviceItemResponse>>, (StatusCode, String)> {
    let conn = state.db.lock().await;

    let devices = list_devices_by_user(&conn, &auth.user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let response: Vec<DeviceItemResponse> = devices
        .into_iter()
        .map(|d| {
            let is_current = d.id == auth.device_id;
            DeviceItemResponse {
                id: d.id,
                name: d.name,
                platform: d.platform,
                last_seen_at: d.last_seen_at,
                created_at: d.created_at,
                is_current,
            }
        })
        .collect();

    Ok(Json(response))
}

pub async fn delete_device_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedDevice>,
    Path(device_id): Path<String>,
) -> Result<Json<DeleteDeviceResponse>, (StatusCode, String)> {
    let conn = state.db.lock().await;

    let target_device = get_device_by_id(&conn, &device_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Device not found".into()))?;

    if target_device.user_id != auth.user_id {
        return Err((
            StatusCode::FORBIDDEN,
            "Cannot delete device of another user".into(),
        ));
    }

    delete_device(&conn, &device_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tracing::info!("User '{}' revoked device '{}'", auth.user_id, device_id);

    Ok(Json(DeleteDeviceResponse { ok: true }))
}
