use gpui::*;
use crate::keymap::{ALL_ACTIONS, ActionMeta, KeymapConfig};

/// Capture session for rebinding one action in the keybindings editor.
#[derive(Clone, Debug)]
pub struct KeybindingCapture {
    pub action_id: String,
    pub label: String,
    /// Live keystroke preview while capturing (`None` until first keypress).
    pub preview: Option<String>,
    /// Set when capture finishes with a conflicting keystroke.
    pub conflict: Option<String>,
}

impl KeybindingCapture {
    pub fn new(action_id: &str, label: &str) -> Self {
        Self {
            action_id: action_id.to_string(),
            label: label.to_string(),
            preview: None,
            conflict: None,
        }
    }
}

/// Serialize a `KeyDownEvent` to the canonical keystroke string used by
/// `KeymapConfig` (`ctrl-k`, `alt-left`, ...). Returns `None` for bare
/// modifier presses and for Escape (cancel).
pub fn keystroke_from_event(event: &KeyDownEvent) -> Option<String> {
    let key = event.keystroke.key.as_str();
    // Bare modifier presses carry no key; Escape cancels instead of binding.
    if key.eq_ignore_ascii_case("escape") || key.eq_ignore_ascii_case("shift") {
        return None;
    }
    let mut parts = Vec::new();
    if event.keystroke.modifiers.control {
        parts.push("ctrl".to_string());
    }
    if event.keystroke.modifiers.alt {
        parts.push("alt".to_string());
    }
    if event.keystroke.modifiers.platform {
        parts.push("super".to_string());
    }
    if event.keystroke.modifiers.shift {
        parts.push("shift".to_string());
    }
    parts.push(key.to_lowercase());
    Some(parts.join("-"))
}

/// Display a stored keystroke string as a human-readable badge label
/// (`ctrl-k` → `Ctrl+K`).
pub fn display_keystroke(stored: &str) -> String {
    stored
        .split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let mut label = first.to_uppercase().to_string();
                    label.push_str(&chars.as_str().to_lowercase());
                    label
                }
            }
        })
        .collect::<Vec<_>>()
        .join("+")
}

/// Group action metadata by category for sectioned display, preserving the
/// declaration order in `ALL_ACTIONS`.
pub fn actions_by_category() -> Vec<(&'static str, Vec<&'static ActionMeta>)> {
    let mut groups: Vec<(&'static str, Vec<&'static ActionMeta>)> = Vec::new();
    for action in ALL_ACTIONS {
        match groups.iter_mut().find(|(cat, _)| *cat == action.category) {
            Some((_, actions)) => actions.push(action),
            None => groups.push((action.category, vec![action])),
        }
    }
    groups
}

/// Apply a captured keystroke: update the config, persist to disk, and rebind
/// GPUI. Returns the conflicting action ids (excluding the rebound action
/// itself) so the caller can surface them.
pub fn apply_captured_keystroke(
    config: &mut KeymapConfig,
    action_id: &str,
    context: Option<&str>,
    keystroke: &str,
    cx: &mut App,
) -> Vec<String> {
    let conflicts: Vec<String> = config
        .actions_for_keystroke(keystroke)
        .into_iter()
        .filter(|(id, _)| id != action_id)
        .map(|(id, _)| id)
        .collect();
    config.set_key_for_action(action_id, keystroke, context);
    if let Err(e) = config.save() {
        eprintln!("[keymap] failed to persist keymap: {e}");
    }
    cx.clear_key_bindings();
    config.bind_to_gpui(cx);
    conflicts
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn keystroke_display_formats_badges() {
        assert_eq!(display_keystroke("ctrl-k"), "Ctrl+K");
        assert_eq!(display_keystroke("alt-left"), "Alt+Left");
        assert_eq!(display_keystroke("f3"), "F3");
        assert_eq!(display_keystroke("ctrl-shift-p"), "Ctrl+Shift+P");
    }

    #[test]
    fn actions_group_preserves_category_order() {
        let groups = actions_by_category();
        let names: Vec<&str> = groups.iter().map(|(cat, _)| *cat).collect();
        assert_eq!(names, vec!["General", "Notes", "Navigation", "Sidebar", "Editor"]);
        assert!(groups.iter().map(|(_, a)| a.len()).sum::<usize>() == ALL_ACTIONS.len());
    }

    #[test]
    fn conflict_lookup_ignores_case_and_plus() {
        let config = KeymapConfig::default_config();
        let hits = config.actions_for_keystroke("Ctrl+K");
        assert!(hits.iter().any(|(id, _)| id == "tnotes::FocusSearch"));
        let none = config.actions_for_keystroke("ctrl-q");
        assert!(none.is_empty());
    }
}
