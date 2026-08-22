mod middleware;
mod routes;
mod state;
mod ws;

use axum::{Router, routing::get};
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::state::{AppState, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "notat_server=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = ServerConfig::from_env();
    tracing::info!("Starting Notat server on {}:{}", config.host, config.port);

    std::fs::create_dir_all(&config.data_dir)?;
    let db_path = format!("{}/notat.db", config.data_dir);
    tracing::info!("Opening database at {}", db_path);

    let conn = notat_core::db::migrations::open_connection(&db_path)?;

    notat_core::db::themes::seed_default_themes(&conn)?;

    let state = AppState::new(conn, config.clone());

    let app = Router::new()
        .route("/api/health", get(routes::health::health_check))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Notat server listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
