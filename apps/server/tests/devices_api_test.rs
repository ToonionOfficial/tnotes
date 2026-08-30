use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::json;
use tnotes_core::db::migrations::open_in_memory;
use tnotes_server::{
    build_router,
    routes::{auth::SetupResponse, devices::DeviceItemResponse},
    state::{AppState, ServerConfig},
};
use tower::ServiceExt;

#[tokio::test]
async fn test_devices_api_flow() {
    let conn = open_in_memory().unwrap();
    let config = ServerConfig {
        host: "127.0.0.1".into(),
        port: 8787,
        data_dir: ":memory:".into(),
        server_url: "http://localhost:8787".into(),
    };
    let state = AppState::new(conn, config);
    let app = build_router(state);

    // 1. Initial setup
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/setup")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({ "username": "alice", "password": "password123!" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let setup_res: SetupResponse = serde_json::from_slice(&body).unwrap();
    let token = setup_res.token;
    let device_id = setup_res.device_id;

    // 2. List devices with auth token
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/devices")
                .header("authorization", format!("Bearer {}", token))
                .header("x-device-id", &device_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let devices: Vec<DeviceItemResponse> = serde_json::from_slice(&body).unwrap();
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].id, device_id);
    assert!(devices[0].is_current);

    // 3. Login 2nd device (Mobile)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "username": "alice",
                        "password": "password123!",
                        "device_id": "pixel_phone",
                        "device_name": "Pixel 9 Pro",
                        "platform": "mobile"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 4. List devices again -> 2 devices
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/devices")
                .header("authorization", format!("Bearer {}", token))
                .header("x-device-id", &device_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let devices: Vec<DeviceItemResponse> = serde_json::from_slice(&body).unwrap();
    assert_eq!(devices.len(), 2);

    // 5. Delete the 2nd device
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/devices/pixel_phone")
                .header("authorization", format!("Bearer {}", token))
                .header("x-device-id", &device_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 6. List devices after deletion -> 1 device remaining
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/devices")
                .header("authorization", format!("Bearer {}", token))
                .header("x-device-id", &device_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let devices: Vec<DeviceItemResponse> = serde_json::from_slice(&body).unwrap();
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].id, device_id);
}
