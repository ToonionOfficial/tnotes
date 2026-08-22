use axum::{
    Json,
    extract::{Extension, State},
    http::{HeaderMap, StatusCode, header},
};
use qrcode_generator::QrCodeEcc;
use serde::{Deserialize, Serialize};
use tnotes_core::{
    Ulid,
    auth::token::{create_session_for_device, generate_pairing_code},
    db::{devices::upsert_device, sessions::create_session, users::get_user_by_id},
    models::device::Device,
};

use crate::{middleware::AuthenticatedDevice, state::AppState};

#[derive(Debug, Serialize, Deserialize)]
pub struct PairingDataResponse {
    pub url: String,
    pub token: String,
    pub device_id: String,
    pub user_id: String,
    pub username: String,
    pub pairing_code: String,
    pub qr_svg: String,
    pub qr_payload: String,
    pub expires_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QrPayload {
    pub v: u32,
    pub url: String,
    pub token: String,
    pub device_id: String,
    pub user_id: String,
    pub username: String,
    pub pairing_code: String,
    pub expires_at: i64,
}

fn resolve_server_url(headers: &HeaderMap, configured_url: &str) -> String {
    let is_local_default = configured_url.contains("localhost")
        || configured_url.contains("127.0.0.1")
        || configured_url.contains("0.0.0.0");

    if !is_local_default {
        return configured_url.to_string();
    }

    if let Some(host) = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get(header::HOST))
        .and_then(|v| v.to_str().ok())
    {
        let proto = headers
            .get("x-forwarded-proto")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("http");
        return format!("{}://{}", proto, host);
    }

    configured_url.to_string()
}

pub async fn pair_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthenticatedDevice>,
) -> Result<Json<PairingDataResponse>, (StatusCode, String)> {
    let new_device_id = Ulid::generate().to_string();
    let session = create_session_for_device(&new_device_id, None);
    let pairing_code = generate_pairing_code();

    let conn = state.db.lock().await;

    // Get the username for the authenticated owner
    let user = get_user_by_id(&conn, &auth.user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "User not found".into()))?;

    let device = Device::new(&new_device_id, "Paired Device", "mobile", &auth.user_id);
    upsert_device(&conn, &device)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    create_session(&conn, &session)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let server_url = resolve_server_url(&headers, &state.config.server_url);

    let qr_payload_struct = QrPayload {
        v: 1,
        url: server_url.clone(),
        token: session.token.clone(),
        device_id: new_device_id.clone(),
        user_id: auth.user_id.clone(),
        username: user.username.clone(),
        pairing_code: pairing_code.clone(),
        expires_at: session.expires_at,
    };

    let qr_payload = serde_json::to_string(&qr_payload_struct)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let qr_svg =
        qrcode_generator::to_svg_to_string(&qr_payload, QrCodeEcc::Medium, 240, None::<&str>)
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("QR generation error: {}", e),
                )
            })?;

    Ok(Json(PairingDataResponse {
        url: server_url,
        token: session.token,
        device_id: new_device_id,
        user_id: auth.user_id,
        username: user.username,
        pairing_code,
        qr_svg,
        qr_payload,
        expires_at: session.expires_at,
    }))
}
