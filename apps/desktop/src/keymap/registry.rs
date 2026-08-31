use crate::components::fps_overlay::{ToggleFps, ToggleStressTest};
use crate::views::{CloseModal, CreateNewNote, ToggleCommandPalette, ToggleSettings};
use gpui::KeyBinding;

#[derive(Debug, Clone)]
pub struct ActionMetadata {
    pub id: &'static str,
    pub label: &'static str,
    pub category: &'static str,
    pub default_shortcut: &'static str,
}

pub const REGISTERED_ACTIONS: &[ActionMetadata] = &[
    ActionMetadata {
        id: "workspace::toggle_command_palette",
        label: "Toggle Command Palette",
        category: "Navigation",
        default_shortcut: "ctrl-p / cmd-p",
    },
    ActionMetadata {
        id: "workspace::create_new_note",
        label: "Create New Note",
        category: "Notes",
        default_shortcut: "ctrl-n / cmd-n",
    },
    ActionMetadata {
        id: "workspace::toggle_settings",
        label: "Open Settings & Preferences",
        category: "Settings",
        default_shortcut: "ctrl-, / cmd-,",
    },
    ActionMetadata {
        id: "workspace::close_modal",
        label: "Close Active Modal Dialog",
        category: "Navigation",
        default_shortcut: "escape",
    },
    ActionMetadata {
        id: "fps::toggle_fps",
        label: "Toggle Performance & FPS Monitor",
        category: "Developer",
        default_shortcut: "f3",
    },
    ActionMetadata {
        id: "fps::toggle_stress_test",
        label: "Toggle 144Hz Frame Stress Test",
        category: "Developer",
        default_shortcut: "shift-f3",
    },
];

pub fn build_key_binding(key: &str, action_name: &str, context: Option<&str>) -> Option<KeyBinding> {
    match action_name {
        "workspace::toggle_command_palette" => Some(KeyBinding::new(key, ToggleCommandPalette, context)),
        "workspace::create_new_note" => Some(KeyBinding::new(key, CreateNewNote, context)),
        "workspace::toggle_settings" => Some(KeyBinding::new(key, ToggleSettings, context)),
        "workspace::close_modal" => Some(KeyBinding::new(key, CloseModal, context)),
        "fps::toggle_fps" => Some(KeyBinding::new(key, ToggleFps, context)),
        "fps::toggle_stress_test" => Some(KeyBinding::new(key, ToggleStressTest, context)),
        _ => {
            eprintln!("Unknown action in keymap: {action_name}");
            None
        }
    }
}
