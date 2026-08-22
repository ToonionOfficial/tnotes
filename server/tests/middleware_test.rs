use axum::{
    body::Body,
    extract::Extension,
    http::{Request, StatusCode},
    middleware::from_fn_with_state,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use notat_core::{
    auth::token::create_session_for_device,
    db::{
        devices::upsert_device,
        migrations::open_in_memory,
        sessions::create_session,
        users::create_user,
    },
    models::{device::Device, user::User},
};
use notat_server::{
    middleware::{require_auth, AuthenticatedDevice},
    state::{AppState, ServerConfig},
};
use serde_json::json;
use tower::ServiceExt;

async fn protected_test_handler(
    Extension(auth): Extension<AuthenticatedDevice>,
) -> impl IntoResponse {
    Json(json!({ "device_id": auth.device_id, "user_id": auth.user_id }))
}

#[tokio::test]
async fn test_require_auth_middleware() {
    let conn = open_in_memory().unwrap();
    let config = ServerConfig {
        host: "127.0.0.1".into(),
        port: 8787,
        data_dir: ":memory:".into(),
        server_url: "http://localhost:8787".into(),
    };

    let user = User::new("testuser", "hash");
    create_user(&conn, &user).unwrap();

    let dev_phone = Device::new("device_phone", "Phone", "mobile", &user.id);
    let dev_old = Device::new("device_old", "Old Device", "mobile", &user.id);
    upsert_device(&conn, &dev_phone).unwrap();
    upsert_device(&conn, &dev_old).unwrap();

    let session_valid = create_session_for_device("device_phone", None);
    let session_expired = create_session_for_device("device_old", Some(-5000));

    create_session(&conn, &session_valid).unwrap();
    create_session(&conn, &session_expired).unwrap();

    let state = AppState::new(conn, config);

    let app = Router::new()
        .route("/protected", get(protected_test_handler))
        .layer(from_fn_with_state(state.clone(), require_auth))
        .with_state(state);

    // 1. Missing Authorization header -> 401
    let res = app
        .clone()
        .oneshot(Request::builder().uri("/protected").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // 2. Invalid Scheme (not Bearer) -> 401
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/protected")
                .header("authorization", "Basic 12345")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // 3. Fake token -> 401
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/protected")
                .header("authorization", "Bearer fake_token_123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // 4. Expired token -> 401
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/protected")
                .header("authorization", format!("Bearer {}", session_expired.token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // 5. Valid token -> 200 OK and receives device context
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/protected")
                .header("authorization", format!("Bearer {}", session_valid.token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
