use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::models::{compute_checksum, current_time_ms};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub folder_id: Option<String>,
    pub title: String,
    pub body: String,
    pub pinned: bool,
    pub trashed: bool,
    pub version: u64,
    pub updated_at: i64,
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,
    pub device_id: String,
    pub checksum: String,
}

impl Note {
    pub fn new(
        title: impl Into<String>,
        body: impl Into<String>,
        folder_id: Option<String>,
        device_id: impl Into<String>,
    ) -> Self {
        let title = title.into();
        let body = body.into();
        let device_id = device_id.into();
        let now = current_time_ms();
        let checksum = compute_checksum(&body);

        Self {
            id: Ulid::generate().to_string(),
            folder_id,
            title: title.into(),
            body,
            pinned: false,
            trashed: false,
            version: 1,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            device_id,
            checksum,
        }
    }

    pub fn update() {}

    pub fn trash() {}

    pub fn restore() {}
}
