use gpui::*;
use tnotes_core::db::folders::FolderNode;
use tnotes_core::models::folder::Folder;
use tnotes_core::models::note::Note;

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
pub struct NoteStore {
    notes: Vec<Note>,
    folders: Vec<FolderNode>,
    active_location: NavigationLocation,
    history: NavigationHistory,
}

impl NoteStore {
    pub fn new() -> Self {
        let now = tnotes_core::models::current_time_ms();
        let minute = 60_000i64;
        let hour = 3_600_000i64;
        let day = 86_400_000i64;

        let mk = |id: &str,
                  title: &str,
                  body: &str,
                      folder_id: Option<&str>,
                      pinned: bool,
                      updated_at: i64| {
            let mut note = Note::new(
                title,
                body,
                body,
                folder_id.map(|s| s.to_string()),
                LOCAL_DEVICE_ID,
                LOCAL_USER_ID,
            );
            note.id = id.to_string();
            note.pinned = pinned;
            note.updated_at = updated_at;
            note.created_at = updated_at;
            note
        };

        let notes = vec![
            mk(
                "note-arch-spec",
                "System Architecture Spec",
                "Local-first SQLite with FTS5, CRDT synchronization, and DankeShell token design system.",
                Some("projects:architecture:core"),
                true,
                now,
            ),
            mk(
                "note-desktop-gpui",
                "Desktop Shell & GPUI Architecture",
                "Unified Obsidian-style sidebar, borderless notes, twrite rope canvas rendering.",
                Some("projects:architecture:desktop"),
                true,
                now - 15 * minute,
            ),
            mk(
                "note-db-schema",
                "Database Schema & Index Design",
                "Tables for notes, folders, tags, sync operations log, and tokenized full-text search indexes.",
                Some("projects"),
                false,
                now - 2 * hour,
            ),
            mk(
                "note-ws-sync",
                "WebSocket Sync Protocol",
                "Bidirectional binary and JSON delta streaming between mobile client and desktop server.",
                Some("projects"),
                false,
                now - 26 * hour,
            ),
            mk(
                "note-q3-roadmap",
                "Q3 Roadmap & Planning",
                "Offline-first conflict resolution, canvas mode, graph view, and end-to-end encryption.",
                Some("personal"),
                false,
                now - 3 * day,
            ),
            mk(
                "note-sprint-goals",
                "Weekly Sprint Goals",
                "Implement single-sidebar layout, DankeShell active states, and twrite editor integration.",
                Some("personal:journal"),
                false,
                now - 5 * day,
            ),
            mk(
                "note-design-inspo",
                "Design Inspiration: Obsidian x Notion",
                "Collapsible sidebar rail, clean hierarchy, distraction-free markdown canvas, quiet chrome.",
                Some("personal:journal"),
                false,
                now - 7 * day,
            ),
        ];

        let folders = Self::mock_folders(now);

        Self {
            notes,
            folders,
            active_location: NavigationLocation::Note("note-arch-spec".to_string()),
            history: NavigationHistory::default(),
        }
    }

    fn mock_folders(now: i64) -> Vec<FolderNode> {
        // Depth-ordered to match the previous hardcoded tree.
        let defs: &[(&str, Option<&str>, &str, i64, &str)] = &[
            ("projects", None, "Projects", 0, "/Projects"),
            (
                "projects:architecture",
                Some("projects"),
                "Architecture",
                1,
                "/Projects/Architecture",
            ),
            (
                "projects:architecture:core",
                Some("projects:architecture"),
                "Core Engine",
                2,
                "/Projects/Architecture/Core Engine",
            ),
            (
                "projects:architecture:desktop",
                Some("projects:architecture"),
                "Desktop Shell",
                2,
                "/Projects/Architecture/Desktop Shell",
            ),
            (
                "specs",
                Some("projects"),
                "Specifications",
                1,
                "/Projects/Specifications",
            ),
            ("personal", None, "Personal Notes", 0, "/Personal Notes"),
            (
                "personal:journal",
                Some("personal"),
                "Journal",
                1,
                "/Personal Notes/Journal",
            ),
        ];

        defs.iter()
            .map(|(id, parent, name, depth, path)| FolderNode {
                folder: Folder {
                    id: id.to_string(),
                    user_id: LOCAL_USER_ID.to_string(),
                    parent_id: parent.map(|s| s.to_string()),
                    name: name.to_string(),
                    icon: "📁".to_string(),
                    sort_order: 0,
                    version: 1,
                    updated_at: now,
                    created_at: now,
                    deleted_at: None,
                    device_id: LOCAL_DEVICE_ID.to_string(),
                },
                depth: *depth,
                path: path.to_string(),
            })
            .collect()
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

    /// In-memory search; signature is ready for an FTS5 swap in Phase 6.
    pub fn search_notes(&self, query: &str) -> Vec<Note> {
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

    // ---- mutations (write through to SQLite in Phase 6) ----

    pub fn create_new_note(&mut self, cx: &mut Context<Self>) {
        self.create_note_in_folder(None, cx);
    }

    pub fn create_note_in_folder(
        &mut self,
        folder_id: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let mut note = Note::new(
            "Untitled Note",
            "Start typing your note here...",
            "Start typing your note here...",
            folder_id,
            LOCAL_DEVICE_ID,
            LOCAL_USER_ID,
        );
        // Keep the human-friendly `note-N` prefix used by existing tests.
        note.id = format!("note-{}", self.notes.len() + 1);
        let target = note.id.clone();
        self.notes.insert(0, note);
        self.select_note(&target, cx);
    }

    pub fn toggle_note_pin(&mut self, note_id: &str, cx: &mut Context<Self>) {
        if let Some(note) = self.notes.iter_mut().find(|n| n.id == note_id) {
            let pinned = !note.pinned;
            note.set_pinned(pinned, LOCAL_DEVICE_ID);
        }
        cx.notify();
    }

    pub fn duplicate_note(&mut self, note_id: &str, cx: &mut Context<Self>) {
        if let Some(index) = self.notes.iter().position(|n| n.id == note_id) {
            let source = self.notes[index].clone();
            let mut copy = Note::new(
                format!("{} (copy)", source.title),
                source.body.clone(),
                source.searchable_text.clone(),
                source.folder_id.clone(),
                LOCAL_DEVICE_ID,
                LOCAL_USER_ID,
            );
            copy.id = format!("note-{}-copy", self.notes.len() + 1);
            let new_id = copy.id.clone();
            self.notes.insert(index + 1, copy);
            self.select_note(&new_id, cx);
        }
    }

    pub fn delete_note(&mut self, note_id: &str, cx: &mut Context<Self>) {
        if let Some(note) = self.notes.iter_mut().find(|n| n.id == note_id) {
            note.trash(LOCAL_DEVICE_ID);
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
            cx.notify();
        }
    }

    pub fn permanently_delete_note(&mut self, note_id: &str, cx: &mut Context<Self>) {
        self.notes.retain(|n| n.id != note_id);
        self.history.remove_note(note_id);
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
        for id in trashed {
            self.history.remove_note(&id);
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
        let store = NoteStore::new();
        assert_eq!(store.trashed_notes().len(), 0);
        assert_eq!(store.active_notes().len(), 7);
    }
}
