pub mod actions;
pub mod config;

pub use actions::{
    create_binding, ActionMeta, CloseSettings, DeleteNote, FocusSearch, NavigateBack,
    NavigateForward, NewNote, OpenSettings, PinNote, Save, SaveAndClose, ToggleFps, ToggleSidebar,
    ALL_ACTIONS,
};
pub use config::{KeymapConfig, KeymapSection};
