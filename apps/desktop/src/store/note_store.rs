use std::path::Path;

use gpui::*;
use tnotes_core::db::{folders, migrations, notes};
use tnotes_core::db::folders::FolderNode;
use tnotes_core::models::note::Note;
use tnotes_core::models::user::User;
use tnotes_core::{Connection, Result as CoreResult};

pub const LOCAL_USER_ID: &str = "local-user";
pub const LOCAL_DEVICE_ID: &str = "desktop-local";

const MAX_HISTORY_SIZE: usize = 50;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NavigationLocation {
    Note(String),
    Starred,
    Trash,
}

impl NavigationLocation {
    pub fn as_note_id(&self) -> Option<&str> {
        match self {
            Self::Note(id) => Some(id.as_str()),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Note(id) => id.as_str(),
            Self::Starred => "starred",
            Self::Trash => "trash",
        }
    }
}

impl std::ops::Deref for NavigationLocation {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl From<&str> for NavigationLocation {
    fn from(s: &str) -> Self {
        match s {
            "starred" => Self::Starred,
            "trash" => Self::Trash,
            other => Self::Note(other.to_string()),
        }
    }
}

impl From<String> for NavigationLocation {
    fn from(s: String) -> Self {
        match s.as_str() {
            "starred" => Self::Starred,
            "trash" => Self::Trash,
            _ => Self::Note(s),
        }
    }
}

impl From<&NavigationLocation> for NavigationLocation {
    fn from(loc: &NavigationLocation) -> Self {
        loc.clone()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NavigationHistory {
    pub back_stack: Vec<NavigationLocation>,
    pub forward_stack: Vec<NavigationLocation>,
}

impl NavigationHistory {
    pub fn can_go_back(&self) -> bool {
        !self.back_stack.is_empty()
    }

    pub fn can_go_forward(&self) -> bool {
        !self.forward_stack.is_empty()
    }

    pub fn push(
        &mut self,
        current: Option<impl Into<NavigationLocation>>,
        next: impl Into<NavigationLocation>,
    ) {
        let next = next.into();
        if let Some(cur) = current {
            let cur = cur.into();
            if cur != next {
                self.back_stack.push(cur);
                if self.back_stack.len() > MAX_HISTORY_SIZE {
                    let excess = self.back_stack.len() - MAX_HISTORY_SIZE;
                    self.back_stack.drain(..excess);
                }
                self.forward_stack.clear();
            }
        }
    }

    pub fn go_back(
        &mut self,
        current: Option<impl Into<NavigationLocation>>,
    ) -> Option<NavigationLocation> {
        let prev = self.back_stack.pop()?;
        if let Some(cur) = current {
            self.forward_stack.push(cur.into());
            if self.forward_stack.len() > MAX_HISTORY_SIZE {
                let excess = self.forward_stack.len() - MAX_HISTORY_SIZE;
                self.forward_stack.drain(..excess);
            }
        }
        Some(prev)
    }

    pub fn go_forward(
        &mut self,
        current: Option<impl Into<NavigationLocation>>,
    ) -> Option<NavigationLocation> {
        let next = self.forward_stack.pop()?;
        if let Some(cur) = current {
            self.back_stack.push(cur.into());
            if self.back_stack.len() > MAX_HISTORY_SIZE {
                let excess = self.back_stack.len() - MAX_HISTORY_SIZE;
                self.back_stack.drain(..excess);
            }
        }
        Some(next)
    }

    pub fn remove_note(&mut self, note_id: &str) {
        let target = NavigationLocation::Note(note_id.to_string());
        self.back_stack.retain(|id| id != &target);
        self.forward_stack.retain(|id| id != &target);
    }
}

/// Central data entity: owns notes, the folder tree, and navigation state.
/// Views hold an `Entity<NoteStore>` and read/mutate through it.
///
/// When opened via [`NoteStore::open`], every mutation is written through to
/// SQLite; otherwise the store is a plain in-memory container (used by tests
/// and as a fallback when the database cannot be opened).
pub struct NoteStore {
    notes: Vec<Note>,
    folders: Vec<FolderNode>,
    active_location: NavigationLocation,
    history: NavigationHistory,
    conn: Option<Connection>,
    user_id: String,
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
        }
    }

