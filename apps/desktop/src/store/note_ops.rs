use gpui::Context;
use tnotes_core::db::notes;
use tnotes_core::models::note::Note;

use super::navigation::NavigationLocation;
use super::note_store::NoteStore;

impl NoteStore {
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

    #[allow(dead_code)]
    pub fn starred_notes(&self) -> Vec<Note> {
        self.notes
            .iter()
            .filter(|n| n.pinned && !n.trashed)
            .cloned()
            .collect()
    }

    #[allow(dead_code)]
    pub fn trashed_notes(&self) -> Vec<Note> {
        self.notes.iter().filter(|n| n.trashed).cloned().collect()
    }

    #[allow(dead_code)]
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

    pub fn create_new_note(&mut self, cx: &mut Context<Self>) {
        self.create_note_in_folder(None, cx);
    }

    pub fn create_note_in_folder(&mut self, folder_id: Option<String>, cx: &mut Context<Self>) {
        // Coerce unknown folders to root: `notes.folder_id` is a real FK, so
        // persisting a dangling reference would fail while the in-memory note
        // survived — a divergence we never want.
        let folder_id = folder_id.filter(|id| self.folder_exists(id));
        let note = Note::new(
            "Untitled Note",
            "",
            "",
            folder_id,
            self.device_id.clone(),
            self.user_id.clone(),
        );
        let target = note.id.clone();
        self.persist_note(&note);
        self.notes.insert(0, note);
        self.select_note(&target, cx);
    }

    /// Persist editor markdown as canonical document JSON (`tnotes-document`).
    pub fn update_note_body(&mut self, note_id: &str, markdown: &str, cx: &mut Context<Self>) {
        use super::{markdown_to_body_json, markdown_to_searchable};

        let body = markdown_to_body_json(markdown);
        let searchable = markdown_to_searchable(markdown);
        if let Some(note) = self.notes.iter_mut().find(|n| n.id == note_id) {
            let title = note.title.clone();
            let folder = note.folder_id.clone();
            note.update(title, body, searchable, folder, self.device_id.clone());
            let updated = note.clone();
            self.persist_note(&updated);
        }
        cx.notify();
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
            note.update(title, body, searchable, folder, self.device_id.clone());
            let updated = note.clone();
            self.persist_note(&updated);
        }
        cx.notify();
    }

    pub fn toggle_note_pin(&mut self, note_id: &str, cx: &mut Context<Self>) {
        if let Some(note) = self.notes.iter_mut().find(|n| n.id == note_id) {
            let pinned = !note.pinned;
            note.set_pinned(pinned, self.device_id.clone());
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
                self.device_id.clone(),
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
            note.trash(self.device_id.clone());
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
            note.restore(self.device_id.clone());
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::note_store::{LOCAL_USER_ID, test_store};
    use core::prelude::v1::test;
    use gpui::TestAppContext;

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
            store.device_id.clone(),
            LOCAL_USER_ID,
        );
        active.updated_at = now;
        let mut trashed = Note::new(
            "Trashed",
            "body",
            "body",
            None,
            store.device_id.clone(),
            LOCAL_USER_ID,
        );
        trashed.trash(store.device_id.clone());
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
                store.device_id.clone(),
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
        assert!(
            reopened
                .search_notes("sqlite body")
                .iter()
                .any(|n| n.id == note_id)
        );
        assert!(reopened.search_notes("no-such-term-xyz").is_empty());

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("db-wal"));
        let _ = std::fs::remove_file(path.with_extension("db-shm"));
    }

    #[test]
    fn open_heals_stale_local_user_id() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!("tnotes-stale-user-test-{nanos}.db"));

        // Simulate a legacy install: the vault user stored under a stale id.
        let note_id = {
            let mut store = NoteStore::open(&path, "local_user").unwrap();
            let note = Note::new(
                "Legacy Note",
                "legacy body",
                "legacy body",
                None,
                store.device_id.clone(),
                "local_user",
            );
            let id = note.id.clone();
            store.notes.push(note.clone());
            store.persist_note(&note);
            id
        };

        // Opening with the canonical id must adopt the legacy rows instead
        // of failing on the username UNIQUE constraint (which previously
        // dropped the session to an empty store).
        let reopened = NoteStore::open(&path, LOCAL_USER_ID).unwrap();
        let found: Vec<Note> = reopened
            .active_notes()
            .into_iter()
            .filter(|n| n.id == note_id)
            .collect();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].title, "Legacy Note");

        // The heal is idempotent: reopening is stable.
        let reopened_again = NoteStore::open(&path, LOCAL_USER_ID).unwrap();
        assert_eq!(
            reopened_again
                .active_notes()
                .iter()
                .filter(|n| n.id == note_id)
                .count(),
            1
        );

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("db-wal"));
        let _ = std::fs::remove_file(path.with_extension("db-shm"));
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
}
