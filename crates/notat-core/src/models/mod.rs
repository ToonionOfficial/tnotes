use blake3::hash;
use std::time::{SystemTime, UNIX_EPOCH};

pub mod device;
pub mod folder;
pub mod note;
pub mod session;
pub mod theme;
pub mod user;

pub fn current_time_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backward")
        .as_millis() as i64
}

pub fn compute_checksum(content: &str) -> String {
    hash(content.as_bytes()).to_hex().to_string()
}
