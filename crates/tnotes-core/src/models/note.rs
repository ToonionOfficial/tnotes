use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::models::{compute_checksum, current_time_ms};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub user_id: String,
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
    /// Creates a new note with version 1 and calculates its initial checksum
    pub fn new(
        title: impl Into<String>,
        body: impl Into<String>,
        folder_id: Option<String>,
        device_id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> Self {
        let title = title.into();
        let body = body.into();
        let device_id = device_id.into();
        let user_id = user_id.into();
        let now = current_time_ms();
        let checksum = compute_checksum(&body);

        Self {
            id: Ulid::generate().to_string(),
            user_id,
            folder_id,
            title,
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

    /// Updates title, body, and folder_id, automatically recalculating checksum,
    /// bumping version, and updating timestamp.
    pub fn update(
        &mut self,
        title: impl Into<String>,
        body: impl Into<String>,
        folder_id: Option<String>,
        device_id: impl Into<String>,
    ) {
        let body = body.into();
        self.checksum = compute_checksum(&body);
        self.title = title.into();
        self.body = body;
        self.folder_id = folder_id;
        self.device_id = device_id.into();
        self.version += 1;
        self.updated_at = current_time_ms();
    }

    /// Moves the note to trash (soft delete)
    pub fn trash(&mut self, device_id: impl Into<String>) {
        let now = current_time_ms();
        self.trashed = true;
        self.deleted_at = Some(now);
        self.updated_at = now;
        self.device_id = device_id.into();
        self.version += 1;
    }

    /// Restores the note from trash
    pub fn restore(&mut self, device_id: impl Into<String>) {
        let now = current_time_ms();
        self.trashed = false;
        self.deleted_at = None;
        self.updated_at = now;
        self.device_id = device_id.into();
        self.version += 1;
    }

    /// Toggles or updates the pinned status
    pub fn set_pinned(&mut self, pinned: bool, device_id: impl Into<String>) {
        self.pinned = pinned;
        self.updated_at = current_time_ms();
        self.device_id = device_id.into();
        self.version += 1;
    }
}
