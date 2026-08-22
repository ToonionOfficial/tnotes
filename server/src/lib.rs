pub mod middleware;
pub mod routes;
pub mod state;
pub mod ws;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(routes::health::health_check))
        .route("/api/setup/status", get(routes::auth::setup_status_handler))
        .route("/api/setup", post(routes::auth::setup_handler))
        .route("/api/login", post(routes::auth::login_handler))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
