use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::models::current_time_ms;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
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
    ) -> Self {
        let now = current_time_ms();

        Self {
            id: Ulid::generate().to_string(),
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
}
