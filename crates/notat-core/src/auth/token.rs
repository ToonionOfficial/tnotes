//! Session token and pairing code generation.

use rand::Rng;
use rand_core::{OsRng, RngCore};

use crate::models::{current_time_ms, session::Session};

/// Default session validity period: 90 days (in milliseconds)
pub const DEFAULT_SESSION_TTL_MS: i64 = 90 * 24 * 60 * 60 * 1000;

/// Generates a cryptographically secure 32-byte (256-bit) session token,
/// returned as a 64-character lowercase hex string.
pub fn generate_session_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let mut hex = String::with_capacity(64);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(hex, "{:02x}", b);
    }
    hex
}

/// Generates a human-friendly 6-digit numeric pairing code (e.g. "842195")
/// for pairing desktop clients with the server.
pub fn generate_pairing_code() -> String {
    let mut rng = rand::thread_rng();
    let code: u32 = rng.gen_range(100_000..=999_999);
    code.to_string()
}

/// Helper to construct a new `Session` with default or custom TTL
pub fn create_session_for_device(
    device_id: impl Into<String>,
    custom_ttl_ms: Option<i64>,
) -> Session {
    let token = generate_session_token();
    let now = current_time_ms();
    let ttl = custom_ttl_ms.unwrap_or(DEFAULT_SESSION_TTL_MS);
    Session::new(token, device_id, now + ttl)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_session_token() {
        let token_1 = generate_session_token();
        let token_2 = generate_session_token();

        assert_eq!(token_1.len(), 64);
        assert_eq!(token_2.len(), 64);
        assert_ne!(token_1, token_2);
        assert!(token_1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_pairing_code() {
        let code = generate_pairing_code();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
        let num: u32 = code.parse().unwrap();
        assert!((100_000..=999_999).contains(&num));
    }

    #[test]
    fn test_create_session_for_device() {
        let session = create_session_for_device("dev_phone", None);
        assert_eq!(session.device_id, "dev_phone");
        assert_eq!(session.token.len(), 64);
        assert!(!session.is_expired());
    }

    #[test]
    fn test_expired_session() {
        let session = create_session_for_device("dev_phone", Some(-1000));
        assert!(session.is_expired());
    }
}
