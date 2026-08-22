use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::state::AppState;
use notat_core::{
    auth::{
        password::{hash_password, verify_password},
        token::create_session_for_device,
    },
    db::{
        devices::upsert_device,
        sessions::create_session,
        users::{create_user, get_user_by_username, has_any_user},
    },
    models::{device::Device, user::User},
};

#[derive(Debug, Deserialize, Validate)]
pub struct SetupRequest {
    #[validate(length(
        min = 1,
        max = 64,
        message = "Username must be between 1 and 64 characters"
    ))]
    pub username: String,
    #[validate(length(
        min = 8,
        max = 128,
        message = "Password must be at least 8 characters long"
    ))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetupResponse {
    pub ok: bool,
    pub user_id: String,
    pub username: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1, max = 64, message = "Invalid username"))]
    pub username: String,
    #[validate(length(min = 1, max = 128, message = "Invalid password"))]
    pub password: String,
    #[validate(length(min = 1, max = 255, message = "Invalid device ID"))]
    pub device_id: String,
    #[validate(length(min = 1, max = 255, message = "Invalid device name"))]
    pub device_name: String,
    #[validate(length(min = 1, max = 50, message = "Invalid platform"))]
    pub platform: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub device_id: String,
    pub expires_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
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

pub async fn setup_handler(
    State(state): State<AppState>,
    Json(req): Json<SetupRequest>,
) -> Result<(StatusCode, Json<SetupResponse>), (StatusCode, String)> {
    req.validate()
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let conn = state.db.lock().await;

    let configured =
        has_any_user(&conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if configured {
        return Err((
            StatusCode::BAD_REQUEST,
            "Server is already configured. Initial setup can only be performed once.".into(),
        ));
    }

    let password_hash = hash_password(&req.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let user = User::new(&req.username, password_hash);
    let user_id = user.id.clone();
    let saved_username = user.username.clone();

    create_user(&conn, &user).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tracing::info!("First-run setup completed for user: {}", saved_username);

    Ok((
        StatusCode::CREATED,
        Json(SetupResponse {
            ok: true,
            user_id,
            username: saved_username,
        }),
    ))
}

pub async fn login_handler(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    req.validate()
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let conn = state.db.lock().await;

    let user = get_user_by_username(&conn, &req.username)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Invalid username or password".into(),
        ))?;

    let is_valid = verify_password(&req.password, &user.password_hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !is_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Invalid username or password".into(),
        ));
    }

    let device = Device::new(&req.device_id, &req.device_name, &req.platform);
    upsert_device(&conn, &device)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let session = create_session_for_device(&req.device_id, None);
    create_session(&conn, &session)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tracing::info!(
        "User '{}' logged in from device '{}' ({})",
        user.username,
        device.name,
        device.platform
    );

    Ok(Json(LoginResponse {
        token: session.token,
        device_id: session.device_id,
        expires_at: session.expires_at,
    }))
}
