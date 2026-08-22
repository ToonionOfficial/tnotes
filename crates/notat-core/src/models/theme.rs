use serde::{Deserialize, Serialize};

/// Stored theme definition. The `schema` field holds the raw JSON theme
/// (colors, typography, spacing) as a string so it can be forwarded to
/// clients without the server needing to understand its internal shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub id: String,
    pub name: String,
    pub builtin: bool,
    /// Raw JSON blob containing colors, typography, spacing tokens.
    pub schema: String, // use a ThemeSchema struct
    pub version: u64,
    pub updated_at: i64,
    pub created_at: i64,
    pub device_id: String,
}
