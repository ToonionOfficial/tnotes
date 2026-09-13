pub mod error;
pub mod extract;
pub mod markdown;
pub mod model;
pub mod parser;
pub mod serializer;

pub use error::ParseError;
pub use model::*;
pub use parser::Parser;
pub use serializer::Serializer;

impl Document {
    pub fn from_markdown(md: &str) -> Result<Self, ParseError> {
        markdown::parse::parse_markdown(md)
    }

    pub fn to_markdown(&self) -> String {
        markdown::serialize::serialize_markdown(&self.blocks)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_html(html: &str) -> Result<Self, ParseError> {
        Parser::new().parse(html)
    }

    pub fn to_html(&self) -> String {
        Serializer::serialize(self)
    }
}
