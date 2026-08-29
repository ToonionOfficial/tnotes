use axum::{
    Json,
    extract::{Extension, Query, State},
    http::{HeaderMap, StatusCode, header},
};
use qrcode_generator::QrCodeEcc;
use serde::{Deserialize, Serialize};
use tnotes_core::{
    Ulid,
    auth::token::{create_session_for_device, generate_pairing_code},
    db::{devices::upsert_device, sessions::create_session, users::get_user_by_id},
    models::{current_time_ms, device::Device},
};

use crate::{
    middleware::AuthenticatedDevice,
    state::{AppState, PendingPairing},
};

const PAIRING_CODE_TTL_MS: i64 = 5 * 60 * 1000; // 5 minutes

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

#[derive(Debug, Deserialize)]
pub struct PairClaimRequest {
    pub code: String,
    pub device_name: Option<String>,
    pub platform: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PairClaimResponse {
    pub ok: bool,
    pub token: String,
    pub user_id: String,
    pub username: String,
    pub device_id: String,
    pub expires_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct PairStatusQuery {
    pub code: Option<String>,
    pub device_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PairStatusResponse {
    pub paired: bool,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
    pub username: Option<String>,
}

fn get_lan_ip() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let addr = socket.local_addr().ok()?;
    let ip = addr.ip();
    if ip.is_loopback() || ip.is_unspecified() {
        None
    } else {
        Some(ip.to_string())
    }
}

fn resolve_server_url(headers: &HeaderMap, configured_url: &str) -> String {
    let is_configured_local = configured_url.contains("localhost")
        || configured_url.contains("127.0.0.1")
        || configured_url.contains("0.0.0.0");

    if !is_configured_local && !configured_url.is_empty() {
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

        let is_host_local = host.starts_with("localhost")
            || host.starts_with("127.0.0.1")
            || host.starts_with("0.0.0.0");

        if is_host_local {
            let port = host.split(':').nth(1).unwrap_or("8787");
            if let Some(lan_ip) = get_lan_ip() {
                return format!("http://{}:{}", lan_ip, port);
            }
        }

        return format!("{}://{}", proto, host);
    }

    if is_configured_local && let Some(lan_ip) = get_lan_ip() {
        let port = configured_url
            .split(':')
            .nth(2)
            .unwrap_or("8787")
            .trim_matches('/');
        return format!("http://{}:{}", lan_ip, port);
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
    let now = current_time_ms();
    let code_expires_at = now + PAIRING_CODE_TTL_MS;

    let conn = state.db.lock().await;

    let user = get_user_by_id(&conn, &auth.user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "User not found".into()))?;

    let device = Device::new(&new_device_id, "Paired Mobile", "mobile", &auth.user_id);
    upsert_device(&conn, &device)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    create_session(&conn, &session)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Register pending pairing in server state with 5-minute expiration
    {
        let mut pairings = state.pending_pairings.lock().await;
        pairings.retain(|_, v| v.expires_at > now);
        pairings.insert(
            pairing_code.clone(),
            PendingPairing {
                code: pairing_code.clone(),
                token: session.token.clone(),
                user_id: auth.user_id.clone(),
                username: user.username.clone(),
                device_id: new_device_id.clone(),
                expires_at: code_expires_at,
                claimed: false,
                claimed_device_name: None,
            },
        );
    }

    let server_url = resolve_server_url(&headers, &state.config.server_url);

    let qr_payload_struct = QrPayload {
        v: 1,
        url: server_url.clone(),
        token: session.token.clone(),
        device_id: new_device_id.clone(),
        user_id: auth.user_id.clone(),
        username: user.username.clone(),
        pairing_code: pairing_code.clone(),
        expires_at: code_expires_at,
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
        expires_at: code_expires_at,
    }))
}

pub async fn pair_claim_handler(
    State(state): State<AppState>,
    Json(req): Json<PairClaimRequest>,
) -> Result<Json<PairClaimResponse>, (StatusCode, String)> {
    let code = req.code.trim();
    let now = current_time_ms();

    let mut pairings = state.pending_pairings.lock().await;
    pairings.retain(|_, v| v.expires_at > now);

    let pending = pairings.get_mut(code).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "Invalid or expired 6-digit pairing code".to_string(),
        )
    })?;

    if now > pending.expires_at {
        return Err((
            StatusCode::BAD_REQUEST,
            "Pairing code has expired. Please refresh the code on your dashboard.".to_string(),
        ));
    }

    let device_name = req.device_name.unwrap_or_else(|| "Mobile App".into());
    let platform = req.platform.unwrap_or_else(|| "mobile".into());

    pending.claimed = true;
    pending.claimed_device_name = Some(device_name.clone());

    let token = pending.token.clone();
    let user_id = pending.user_id.clone();
    let username = pending.username.clone();
    let device_id = pending.device_id.clone();
    let expires_at = pending.expires_at;

    let conn = state.db.lock().await;
    let device = Device::new(&device_id, &device_name, &platform, &user_id);
    let _ = upsert_device(&conn, &device);

    Ok(Json(PairClaimResponse {
        ok: true,
        token,
        user_id,
        username,
        device_id,
        expires_at,
    }))
}

pub async fn pair_status_handler(
    State(state): State<AppState>,
    Query(query): Query<PairStatusQuery>,
) -> Result<Json<PairStatusResponse>, (StatusCode, String)> {
    let pairings = state.pending_pairings.lock().await;
    let now = current_time_ms();

    for p in pairings.values() {
        if p.expires_at > now {
            let matches_code = query.code.as_deref() == Some(&p.code);
            let matches_device = query.device_id.as_deref() == Some(&p.device_id);

            if matches_code || matches_device {
                return Ok(Json(PairStatusResponse {
                    paired: p.claimed,
                    device_id: Some(p.device_id.clone()),
                    device_name: p.claimed_device_name.clone(),
                    username: Some(p.username.clone()),
                }));
            }
        }
    }

    Ok(Json(PairStatusResponse {
        paired: false,
        device_id: None,
        device_name: None,
        username: None,
    }))
}
