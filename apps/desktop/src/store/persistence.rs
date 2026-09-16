use tnotes_core::db::{folders, notes};
use tnotes_core::models::folder::Folder;
use tnotes_core::models::note::Note;
use tnotes_core::models::user::User;
use tnotes_core::{Connection, Result as CoreResult, params};

use super::note_store::{LOCAL_USERNAME, NoteStore};

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
    /// Stable per-install device id, mirroring mobile `getOrCreateDeviceId`:
    /// read `device_id` from `sync_meta`, else generate a ULID and persist it.
    /// Legacy rows stamped `"desktop-local"` keep their value; only new
    /// writes use the generated id.
    pub(crate) fn device_id_for_connection(conn: &Connection) -> CoreResult<String> {
        use tnotes_core::db::sync::{get_sync_meta, set_sync_meta};

        if let Some(id) = get_sync_meta(conn, "device_id")?
            && !id.is_empty()
        {
            return Ok(id);
        }
        let id = tnotes_core::Ulid::generate().to_string();
        set_sync_meta(conn, "device_id", &id)?;
        Ok(id)
    }

    /// `notes.user_id` / `folders.user_id` are FKs into `users`, so the local
    /// vault user must exist before any note can be persisted.
    pub(crate) fn ensure_local_user(conn: &mut Connection, user_id: &str) -> CoreResult<()> {
        use tnotes_core::db::users::{get_user_by_id, get_user_by_username};

        if get_user_by_id(&*conn, user_id)?.is_some() {
            return Ok(());
        }
        // Heal installs where the vault user exists under a stale id
        // (same username, different id): adopt its rows instead of failing
        // on the username UNIQUE constraint, which used to drop the whole
        // session to an empty store.
        if let Some(legacy) = get_user_by_username(&*conn, LOCAL_USERNAME)? {
            Self::rename_user_id(&mut *conn, &legacy.id, user_id)?;
            return Ok(());
        }
        let now = tnotes_core::models::current_time_ms();
        // `users.id` is the FK target for notes/folders; a bare local
        // vault user (no password) is enough until auth lands.
        let user = User {
            id: user_id.to_string(),
            username: LOCAL_USERNAME.to_string(),
            password_hash: String::new(),
            created_at: now,
        };
        if let Err(e) = tnotes_core::db::users::create_user(&*conn, &user) {
            // Tolerate a concurrent first-boot race; anything else propagates.
            if get_user_by_id(&*conn, user_id)?.is_none() {
                return Err(e.into());
            }
        }
        Ok(())
    }

    /// Rename a user id across `users` and every `user_id` FK table.
    /// Runs in one transaction with deferred FK enforcement so the
    /// intermediate state (children pointing at the new id before the
    /// parent row is renamed, or vice versa) never trips a violation.
    pub(crate) fn rename_user_id(conn: &mut Connection, from: &str, to: &str) -> CoreResult<()> {
        let tx = conn.transaction()?;
        tx.execute_batch("PRAGMA defer_foreign_keys = ON")?;
        for table in ["notes", "folders", "devices", "changes", "users"] {
            let column = if table == "users" { "id" } else { "user_id" };
            tx.execute(
                &format!("UPDATE {table} SET {column} = ?1 WHERE {column} = ?2"),
                params![to, from],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Best-effort write-through: the in-memory state is authoritative for the
    /// UI, a persistence failure must not break interaction (it is logged).
    pub(crate) fn persist_note(&self, note: &Note) {
        if let Some(conn) = self.conn.as_ref()
            && let Err(e) = notes::upsert_note(conn, note)
        {
            eprintln!("tnotes: failed to persist note {}: {e}", note.id);
        }
    }

    pub(crate) fn delete_persisted_note(&self, note_id: &str) {
        if let Some(conn) = self.conn.as_ref()
            && let Err(e) = notes::delete_note_permanently(conn, note_id)
        {
            eprintln!("tnotes: failed to delete note {note_id}: {e}");
        }
    }

    pub(crate) fn persist_folder(&self, folder: &Folder) {
        if let Some(conn) = self.conn.as_ref()
            && let Err(e) = folders::upsert_folder(conn, folder)
        {
            eprintln!("tnotes: failed to persist folder {}: {e}", folder.id);
        }
    }

    /// Reload the in-memory tree from SQLite (authoritative ordering).
    /// No-op without a database connection.
    pub(crate) fn reload_folders(&mut self) {
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
                let url = tnotes_core::db::sync::get_sync_meta(conn, "server_url").unwrap_or(None);
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

    pub fn vacuum(&self) -> CoreResult<()> {
        if let Some(conn) = self.conn.as_ref() {
            conn.execute_batch("VACUUM;")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::note_store::{LOCAL_USER_ID, test_store};
    use core::prelude::v1::test;
    use gpui::{AppContext, TestAppContext};

    #[test]
    fn device_id_is_stable_across_opens() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!("tnotes-device-id-test-{nanos}.db"));

        let first = NoteStore::open(&path, LOCAL_USER_ID).unwrap();
        assert_eq!(first.device_id.len(), 26);
        assert_ne!(first.device_id, "desktop-local");
        let saved = first.device_id.clone();
        drop(first);

        let second = NoteStore::open(&path, LOCAL_USER_ID).unwrap();
        assert_eq!(second.device_id, saved);

        drop(second);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("db-wal"));
        let _ = std::fs::remove_file(path.with_extension("db-shm"));
    }

    #[test]
    fn device_id_stamps_new_rows() {
        let mut cx = TestAppContext::single();
        let store = test_store(&mut cx);
        cx.update(|cx| {
            store.update(cx, |s, cx| {
                s.create_new_note(cx);
            });
        });
        cx.update(|cx| {
            let s = store.read(cx);
            let note = s.selected_note().unwrap();
            assert_eq!(note.device_id, s.device_id);
            assert_ne!(note.device_id, "desktop-local");
        });
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
}
