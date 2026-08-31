pub mod defaults;
pub mod registry;
pub mod schema;

use gpui::App;
use registry::build_key_binding;
use schema::KeymapConfig;
use std::path::PathBuf;

pub use registry::{ActionMetadata, REGISTERED_ACTIONS};

pub fn init_keymap(cx: &mut App) {
    let keymap = load_keymap();
    let mut bindings = Vec::new();

    for section in keymap {
        let context_str = section.context.as_deref();
        for (key, action) in section.bindings {
            if let Some(binding) = build_key_binding(&key, &action, context_str) {
                bindings.push(binding);
            }
        }
    }

    cx.bind_keys(bindings);
}

pub fn load_keymap() -> KeymapConfig {
    let mut config = defaults::load_default_keymap();

    let user_path = user_keymap_path();
    if user_path.exists()
        && let Ok(content) = std::fs::read_to_string(&user_path)
        && let Ok(user_config) = serde_json::from_str::<KeymapConfig>(&content)
    {
        for user_section in user_config {
            if let Some(existing) = config
                .iter_mut()
                .find(|s| s.context == user_section.context)
            {
                for (key, action) in user_section.bindings {
                    existing.bindings.insert(key, action);
                }
            } else {
                config.push(user_section);
            }
        }
    }

    config
}

pub fn user_keymap_path() -> PathBuf {
    if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(config_home).join("tnotes/keymap.json")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config/tnotes/keymap.json")
    } else {
        PathBuf::from("keymap.json")
    }
}
