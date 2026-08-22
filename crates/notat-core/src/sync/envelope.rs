//! Sync protocol types: SyncEnvelope, Change, SyncResponse.

pub struct SyncEnvelope {
    pub device_id: String,
    pub last_sync_at: i64,
    pub changes: Vec<Change>,
}

pub struct Change {
    pub entity_type: String,
    pub entity_id: String,
    pub version: u64,
    pub updated_at: i64,
    pub tombstone: bool,
    pub payload: String,
}

pub struct SyncResponse {
    pub server_time: i64,
    pub changes: Vec<Change>,
}
