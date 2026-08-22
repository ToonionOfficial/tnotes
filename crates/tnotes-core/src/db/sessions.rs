use crate::models::session::Session;
use rusqlite::{Connection, OptionalExtension, Result, Row, params};

pub fn row_to_session(row: &Row) -> Result<Session> {
    Ok(Session {
        token: row.get("token")?,
        device_id: row.get("device_id")?,
        created_at: row.get("created_at")?,
        expires_at: row.get("expires_at")?,
    })
}

/// Store a new session token
pub fn create_session(conn: &Connection, session: &Session) -> Result<()> {
    conn.execute(
        "INSERT INTO sessions (token, device_id, created_at, expires_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            session.token,
            session.device_id,
            session.created_at,
            session.expires_at,
        ],
    )?;
    Ok(())
}

/// Fetch a session by its token
pub fn get_session(conn: &Connection, token: &str) -> Result<Option<Session>> {
    conn.query_row(
        "SELECT * FROM sessions WHERE token = ?1",
        params![token],
        row_to_session,
    )
    .optional()
}

/// Delete a session (logout)
pub fn delete_session(conn: &Connection, token: &str) -> Result<()> {
    conn.execute("DELETE FROM sessions WHERE token = ?1", params![token])?;
    Ok(())
}

/// Delete all sessions associated with a specific device
pub fn delete_sessions_by_device(conn: &Connection, device_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM sessions WHERE device_id = ?1",
        params![device_id],
    )?;
    Ok(())
}

/// Clean up expired sessions from the database
pub fn cleanup_expired_sessions(conn: &Connection, now_ms: i64) -> Result<usize> {
    let count = conn.execute(
        "DELETE FROM sessions WHERE expires_at < ?1",
        params![now_ms],
    )?;
    Ok(count)
}
