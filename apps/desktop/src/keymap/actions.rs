use gpui::{actions, KeyBinding};

actions!(tnotes, [
    ToggleFps,
    ToggleSidebar,
    NewNote,
    FocusSearch,
    PinNote,
    DeleteNote,
    Save,
    SaveAndClose,
    OpenSettings,
    CloseSettings,
    NavigateBack,
    NavigateForward,
]);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionMeta {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub default_context: Option<&'static str>,
    pub default_key: &'static str,
}

pub const ALL_ACTIONS: &[ActionMeta] = &[
    ActionMeta {
        id: "tnotes::ToggleFps",
        label: "Toggle Performance HUD",
        description: "Show or hide the realtime FPS and vitals monitor",
        category: "General",
        default_context: None,
        default_key: "f3",
    },
    ActionMeta {
        id: "tnotes::ToggleSidebar",
        label: "Toggle Sidebar",
        description: "Expand or collapse the left sidebar",
        category: "General",
        default_context: None,
        default_key: "ctrl-b",
    },
    ActionMeta {
        id: "tnotes::NewNote",
        label: "New Note",
        description: "Create a new note in the selected folder",
        category: "Notes",
        default_context: None,
        default_key: "ctrl-n",
    },
    ActionMeta {
        id: "tnotes::FocusSearch",
        label: "Search Notes",
        description: "Focus the notes search input",
        category: "Navigation",
        default_context: None,
        default_key: "ctrl-k",
    },
    ActionMeta {
        id: "tnotes::OpenSettings",
        label: "Settings",
        description: "Open application settings",
        category: "General",
        default_context: None,
        default_key: "ctrl-,",
    },
    ActionMeta {
        id: "tnotes::PinNote",
        label: "Pin / Unpin Note",
        description: "Toggle pin status of the selected note",
        category: "Sidebar",
        default_context: Some("Sidebar"),
        default_key: "p",
    },
    ActionMeta {
        id: "tnotes::DeleteNote",
        label: "Delete Note",
        description: "Delete the selected note",
        category: "Sidebar",
        default_context: Some("Sidebar"),
        default_key: "delete",
    },
    ActionMeta {
        id: "tnotes::Save",
        label: "Save Note",
        description: "Save changes in the editor",
        category: "Editor",
        default_context: Some("Editor"),
        default_key: "ctrl-s",
    },
    ActionMeta {
        id: "tnotes::SaveAndClose",
        label: "Save and Close",
        description: "Save changes and exit note editor",
        category: "Editor",
        default_context: Some("Editor"),
        default_key: "ctrl-enter",
    },
    ActionMeta {
        id: "tnotes::NavigateBack",
        label: "Go Back",
        description: "Navigate to previous note in history",
        category: "Navigation",
        default_context: None,
        default_key: "alt-left",
    },
    ActionMeta {
        id: "tnotes::NavigateForward",
        label: "Go Forward",
        description: "Navigate to next note in history",
        category: "Navigation",
        default_context: None,
        default_key: "alt-right",
    },
];

pub fn create_binding(id: &str, raw_keystrokes: &str, context: Option<&str>) -> Option<KeyBinding> {
    let normalized = raw_keystrokes.trim().replace('+', "-").to_lowercase();
    match id {
        "tnotes::ToggleFps" => Some(KeyBinding::new(&normalized, ToggleFps, context)),
        "tnotes::ToggleSidebar" => Some(KeyBinding::new(&normalized, ToggleSidebar, context)),
        "tnotes::NewNote" => Some(KeyBinding::new(&normalized, NewNote, context)),
        "tnotes::FocusSearch" => Some(KeyBinding::new(&normalized, FocusSearch, context)),
        "tnotes::OpenSettings" => Some(KeyBinding::new(&normalized, OpenSettings, context)),
        "tnotes::PinNote" => Some(KeyBinding::new(&normalized, PinNote, context)),
        "tnotes::DeleteNote" => Some(KeyBinding::new(&normalized, DeleteNote, context)),
        "tnotes::Save" => Some(KeyBinding::new(&normalized, Save, context)),
        "tnotes::SaveAndClose" => Some(KeyBinding::new(&normalized, SaveAndClose, context)),
        "tnotes::CloseSettings" => Some(KeyBinding::new(&normalized, CloseSettings, context)),
        "tnotes::NavigateBack" => Some(KeyBinding::new(&normalized, NavigateBack, context)),
        "tnotes::NavigateForward" => Some(KeyBinding::new(&normalized, NavigateForward, context)),
        _ => None,
    }
}
