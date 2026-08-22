use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use notat_core::db::migrations::open_in_memory;
use notat_server::{
    build_router,
    routes::pair::PairingDataResponse,
    state::{AppState, ServerConfig},
};
use tower::ServiceExt;

#[tokio::test]
async fn test_web_index_and_pair_api() {
    let conn = open_in_memory().unwrap();
    let config = ServerConfig {
        host: "127.0.0.1".into(),
        port: 8787,
        data_dir: ":memory:".into(),
        server_url: "http://192.168.1.50:8787".into(),
    };
    let state = AppState::new(conn, config);
    let app = build_router(state);

    // 1. GET / returns HTML page
    let res = app
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.to_lowercase().contains("<!doctype html>"));
    assert!(html.contains("Notat"));

    // 2. GET /api/pair returns valid pairing payload with SVG QR Code
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/pair")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let data: PairingDataResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(data.url, "http://192.168.1.50:8787");
    assert_eq!(data.token.len(), 64);
    assert_eq!(data.pairing_code.len(), 6);
    assert!(data.qr_payload.contains("\"token\":"));
    // Verify real SVG was generated
    assert!(data.qr_svg.contains("<svg"));
    assert!(data.qr_svg.contains("<rect"));
    assert!(data.qr_svg.contains("</svg>"));
}
