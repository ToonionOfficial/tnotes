use std::path::Path;

#[cfg(test)]
use gpui::*;
use tnotes_core::db::folders::FolderNode;
use tnotes_core::db::{folders, migrations, notes};
use tnotes_core::models::note::Note;
use tnotes_core::{Connection, Result as CoreResult};

pub const LOCAL_USER_ID: &str = "local-user";
pub const LOCAL_USERNAME: &str = "local";

use super::navigation::{NavigationHistory, NavigationLocation};

/// Shared test helper: wraps a fresh in-memory store in an entity.
#[cfg(test)]
pub(crate) fn test_store(cx: &mut TestAppContext) -> Entity<NoteStore> {
    cx.update(|cx| cx.new(|_| NoteStore::new()))
}

/// Central data entity: owns notes, the folder tree, and navigation state.
/// Views hold an `Entity<NoteStore>` and read/mutate through it.
///
/// When opened via [`NoteStore::open`], every mutation is written through to
/// SQLite; otherwise the store is a plain in-memory container (used by tests
/// and as a fallback when the database cannot be opened).
pub struct NoteStore {
    pub(crate) notes: Vec<Note>,
    pub(crate) folders: Vec<FolderNode>,
    pub(crate) active_location: NavigationLocation,
    pub(crate) history: NavigationHistory,
    pub(crate) conn: Option<Connection>,
    pub(crate) user_id: String,
    /// Stable per-install id stamping every local write (ULID, like mobile).
    /// Persisted in `sync_meta` for database stores; ephemeral for `new()`.
    pub(crate) device_id: String,
}

impl NoteStore {
    /// Empty in-memory store. The default location is Starred so first boot
    /// shows a valid empty state instead of a dangling note reference.
    pub fn new() -> Self {
        Self {
            notes: Vec::new(),
            folders: Vec::new(),
            active_location: NavigationLocation::Starred,
            history: NavigationHistory::default(),
            conn: None,
            user_id: LOCAL_USER_ID.to_string(),
            device_id: tnotes_core::Ulid::generate().to_string(),
        }
    }

    /// Open (creating if needed) the SQLite database at `db_path` and load
    /// this user's notes and folder tree into memory.
    pub fn open(db_path: &Path, user_id: &str) -> CoreResult<Self> {
        let mut conn = migrations::open_connection(db_path)?;
        Self::ensure_local_user(&mut conn, user_id)?;
        let device_id = Self::device_id_for_connection(&conn)?;

        let mut stored_notes = notes::list_active_notes(&conn, user_id)?;
        stored_notes.extend(notes::list_trashed_notes(&conn, user_id)?);
        let stored_folders = folders::get_folder_tree(&conn, user_id)?;

        // Default to the most recently updated active note, if any.
        let active_location = stored_notes
            .iter()
            .filter(|n| !n.trashed)
            .max_by_key(|n| n.updated_at)
            .map(|n| NavigationLocation::Note(n.id.clone()))
            .unwrap_or(NavigationLocation::Starred);

        Ok(Self {
            notes: stored_notes,
            folders: stored_folders,
            active_location,
            history: NavigationHistory::default(),
            conn: Some(conn),
            user_id: user_id.to_string(),
            device_id,
        })
    }

    /// The vault owner's id (owning user of the loaded notes/folders).
    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    pub fn active_location(&self) -> &NavigationLocation {
        &self.active_location
    }
}

impl Default for NoteStore {
    fn default() -> Self {
        Self::new()
    }
}
