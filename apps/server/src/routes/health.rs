use axum::{Json, response::IntoResponse};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub ok: bool,
    pub version: &'static str,
}

pub async fn health_check() -> impl IntoResponse {
    Json(HealthResponse {
        ok: true,
        version: env!("CARGO_PKG_VERSION"),
    })
}
