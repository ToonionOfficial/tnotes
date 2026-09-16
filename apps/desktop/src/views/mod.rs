pub mod login;
pub mod note_editor;
pub mod note_view;
pub mod settings;
pub mod sidebar;
pub mod starred;
pub mod trash;

#[allow(unused_imports)]
pub use note_view::NoteView;
#[allow(unused_imports)]
pub use settings::SettingsView;
#[allow(unused_imports)]
pub use sidebar::SidebarView;
#[allow(unused_imports)]
pub use starred::StarredView;
#[allow(unused_imports)]
pub use trash::TrashView;
