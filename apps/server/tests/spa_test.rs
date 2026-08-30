use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tnotes_core::db::migrations::open_in_memory;
use tnotes_server::{
    build_router,
    state::{AppState, ServerConfig},
};
use tower::ServiceExt;

#[tokio::test]
async fn test_embedded_spa_and_fallback_routing() {
    let conn = open_in_memory().unwrap();
    let config = ServerConfig {
        host: "127.0.0.1".into(),
        port: 8787,
        data_dir: ":memory:".into(),
        server_url: "http://localhost:8787".into(),
    };
    let state = AppState::new(conn, config);
    let app = build_router(state);

    // 1. Root route / serves SPA index.html
    let res = app
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let content_type = res.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(content_type.contains("text/html"));

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(
        html.contains("<div id=\"root\"")
            || html.contains("<html")
            || html.contains("<!doctype html>")
    );

    // 2. Client route /login serves SPA fallback index.html
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/login")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let content_type = res.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(content_type.contains("text/html"));

    // 3. Client route /pair serves SPA fallback index.html
    let res = app
        .clone()
        .oneshot(Request::builder().uri("/pair").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // 4. API routes take precedence and do NOT serve the SPA fallback
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
    let content_type = res.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(content_type.contains("application/json"));
}
