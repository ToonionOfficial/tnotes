use std::net::SocketAddr;
use tnotes_server::{
    build_router,
    state::{AppState, ServerConfig},
    tasks::start_housekeeping_task,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tnotes_server=info,tnotes_server=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = ServerConfig::from_env();
    tracing::info!("Starting TNotes server on {}:{}", config.host, config.port);

    std::fs::create_dir_all(&config.data_dir)?;
    
    // Check if legacy notat.db exists, otherwise use tnotes.db
    let legacy_db = format!("{}/notat.db", config.data_dir);
    let primary_db = format!("{}/tnotes.db", config.data_dir);
    let db_path = if std::path::Path::new(&legacy_db).exists() && !std::path::Path::new(&primary_db).exists() {
        legacy_db
    } else {
        primary_db
    };

    tracing::info!("Opening database at {}", db_path);

    let conn = tnotes_core::db::migrations::open_connection(&db_path)?;
    tnotes_core::db::themes::seed_default_themes(&conn)?;

    let state = AppState::new(conn, config.clone());

    // Start background housekeeping task (runs every 1 hour)
    start_housekeeping_task(state.db.clone(), 3600);

    let app = build_router(state);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("TNotes server listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
