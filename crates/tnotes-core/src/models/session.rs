use serde::{Deserialize, Serialize};

use crate::models::current_time_ms;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub token: String,
    pub device_id: String,
    pub created_at: i64,
    pub expires_at: i64,
}

impl Session {
    pub fn new(token: impl Into<String>, device_id: impl Into<String>, expires_at: i64) -> Self {
        Self {
            token: token.into(),
            device_id: device_id.into(),
            created_at: current_time_ms(),
            expires_at,
        }
    }

    pub fn is_expired(&self) -> bool {
        current_time_ms() > self.expires_at
    }
}
