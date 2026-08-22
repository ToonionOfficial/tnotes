use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::db::{
    folders::{get_folder_by_id, upsert_folder},
    notes::{get_note_by_id, upsert_note},
    sync::get_changes_since,
    themes::{get_theme_by_id, upsert_theme},
};
use crate::errors::{Error, Result};
use crate::models::{current_time_ms, folder::Folder, note::Note, theme::Theme};
use crate::sync::{
    conflict::{VersionMeta, should_apply_remote},
    envelope::{Change, EntityType, SyncEnvelope, SyncResponse},
};

/// Summary of applied vs skipped changes in a sync batch
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncApplyReport {
    pub applied: usize,
    pub skipped: usize,
}

/// Applies a batch of incoming changes to the local database,
/// using deterministic conflict resolution for existing records.
pub fn apply_incoming_changes(
    conn: &mut Connection,
    changes: &[Change],
) -> Result<SyncApplyReport> {
    let mut report = SyncApplyReport::default();
    let tx = conn.transaction()?;

    for change in changes {
        match change.entity_type {
            EntityType::Note => {
                let remote_note: Note =
                    serde_json::from_value(change.payload.clone()).map_err(Error::Json)?;

                let local_note = get_note_by_id(&tx, &remote_note.id)?;
                let should_apply = match &local_note {
                    Some(local) => {
                        let local_meta =
                            VersionMeta::new(local.version, local.updated_at, &local.device_id);
                        let remote_meta = VersionMeta::new(
                            remote_note.version,
                            remote_note.updated_at,
                            &remote_note.device_id,
                        );
                        should_apply_remote(&local_meta, &remote_meta)
                    }
                    None => true,
                };

                if should_apply {
                    upsert_note(&tx, &remote_note)?;
                    report.applied += 1;
                } else {
                    report.skipped += 1;
                }
            }
            EntityType::Folder => {
                let remote_folder: Folder =
                    serde_json::from_value(change.payload.clone()).map_err(Error::Json)?;

                let local_folder = get_folder_by_id(&tx, &remote_folder.id)?;
                let should_apply = match &local_folder {
                    Some(local) => {
                        let local_meta =
                            VersionMeta::new(local.version, local.updated_at, &local.device_id);
                        let remote_meta = VersionMeta::new(
                            remote_folder.version,
                            remote_folder.updated_at,
                            &remote_folder.device_id,
                        );
                        should_apply_remote(&local_meta, &remote_meta)
                    }
                    None => true,
                };

                if should_apply {
                    upsert_folder(&tx, &remote_folder)?;
                    report.applied += 1;
                } else {
                    report.skipped += 1;
                }
            }
            EntityType::Theme => {
                let remote_theme: Theme =
                    serde_json::from_value(change.payload.clone()).map_err(Error::Json)?;

                let local_theme = get_theme_by_id(&tx, &remote_theme.id)?;
                let should_apply = match &local_theme {
                    Some(local) => {
                        let local_meta =
                            VersionMeta::new(local.version, local.updated_at, &local.device_id);
                        let remote_meta = VersionMeta::new(
                            remote_theme.version,
                            remote_theme.updated_at,
                            &remote_theme.device_id,
                        );
                        should_apply_remote(&local_meta, &remote_meta)
                    }
                    None => true,
                };

                if should_apply {
                    upsert_theme(&tx, &remote_theme)?;
                    report.applied += 1;
                } else {
                    report.skipped += 1;
                }
            }
        }
    }

    tx.commit()?;
    Ok(report)
}

