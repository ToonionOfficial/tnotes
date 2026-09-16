use gpui::{App, KeyBinding};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::Result;

use crate::keymap::actions::create_binding;
use crate::paths;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeymapSection {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    pub bindings: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeymapConfig(pub Vec<KeymapSection>);

impl Default for KeymapConfig {
    fn default() -> Self {
        Self::default_config()
    }
}

impl KeymapConfig {
    pub fn default_config() -> Self {
        let mut global_bindings = BTreeMap::new();
        global_bindings.insert("f3".to_string(), "tnotes::ToggleFps".to_string());
        global_bindings.insert("ctrl-b".to_string(), "tnotes::ToggleSidebar".to_string());
        global_bindings.insert("ctrl-n".to_string(), "tnotes::NewNote".to_string());
        global_bindings.insert("ctrl-k".to_string(), "tnotes::FocusSearch".to_string());
        global_bindings.insert("ctrl-,".to_string(), "tnotes::OpenSettings".to_string());
        global_bindings.insert("alt-left".to_string(), "tnotes::NavigateBack".to_string());
        global_bindings.insert(
            "alt-right".to_string(),
            "tnotes::NavigateForward".to_string(),
        );
        global_bindings.insert("ctrl-[".to_string(), "tnotes::NavigateBack".to_string());
        global_bindings.insert("ctrl-]".to_string(), "tnotes::NavigateForward".to_string());

        let mut sidebar_bindings = BTreeMap::new();
        sidebar_bindings.insert("delete".to_string(), "tnotes::DeleteNote".to_string());
        sidebar_bindings.insert("p".to_string(), "tnotes::PinNote".to_string());

        let mut editor_bindings = BTreeMap::new();
        editor_bindings.insert("ctrl-s".to_string(), "tnotes::Save".to_string());
        editor_bindings.insert("ctrl-enter".to_string(), "tnotes::SaveAndClose".to_string());

        Self(vec![
            KeymapSection {
                context: None,
                bindings: global_bindings,
            },
            KeymapSection {
                context: Some("Sidebar".to_string()),
                bindings: sidebar_bindings,
            },
            KeymapSection {
                context: Some("Editor".to_string()),
                bindings: editor_bindings,
            },
        ])
    }

    pub fn load() -> Self {
        if let Err(e) = paths::ensure_dirs() {
            eprintln!("[keymap] failed to ensure directories: {e}");
            return Self::default_config();
        }

        let path = paths::keymap_file();
        if !path.exists() {
            let default_cfg = Self::default_config();
            if let Err(e) = default_cfg.save() {
                eprintln!("[keymap] failed to save default keymap: {e}");
            }
            return default_cfg;
        }

        match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<Self>(&content) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("[keymap] failed to parse {}: {e}", path.display());
                    Self::default_config()
                }
            },
            Err(e) => {
                eprintln!("[keymap] failed to read {}: {e}", path.display());
                Self::default_config()
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        paths::ensure_dirs()?;
        let path = paths::keymap_file();
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(path, json)
    }

    pub fn to_gpui_bindings(&self) -> Vec<KeyBinding> {
        let mut result = Vec::new();
        for section in &self.0 {
            let ctx = section.context.as_deref();
            for (keystroke, action_id) in &section.bindings {
                if let Some(binding) = create_binding(action_id, keystroke, ctx) {
                    result.push(binding);
                }
            }
        }
        result
    }

    pub fn bind_to_gpui(&self, cx: &mut App) {
        cx.bind_keys(self.to_gpui_bindings());
    }

    #[allow(dead_code)]
    pub fn get_key_for_action(&self, action_id: &str) -> Option<(String, Option<String>)> {
        for section in &self.0 {
            for (key, act) in &section.bindings {
                if act == action_id {
                    return Some((key.clone(), section.context.clone()));
                }
            }
        }
        None
    }

    /// All actions bound to `keystroke` (normalized comparison), with their
    /// contexts. Used for conflict display in the keybindings editor.
    pub fn actions_for_keystroke(&self, keystroke: &str) -> Vec<(String, Option<String>)> {
        let wanted = normalize_keystroke(keystroke);
        self.0
            .iter()
            .flat_map(|section| {
                section
                    .bindings
                    .iter()
                    .filter(|(key, _)| normalize_keystroke(key) == wanted)
                    .map(|(_, act)| (act.clone(), section.context.clone()))
            })
            .collect()
    }

    pub fn set_key_for_action(
        &mut self,
        action_id: &str,
        new_key: &str,
        target_context: Option<&str>,
    ) {
        for section in &mut self.0 {
            section.bindings.retain(|_, act| act != action_id);
        }

        let target_ctx_str = target_context.map(|s| s.to_string());
        if let Some(section) = self.0.iter_mut().find(|s| s.context == target_ctx_str) {
            section
                .bindings
                .insert(new_key.to_string(), action_id.to_string());
        } else {
            let mut bindings = BTreeMap::new();
            bindings.insert(new_key.to_string(), action_id.to_string());
            self.0.push(KeymapSection {
                context: target_ctx_str,
                bindings,
            });
        }
    }
}

/// Normalize a keystroke string the same way `create_binding` does, so
/// conflict checks compare apples to apples.
fn normalize_keystroke(raw: &str) -> String {
    raw.trim().replace('+', "-").to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_roundtrips_json() {
        let config = KeymapConfig::default_config();
        let json = serde_json::to_string_pretty(&config).unwrap();

        assert!(json.contains("f3"));
        assert!(json.contains("tnotes::ToggleFps"));
        assert!(json.contains("Sidebar"));
        assert!(json.contains("Editor"));

        let deserialized: KeymapConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn default_config_generates_gpui_bindings() {
        let config = KeymapConfig::default_config();
        let bindings = config.to_gpui_bindings();
        assert!(!bindings.is_empty());
    }

    #[test]
    fn set_and_get_key_for_action() {
        let mut config = KeymapConfig::default_config();
        config.set_key_for_action("tnotes::ToggleFps", "f4", None);

        let (key, ctx) = config.get_key_for_action("tnotes::ToggleFps").unwrap();
        assert_eq!(key, "f4");
        assert_eq!(ctx, None);
    }
}
