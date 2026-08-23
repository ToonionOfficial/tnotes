use crate::models::current_time_ms;
use rusqlite::{Connection, Result, params};
use serde_json::Value;

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
}
