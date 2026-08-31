use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeymapSection {
    pub context: Option<String>,
    pub bindings: HashMap<String, String>,
}

pub type KeymapConfig = Vec<KeymapSection>;
