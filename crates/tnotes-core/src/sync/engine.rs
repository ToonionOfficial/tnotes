use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::db::{
    changes::{get_changes_after_seq, get_device_cursor, record_change, upsert_device_cursor},
    devices::get_device_by_id,
    folders::{get_folder_by_id, upsert_folder},
    notes::{get_note_by_id, upsert_note},
};
use crate::errors::{Error, Result};
use crate::models::{current_time_ms, folder::Folder, note::Note};
use crate::sync::{
    conflict::{VersionMeta, should_apply_remote},
    envelope::{Change, EntityType, SyncEnvelope, SyncResponse},
    integrity::{CycleCheckResult, check_proposed_folder_parent},
};

/// Summary of applied vs skipped changes in a sync batch
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncApplyReport {
    pub applied: usize,
    pub skipped: usize,
}

/// Metadata derived directly from a validated and applied entity
#[derive(Debug, Clone)]
pub struct AppliedChange {
    pub entity_id: String,
    pub version: u64,
    pub updated_at: i64,
    pub is_tombstone: bool,
    pub payload: serde_json::Value,
}

fn folder_exists_and_owned_by_user(
    conn: &Connection,
    folder_id: &str,
    user_id: &str,
) -> Result<bool> {
    let exists = conn
        .query_row(
            "SELECT 1 FROM folders WHERE id = ?1 AND user_id = ?2",
            rusqlite::params![folder_id, user_id],
            |_| Ok(()),
        )
        .optional()?
        .is_some();
    Ok(exists)
}

