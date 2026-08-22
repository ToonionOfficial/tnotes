use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::cookie::CookieJar;
use tnotes_core::{
    db::{
        devices::{get_device_by_id, touch_device},
        sessions::{delete_session, get_session},
    },
    models::current_time_ms,
};

use crate::state::AppState;

pub const SESSION_COOKIE_NAME: &str = "tnotes_session";
pub const LEGACY_SESSION_COOKIE_NAME: &str = "notat_session";

#[derive(Debug, Clone)]
pub struct AuthenticatedDevice {
    pub token: String,
    pub device_id: String,
    pub user_id: String,
}

/// Extracts session token from Authorization: Bearer header,
/// falling back to the `tnotes_session` (or legacy `notat_session`) httpOnly cookie.
pub fn extract_token_from_request(req: &Request) -> Option<String> {
    // 1. Check Authorization: Bearer <token>
    if let Some(auth_val) = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|val| val.to_str().ok())
    {
        if let Some(token) = auth_val.strip_prefix("Bearer ") {
            let trimmed = token.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    // 2. Fallback to tnotes_session / notat_session cookie
    let jar = CookieJar::from_headers(req.headers());
    if let Some(cookie) = jar.get(SESSION_COOKIE_NAME).or_else(|| jar.get(LEGACY_SESSION_COOKIE_NAME)) {
        let val = cookie.value().trim();
        if !val.is_empty() {
            return Some(val.to_string());
        }
    }

    None
}

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, (StatusCode, &'static str)> {
    let token = extract_token_from_request(&req)
        .ok_or((StatusCode::UNAUTHORIZED, "Missing authentication token or cookie"))?;

    let conn = state.db.lock().await;

    let session = get_session(&conn, &token)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid session token"))?;

    if session.is_expired() {
        let _ = delete_session(&conn, &token);
        return Err((StatusCode::UNAUTHORIZED, "Session expired"));
    }

    let device = get_device_by_id(&conn, &session.device_id)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
        .ok_or((StatusCode::UNAUTHORIZED, "Associated device not found"))?;

    let now = current_time_ms();
    let _ = touch_device(&conn, &session.device_id, now);

    let auth_device = AuthenticatedDevice {
        token: session.token,
        device_id: session.device_id,
        user_id: device.user_id,
    };

    req.extensions_mut().insert(auth_device);

    drop(conn);

    Ok(next.run(req).await)
}
