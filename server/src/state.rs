use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tnotes_core::{Connection, sync::envelope::Change};
use tokio::sync::{Mutex, broadcast};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsBroadcastMessage {
    pub sender_device_id: String,
    pub user_id: String,
    pub changes: Vec<Change>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingPairing {
    pub code: String,
    pub token: String,
    pub user_id: String,
    pub username: String,
    pub device_id: String,
    pub expires_at: i64,
    pub claimed: bool,
    pub claimed_device_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WsTicket {
    pub user_id: String,
    pub device_id: String,
    pub expires_at: i64,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub data_dir: String,
    pub server_url: String,
}

impl ServerConfig {
    pub fn from_env() -> Self {
        let host = std::env::var("TNOTES_HOST")
            .or_else(|_| std::env::var("NOTAT_HOST"))
            .unwrap_or_else(|_| "0.0.0.0".into());

        let port: u16 = std::env::var("TNOTES_PORT")
            .or_else(|_| std::env::var("NOTAT_PORT"))
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8787);

        let data_dir = std::env::var("TNOTES_DATA_DIR")
            .or_else(|_| std::env::var("NOTAT_DATA_DIR"))
            .unwrap_or_else(|_| "./data".into());

        let server_url = std::env::var("TNOTES_SERVER_URL")
            .or_else(|_| std::env::var("NOTAT_SERVER_URL"))
            .unwrap_or_else(|_| format!("http://localhost:{}", port));

        Self {
            host,
            port,
            data_dir,
            server_url,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub ws_sender: broadcast::Sender<WsBroadcastMessage>,
    pub config: Arc<ServerConfig>,
    pub pending_pairings: Arc<Mutex<HashMap<String, PendingPairing>>>,
    pub pending_ws_tickets: Arc<Mutex<HashMap<String, WsTicket>>>,
}

impl AppState {
    pub fn new(conn: Connection, config: ServerConfig) -> Self {
        let (ws_sender, _) = broadcast::channel(128);

        Self {
            db: Arc::new(Mutex::new(conn)),
            ws_sender,
            config: Arc::new(config),
            pending_pairings: Arc::new(Mutex::new(HashMap::new())),
            pending_ws_tickets: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
