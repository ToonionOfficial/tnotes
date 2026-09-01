pub mod editor;
pub mod folder_dashboard;
pub mod modals;
pub mod settings;
pub mod sidebar;
pub mod workspace;

#[allow(unused_imports)]
pub use folder_dashboard::FolderDashboardView;
pub use workspace::{
    CloseModal, CreateNewNote, TNotesWorkspace, ToggleCommandPalette, ToggleFps, ToggleSettings,
};
