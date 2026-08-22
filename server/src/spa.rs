use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::Response,
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "../web/dist"]
pub struct WebAssets;

pub async fn spa_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    // 1. Try to serve the exact asset file (e.g. /assets/index-xxx.js, /favicon.ico, etc.)
    if !path.is_empty() {
        if let Some(file) = WebAssets::get(path) {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            let cache_control = if path.starts_with("assets/") {
                "public, max-age=31536000, immutable"
            } else {
                "public, max-age=3600"
            };

            return Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .header(header::CACHE_CONTROL, cache_control)
                .body(Body::from(file.data))
                .unwrap();
        }
    }

    // 2. SPA Fallback: Serve index.html for all navigation routes (/, /setup, /login, /pair, etc.)
    if let Some(index) = WebAssets::get("index.html") {
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
            .body(Body::from(index.data))
            .unwrap();
    }

    // 3. 404 Fallback if assets were not compiled
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from("404 Not Found: Frontend assets not built."))
        .unwrap()
}