/// Server-side sync endpoint processor:
/// 1. Applies client's incoming changes atomically.
/// 2. Queries server's changes since client's `last_sync_at` (excluding client's device ID) for `user_id`.
/// 3. Returns the `SyncResponse` envelope.
pub fn process_sync_envelope(
    conn: &mut Connection,
    envelope: &SyncEnvelope,
    user_id: &str,
) -> Result<SyncResponse> {
    // 1. Apply incoming changes
    apply_incoming_changes(conn, &envelope.changes)?;

    // 2. Fetch server changes since client's cursor scoped to this user
    let outgoing_changes = get_changes_since(
        conn,
        envelope.last_sync_at,
        Some(&envelope.device_id),
        user_id,
    )?;

    // 3. Return response with current server timestamp
    Ok(SyncResponse {
        server_time: current_time_ms(),
        changes: outgoing_changes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::open_in_memory;
    use crate::db::users::create_user;
    use crate::models::user::User;

    #[test]
    fn test_sync_apply_new_and_conflict() {
        let mut conn = open_in_memory().unwrap();
        let user = User::new("testuser", "hash");
        create_user(&conn, &user).unwrap();

        // 1. Client A creates a note
        let note_a = Note::new("Title A", "Body A", None, "dev_a", &user.id);
        let change_a = Change {
            entity_type: EntityType::Note,
            entity_id: note_a.id.clone(),
            version: note_a.version,
            updated_at: note_a.updated_at,
            tombstone: false,
            payload: serde_json::to_value(&note_a).unwrap(),
        };

        let report = apply_incoming_changes(&mut conn, &[change_a]).unwrap();
        assert_eq!(report.applied, 1);
        assert_eq!(report.skipped, 0);

        // 2. Client B sends an older version of the same note (should be skipped)
        let mut old_note_b = note_a.clone();
        old_note_b.version = 0; // older version
        old_note_b.body = "Old body".into();
        let change_old = Change {
            entity_type: EntityType::Note,
            entity_id: old_note_b.id.clone(),
            version: old_note_b.version,
            updated_at: old_note_b.updated_at,
            tombstone: false,
            payload: serde_json::to_value(&old_note_b).unwrap(),
        };

        let report = apply_incoming_changes(&mut conn, &[change_old]).unwrap();
        assert_eq!(report.applied, 0);
        assert_eq!(report.skipped, 1);

        // Verify local note body is still "Body A"
        let fetched = get_note_by_id(&conn, &note_a.id).unwrap().unwrap();
        assert_eq!(fetched.body, "Body A");

        // 3. Client B sends a newer version (version = 2)
        let mut new_note_b = note_a.clone();
        new_note_b.update("Title B", "Updated Body B", None, "dev_b");
        let change_new = Change {
            entity_type: EntityType::Note,
            entity_id: new_note_b.id.clone(),
            version: new_note_b.version,
            updated_at: new_note_b.updated_at,
            tombstone: false,
            payload: serde_json::to_value(&new_note_b).unwrap(),
        };

        let report = apply_incoming_changes(&mut conn, &[change_new]).unwrap();
        assert_eq!(report.applied, 1);
        assert_eq!(report.skipped, 0);

        let fetched = get_note_by_id(&conn, &note_a.id).unwrap().unwrap();
        assert_eq!(fetched.body, "Updated Body B");
        assert_eq!(fetched.version, 2);
    }

    #[test]
    fn test_process_sync_envelope_roundtrip() {
        let mut conn = open_in_memory().unwrap();
        let user = User::new("testuser", "hash");
        create_user(&conn, &user).unwrap();

        // 1. Seed a note created by dev_a on the server
        let note_server = Note::new("Server Note", "From dev_a", None, "dev_a", &user.id);
        upsert_note(&conn, &note_server).unwrap();

        // 2. Client dev_b connects with a new note and last_sync_at = 0
        let note_client = Note::new("Client Note", "From dev_b", None, "dev_b", &user.id);
        let envelope = SyncEnvelope {
            device_id: "dev_b".to_string(),
            last_sync_at: 0,
            changes: vec![Change {
                entity_type: EntityType::Note,
                entity_id: note_client.id.clone(),
                version: note_client.version,
                updated_at: note_client.updated_at,
                tombstone: false,
                payload: serde_json::to_value(&note_client).unwrap(),
            }],
        };

        let response = process_sync_envelope(&mut conn, &envelope, &user.id).unwrap();

        // dev_b should receive the note from dev_a, but not its own note
        assert_eq!(response.changes.len(), 1);
        assert_eq!(response.changes[0].entity_id, note_server.id);

        // Server should now contain both notes
        assert!(get_note_by_id(&conn, &note_server.id).unwrap().is_some());
        assert!(get_note_by_id(&conn, &note_client.id).unwrap().is_some());
    }
}
