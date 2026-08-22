pub mod middleware;
pub mod routes;
pub mod state;
pub mod tasks;
pub mod ws;

use axum::{
    middleware::from_fn_with_state,
    response::Html,
    routing::{get, post},
    Router,
};
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    trace::TraceLayer,
};

use crate::state::AppState;

const INDEX_HTML: &str = include_str!("web/index.html");

async fn index_handler() -> Html<&'static str> {
    Html(INDEX_HTML)
}

pub fn build_router(state: AppState) -> Router {
    let protected_routes = Router::new()
        .route("/api/sync", post(routes::sync::sync_handler))
        .route("/api/pair", get(routes::pair::pair_handler))
        .route("/api/me", get(routes::auth::me_handler))
        .route("/api/logout", post(routes::auth::logout_handler))
        .layer(from_fn_with_state(state.clone(), middleware::require_auth));

    Router::new()
        .route("/", get(index_handler))
        .route("/ws/sync", get(ws::ws_sync_handler))
        .route("/api/health", get(routes::health::health_check))
        .route("/api/setup/status", get(routes::auth::setup_status_handler))
        .route("/api/setup", post(routes::auth::setup_handler))
        .route("/api/login", post(routes::auth::login_handler))
        .merge(protected_routes)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .with_state(state)
}
