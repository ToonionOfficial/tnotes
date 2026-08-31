pub mod error;
pub mod model;
pub mod parser;
pub mod serializer;

pub use error::ParseError;
pub use model::*;
pub use parser::Parser;
pub use serializer::Serializer;

impl Document {
    /// Convenient 1-line parser on Document
    pub fn from_html(html: &str) -> Result<Self, ParseError> {
        Parser::new().parse(html)
    }

    /// Convenient 1-line serializer on Document
    pub fn to_html(&self) -> String {
        Serializer::serialize(self)
    }
}
