use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    #[error("Failed to parse HTML structure")]
    InvalidHtml,

    #[error("Unsupported block element: <{0}>")]
    UnsupportedBlock(String),

    #[error("Malformed block attributes in <{tag}>: {reason}")]
    MalformedAttributes { tag: String, reason: String },
}
