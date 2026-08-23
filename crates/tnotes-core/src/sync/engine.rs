use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::db::{
    changes::{get_changes_after_seq, record_change, upsert_device_cursor},
    folders::{get_folder_by_id, upsert_folder},
    notes::{get_note_by_id, upsert_note},
};
use crate::errors::{Error, Result};
use crate::models::{current_time_ms, folder::Folder, note::Note};
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

/// Applies a single incoming change using conflict resolution.
/// Returns Ok(true) if applied, or Ok(false) if skipped.
pub fn apply_single_change(conn: &Connection, change: &Change) -> Result<bool> {
    match change.entity_type {
        EntityType::Note => {
            let remote_note: Note =
                serde_json::from_value(change.payload.clone()).map_err(Error::Json)?;

            let local_note = get_note_by_id(conn, &remote_note.id)?;
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
                upsert_note(conn, &remote_note)?;
                Ok(true)
            } else {
                Ok(false)
            }
        }
        EntityType::Folder => {
            let remote_folder: Folder =
                serde_json::from_value(change.payload.clone()).map_err(Error::Json)?;

            let local_folder = get_folder_by_id(conn, &remote_folder.id)?;
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
                upsert_folder(conn, &remote_folder)?;
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }
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
        if apply_single_change(&tx, change)? {
            report.applied += 1;
        } else {
            report.skipped += 1;
        }
    }

    tx.commit()?;
    Ok(report)
}

/// Server-side sync endpoint processor:
/// 1. Applies client's incoming changes atomically and logs them in `changes`.
/// 2. Queries server's changes since `last_seq` cursor (excluding client's device ID) for `user_id`.
/// 3. Updates device cursor and returns the `SyncResponse` envelope.
pub fn process_sync_envelope(
    conn: &mut Connection,
    envelope: &SyncEnvelope,
    user_id: &str,
) -> Result<SyncResponse> {
    let tx = conn.transaction()?;
    let server_time = current_time_ms();

    for change in &envelope.changes {
        if apply_single_change(&tx, change)? {
            let entity_type_str = match change.entity_type {
                EntityType::Note => "note",
                EntityType::Folder => "folder",
            };
            record_change(
                &tx,
                user_id,
                entity_type_str,
                &change.entity_id,
                &envelope.device_id,
                change.version,
                change.updated_at,
                change.tombstone,
                &change.payload,
            )?;
        }
    }

    let (change_records, has_more) = get_changes_after_seq(
        &tx,
        user_id,
        envelope.last_seq,
        &envelope.device_id,
        500,
    )?;

    let new_cursor = change_records
        .last()
        .map(|r| r.seq)
        .unwrap_or(envelope.last_seq);

    upsert_device_cursor(&tx, &envelope.device_id, new_cursor, server_time)?;

    tx.commit()?;

    let outgoing_changes: Vec<Change> = change_records
        .into_iter()
        .map(|r| {
            let entity_type = match r.entity_type.as_str() {
                "folder" => EntityType::Folder,
                _ => EntityType::Note,
            };
            Change {
                entity_type,
                entity_id: r.entity_id,
                version: r.entity_version,
                updated_at: r.entity_updated_at,
                tombstone: r.is_tombstone,
                payload: r.payload,
            }
        })
        .collect();

    Ok(SyncResponse {
        server_time,
        cursor: new_cursor,
        has_more,
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
        use crate::db::devices::upsert_device;
        use crate::models::device::Device;

        let mut conn = open_in_memory().unwrap();
        let user = User::new("testuser", "hash");
        create_user(&conn, &user).unwrap();

        let dev_a = Device::new("dev_a", "Device A", "desktop", &user.id);
        let dev_b = Device::new("dev_b", "Device B", "mobile", &user.id);
        upsert_device(&conn, &dev_a).unwrap();
        upsert_device(&conn, &dev_b).unwrap();

        // 1. Client dev_a syncs a new note to the server
        let note_a = Note::new("Server Note", "From dev_a", None, "dev_a", &user.id);
        let envelope_a = SyncEnvelope {
            device_id: "dev_a".to_string(),
            last_seq: 0,
            last_sync_at: 0,
            changes: vec![Change {
                entity_type: EntityType::Note,
                entity_id: note_a.id.clone(),
                version: note_a.version,
                updated_at: note_a.updated_at,
                tombstone: false,
                payload: serde_json::to_value(&note_a).unwrap(),
            }],
        };

        let response_a = process_sync_envelope(&mut conn, &envelope_a, &user.id).unwrap();
        assert_eq!(response_a.cursor, 0);
        assert_eq!(response_a.changes.len(), 0);

        // 2. Client dev_b connects with last_seq = 0 and pushes its own note
        let note_b = Note::new("Client Note", "From dev_b", None, "dev_b", &user.id);
        let envelope_b = SyncEnvelope {
            device_id: "dev_b".to_string(),
            last_seq: 0,
            last_sync_at: 0,
            changes: vec![Change {
                entity_type: EntityType::Note,
                entity_id: note_b.id.clone(),
                version: note_b.version,
                updated_at: note_b.updated_at,
                tombstone: false,
                payload: serde_json::to_value(&note_b).unwrap(),
            }],
        };

        let response_b = process_sync_envelope(&mut conn, &envelope_b, &user.id).unwrap();

        // dev_b receives dev_a's note (seq 1), but not its own note (seq 2)
        assert_eq!(response_b.changes.len(), 1);
        assert_eq!(response_b.changes[0].entity_id, note_a.id);
        assert_eq!(response_b.cursor, 1);

        // 3. dev_b syncs again with returned cursor (1) -> receives 0 new changes
        let envelope_b2 = SyncEnvelope {
            device_id: "dev_b".to_string(),
            last_seq: response_b.cursor,
            last_sync_at: 0,
            changes: vec![],
        };
        let response_b2 = process_sync_envelope(&mut conn, &envelope_b2, &user.id).unwrap();
        assert_eq!(response_b2.changes.len(), 0);
        assert_eq!(response_b2.cursor, 1);

        // 4. dev_a syncs with last_seq = 0 -> receives dev_b's note (seq 2)
        let envelope_a2 = SyncEnvelope {
            device_id: "dev_a".to_string(),
            last_seq: 0,
            last_sync_at: 0,
            changes: vec![],
        };
        let response_a2 = process_sync_envelope(&mut conn, &envelope_a2, &user.id).unwrap();
        assert_eq!(response_a2.changes.len(), 1);
        assert_eq!(response_a2.changes[0].entity_id, note_b.id);
        assert_eq!(response_a2.cursor, 2);

        // Server contains both notes
        assert!(get_note_by_id(&conn, &note_a.id).unwrap().is_some());
        assert!(get_note_by_id(&conn, &note_b.id).unwrap().is_some());
    }
}
