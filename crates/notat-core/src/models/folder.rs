use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::models::current_time_ms;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub name: String,
    pub icon: String,
    pub sort_order: i32,
    pub version: u64,
    pub updated_at: i64,
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,
    pub device_id: String,
}

impl Folder {
    pub fn new(
        name: impl Into<String>,
        icon: Option<String>,
        parent_id: Option<String>,
        sort_order: i32,
        device_id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> Self {
        let now = current_time_ms();

        Self {
            id: Ulid::generate().to_string(),
            user_id: user_id.into(),
            parent_id,
            name: name.into(),
            icon: icon.unwrap_or_else(|| "📁".to_string()),
            sort_order,
            version: 1,
            updated_at: now,
            created_at: now,
            deleted_at: None,
            device_id: device_id.into(),
        }
    }

    /// Renames the folder and bumps version + timestamp
    pub fn rename(&mut self, name: impl Into<String>, device_id: impl Into<String>) {
        self.name = name.into();
        self.version += 1;
        self.updated_at = current_time_ms();
        self.device_id = device_id.into();
    }

    /// Moves the folder to a new parent folder
    pub fn move_to(&mut self, parent_id: Option<String>, device_id: impl Into<String>) {
        self.parent_id = parent_id;
        self.version += 1;
        self.updated_at = current_time_ms();
        self.device_id = device_id.into();
    }

    /// Sets the icon emoji
    pub fn set_icon(&mut self, icon: impl Into<String>, device_id: impl Into<String>) {
        self.icon = icon.into();
        self.version += 1;
        self.updated_at = current_time_ms();
        self.device_id = device_id.into();
    }

    /// Soft deletes the folder
    pub fn soft_delete(&mut self, device_id: impl Into<String>) {
        let now = current_time_ms();
        self.deleted_at = Some(now);
        self.version += 1;
        self.updated_at = now;
        self.device_id = device_id.into();
    }

    /// Restores a soft-deleted folder
    pub fn restore(&mut self, device_id: impl Into<String>) {
        self.deleted_at = None;
        self.version += 1;
        self.updated_at = current_time_ms();
        self.device_id = device_id.into();
    }
}
