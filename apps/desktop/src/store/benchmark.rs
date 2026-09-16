use gpui::Context;
use tnotes_core::db::folders::FolderNode;
use tnotes_core::db::{folders, notes};
use tnotes_core::models::folder::Folder;
use tnotes_core::models::note::Note;

use super::note_store::NoteStore;

#[cfg(test)]
use super::note_store::LOCAL_USER_ID;

impl NoteStore {
    pub fn create_benchmark_notes(
        &mut self,
        count: usize,
        cx: &mut Context<Self>,
    ) -> (usize, usize) {
        let folder_count = count.div_ceil(10);
        let mut folder_ids = Vec::with_capacity(folder_count);
        let mut new_folders = Vec::with_capacity(folder_count);

        for i in 0..folder_count {
            let folder_name = format!("__tnotes_benchmark_folder_v1__:Benchmark Folder {}", i + 1);
            let folder = Folder::new(
                &folder_name,
                None,
                None,
                i as i32,
                self.device_id.clone(),
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
            let body = format!(
                "__tnotes_benchmark_note_v1__:This is benchmark note number {} generated for performance testing.",
                i + 1
            );
            let note = Note::new(
                &title,
                &body,
                &body,
                target_folder,
                self.device_id.clone(),
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
            .filter(|n| {
                n.searchable_text
                    .starts_with("__tnotes_benchmark_note_v1__:")
                    || n.title.starts_with("Benchmark Note")
            })
            .map(|n| n.id.clone())
            .collect();
        let count = benchmark_note_ids.len();

        let benchmark_folder_ids: std::collections::HashSet<String> = self
            .folders
            .iter()
            .filter(|f| {
                f.folder.name.starts_with("__tnotes_benchmark_folder_v1__:")
                    || f.folder.name.starts_with("Benchmark Folder")
            })
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
            self.folders
                .retain(|f| !benchmark_folder_ids.contains(&f.folder.id));
        }

        self.notes.retain(|n| !benchmark_note_ids.contains(&n.id));
        for id in &benchmark_note_ids {
            self.history.remove_note(id);
        }

        cx.notify();
        count
    }
}

/// Deterministic fixture for view tests: one folder with three notes
/// (two pinned, one plain). Not part of the production API.
#[cfg(test)]
impl NoteStore {
    pub fn seed_test_data(&mut self) {
        use tnotes_core::models::folder::Folder;

        use super::navigation::NavigationLocation;

        let now = tnotes_core::models::current_time_ms();
        let mk = |id: &str, title: &str, folder: Option<&str>, pinned: bool| {
            let mut note = Note::new(
                title,
                title,
                title,
                folder.map(|s| s.to_string()),
                self.device_id.clone(),
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
                device_id: self.device_id.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::note_store::LOCAL_USER_ID;
    use core::prelude::v1::test;
    use gpui::{AppContext, TestAppContext};

    #[test]
    fn sqlite_benchmark_large_batch_persists_and_deletes() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!("tnotes-bench-test-{nanos}.db"));

        let cx = TestAppContext::single();
        let store = cx.update(|cx| cx.new(|_| NoteStore::open(&path, LOCAL_USER_ID).unwrap()));

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
}
