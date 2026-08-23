use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Sent by the client in `POST /api/sync`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncEnvelope {
    pub device_id: String,
    #[serde(default)]
    pub last_seq: i64,
    #[serde(default)]
    pub last_sync_at: i64,
    pub changes: Vec<Change>,
}

/// A single entity change (note, folder, or theme).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncResponse {
    pub server_time: i64,
    #[serde(default)]
    pub cursor: i64,
    #[serde(default)]
    pub has_more: bool,
    pub changes: Vec<Change>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Note,
    Folder,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_sync_envelope_defaults_and_roundtrip() {
        // Test parsing JSON without last_seq or last_sync_at defaults them to 0
        let raw_json = json!({
            "device_id": "dev_1",
            "changes": []
        });

        let envelope: SyncEnvelope = serde_json::from_value(raw_json).unwrap();
        assert_eq!(envelope.device_id, "dev_1");
        assert_eq!(envelope.last_seq, 0);
        assert_eq!(envelope.last_sync_at, 0);
        assert!(envelope.changes.is_empty());

        let serialized = serde_json::to_string(&envelope).unwrap();
        let parsed: SyncEnvelope = serde_json::from_str(&serialized).unwrap();
        assert_eq!(envelope, parsed);
    }

    #[test]
    fn test_sync_response_defaults_and_roundtrip() {
        // Test parsing legacy response without cursor or has_more
        let raw_json = json!({
            "server_time": 1000,
            "changes": []
        });

        let res: SyncResponse = serde_json::from_value(raw_json).unwrap();
        assert_eq!(res.server_time, 1000);
        assert_eq!(res.cursor, 0);
        assert!(!res.has_more);

        let full_res = SyncResponse {
            server_time: 2000,
            cursor: 42,
            has_more: true,
            changes: vec![],
        };
        let serialized = serde_json::to_string(&full_res).unwrap();
        let parsed: SyncResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(full_res, parsed);
    }
}
