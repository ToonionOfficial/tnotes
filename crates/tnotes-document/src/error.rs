use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    #[error("Failed to parse Markdown structure: {0}")]
    InvalidMarkdown(String),
}
