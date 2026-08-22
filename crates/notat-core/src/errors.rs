use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Sync error: {0}")]
    Sync(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Password hashing error: {0}")]
    PasswordHash(String),
}

pub type Result<T> = std::result::Result<T, Error>;
