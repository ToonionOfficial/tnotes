use crate::models::current_time_ms;
use rusqlite::{Connection, Result, Row, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChangeRecord {
    pub seq: i64,
    pub user_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub device_id: String,
    pub entity_version: u64,
    pub entity_updated_at: i64,
    pub is_tombstone: bool,
    pub payload: Value,
    pub created_at: i64,
}

pub fn row_to_change_record(row: &Row) -> Result<ChangeRecord> {
    let payload_str: String = row.get("payload")?;
    let payload: Value = serde_json::from_str(&payload_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    Ok(ChangeRecord {
        seq: row.get("seq")?,
        user_id: row.get("user_id")?,
        entity_type: row.get("entity_type")?,
        entity_id: row.get("entity_id")?,
        device_id: row.get("device_id")?,
        entity_version: row.get::<_, i64>("entity_version")? as u64,
        entity_updated_at: row.get("entity_updated_at")?,
        is_tombstone: row.get::<_, i64>("is_tombstone")? != 0,
        payload,
        created_at: row.get("created_at")?,
    })
}

pub fn record_change(
    conn: &Connection,
    user_id: &str,
    entity_type: &str,
    entity_id: &str,
    device_id: &str,
    entity_version: u64,
    entity_updated_at: i64,
    is_tombstone: bool,
    payload: &Value,
) -> Result<i64> {
    let now = current_time_ms();
    let payload_str = payload.to_string();

    conn.execute(
        "INSERT INTO changes (
            user_id, entity_type, entity_id, device_id,
            entity_version, entity_updated_at, is_tombstone,
            payload, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            user_id,
            entity_type,
            entity_id,
            device_id,
            entity_version as i64,
            entity_updated_at,
            if is_tombstone { 1 } else { 0 },
            payload_str,
            now,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn get_changes_after_seq(
    conn: &Connection,
    user_id: &str,
    after_seq: i64,
    exclude_device_id: &str,
    limit: i64,
) -> Result<(Vec<ChangeRecord>, bool)> {
    let fetch_limit = limit.max(1) + 1;
    let mut stmt = conn.prepare(
        "SELECT seq, user_id, entity_type, entity_id, device_id,
                entity_version, entity_updated_at, is_tombstone,
                payload, created_at
         FROM changes
         WHERE user_id = ?1 AND seq > ?2 AND device_id != ?3
         ORDER BY seq ASC
         LIMIT ?4",
    )?;

    let mut records = stmt
        .query_map(
            params![user_id, after_seq, exclude_device_id, fetch_limit],
            row_to_change_record,
        )?
        .collect::<Result<Vec<_>>>()?;

    let has_more = records.len() > limit as usize;
    if has_more {
        records.truncate(limit as usize);
    }

    Ok((records, has_more))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::open_in_memory;
    use crate::db::users::create_user;
    use crate::models::user::User;
    use serde_json::json;

    #[test]
    fn test_record_change_monotonic_seq() {
        let conn = open_in_memory().unwrap();
        let user = User::new("testuser", "hash");
        create_user(&conn, &user).unwrap();

        let payload = json!({"title": "test"});

        let seq1 = record_change(
            &conn, &user.id, "note", "n1", "d1", 1, 1000, false, &payload,
        )
        .unwrap();
        let seq2 = record_change(
            &conn, &user.id, "note", "n2", "d1", 1, 1001, false, &payload,
        )
        .unwrap();
        let seq3 =
            record_change(&conn, &user.id, "note", "n1", "d1", 2, 1002, true, &payload).unwrap();

        assert_eq!(seq1, 1);
        assert_eq!(seq2, 2);
        assert_eq!(seq3, 3);
    }

    #[test]
    fn test_get_changes_after_seq_filtering_and_pagination() {
        let conn = open_in_memory().unwrap();
        let user = User::new("testuser", "hash");
        create_user(&conn, &user).unwrap();

        let other_user = User::new("otheruser", "hash");
        create_user(&conn, &other_user).unwrap();

        let payload = json!({"data": "sample"});

        // 3 changes from dev1 for user
        record_change(&conn, &user.id, "note", "n1", "dev1", 1, 100, false, &payload).unwrap();
        record_change(&conn, &user.id, "note", "n2", "dev1", 1, 200, false, &payload).unwrap();
        record_change(&conn, &user.id, "note", "n3", "dev1", 1, 300, false, &payload).unwrap();

        // 1 change from dev2 for user
        record_change(&conn, &user.id, "note", "n4", "dev2", 1, 400, false, &payload).unwrap();

        // 1 change for other user
        record_change(&conn, &other_user.id, "note", "n5", "dev1", 1, 500, false, &payload).unwrap();

        // dev2 queries after seq 0: should receive 3 changes from dev1 (excluding dev2)
        let (records, has_more) = get_changes_after_seq(&conn, &user.id, 0, "dev2", 10).unwrap();
        assert_eq!(records.len(), 3);
        assert!(!has_more);
        assert_eq!(records[0].entity_id, "n1");
        assert_eq!(records[1].entity_id, "n2");
        assert_eq!(records[2].entity_id, "n3");

        // dev2 queries with limit = 2 (pagination)
        let (paged_records, has_more_paged) = get_changes_after_seq(&conn, &user.id, 0, "dev2", 2).unwrap();
        assert_eq!(paged_records.len(), 2);
        assert!(has_more_paged);
        assert_eq!(paged_records[0].seq, 1);
        assert_eq!(paged_records[1].seq, 2);

        // query remaining page after seq 2
        let (remaining, has_more_rem) = get_changes_after_seq(&conn, &user.id, 2, "dev2", 2).unwrap();
        assert_eq!(remaining.len(), 1);
        assert!(!has_more_rem);
        assert_eq!(remaining[0].seq, 3);
    }
}

