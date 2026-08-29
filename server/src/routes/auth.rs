use axum::{
    Json,
    extract::{Extension, State},
    http::StatusCode,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::{Deserialize, Serialize};
use time::Duration;
use validator::Validate;

use crate::{
    middleware::{AuthenticatedDevice, SESSION_COOKIE_NAME},
    state::AppState,
};
use tnotes_core::{
    Ulid,
    auth::{
        password::{hash_password, verify_password},
        token::create_session_for_device,
    },
    db::{
        devices::{get_device_by_id, list_devices_by_user, upsert_device},
        folders::count_active_folders_for_user,
        notes::count_active_notes_for_user,
        sessions::{create_session, delete_session},
        users::{create_user, get_user_by_id, get_user_by_username, has_any_user},
    },
    models::{current_time_ms, device::Device, user::User},
};

pub fn build_session_cookie(token: &str, expires_at: i64) -> Cookie<'static> {
    let now = current_time_ms();
    let max_age_secs = if expires_at > now {
        (expires_at - now) / 1000
    } else {
        7776000 // 90 days default
    };

    Cookie::build((SESSION_COOKIE_NAME, token.to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::seconds(max_age_secs))
        .build()
}

pub fn build_removal_cookie() -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::ZERO)
        .build()
}

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
    pub token: String,
    pub device_id: String,
    pub expires_at: i64,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
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
pub struct RegisterResponse {
    pub ok: bool,
    pub user_id: String,
    pub username: String,
    pub token: String,
    pub device_id: String,
    pub expires_at: i64,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct MeResponse {
    pub user_id: String,
    pub username: String,
    pub device_id: String,
    pub device_name: String,
    pub platform: String,
    pub has_paired_devices: bool,
    pub paired_devices_count: usize,
    #[serde(default)]
    pub notes_count: usize,
    #[serde(default)]
    pub folders_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogoutResponse {
    pub ok: bool,
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
    jar: CookieJar,
    Json(req): Json<SetupRequest>,
) -> Result<(StatusCode, CookieJar, Json<SetupResponse>), (StatusCode, String)> {
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

    // Create an initial web session for the owner
    let device_id = Ulid::generate().to_string();
    let device = Device::new(&device_id, "Web Browser", "web", &user_id);
    upsert_device(&conn, &device)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let session = create_session_for_device(&device_id, None);
    create_session(&conn, &session)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tracing::info!("First-run setup completed for user: {}", saved_username);

    let cookie = build_session_cookie(&session.token, session.expires_at);

    Ok((
        StatusCode::CREATED,
        jar.add(cookie),
        Json(SetupResponse {
            ok: true,
            user_id,
            username: saved_username,
            token: session.token,
            device_id: session.device_id,
            expires_at: session.expires_at,
        }),
    ))
}

pub async fn register_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, CookieJar, Json<RegisterResponse>), (StatusCode, String)> {
    req.validate()
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let trimmed_username = req.username.trim();
    if trimmed_username.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Username cannot be empty".into()));
    }

    let conn = state.db.lock().await;

    let existing_user = get_user_by_username(&conn, trimmed_username)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if existing_user.is_some() {
        return Err((
            StatusCode::CONFLICT,
            "Username is already taken. Please choose another username.".into(),
        ));
    }

    let password_hash = hash_password(&req.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let user = User::new(trimmed_username, password_hash);
    let user_id = user.id.clone();
    let saved_username = user.username.clone();

    create_user(&conn, &user).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let device_id = Ulid::generate().to_string();
    let device = Device::new(&device_id, "Web Browser", "web", &user_id);
    upsert_device(&conn, &device)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let session = create_session_for_device(&device_id, None);
    create_session(&conn, &session)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tracing::info!("New user registered: {}", saved_username);

    let cookie = build_session_cookie(&session.token, session.expires_at);

    Ok((
        StatusCode::CREATED,
        jar.add(cookie),
        Json(RegisterResponse {
            ok: true,
            user_id,
            username: saved_username,
            token: session.token,
            device_id: session.device_id,
            expires_at: session.expires_at,
        }),
    ))
}

pub async fn login_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<LoginRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), (StatusCode, String)> {
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

    let device = Device::new(&req.device_id, &req.device_name, &req.platform, &user.id);
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

    let cookie = build_session_cookie(&session.token, session.expires_at);

    Ok((
        jar.add(cookie),
        Json(LoginResponse {
            token: session.token,
            device_id: session.device_id,
            expires_at: session.expires_at,
        }),
    ))
}

pub async fn me_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedDevice>,
) -> Result<Json<MeResponse>, (StatusCode, String)> {
    let conn = state.db.lock().await;

    let user = get_user_by_id(&conn, &auth.user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "User not found".into()))?;

    let device = get_device_by_id(&conn, &auth.device_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Device not found".into()))?;

    {
        let mut pairings = state.pending_pairings.lock().await;
        for p in pairings.values_mut() {
            if p.device_id == auth.device_id {
                p.claimed = true;
                p.claimed_device_name = Some(device.name.clone());
            }
        }
    }

    let all_devices = list_devices_by_user(&conn, &auth.user_id).unwrap_or_default();
    let paired_devices: Vec<_> = all_devices
        .into_iter()
        .filter(|d| d.id != auth.device_id)
        .collect();
    let has_paired_devices = !paired_devices.is_empty();
    let paired_devices_count = paired_devices.len();

    let notes_count = count_active_notes_for_user(&conn, &auth.user_id).unwrap_or(0);
    let folders_count = count_active_folders_for_user(&conn, &auth.user_id).unwrap_or(0);

    Ok(Json(MeResponse {
        user_id: user.id,
        username: user.username,
        device_id: device.id,
        device_name: device.name,
        platform: device.platform,
        has_paired_devices,
        paired_devices_count,
        notes_count,
        folders_count,
    }))
}

pub async fn logout_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    Extension(auth): Extension<AuthenticatedDevice>,
) -> Result<(CookieJar, Json<LogoutResponse>), (StatusCode, String)> {
    let conn = state.db.lock().await;
    let _ = delete_session(&conn, &auth.token);

    tracing::info!("User session logged out: {}", auth.device_id);

    let cookie = build_removal_cookie();

    Ok((jar.add(cookie), Json(LogoutResponse { ok: true })))
}
