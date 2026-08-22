use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use notat_core::db::migrations::open_in_memory;
use notat_server::{
    build_router,
    routes::auth::LoginResponse,
    routes::pair::PairingDataResponse,
    state::{AppState, ServerConfig},
};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_pair_api_flow() {
    let conn = open_in_memory().unwrap();
    let config = ServerConfig {
        host: "127.0.0.1".into(),
        port: 8787,
        data_dir: ":memory:".into(),
        server_url: "http://192.168.1.50:8787".into(),
    };
    let state = AppState::new(conn, config);
    let app = build_router(state);

    // 1. GET /api/pair without auth → 401 Unauthorized
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
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // 2. Setup admin user
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/setup")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({ "username": "admin", "password": "password123!" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // 3. Login to get a session token
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "username": "admin",
                        "password": "password123!",
                        "device_id": "web_test",
                        "device_name": "Test Browser",
                        "platform": "web"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let login_res: LoginResponse = serde_json::from_slice(&body).unwrap();
    let token = login_res.token;

    // 4. GET /api/pair with valid Bearer token → 200 OK
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/pair")
                .header("authorization", format!("Bearer {}", token))
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
