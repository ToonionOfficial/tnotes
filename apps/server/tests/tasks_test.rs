use serde_json::json;
use tnotes_core::{
    auth::token::create_session_for_device,
    db::{
        changes::{get_changes_after_seq, record_change, upsert_device_cursor},
        devices::upsert_device,
        migrations::open_in_memory,
        notes::{get_note_by_id, insert_note},
        sessions::{create_session, get_session},
        users::create_user,
    },
    models::{current_time_ms, device::Device, note::Note, user::User},
    params,
};
use tnotes_server::tasks::run_housekeeping;

#[tokio::test]
async fn test_housekeeping_task() {
    let conn = open_in_memory().unwrap();
    let now = current_time_ms();
    let ms_per_day = 24 * 60 * 60 * 1000;

    let user = User::new("testuser", "hash");
    create_user(&conn, &user).unwrap();

    let dev_active = Device::new("device_active", "Active Device", "desktop", &user.id);
    let dev_expired = Device::new("device_expired", "Expired Device", "mobile", &user.id);
    upsert_device(&conn, &dev_active).unwrap();
    upsert_device(&conn, &dev_expired).unwrap();

    // 1. Setup Sessions: one active, one expired
    let active_session = create_session_for_device("device_active", None);
    let expired_session = create_session_for_device("device_expired", Some(-10_000));

    create_session(&conn, &active_session).unwrap();
    create_session(&conn, &expired_session).unwrap();

    // 2. Setup Notes: one recently trashed (5 days ago), one old trashed (35 days ago)
    let mut recent_trashed = Note::new("Recent Note", "Body", None, "device_active", &user.id);
    recent_trashed.trashed = true;
    recent_trashed.deleted_at = Some(now - (5 * ms_per_day));

    let mut old_trashed = Note::new("Old Note", "Body", None, "device_active", &user.id);
    old_trashed.trashed = true;
    old_trashed.deleted_at = Some(now - (35 * ms_per_day));

    insert_note(&conn, &recent_trashed).unwrap();
    insert_note(&conn, &old_trashed).unwrap();

    // 3. Setup Changes:
    // seq 1: 35 days old tombstone
    // seq 2: 5 days old tombstone
    let old_change_time = now - (35 * ms_per_day);
    let recent_change_time = now - (5 * ms_per_day);
    record_change(
        &conn,
        &user.id,
        "note",
        "n_old",
        "device_active",
        1,
        old_change_time,
        true,
        &json!({}),
    )
    .unwrap();
    record_change(
        &conn,
        &user.id,
        "note",
        "n_rec",
        "device_active",
        2,
        recent_change_time,
        true,
        &json!({}),
    )
    .unwrap();

    conn.execute(
        "UPDATE changes SET created_at = ?1 WHERE entity_id = 'n_old'",
        params![old_change_time],
    )
    .unwrap();
    conn.execute(
        "UPDATE changes SET created_at = ?1 WHERE entity_id = 'n_rec'",
        params![recent_change_time],
    )
    .unwrap();

    // Both devices have reached seq 2
    upsert_device_cursor(&conn, "device_active", 2, now).unwrap();
    upsert_device_cursor(&conn, "device_expired", 2, now).unwrap();

    // 4. Execute Housekeeping Run
    let (purged_sessions, purged_notes, purged_tombstones) = run_housekeeping(&conn);
    assert_eq!(purged_sessions, 1);
    assert_eq!(purged_notes, 1);
    assert_eq!(purged_tombstones, 1); // seq 1 was purged, seq 2 kept (only 5 days old)

    // 5. Verify outcomes
    // Expired session should be gone
    assert!(
        get_session(&conn, &expired_session.token)
            .unwrap()
            .is_none()
    );
    // Active session should remain
    assert!(get_session(&conn, &active_session.token).unwrap().is_some());

    // 35-day-old trashed note should be permanently deleted
    assert!(get_note_by_id(&conn, &old_trashed.id).unwrap().is_none());
    // 5-day-old trashed note should still exist in trash
    assert!(get_note_by_id(&conn, &recent_trashed.id).unwrap().is_some());

    // seq 1 change should be gone, seq 2 change should still exist
    let (changes, _) = get_changes_after_seq(&conn, &user.id, 0, "other", 10).unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].seq, 2);
}
