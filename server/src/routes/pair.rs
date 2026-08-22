use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use notat_core::{
    auth::token::{create_session_for_device, generate_pairing_code},
    db::sessions::create_session,
    Ulid,
};
use qrcode_generator::QrCodeEcc;
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct PairingDataResponse {
    pub url: String,
    pub token: String,
    pub device_id: String,
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
}

/// GET /api/pair
/// Generates pairing credentials and a fully rendered SVG QR code for new clients.
pub async fn pair_handler(
    State(state): State<AppState>,
) -> Result<Json<PairingDataResponse>, (StatusCode, String)> {
    let new_device_id = Ulid::generate().to_string();
    let session = create_session_for_device(&new_device_id, None);
    let pairing_code = generate_pairing_code();

    let conn = state.db.lock().await;
    create_session(&conn, &session)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let qr_payload_struct = QrPayload {
        v: 1,
        url: state.config.server_url.clone(),
        token: session.token.clone(),
        device_id: new_device_id.clone(),
    };

    let qr_payload = serde_json::to_string(&qr_payload_struct)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Generate real SVG QR code using qrcode-generator
    let qr_svg = qrcode_generator::to_svg_to_string(
        &qr_payload,
        QrCodeEcc::Medium,
        220,
        None::<&str>,
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("QR generation error: {}", e)))?;

    Ok(Json(PairingDataResponse {
        url: state.config.server_url.clone(),
        token: session.token,
        device_id: new_device_id,
        pairing_code,
        qr_svg,
        qr_payload,
        expires_at: session.expires_at,
    }))
}
