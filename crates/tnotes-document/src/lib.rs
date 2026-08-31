pub mod error;
pub mod model;
pub mod parser;

pub use error::ParseError;
pub use model::*;
pub use parser::Parser;

impl Document {
    /// Convenient 1-line parser on Document
    pub fn from_html(html: &str) -> Result<Self, ParseError> {
        Parser::new().parse(html)
    }
}