/// Applies a single incoming change using conflict resolution and user ownership validation.
/// Returns Ok(Some(AppliedChange)) if applied, or Ok(None) if skipped/rejected.
pub fn apply_single_change(
    conn: &Connection,
    change: &Change,
    authenticated_user_id: &str,
) -> Result<Option<AppliedChange>> {
    match change.entity_type {
        EntityType::Note => {
            let mut remote_note: Note =
                serde_json::from_value(change.payload.clone()).map_err(Error::Json)?;

            // Enforce authenticated user ownership
            remote_note.user_id = authenticated_user_id.to_string();

            // Validate parent folder belongs to the same user
            if let Some(ref fid) = remote_note.folder_id
                && !folder_exists_and_owned_by_user(conn, fid, authenticated_user_id)?
            {
                remote_note.folder_id = None;
            }

            let local_note = get_note_by_id(conn, &remote_note.id)?;
            let should_apply = match &local_note {
                Some(local) => {
                    if local.user_id != authenticated_user_id {
                        return Ok(None);
                    }
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
                let version = remote_note.version;
                let updated_at = remote_note.updated_at;
                let is_tombstone = remote_note.trashed || remote_note.deleted_at.is_some();
                let entity_id = remote_note.id.clone();
                let payload = serde_json::to_value(&remote_note).map_err(Error::Json)?;

                Ok(Some(AppliedChange {
                    entity_id,
                    version,
                    updated_at,
                    is_tombstone,
                    payload,
                }))
            } else {
                Ok(None)
            }
        }
        EntityType::Folder => {
            let mut remote_folder: Folder =
                serde_json::from_value(change.payload.clone()).map_err(Error::Json)?;

            // Enforce authenticated user ownership
            remote_folder.user_id = authenticated_user_id.to_string();

            // Validate parent folder belongs to the same user
            if let Some(ref pid) = remote_folder.parent_id
                && !folder_exists_and_owned_by_user(conn, pid, authenticated_user_id)?
            {
                remote_folder.parent_id = None;
            }

            let local_folder = get_folder_by_id(conn, &remote_folder.id)?;
            let should_apply = match &local_folder {
                Some(local) => {
                    if local.user_id != authenticated_user_id {
                        return Ok(None);
                    }
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
                match check_proposed_folder_parent(
                    conn,
                    &remote_folder.id,
                    remote_folder.parent_id.as_deref(),
                    50,
                )? {
                    CycleCheckResult::CycleDetected => remote_folder.parent_id = None,
                    // Do not write the folder before rejecting an over-deep hierarchy.
                    CycleCheckResult::DepthLimitExceeded => return Ok(None),
                    CycleCheckResult::NoCycle => {}
                }

                upsert_folder(conn, &remote_folder)?;

                let version = remote_folder.version;
                let updated_at = remote_folder.updated_at;
                let is_tombstone = remote_folder.deleted_at.is_some();
                let entity_id = remote_folder.id.clone();
                let payload = serde_json::to_value(&remote_folder).map_err(Error::Json)?;

                Ok(Some(AppliedChange {
                    entity_id,
                    version,
                    updated_at,
                    is_tombstone,
                    payload,
                }))
            } else {
                Ok(None)
            }
        }
    }
}

/// Applies a batch of incoming changes to the local database,
/// using deterministic conflict resolution for existing records.
pub fn apply_incoming_changes(
    conn: &mut Connection,
    changes: &[Change],
    authenticated_user_id: &str,
) -> Result<SyncApplyReport> {
    let mut report = SyncApplyReport::default();
    let tx = conn.transaction()?;

    for change in changes {
        if apply_single_change(&tx, change, authenticated_user_id)?.is_some() {
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
/// 2. Queries server's changes since the server-recorded device cursor (excluding the client's
///    device ID) for `user_id`.
/// 3. Updates device cursor and returns the `SyncResponse` envelope.
pub fn process_sync_envelope(
    conn: &mut Connection,
    envelope: &SyncEnvelope,
    user_id: &str,
) -> Result<SyncResponse> {
    let device = get_device_by_id(conn, &envelope.device_id)?
        .ok_or_else(|| Error::Auth("Unknown sync device".to_string()))?;
    if device.user_id != user_id {
        return Err(Error::Auth(
            "Sync device does not belong to the authenticated user".to_string(),
        ));
    }

    let tx = conn.transaction()?;
    let server_time = current_time_ms();

    for change in &envelope.changes {
        if let Some(applied) = apply_single_change(&tx, change, user_id)? {
            let entity_type_str = match change.entity_type {
                EntityType::Note => "note",
                EntityType::Folder => "folder",
            };
            record_change(
                &tx,
                user_id,
                entity_type_str,
                &applied.entity_id,
                &envelope.device_id,
                applied.version,
                applied.updated_at,
                applied.is_tombstone,
                &applied.payload,
            )?;
        }
    }

    // Never trust a cursor claimed by the client. Advancing a cursor beyond changes actually
    // returned by this server can make tombstone retention unsafe.
    let recorded_cursor = get_device_cursor(&tx, &envelope.device_id)?
        .map(|(last_seq, _)| last_seq)
        .unwrap_or(0);
    let (change_records, has_more) =
        get_changes_after_seq(&tx, user_id, recorded_cursor, &envelope.device_id, 500)?;

    let new_cursor = change_records
        .last()
        .map(|r| r.seq)
        .unwrap_or(recorded_cursor);

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
    use crate::db::{devices::upsert_device, migrations::open_in_memory, users::create_user};
    use crate::models::{device::Device, user::User};

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

        let report = apply_incoming_changes(&mut conn, &[change_a], &user.id).unwrap();
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

        let report = apply_incoming_changes(&mut conn, &[change_old], &user.id).unwrap();
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

        let report = apply_incoming_changes(&mut conn, &[change_new], &user.id).unwrap();
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

    #[test]
    fn test_sync_does_not_advance_cursor_from_client_claim() {
        let mut conn = open_in_memory().unwrap();
        let user = User::new("testuser", "hash");
        create_user(&conn, &user).unwrap();

        let sender = Device::new("sender", "Sender", "desktop", &user.id);
        let receiver = Device::new("receiver", "Receiver", "mobile", &user.id);
        upsert_device(&conn, &sender).unwrap();
        upsert_device(&conn, &receiver).unwrap();
        record_change(
            &conn,
            &user.id,
            "note",
            "pending-tombstone",
            &sender.id,
            1,
            1,
            true,
            &serde_json::json!({}),
        )
        .unwrap();

        let response = process_sync_envelope(
            &mut conn,
            &SyncEnvelope {
                device_id: receiver.id.clone(),
                last_seq: i64::MAX,
                last_sync_at: 0,
                changes: vec![],
            },
            &user.id,
        )
        .unwrap();

        assert_eq!(response.changes.len(), 1);
        assert_eq!(response.changes[0].entity_id, "pending-tombstone");
        assert_eq!(response.cursor, 1);
        assert_eq!(
            get_device_cursor(&conn, &receiver.id).unwrap(),
            Some((1, response.server_time))
        );
    }

    #[test]
    fn test_sync_rejects_envelope_for_another_users_device() {
        let mut conn = open_in_memory().unwrap();
        let user_a = User::new("user-a", "hash");
        let user_b = User::new("user-b", "hash");
        create_user(&conn, &user_a).unwrap();
        create_user(&conn, &user_b).unwrap();

        let device_b = Device::new("device-b", "Device B", "desktop", &user_b.id);
        upsert_device(&conn, &device_b).unwrap();

        let result = process_sync_envelope(
            &mut conn,
            &SyncEnvelope {
                device_id: device_b.id.clone(),
                last_seq: 0,
                last_sync_at: 0,
                changes: vec![],
            },
            &user_a.id,
        );

        assert!(matches!(result, Err(Error::Auth(_))));
        assert!(get_device_cursor(&conn, &device_b.id).unwrap().is_none());
    }

    #[test]
    fn test_note_deleted_at_is_logged_as_a_tombstone() {
        let mut conn = open_in_memory().unwrap();
        let user = User::new("testuser", "hash");
        create_user(&conn, &user).unwrap();

        let sender = Device::new("sender", "Sender", "desktop", &user.id);
        let receiver = Device::new("receiver", "Receiver", "mobile", &user.id);
        upsert_device(&conn, &sender).unwrap();
        upsert_device(&conn, &receiver).unwrap();

        let mut note = Note::new("Deleted", "Body", None, &sender.id, &user.id);
        note.deleted_at = Some(note.updated_at);
        let response = process_sync_envelope(
            &mut conn,
            &SyncEnvelope {
                device_id: sender.id.clone(),
                last_seq: 0,
                last_sync_at: 0,
                changes: vec![Change {
                    entity_type: EntityType::Note,
                    entity_id: note.id.clone(),
                    version: note.version,
                    updated_at: note.updated_at,
                    tombstone: false,
                    payload: serde_json::to_value(&note).unwrap(),
                }],
            },
            &user.id,
        )
        .unwrap();
        assert!(response.changes.is_empty());

        let receiver_response = process_sync_envelope(
            &mut conn,
            &SyncEnvelope {
                device_id: receiver.id,
                last_seq: 0,
                last_sync_at: 0,
                changes: vec![],
            },
            &user.id,
        )
        .unwrap();
        assert_eq!(receiver_response.changes.len(), 1);
        assert!(receiver_response.changes[0].tombstone);
    }

    #[test]
    fn test_sync_folder_cycle_auto_break() {
        let mut conn = open_in_memory().unwrap();
        let user = User::new("testuser", "hash");
        create_user(&conn, &user).unwrap();

        let dev_a = Device::new("dev_a", "Device A", "desktop", &user.id);
        let dev_b = Device::new("dev_b", "Device B", "mobile", &user.id);
        upsert_device(&conn, &dev_a).unwrap();
        upsert_device(&conn, &dev_b).unwrap();

        // 1. Initially: Folder A is root, Folder B has parent A
        let mut folder_a = Folder::new("Folder A", None, None, 0, &dev_a.id, &user.id);
        folder_a.id = "folder_a".into();
        let mut folder_b = Folder::new(
            "Folder B",
            None,
            Some("folder_a".into()),
            0,
            &dev_a.id,
            &user.id,
        );
        folder_b.id = "folder_b".into();

        upsert_folder(&conn, &folder_a).unwrap();
        upsert_folder(&conn, &folder_b).unwrap();

        // 2. Dev B offline moves Folder A inside Folder B (creating cycle B -> A -> B)
        folder_a.parent_id = Some("folder_b".into());
        folder_a.version = 2;
        folder_a.updated_at += 1000;
        folder_a.device_id = dev_b.id.clone();

        let envelope = SyncEnvelope {
            device_id: dev_b.id.clone(),
            last_seq: 0,
            last_sync_at: 0,
            changes: vec![Change {
                entity_type: EntityType::Folder,
                entity_id: folder_a.id.clone(),
                version: folder_a.version,
                updated_at: folder_a.updated_at,
                tombstone: false,
                payload: serde_json::to_value(&folder_a).unwrap(),
            }],
        };

        // Process sync envelope from dev_b
        let res = process_sync_envelope(&mut conn, &envelope, &user.id).unwrap();
        assert_eq!(res.changes.len(), 0);
        assert_eq!(res.cursor, 0);

        // Verify that the cycle was automatically detected and broken (folder_a.parent_id is None)
        let resolved_a = get_folder_by_id(&conn, "folder_a").unwrap().unwrap();
        assert!(resolved_a.parent_id.is_none());

        let resolved_b = get_folder_by_id(&conn, "folder_b").unwrap().unwrap();
        assert_eq!(resolved_b.parent_id.as_deref(), Some("folder_a"));

        // 3. dev_a pulls: verify dev_a receives the REPAIRED payload (parent_id: null)
        let envelope_a = SyncEnvelope {
            device_id: dev_a.id.clone(),
            last_seq: 0,
            last_sync_at: 0,
            changes: vec![],
        };
        let res_a = process_sync_envelope(&mut conn, &envelope_a, &user.id).unwrap();
        assert_eq!(res_a.changes.len(), 1);
        let pulled_folder: Folder =
            serde_json::from_value(res_a.changes[0].payload.clone()).unwrap();
        assert_eq!(pulled_folder.id, "folder_a");
        assert!(pulled_folder.parent_id.is_none());
    }

    #[test]
    fn test_sync_user_ownership_and_cross_user_isolation() {
        let mut conn = open_in_memory().unwrap();
        let user1 = User::new("user1", "hash");
        let user2 = User::new("user2", "hash");
        create_user(&conn, &user1).unwrap();
        create_user(&conn, &user2).unwrap();

        let dev1 = Device::new("dev1", "Dev 1", "desktop", &user1.id);
        let dev2 = Device::new("dev2", "Dev 2", "desktop", &user2.id);
        upsert_device(&conn, &dev1).unwrap();
        upsert_device(&conn, &dev2).unwrap();

        // 1. User 1 creates Folder 1
        let mut folder1 = Folder::new("User 1 Folder", None, None, 0, &dev1.id, &user1.id);
        folder1.id = "f1".into();
        upsert_folder(&conn, &folder1).unwrap();

        // 2. User 2 tries to push a note pointing to User 1's folder
        let malicious_note = Note::new(
            "Attacker Note",
            "Body",
            Some("f1".into()),
            &dev2.id,
            &user2.id,
        );
        let change = Change {
            entity_type: EntityType::Note,
            entity_id: malicious_note.id.clone(),
            version: 1,
            updated_at: 1000,
            tombstone: false,
            payload: serde_json::to_value(&malicious_note).unwrap(),
        };

        let env2 = SyncEnvelope {
            device_id: dev2.id.clone(),
            last_seq: 0,
            last_sync_at: 0,
            changes: vec![change],
        };

        process_sync_envelope(&mut conn, &env2, &user2.id).unwrap();

        // The note was saved under user2, but folder_id was sanitized to None (cross-user relationship stripped)
        let saved_note = get_note_by_id(&conn, &malicious_note.id).unwrap().unwrap();
        assert_eq!(saved_note.user_id, user2.id);
        assert!(saved_note.folder_id.is_none());
    }

    #[test]
    fn test_sync_rejects_over_deep_folder_without_persisting_it() {
        let mut conn = open_in_memory().unwrap();
        let user = User::new("testuser", "hash");
        create_user(&conn, &user).unwrap();

        let mut parent_id = None;
        for index in 0..51 {
            let mut folder = Folder::new("Nested", None, parent_id, 0, "dev", &user.id);
            folder.id = format!("folder_{index}");
            parent_id = Some(folder.id.clone());
            upsert_folder(&conn, &folder).unwrap();
        }

        let mut too_deep = Folder::new("Too deep", None, parent_id, 0, "dev", &user.id);
        too_deep.id = "too_deep".into();
        let change = Change {
            entity_type: EntityType::Folder,
            entity_id: too_deep.id.clone(),
            version: too_deep.version,
            updated_at: too_deep.updated_at,
            tombstone: false,
            payload: serde_json::to_value(&too_deep).unwrap(),
        };

        let report = apply_incoming_changes(&mut conn, &[change], &user.id).unwrap();
        assert_eq!(report.applied, 0);
        assert_eq!(report.skipped, 1);
        assert!(get_folder_by_id(&conn, &too_deep.id).unwrap().is_none());
    }
}
