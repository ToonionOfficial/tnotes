//! Sync protocol types: SyncEnvelope, Change, SyncResponse.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Sent by the client in `POST /api/sync`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEnvelope {
    pub device_id: String,
    pub last_sync_at: i64,
    pub changes: Vec<Change>,
}

/// A single entity change (note, folder, or theme).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub entity_type: EntityType,
    pub entity_id: String,
    pub version: u64,
    pub updated_at: i64,
    pub tombstone: bool,
    /// Full serialized entity (Note, Folder, or Theme as JSON).
    pub payload: Value,
}

/// Returned by the server from `POST /api/sync`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    pub server_time: i64,
    pub changes: Vec<Change>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Note,
    Folder,
    Theme,
}
