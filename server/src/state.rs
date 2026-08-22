use notat_core::{Connection, sync::envelope::Change};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};

/// Broadcast message sent over WebSockets when changes occur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsBroadcastMessage {
    pub sender_device_id: String,
    pub changes: Vec<Change>,
}

/// Server runtime configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub data_dir: String,
    pub server_url: String,
}

impl ServerConfig {
    pub fn from_env() -> Self {
        let host = std::env::var("NOTAT_HOST").unwrap_or_else(|_| "0.0.0.0".into());
        let port: u16 = std::env::var("NOTAT_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8787);
        let data_dir = std::env::var("NOTAT_DATA_DIR").unwrap_or_else(|_| "./data".into());
        let server_url = std::env::var("NOTAT_SERVER_URL")
            .unwrap_or_else(|_| format!("http://localhost:{}", port));

        Self {
            host,
            port,
            data_dir,
            server_url,
        }
    }
}

/// Shared application state injected into all Axum route handlers
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub ws_sender: broadcast::Sender<WsBroadcastMessage>,
    pub config: Arc<ServerConfig>,
}

impl AppState {
    pub fn new(conn: Connection, config: ServerConfig) -> Self {
        // Buffer up to 128 broadcast messages
        let (ws_sender, _) = broadcast::channel(128);

        Self {
            db: Arc::new(Mutex::new(conn)),
            ws_sender,
            config: Arc::new(config),
        }
    }
}
