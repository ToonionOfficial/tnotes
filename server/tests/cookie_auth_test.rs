use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tnotes_core::db::migrations::open_in_memory;
use tnotes_server::{
    build_router,
    middleware::SESSION_COOKIE_NAME,
    routes::auth::{LoginResponse, MeResponse, SetupResponse},
    routes::pair::PairingDataResponse,
    state::{AppState, ServerConfig},
};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_httponly_cookie_authentication_flow() {
    let conn = open_in_memory().unwrap();
    let config = ServerConfig {
        host: "127.0.0.1".into(),
        port: 8787,
        data_dir: ":memory:".into(),
        server_url: "http://localhost:8787".into(),
    };
    let state = AppState::new(conn, config);
    let app = build_router(state);

    // 1. Initial Setup: POST /api/setup
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/setup")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({ "username": "owner", "password": "secure_password_123" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);

    // Verify Set-Cookie header contains notat_session with HttpOnly and SameSite=Lax
    let cookie_header = res
        .headers()
        .get("set-cookie")
        .expect("Expected set-cookie header on setup")
        .to_str()
        .unwrap();
    assert!(cookie_header.contains(&format!("{}=", SESSION_COOKIE_NAME)));
    assert!(cookie_header.to_lowercase().contains("httponly"));
    assert!(cookie_header.to_lowercase().contains("samesite=lax"));
    assert!(cookie_header.contains("Path=/"));

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let setup_res: SetupResponse = serde_json::from_slice(&body).unwrap();
    assert!(setup_res.ok);
    assert_eq!(setup_res.username, "owner");
    assert!(!setup_res.token.is_empty());
    assert!(!setup_res.device_id.is_empty());

    let session_cookie = format!("{}={}", SESSION_COOKIE_NAME, setup_res.token);

    // 2. GET /api/me using the session cookie -> 200 OK
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/me")
                .header("cookie", &session_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let me_res: MeResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(me_res.username, "owner");
    assert_eq!(me_res.user_id, setup_res.user_id);
    assert_eq!(me_res.device_id, setup_res.device_id);
    assert_eq!(me_res.platform, "web");

    // 3. GET /api/pair using the session cookie -> 200 OK
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/pair")
                .header("cookie", &session_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let pair_res: PairingDataResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(pair_res.pairing_code.len(), 6);
    assert!(pair_res.qr_svg.contains("<svg"));

    // 4. POST /api/login sets new httpOnly cookie
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "username": "owner",
                        "password": "secure_password_123",
                        "device_id": "browser_tab_2",
                        "device_name": "Firefox",
                        "platform": "web"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let login_cookie_header = res
        .headers()
        .get("set-cookie")
        .expect("Expected set-cookie header on login")
        .to_str()
        .unwrap();
    assert!(login_cookie_header.contains(&format!("{}=", SESSION_COOKIE_NAME)));

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let login_res: LoginResponse = serde_json::from_slice(&body).unwrap();
    let login_cookie = format!("{}={}", SESSION_COOKIE_NAME, login_res.token);

    // 5. POST /api/logout clears session and unsets cookie
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/logout")
                .header("cookie", &login_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let logout_cookie_header = res
        .headers()
        .get("set-cookie")
        .expect("Expected set-cookie header on logout")
        .to_str()
        .unwrap();
    assert!(logout_cookie_header.contains("Max-Age=0"));

    // 6. Accessing /api/me with the logged-out cookie -> 401 Unauthorized
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/me")
                .header("cookie", &login_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}
