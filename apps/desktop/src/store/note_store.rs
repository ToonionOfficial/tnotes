use std::path::Path;

use gpui::*;
use tnotes_core::db::{folders, migrations, notes};
use tnotes_core::db::folders::FolderNode;
use tnotes_core::models::folder::Folder;
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

/// Read-only sync status snapshot for the Sync settings section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncSummary {
    /// Epoch-ms of the last successful sync, or `None` if never synced.
    pub last_sync_at: Option<i64>,
    /// Paired server URL, or `None` when this vault has never been paired.
    pub server_url: Option<String>,
}

impl SyncSummary {
    pub fn is_paired(&self) -> bool {
        self.server_url.as_deref().is_some_and(|u| !u.is_empty())
    }
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

    fn persist_folder(&self, folder: &Folder) {
        if let Some(conn) = self.conn.as_ref()
            && let Err(e) = folders::upsert_folder(conn, folder)
        {
            eprintln!("tnotes: failed to persist folder {}: {e}", folder.id);
        }
    }

    /// Reload the in-memory tree from SQLite (authoritative ordering).
    /// No-op without a database connection.
    fn reload_folders(&mut self) {
        let user_id = self.user_id.clone();
        if let Some(conn) = self.conn.as_ref() {
            match folders::get_folder_tree(conn, &user_id) {
                Ok(tree) => self.folders = tree,
                Err(e) => eprintln!("tnotes: failed to reload folder tree: {e}"),
            }
        }
    }

    /// Read-only sync status for the Sync settings section. `last_sync_at`
    /// comes from `sync_meta` when a database is open (0/None = never synced);
    /// `paired` is true once a server URL has been stored there.
    pub fn sync_summary(&self) -> SyncSummary {
        let (last_sync_at, server_url) = match self.conn.as_ref() {
            Some(conn) => {
                let last = tnotes_core::db::sync::get_last_sync_at(conn).unwrap_or(0);
                let url = tnotes_core::db::sync::get_sync_meta(conn, "server_url")
                    .unwrap_or(None);
                (last, url)
            }
            None => (0, None),
        };
        SyncSummary {
            last_sync_at: if last_sync_at > 0 {
                Some(last_sync_at)
            } else {
                None
            },
            server_url,
        }
    }

    // ---- navigation ----

    /// The vault owner's id (owning user of the loaded notes/folders).
    pub fn user_id(&self) -> &str {
        &self.user_id
    }

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

    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    pub fn active_note_count(&self) -> usize {
        self.notes.iter().filter(|n| !n.trashed).count()
    }

    pub fn trashed_note_count(&self) -> usize {
        self.notes.iter().filter(|n| n.trashed).count()
    }

    pub fn starred_note_count(&self) -> usize {
        self.notes.iter().filter(|n| n.pinned && !n.trashed).count()
    }

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
        // Coerce unknown folders to root: `notes.folder_id` is a real FK, so
        // persisting a dangling reference would fail while the in-memory note
        // survived — a divergence we never want.
        let folder_id = folder_id.filter(|id| self.folder_exists(id));
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

