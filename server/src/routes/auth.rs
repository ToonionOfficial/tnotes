use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::state::AppState;
use notat_core::db::users::has_any_user;

#[derive(Debug, Deserialize)]
pub struct SetupRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct SetupResponse {
    pub ok: bool,
    pub user_id: String,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub device_id: String,
    pub device_name: String,
    pub platform: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub device_id: String,
    pub expires_at: i64,
}

#[derive(Debug, Serialize)]
pub struct SetupStatusResponse {
    pub is_configured: bool,
}

pub async fn setup_status_handler(
    State(state): State<AppState>,
) -> Result<Json<SetupStatusResponse>, (StatusCode, String)> {
    let conn = state.db.lock().await;
    let is_configured =
        has_any_user(&conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(SetupStatusResponse { is_configured }))
}
