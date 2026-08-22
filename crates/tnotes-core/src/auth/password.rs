use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use rand_core::OsRng;

use crate::errors::{Error, Result};

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| Error::PasswordHash(e.to_string()))?
        .to_string();
    Ok(hash)
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool> {
    let parsed_hash =
        PasswordHash::new(password_hash).map_err(|e| Error::PasswordHash(e.to_string()))?;
    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(Error::PasswordHash(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_success() {
        let password = "super_secret_password_123!";
        let hash = hash_password(password).unwrap();

        assert!(hash.starts_with("$argon2id$"));
        assert!(verify_password(password, &hash).unwrap());
    }

    #[test]
    fn test_verify_wrong_password() {
        let password = "correct_password";
        let wrong_password = "wrong_password";
        let hash = hash_password(password).unwrap();

        assert!(!verify_password(wrong_password, &hash).unwrap());
    }

    #[test]
    fn test_verify_invalid_hash_format() {
        let res = verify_password("test", "not_a_valid_argon2_hash");
        assert!(res.is_err());
    }
}
