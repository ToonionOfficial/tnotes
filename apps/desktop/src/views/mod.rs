pub mod editor;
pub mod modals;
pub mod settings;
pub mod sidebar;
pub mod workspace;

pub use workspace::{
    CloseModal, CreateNewNote, TNotesWorkspace, ToggleCommandPalette, ToggleSettings,
};
