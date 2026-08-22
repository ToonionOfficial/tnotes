use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use notat_core::{
    db::{
        devices::touch_device,
        sessions::{delete_session, get_session},
    },
    models::current_time_ms,
};

use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct AuthenticatedDevice {
    pub token: String,
    pub device_id: String,
}

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, (StatusCode, &'static str)> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|val| val.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header"))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid Authorization scheme"))?
        .trim();

    if token.is_empty() {
        return Err((StatusCode::UNAUTHORIZED, "Empty Bearer token"));
    }

    let conn = state.db.lock().await;

    let session = get_session(&conn, token)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid session token"))?;

    if session.is_expired() {
        let _ = delete_session(&conn, token);
        return Err((StatusCode::UNAUTHORIZED, "Session expired"));
    }

    let now = current_time_ms();
    let _ = touch_device(&conn, &session.device_id, now);

    let auth_device = AuthenticatedDevice {
        token: session.token,
        device_id: session.device_id,
    };

    req.extensions_mut().insert(auth_device);

    drop(conn);

    Ok(next.run(req).await)
}
