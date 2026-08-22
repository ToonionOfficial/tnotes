use serde::{Deserialize, Serialize};

use crate::models::current_time_ms;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub platform: String,
    pub last_seen_at: i64,
    pub created_at: i64,
}

impl Device {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        platform: impl Into<String>,
    ) -> Self {
        let now = current_time_ms();

        Self {
            id: id.into(),
            name: name.into(),
            platform: platform.into(),
            last_seen_at: now,
            created_at: now,
        }
    }

    pub fn touch(&mut self) {
        self.last_seen_at = current_time_ms()
    }
}
