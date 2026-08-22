pub mod middleware;
pub mod routes;
pub mod state;
pub mod ws;

use axum::{
    Router,
    response::Html,
    routing::{get, post},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::state::AppState;

const INDEX_HTML: &str = include_str!("web/index.html");

async fn index_handler() -> Html<&'static str> {
    Html(INDEX_HTML)
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/api/health", get(routes::health::health_check))
        .route("/api/setup/status", get(routes::auth::setup_status_handler))
        .route("/api/setup", post(routes::auth::setup_handler))
        .route("/api/login", post(routes::auth::login_handler))
        .route("/api/pair", get(routes::pair::pair_handler))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
