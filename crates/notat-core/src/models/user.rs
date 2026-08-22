use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::models::current_time_ms;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: i64,
}

impl User {
    pub fn new(username: impl Into<String>, password_hash: impl Into<String>) -> Self {
        Self {
            id: Ulid::generate().to_string(),
            username: username.into(),
            password_hash: password_hash.into(),
            created_at: current_time_ms(),
        }
    }
}