    pub fn rename_note(&mut self, note_id: &str, title: &str, cx: &mut Context<Self>) {
        let title = title.trim();
        if title.is_empty() {
            return;
        }
        if let Some(note) = self.notes.iter_mut().find(|n| n.id == note_id)
            && note.title != title
        {
            let (body, searchable, folder) = (
                note.body.clone(),
                note.searchable_text.clone(),
                note.folder_id.clone(),
            );
            note.update(title, body, searchable, folder, LOCAL_DEVICE_ID);
            let updated = note.clone();
            self.persist_note(&updated);
        }
        cx.notify();
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
        // Mobile orphan rule: a restored note whose folder is gone (deleted
        // or never existed) moves to root instead of dangling.
        let orphaned = self
            .notes
            .iter()
            .find(|n| n.id == note_id)
            .and_then(|n| n.folder_id.clone())
            .is_some_and(|fid| !self.folder_exists(&fid));
        if let Some(note) = self.notes.iter_mut().find(|n| n.id == note_id) {
            if orphaned {
                note.folder_id = None;
            }
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

    pub fn vacuum(&self) -> CoreResult<()> {
        if let Some(conn) = self.conn.as_ref() {
            conn.execute_batch("VACUUM;")?;
        }
        Ok(())
    }

    pub fn create_benchmark_notes(&mut self, count: usize, cx: &mut Context<Self>) -> (usize, usize) {
        let folder_count = (count + 9) / 10;
        let mut folder_ids = Vec::with_capacity(folder_count);
        let mut new_folders = Vec::with_capacity(folder_count);

        for i in 0..folder_count {
            let folder_name = format!("__tnotes_benchmark_folder_v1__:Benchmark Folder {}", i + 1);
            let folder = Folder::new(
                &folder_name,
                None,
                None,
                i as i32,
                LOCAL_DEVICE_ID,
                self.user_id.clone(),
            );
            folder_ids.push(folder.id.clone());
            new_folders.push(folder);
        }

        let mut new_notes = Vec::with_capacity(count);
        for i in 0..count {
            let target_folder = if !folder_ids.is_empty() {
                Some(folder_ids[i % folder_ids.len()].clone())
            } else {
                None
            };
            let title = format!("Benchmark Note {}", i + 1);
            let body = format!("__tnotes_benchmark_note_v1__:This is benchmark note number {} generated for performance testing.", i + 1);
            let note = Note::new(
                &title,
                &body,
                &body,
                target_folder,
                LOCAL_DEVICE_ID,
                self.user_id.clone(),
            );
            new_notes.push(note);
        }

        if let Some(conn) = self.conn.as_ref() {
            let _ = conn.execute_batch("BEGIN TRANSACTION;");
            for folder in &new_folders {
                let _ = folders::upsert_folder(conn, folder);
            }
            for note in &new_notes {
                let _ = notes::upsert_note(conn, note);
            }
            let _ = conn.execute_batch("COMMIT;");
            self.reload_folders();
        } else {
            for folder in new_folders {
                let node = FolderNode {
                    folder: folder.clone(),
                    depth: 0,
                    path: folder.name.clone(),
                };
                self.folders.push(node);
            }
        }

        new_notes.extend(std::mem::take(&mut self.notes));
        self.notes = new_notes;

        cx.notify();
        (count, folder_count)
    }

    pub fn delete_benchmark_notes(&mut self, cx: &mut Context<Self>) -> usize {
        let benchmark_note_ids: std::collections::HashSet<String> = self
            .notes
            .iter()
            .filter(|n| n.searchable_text.starts_with("__tnotes_benchmark_note_v1__:") || n.title.starts_with("Benchmark Note"))
            .map(|n| n.id.clone())
            .collect();
        let count = benchmark_note_ids.len();

        let benchmark_folder_ids: std::collections::HashSet<String> = self
            .folders
            .iter()
            .filter(|f| f.folder.name.starts_with("__tnotes_benchmark_folder_v1__:") || f.folder.name.starts_with("Benchmark Folder"))
            .map(|f| f.folder.id.clone())
            .collect();

        if let Some(conn) = self.conn.as_ref() {
            let _ = conn.execute_batch(
                "BEGIN TRANSACTION;
                 DELETE FROM notes WHERE searchable_text LIKE '__tnotes_benchmark_note_v1__:%' OR title LIKE 'Benchmark Note %';
                 DELETE FROM folders WHERE name LIKE '__tnotes_benchmark_folder_v1__:%' OR name LIKE 'Benchmark Folder %';
                 COMMIT;",
            );
            self.reload_folders();
        } else {
            self.folders.retain(|f| !benchmark_folder_ids.contains(&f.folder.id));
        }

        self.notes.retain(|n| !benchmark_note_ids.contains(&n.id));
        for id in &benchmark_note_ids {
            self.history.remove_note(id);
        }

        cx.notify();
        count
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn notes_in_folder(&self, folder_id: &str) -> Vec<Note> {
        self.notes
            .iter()
            .filter(|n| !n.trashed)
            .filter(|n| n.folder_id.as_deref() == Some(folder_id))
            .cloned()
            .collect()
    }

    #[allow(dead_code)]
    pub fn notes_without_folder(&self) -> Vec<Note> {
        self.notes
            .iter()
            .filter(|n| !n.trashed && n.folder_id.is_none())
            .cloned()
            .collect()
    }

    #[allow(dead_code)]
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

    // ---- folder mutations (mirror mobile createFolder/deleteFolder) ----

    pub fn folder_exists(&self, folder_id: &str) -> bool {
        self.folders.iter().any(|n| n.folder.id == folder_id)
    }

    /// IDs of `folder_id` plus all its descendants (BFS over parent links).
    /// Used by delete (cascade set) and by the sidebar (expansion cleanup).
    pub fn folder_subtree_ids(&self, folder_id: &str) -> Vec<String> {
        let mut ids = vec![folder_id.to_string()];
        let mut queue = vec![folder_id.to_string()];
        while let Some(parent) = queue.pop() {
            for node in self
                .folders
                .iter()
                .filter(|n| n.folder.parent_id.as_deref() == Some(parent.as_str()))
            {
                ids.push(node.folder.id.clone());
                queue.push(node.folder.id.clone());
            }
        }
        ids
    }

    /// Create a folder; unknown parents coerce to top level. Returns the new id.
    /// New folders prepend among siblings (`MIN(sort_order) - 1`), like mobile.
    pub fn create_folder(
        &mut self,
        name: &str,
        parent_id: Option<String>,
        cx: &mut Context<Self>,
    ) -> String {
        let parent_id = parent_id.filter(|id| self.folder_exists(id));
        let min_order = self
            .folders
            .iter()
            .filter(|n| n.folder.parent_id == parent_id)
            .map(|n| n.folder.sort_order)
            .min()
            .unwrap_or(1);
        let mut folder = Folder::new(
            name,
            None,
            parent_id.clone(),
            min_order - 1,
            LOCAL_DEVICE_ID,
            self.user_id.clone(),
        );
        let id = folder.id.clone();
        // Keep version/timestamps coherent if this id ever collides on upsert.
        folder.version = 1;
        self.persist_folder(&folder);

        if self.conn.is_some() {
            self.reload_folders();
        } else {
            // Ordered insert: directly after the parent's existing subtree so
            // the depth-ordered tree render stays correct without a db.
            let (depth, path) = match parent_id.as_deref().and_then(|pid| {
                self.folders.iter().find(|n| n.folder.id == pid)
            }) {
                Some(parent) => (
                    parent.depth + 1,
                    format!("{} / {}", parent.path, folder.name),
                ),
                None => (0, folder.name.clone()),
            };
            let node = FolderNode {
                folder,
                depth,
                path,
            };
            let insert_at = match &parent_id {
                Some(pid) => {
                    let parent_idx = self
                        .folders
                        .iter()
                        .position(|n| n.folder.id == *pid)
                        .unwrap_or(self.folders.len());
                    let parent_depth = self.folders[parent_idx].depth;
                    self.folders
                        .iter()
                        .skip(parent_idx + 1)
                        .take_while(|n| n.depth > parent_depth)
                        .count()
                        + parent_idx
                        + 1
                }
                None => self.folders.len(),
            };
            self.folders.insert(insert_at.min(self.folders.len()), node);
        }
        cx.notify();
        id
    }

    pub fn create_top_level_folder(&mut self, cx: &mut Context<Self>) -> String {
        self.create_folder("Untitled Folder", None, cx)
    }

    /// Soft-delete the folder subtree and trash every contained note, mirroring
    /// mobile `deleteFolder`. Notes keep their `folder_id` so they show their
    /// origin in Trash; restoring an orphaned note moves it to root.
    pub fn delete_folder(&mut self, folder_id: &str, cx: &mut Context<Self>) {
        if !self.folder_exists(folder_id) {
            return;
        }
        let subtree = self.folder_subtree_ids(folder_id);

        for fid in &subtree {
            if let Some(node) = self.folders.iter_mut().find(|n| n.folder.id == *fid) {
                node.folder.soft_delete(LOCAL_DEVICE_ID);
                let updated = node.folder.clone();
                self.persist_folder(&updated);
            }
        }
        self.folders.retain(|n| !subtree.contains(&n.folder.id));

        // Collect ids first so no note borrow is held across persistence.
        let to_trash: Vec<String> = self
            .notes
            .iter()
            .filter(|n| {
                !n.trashed
                    && n.folder_id
                        .as_deref()
                        .is_some_and(|fid| subtree.iter().any(|s| s == fid))
            })
            .map(|n| n.id.clone())
            .collect();
        let mut trashed_selection = false;
        for id in &to_trash {
            if let Some(note) = self.notes.iter_mut().find(|n| &n.id == id) {
                note.trash(LOCAL_DEVICE_ID);
                let updated = note.clone();
                self.persist_note(&updated);
            }
            if self.active_location == NavigationLocation::Note(id.clone()) {
                trashed_selection = true;
            }
            self.history.remove_note(id);
        }

        if self.conn.is_some() {
            self.reload_folders();
        }
        if trashed_selection {
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

    /// Rename a folder; blank names are ignored. Unknown ids are no-ops.
    pub fn rename_folder(&mut self, folder_id: &str, name: &str, cx: &mut Context<Self>) {
        let name = name.trim();
        if name.is_empty() {
            return;
        }
        if let Some(node) = self.folders.iter_mut().find(|n| n.folder.id == folder_id)
            && node.folder.name != name
        {
            node.folder.rename(name, LOCAL_DEVICE_ID);
            let updated = node.folder.clone();
            self.persist_folder(&updated);
        }
        cx.notify();
    }

    /// Set the folder icon (a lowercase name like `"briefcase"`; see the
    /// sidebar's icon table). Blank values are ignored; rendering falls back
    /// to the default folder icon for unknown names.
    pub fn set_folder_icon(&mut self, folder_id: &str, icon: &str, cx: &mut Context<Self>) {
        let icon = icon.trim();
        if icon.is_empty() {
            return;
        }
        if let Some(node) = self.folders.iter_mut().find(|n| n.folder.id == folder_id)
            && node.folder.icon != icon
        {
            node.folder.set_icon(icon, LOCAL_DEVICE_ID);
            let updated = node.folder.clone();
            self.persist_folder(&updated);
        }
        cx.notify();
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
    use gpui::{AppContext, TestAppContext};

    fn test_store(cx: &mut TestAppContext) -> Entity<NoteStore> {
        cx.update(|cx| cx.new(|_| NoteStore::new()))
    }

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
    fn sync_summary_defaults_to_unpaired_never_synced() {
        let store = NoteStore::new();
        let summary = store.sync_summary();
        assert_eq!(summary.last_sync_at, None);
        assert_eq!(summary.server_url, None);
        assert!(!summary.is_paired());
    }

    #[test]
    fn sync_summary_reads_sync_meta_from_sqlite() {
        use tnotes_core::db::sync::{get_last_sync_at, set_last_sync_at, set_sync_meta};

        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!("tnotes-sync-test-{nanos}.db"));

        let store = NoteStore::open(&path, LOCAL_USER_ID).unwrap();
        assert!(!store.sync_summary().is_paired());
        assert_eq!(store.sync_summary().last_sync_at, None);

        // Write sync_meta directly (no sync client exists yet on desktop).
        if let Some(conn) = store.conn.as_ref() {
            set_sync_meta(conn, "server_url", "https://sync.example.com").unwrap();
            set_last_sync_at(conn, 1_700_000_000_000).unwrap();
            assert_eq!(get_last_sync_at(conn).unwrap(), 1_700_000_000_000);
        } else {
            panic!("expected an open database connection");
        }
        let summary = store.sync_summary();
        assert!(summary.is_paired());
        assert_eq!(
            summary.server_url.as_deref(),
            Some("https://sync.example.com")
        );
        assert_eq!(summary.last_sync_at, Some(1_700_000_000_000));

        drop(store);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("db-wal"));
        let _ = std::fs::remove_file(path.with_extension("db-shm"));
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

    #[test]
    fn sqlite_persists_folders_and_folder_notes() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!("tnotes-folders-test-{nanos}.db"));

        let cx = TestAppContext::single();
        let store = cx.update(|cx| {
            cx.new(|_| NoteStore::open(&path, LOCAL_USER_ID).unwrap())
        });
        let (fid, nid) = cx.update(|cx| {
            store.update(cx, |s, cx| {
                let fid = s.create_folder("Work", None, cx);
                s.create_note_in_folder(Some(fid.clone()), cx);
                let nid = s.selected_note_id().unwrap();
                (fid, nid)
            })
        });

        let reopened = NoteStore::open(&path, LOCAL_USER_ID).unwrap();
        assert!(reopened.folder_tree().iter().any(|n| n.folder.id == fid));
        assert!(
            reopened
                .notes_in_folder(&fid)
                .iter()
                .any(|n| n.id == nid)
        );

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("db-wal"));
        let _ = std::fs::remove_file(path.with_extension("db-shm"));
    }

    #[test]
    fn create_folder_prepends_among_siblings() {
        let mut cx = TestAppContext::single();
        let store = test_store(&mut cx);
        let (b, a) = cx.update(|cx| {
            store.update(cx, |s, cx| {
                let b = s.create_folder("B", None, cx);
                let a = s.create_folder("A", None, cx);
                (b, a)
            })
        });
        cx.update(|cx| {
            let s = store.read(cx);
            let order = |id: &str| {
                s.folder_tree()
                    .iter()
                    .find(|n| n.folder.id == id)
                    .map(|n| n.folder.sort_order)
                    .unwrap()
            };
            // New folders prepend: A created after B sorts first.
            assert!(order(&a) < order(&b));
            assert!(
                s.folder_tree()
                    .iter()
                    .all(|n| n.folder.parent_id.is_none())
            );
        });
    }

    #[test]
    fn create_subfolder_links_parent() {
        let mut cx = TestAppContext::single();
        let store = test_store(&mut cx);
        let (parent, child) = cx.update(|cx| {
            store.update(cx, |s, cx| {
                let parent = s.create_folder("Parent", None, cx);
                let child = s.create_folder("Child", Some(parent.clone()), cx);
                (parent, child)
            })
        });
        cx.update(|cx| {
            let s = store.read(cx);
            let node = s
                .folder_tree()
                .iter()
                .find(|n| n.folder.id == child)
                .unwrap();
            assert_eq!(node.folder.parent_id.as_deref(), Some(parent.as_str()));
            assert_eq!(node.depth, 1);
        });
    }

    #[test]
    fn create_note_in_unknown_folder_falls_back_to_root() {
        let mut cx = TestAppContext::single();
        let store = test_store(&mut cx);
        cx.update(|cx| {
            store.update(cx, |s, cx| {
                s.create_note_in_folder(Some("missing-folder".to_string()), cx);
            });
        });
        cx.update(|cx| {
            let s = store.read(cx);
            let note = s.selected_note().unwrap();
            assert_eq!(note.folder_id, None);
        });
    }

    #[test]
    fn delete_folder_trashes_subtree_notes_and_hides_tree() {
        let mut cx = TestAppContext::single();
        let store = test_store(&mut cx);
        cx.update(|cx| {
            store.update(cx, |s, cx| {
                let parent = s.create_folder("Parent", None, cx);
                let child = s.create_folder("Child", Some(parent.clone()), cx);
                s.create_note_in_folder(Some(parent.clone()), cx);
                let n1 = s.selected_note_id().unwrap();
                s.create_note_in_folder(Some(child.clone()), cx);
                s.create_note_in_folder(None, cx);
                // Select a note that is about to be trashed: selection must
                // fall back to the surviving root note.
                s.select_note(&n1, cx);
                let doomed = s.folder_tree()[0].folder.id.clone();
                s.delete_folder(&doomed, cx);
            });
        });
        cx.update(|cx| {
            let s = store.read(cx);
            assert!(s.folder_tree().is_empty());
            assert_eq!(s.trashed_notes().len(), 2);
            assert_eq!(s.active_notes().len(), 1);
            assert!(s.active_notes()[0].folder_id.is_none());
            // Selection fell back to the surviving root note.
            assert_eq!(
                s.selected_note_id(),
                Some(s.active_notes()[0].id.clone())
            );
        });
    }

    #[test]
    fn restore_note_orphans_to_root_when_folder_deleted() {
        let mut cx = TestAppContext::single();
        let store = test_store(&mut cx);
        let nid = cx.update(|cx| {
            store.update(cx, |s, cx| {
                let fid = s.create_folder("Doomed", None, cx);
                s.create_note_in_folder(Some(fid.clone()), cx);
                let nid = s.selected_note_id().unwrap();
                s.delete_folder(&fid, cx);
                nid
            })
        });
        cx.update(|cx| {
            let s = store.read(cx);
            assert!(s.trashed_notes().iter().any(|n| n.id == nid));
        });
        cx.update(|cx| {
            store.update(cx, |s, cx| s.restore_note(&nid, cx));
        });
        cx.update(|cx| {
            let s = store.read(cx);
            let note = s.active_notes().into_iter().find(|n| n.id == nid).unwrap();
            assert_eq!(note.folder_id, None);
        });
    }
    #[test]
    fn rename_folder_trims_and_ignores_blank() {
        let mut cx = TestAppContext::single();
        let store = test_store(&mut cx);
        let fid = cx.update(|cx| {
            store.update(cx, |s, cx| s.create_folder("Work", None, cx))
        });
        cx.update(|cx| {
            store.update(cx, |s, cx| s.rename_folder(&fid, "  Play  ", cx));
        });
        cx.update(|cx| {
            let s = store.read(cx);
            let node = s.folder_tree().iter().find(|n| n.folder.id == fid).unwrap();
            assert_eq!(node.folder.name, "Play");
        });
        cx.update(|cx| {
            store.update(cx, |s, cx| s.rename_folder(&fid, "   ", cx));
        });
        cx.update(|cx| {
            let s = store.read(cx);
            let node = s.folder_tree().iter().find(|n| n.folder.id == fid).unwrap();
            assert_eq!(node.folder.name, "Play");
        });
    }

    #[test]
    fn rename_note_keeps_body_and_folder() {
        let mut cx = TestAppContext::single();
        let store = test_store(&mut cx);
        let nid = cx.update(|cx| {
            store.update(cx, |s, cx| {
                let fid = s.create_folder("F", None, cx);
                s.create_note_in_folder(Some(fid), cx);
                s.selected_note_id().unwrap()
            })
        });
        cx.update(|cx| {
            store.update(cx, |s, cx| s.rename_note(&nid, "  New Title ", cx));
        });
        cx.update(|cx| {
            let s = store.read(cx);
            let note = s.active_notes().into_iter().find(|n| n.id == nid).unwrap();
            assert_eq!(note.title, "New Title");
            assert_eq!(note.body, "Start typing your note here...");
            assert!(note.folder_id.is_some());
        });
        cx.update(|cx| {
            store.update(cx, |s, cx| s.rename_note(&nid, "  ", cx));
        });
        cx.update(|cx| {
            let s = store.read(cx);
            let note = s.active_notes().into_iter().find(|n| n.id == nid).unwrap();
            assert_eq!(note.title, "New Title");
        });
    }

    #[test]
    fn set_folder_icon_stores_lowercase_name() {
        let mut cx = TestAppContext::single();
        let store = test_store(&mut cx);
        let fid = cx.update(|cx| {
            store.update(cx, |s, cx| s.create_folder("Work", None, cx))
        });
        cx.update(|cx| {
            store.update(cx, |s, cx| s.set_folder_icon(&fid, "briefcase", cx));
        });
        cx.update(|cx| {
            let s = store.read(cx);
            let node = s.folder_tree().iter().find(|n| n.folder.id == fid).unwrap();
            assert_eq!(node.folder.icon, "briefcase");
        });
        cx.update(|cx| {
            store.update(cx, |s, cx| s.set_folder_icon(&fid, "  ", cx));
        });
        cx.update(|cx| {
            let s = store.read(cx);
            let node = s.folder_tree().iter().find(|n| n.folder.id == fid).unwrap();
            assert_eq!(node.folder.icon, "briefcase");
        });
    }

    #[test]
    fn sqlite_benchmark_large_batch_persists_and_deletes() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!("tnotes-bench-test-{nanos}.db"));

        let cx = TestAppContext::single();
        let store = cx.update(|cx| {
            cx.new(|_| NoteStore::open(&path, LOCAL_USER_ID).unwrap())
        });

        cx.update(|cx| {
            store.update(cx, |s, cx| {
                let (notes, folders) = s.create_benchmark_notes(5000, cx);
                assert_eq!(notes, 5000);
                assert_eq!(folders, 500);
            });
        });

        let reopened = NoteStore::open(&path, LOCAL_USER_ID).unwrap();
        assert_eq!(reopened.active_note_count(), 5000);
        assert_eq!(reopened.folder_tree().len(), 500);

        cx.update(|cx| {
            store.update(cx, |s, cx| {
                let deleted = s.delete_benchmark_notes(cx);
                assert_eq!(deleted, 5000);
            });
        });

        let reopened_empty = NoteStore::open(&path, LOCAL_USER_ID).unwrap();
        assert_eq!(reopened_empty.active_note_count(), 0);
        assert_eq!(reopened_empty.folder_tree().len(), 0);

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