    /// Open (creating if needed) the SQLite database at `db_path` and load
    /// this user's notes and folder tree into memory.
    pub fn open(db_path: &Path, user_id: &str) -> CoreResult<Self> {
        let conn = migrations::open_connection(db_path)?;
        Self::ensure_local_user(&conn, user_id)?;

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
        })
    }

    /// `notes.user_id` / `folders.user_id` are FKs into `users`, so the local
    /// vault user must exist before any note can be persisted.
    fn ensure_local_user(conn: &Connection, user_id: &str) -> CoreResult<()> {
        use tnotes_core::db::users::get_user_by_id;

        if get_user_by_id(conn, user_id)?.is_none() {
            let now = tnotes_core::models::current_time_ms();
            // `users.id` is the FK target for notes/folders; a bare local
            // vault user (no password) is enough until auth lands.
            let user = User {
                id: user_id.to_string(),
                username: "local".to_string(),
                password_hash: String::new(),
                created_at: now,
            };
            if let Err(e) = tnotes_core::db::users::create_user(conn, &user) {
                // Tolerate a concurrent first-boot race; anything else propagates.
                if get_user_by_id(conn, user_id)?.is_none() {
                    return Err(e.into());
                }
            }
        }
        Ok(())
    }

    /// Best-effort write-through: the in-memory state is authoritative for the
    /// UI, a persistence failure must not break interaction (it is logged).
    fn persist_note(&self, note: &Note) {
        if let Some(conn) = self.conn.as_ref()
            && let Err(e) = notes::upsert_note(conn, note)
        {
            eprintln!("tnotes: failed to persist note {}: {e}", note.id);
        }
    }

    fn delete_persisted_note(&self, note_id: &str) {
        if let Some(conn) = self.conn.as_ref()
            && let Err(e) = notes::delete_note_permanently(conn, note_id)
        {
            eprintln!("tnotes: failed to delete note {note_id}: {e}");
        }
    }

    // ---- navigation ----

    pub fn active_location(&self) -> &NavigationLocation {
        &self.active_location
    }

    pub fn selected_note_id(&self) -> Option<String> {
        self.active_location.as_note_id().map(|s| s.to_string())
    }

    pub fn selected_note(&self) -> Option<Note> {
        let id = self.selected_note_id()?;
        self.notes.iter().find(|n| n.id == id).cloned()
    }

    pub fn open_starred(&mut self, cx: &mut Context<Self>) {
        let next = NavigationLocation::Starred;
        if self.active_location == next {
            return;
        }
        self.history.push(Some(&self.active_location), next.clone());
        self.active_location = next;
        cx.notify();
    }

    pub fn open_trash(&mut self, cx: &mut Context<Self>) {
        let next = NavigationLocation::Trash;
        if self.active_location == next {
            return;
        }
        self.history.push(Some(&self.active_location), next.clone());
        self.active_location = next;
        cx.notify();
    }

    pub fn select_note(&mut self, note_id: &str, cx: &mut Context<Self>) {
        let next = NavigationLocation::Note(note_id.to_string());
        if self.active_location == next {
            return;
        }
        self.history.push(Some(&self.active_location), next.clone());
        self.active_location = next;
        cx.notify();
    }

    pub fn navigate_back(&mut self, cx: &mut Context<Self>) -> bool {
        if let Some(prev) = self.history.go_back(Some(&self.active_location)) {
            self.active_location = prev;
            cx.notify();
            true
        } else {
            false
        }
    }

    pub fn navigate_forward(&mut self, cx: &mut Context<Self>) -> bool {
        if let Some(next) = self.history.go_forward(Some(&self.active_location)) {
            self.active_location = next;
            cx.notify();
            true
        } else {
            false
        }
    }

    pub fn can_navigate_back(&self) -> bool {
        self.history.can_go_back()
    }

    pub fn can_navigate_forward(&self) -> bool {
        self.history.can_go_forward()
    }

    // ---- note collections (owned clones: keeps the FTS swap signature-stable) ----

    pub fn active_notes(&self) -> Vec<Note> {
        self.notes.iter().filter(|n| !n.trashed).cloned().collect()
    }

    pub fn starred_notes(&self) -> Vec<Note> {
        self.notes
            .iter()
            .filter(|n| n.pinned && !n.trashed)
            .cloned()
            .collect()
    }

    pub fn trashed_notes(&self) -> Vec<Note> {
        self.notes.iter().filter(|n| n.trashed).cloned().collect()
    }

    /// Back-compat alias used by TrashView.
    pub fn trash_notes(&self) -> Vec<Note> {
        self.trashed_notes()
    }

    /// Full-text search over SQLite FTS5 when a database is open,
    /// in-memory substring filter otherwise (tests, db fallback).
    /// Returned notes are owned clones in both cases.
    pub fn search_notes(&self, query: &str) -> Vec<Note> {
        if query.trim().is_empty() {
            return Vec::new();
        }
        if let Some(conn) = self.conn.as_ref() {
            match notes::search_notes(conn, &self.user_id, query) {
                Ok(found) => return found,
                Err(e) => {
                    eprintln!("tnotes: FTS search failed ({e}); using in-memory filter");
                }
            }
        }
        self.search_in_memory(query)
    }

    fn search_in_memory(&self, query: &str) -> Vec<Note> {
        let query = query.trim().to_lowercase();
        if query.is_empty() {
            return Vec::new();
        }
        self.notes
            .iter()
            .filter(|n| !n.trashed)
            .filter(|n| {
                n.title.to_lowercase().contains(&query)
                    || n.searchable_text.to_lowercase().contains(&query)
                    || self
                        .folder_name(n.folder_id.as_deref())
                        .map(|f| f.to_lowercase().contains(&query))
                        .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    // ---- mutations (written through to SQLite when open) ----

    pub fn create_new_note(&mut self, cx: &mut Context<Self>) {
        self.create_note_in_folder(None, cx);
    }

    pub fn create_note_in_folder(
        &mut self,
        folder_id: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let note = Note::new(
            "Untitled Note",
            "Start typing your note here...",
            "Start typing your note here...",
            folder_id,
            LOCAL_DEVICE_ID,
            self.user_id.clone(),
        );
        let target = note.id.clone();
        self.persist_note(&note);
        self.notes.insert(0, note);
        self.select_note(&target, cx);
    }

    pub fn toggle_note_pin(&mut self, note_id: &str, cx: &mut Context<Self>) {
        if let Some(note) = self.notes.iter_mut().find(|n| n.id == note_id) {
            let pinned = !note.pinned;
            note.set_pinned(pinned, LOCAL_DEVICE_ID);
            let updated = note.clone();
            self.persist_note(&updated);
        }
        cx.notify();
    }

    pub fn duplicate_note(&mut self, note_id: &str, cx: &mut Context<Self>) {
        if let Some(index) = self.notes.iter().position(|n| n.id == note_id) {
            let source = self.notes[index].clone();
            let copy = Note::new(
                format!("{} (copy)", source.title),
                source.body.clone(),
                source.searchable_text.clone(),
                source.folder_id.clone(),
                LOCAL_DEVICE_ID,
                self.user_id.clone(),
            );
            let new_id = copy.id.clone();
            self.persist_note(&copy);
            self.notes.insert(index + 1, copy);
            self.select_note(&new_id, cx);
        }
    }

    pub fn delete_note(&mut self, note_id: &str, cx: &mut Context<Self>) {
        if let Some(note) = self.notes.iter_mut().find(|n| n.id == note_id) {
            note.trash(LOCAL_DEVICE_ID);
            let updated = note.clone();
            self.persist_note(&updated);
        }
        self.history.remove_note(note_id);
        if self.active_location == NavigationLocation::Note(note_id.to_string()) {
            if let Some(prev) = self.history.back_stack.pop() {
                self.active_location = prev;
            } else if let Some(first) = self.notes.iter().find(|n| !n.trashed) {
                self.active_location = NavigationLocation::Note(first.id.clone());
            } else {
                self.active_location = NavigationLocation::Starred;
            }
        }
        cx.notify();
    }

    pub fn restore_note(&mut self, note_id: &str, cx: &mut Context<Self>) {
        if let Some(note) = self.notes.iter_mut().find(|n| n.id == note_id) {
            note.restore(LOCAL_DEVICE_ID);
            let updated = note.clone();
            self.persist_note(&updated);
            cx.notify();
        }
    }

    pub fn permanently_delete_note(&mut self, note_id: &str, cx: &mut Context<Self>) {
        self.notes.retain(|n| n.id != note_id);
        self.history.remove_note(note_id);
        self.delete_persisted_note(note_id);
        cx.notify();
    }

    pub fn empty_trash(&mut self, cx: &mut Context<Self>) {
        let trashed: Vec<String> = self
            .notes
            .iter()
            .filter(|n| n.trashed)
            .map(|n| n.id.clone())
            .collect();
        self.notes.retain(|n| !n.trashed);
        for id in &trashed {
            self.delete_persisted_note(id);
            self.history.remove_note(id);
        }
        cx.notify();
    }

    // ---- folders ----

    pub fn folder_tree(&self) -> &[FolderNode] {
        &self.folders
    }

    pub fn folder_name(&self, folder_id: Option<&str>) -> Option<&str> {
        let id = folder_id?;
        self.folders
            .iter()
            .find(|n| n.folder.id == id)
            .map(|n| n.folder.name.as_str())
    }

    fn is_descendant(&self, folder_id: &str, ancestor_id: &str) -> bool {
        if folder_id == ancestor_id {
            return true;
        }
        let mut current = self
            .folders
            .iter()
            .find(|n| n.folder.id == folder_id)
            .and_then(|n| n.folder.parent_id.clone());
        while let Some(id) = current {
            if id == ancestor_id {
                return true;
            }
            current = self
                .folders
                .iter()
                .find(|n| n.folder.id == id)
                .and_then(|n| n.folder.parent_id.clone());
        }
        false
    }

    pub fn notes_in_folder(&self, folder_id: &str) -> Vec<Note> {
        self.notes
            .iter()
            .filter(|n| !n.trashed)
            .filter(|n| n.folder_id.as_deref() == Some(folder_id))
            .cloned()
            .collect()
    }

    pub fn notes_without_folder(&self) -> Vec<Note> {
        self.notes
            .iter()
            .filter(|n| !n.trashed && n.folder_id.is_none())
            .cloned()
            .collect()
    }

    /// Subtree count (matches the old hardcoded badge totals).
    pub fn note_count_in_folder(&self, folder_id: &str) -> usize {
        self.notes
            .iter()
            .filter(|n| !n.trashed)
            .filter(|n| match n.folder_id.as_deref() {
                Some(fid) => self.is_descendant(fid, folder_id),
                None => false,
            })
            .count()
    }
}

impl Default for NoteStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn history_caps_at_50_entries() {
        let mut history = NavigationHistory::default();
        for i in 0..60 {
            history.push(
                Some(format!("note-{i}")),
                NavigationLocation::Note(format!("note-{}", i + 1)),
            );
        }
        assert_eq!(history.back_stack.len(), 50);
    }

    #[test]
    fn navigation_history_stack_operations() {
        let mut history = NavigationHistory::default();
        assert!(!history.can_go_back());
        assert!(!history.can_go_forward());

        history.push(Some("note-a"), "note-b");
        history.push(Some("note-b"), "note-c");
        assert!(history.can_go_back());
        assert!(!history.can_go_forward());

        history.push(Some("note-c"), "note-c");
        assert_eq!(history.back_stack.len(), 2);

        let prev = history.go_back(Some("note-c"));
        assert_eq!(prev.as_deref(), Some("note-b"));
        assert!(history.can_go_back());
        assert!(history.can_go_forward());

        let prev = history.go_back(Some("note-b"));
        assert_eq!(prev.as_deref(), Some("note-a"));
        assert!(!history.can_go_back());
        assert!(history.can_go_forward());

        let next = history.go_forward(Some("note-a"));
        assert_eq!(next.as_deref(), Some("note-b"));
        assert!(history.can_go_back());
        assert!(history.can_go_forward());

        history.push(Some("note-b"), "note-d");
        assert!(history.can_go_back());
        assert!(!history.can_go_forward());

        history.remove_note("note-a");
        assert_eq!(
            history.back_stack,
            vec![NavigationLocation::Note("note-b".to_string())]
        );
    }

    #[test]
    fn trash_is_a_filter_not_a_second_collection() {
        let mut store = NoteStore::new();
        assert!(store.active_notes().is_empty());
        assert!(store.trashed_notes().is_empty());

        let now = tnotes_core::models::current_time_ms();
        let mut active = Note::new(
            "Active",
            "body",
            "body",
            None,
            LOCAL_DEVICE_ID,
            LOCAL_USER_ID,
        );
        active.updated_at = now;
        let mut trashed = Note::new(
            "Trashed",
            "body",
            "body",
            None,
            LOCAL_DEVICE_ID,
            LOCAL_USER_ID,
        );
        trashed.trash(LOCAL_DEVICE_ID);
        store.notes.push(active);
        store.notes.push(trashed);

        assert_eq!(store.active_notes().len(), 1);
        assert_eq!(store.trashed_notes().len(), 1);
    }

    #[test]
    fn sqlite_roundtrip_persists_notes() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!("tnotes-test-{nanos}.db"));

        let note_id = {
            let mut store = NoteStore::open(&path, LOCAL_USER_ID).unwrap();
            assert!(store.active_notes().is_empty());
            let note = Note::new(
                "Persistent Note",
                "sqlite body content",
                "sqlite body content",
                None,
                LOCAL_DEVICE_ID,
                LOCAL_USER_ID,
            );
            let id = note.id.clone();
            store.notes.push(note.clone());
            store.persist_note(&note);
            id
        };

        let reopened = NoteStore::open(&path, LOCAL_USER_ID).unwrap();
        let found: Vec<Note> = reopened
            .active_notes()
            .into_iter()
            .filter(|n| n.id == note_id)
            .collect();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].title, "Persistent Note");
        // Exercises the FTS5 branch of search_notes.
        assert!(reopened.search_notes("sqlite body").iter().any(|n| n.id == note_id));
        assert!(reopened.search_notes("no-such-term-xyz").is_empty());

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("db-wal"));
        let _ = std::fs::remove_file(path.with_extension("db-shm"));
    }
    /// Deterministic fixture for view tests: one folder with three notes
    /// (two pinned, one plain). Not part of the production API.
    #[cfg(test)]
    impl NoteStore {
        pub fn seed_test_data(&mut self) {
            use tnotes_core::models::folder::Folder;

            let now = tnotes_core::models::current_time_ms();
            let mk = |id: &str, title: &str, folder: Option<&str>, pinned: bool| {
                let mut note = Note::new(
                    title,
                    title,
                    title,
                    folder.map(|s| s.to_string()),
                    LOCAL_DEVICE_ID,
                    LOCAL_USER_ID,
                );
                note.id = id.to_string();
                note.pinned = pinned;
                note.updated_at = now;
                note.created_at = now;
                note
            };

            self.folders.push(FolderNode {
                folder: Folder {
                    id: "projects".to_string(),
                    user_id: self.user_id.clone(),
                    parent_id: None,
                    name: "Projects".to_string(),
                    icon: "📁".to_string(),
                    sort_order: 0,
                    version: 1,
                    updated_at: now,
                    created_at: now,
                    deleted_at: None,
                    device_id: LOCAL_DEVICE_ID.to_string(),
                },
                depth: 0,
                path: "/Projects".to_string(),
            });
            self.notes.push(mk(
                "note-arch-spec",
                "System Architecture Spec",
                Some("projects"),
                true,
            ));
            self.notes.push(mk(
                "note-desktop-gpui",
                "Desktop Shell & GPUI Architecture",
                Some("projects"),
                true,
            ));
            self.notes.push(mk(
                "note-db-schema",
                "Database Schema & Index Design",
                Some("projects"),
                false,
            ));
            self.active_location = NavigationLocation::Note("note-arch-spec".to_string());
        }
    }
}
