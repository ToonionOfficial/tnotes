use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use notat_core::db::migrations::open_in_memory;
use notat_server::{
    build_router,
    routes::auth::{LoginResponse, SetupResponse, SetupStatusResponse},
    state::{AppState, ServerConfig},
};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_auth_api_flow() {
    let conn = open_in_memory().unwrap();
    let config = ServerConfig {
        host: "127.0.0.1".into(),
        port: 8787,
        data_dir: ":memory:".into(),
        server_url: "http://localhost:8787".into(),
    };
    let state = AppState::new(conn, config);
    let app = build_router(state);

    // 1. Health check
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 2. Setup status before setup -> is_configured = false
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/setup/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let status: SetupStatusResponse = serde_json::from_slice(&body).unwrap();
    assert!(!status.is_configured);

    // 3. Short password -> 400 Bad Request
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/setup")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({ "username": "admin", "password": "123" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // 4. Valid setup -> 201 Created
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
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let setup_res: SetupResponse = serde_json::from_slice(&body).unwrap();
    assert!(setup_res.ok);
    assert_eq!(setup_res.username, "admin");

    // 5. Setup status after setup -> is_configured = true
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/setup/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let status: SetupStatusResponse = serde_json::from_slice(&body).unwrap();
    assert!(status.is_configured);

    // 6. Setup again -> 400 Bad Request
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
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // 7. Login with wrong password -> 401 Unauthorized
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
                        "password": "wrong_password",
                        "device_id": "pixel_1",
                        "device_name": "Pixel 9",
                        "platform": "mobile"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // 8. Login with correct password -> 200 OK
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
                        "device_id": "pixel_1",
                        "device_name": "Pixel 9",
                        "platform": "mobile"
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
    assert_eq!(login_res.device_id, "pixel_1");
    assert_eq!(login_res.token.len(), 64);
}
