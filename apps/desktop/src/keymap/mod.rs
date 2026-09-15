pub mod actions;
pub mod config;

pub use actions::{
    create_binding, ActionMeta, DeleteNote, FocusSearch, NewNote, OpenSettings, PinNote, Save,
    SaveAndClose, ToggleFps, ToggleSidebar, ALL_ACTIONS,
};
pub use config::{KeymapConfig, KeymapSection};
