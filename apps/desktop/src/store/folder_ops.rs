use gpui::Context;
use tnotes_core::db::folders::FolderNode;
use tnotes_core::models::folder::Folder;
use tnotes_core::models::note::Note;

use super::navigation::NavigationLocation;
use super::note_store::NoteStore;

impl NoteStore {
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
            self.device_id.clone(),
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
            let (depth, path) = match parent_id
                .as_deref()
                .and_then(|pid| self.folders.iter().find(|n| n.folder.id == pid))
            {
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
                node.folder.soft_delete(self.device_id.clone());
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
                note.trash(self.device_id.clone());
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
            node.folder.rename(name, self.device_id.clone());
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
            node.folder.set_icon(icon, self.device_id.clone());
            let updated = node.folder.clone();
            self.persist_folder(&updated);
        }
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::note_store::{LOCAL_USER_ID, test_store};
    use core::prelude::v1::test;
    use gpui::{AppContext, TestAppContext};

    #[test]
    fn sqlite_persists_folders_and_folder_notes() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!("tnotes-folders-test-{nanos}.db"));

        let cx = TestAppContext::single();
        let store = cx.update(|cx| cx.new(|_| NoteStore::open(&path, LOCAL_USER_ID).unwrap()));
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
        assert!(reopened.notes_in_folder(&fid).iter().any(|n| n.id == nid));

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
            assert!(s.folder_tree().iter().all(|n| n.folder.parent_id.is_none()));
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
            assert_eq!(s.selected_note_id(), Some(s.active_notes()[0].id.clone()));
        });
    }

    #[test]
    fn rename_folder_trims_and_ignores_blank() {
        let mut cx = TestAppContext::single();
        let store = test_store(&mut cx);
        let fid = cx.update(|cx| store.update(cx, |s, cx| s.create_folder("Work", None, cx)));
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
    fn set_folder_icon_stores_lowercase_name() {
        let mut cx = TestAppContext::single();
        let store = test_store(&mut cx);
        let fid = cx.update(|cx| store.update(cx, |s, cx| s.create_folder("Work", None, cx)));
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
}
