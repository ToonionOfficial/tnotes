pub mod editor;
pub mod modals;
pub mod sidebar;
pub mod workspace;

pub use workspace::{
    CloseModal, CreateNewNote, TNotesWorkspace, ToggleCommandPalette, ToggleSettingsModal,
};
