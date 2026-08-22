use crate::models::user::User;
use rusqlite::{params, Connection, OptionalExtension, Result, Row};

pub fn row_to_user(row: &Row) -> Result<User> {
    Ok(User {
        id: row.get("id")?,
        username: row.get("username")?,
        password_hash: row.get("password_hash")?,
        created_at: row.get("created_at")?,
    })
}

/// Create a new user account
pub fn create_user(conn: &Connection, user: &User) -> Result<()> {
    conn.execute(
        "INSERT INTO users (id, username, password_hash, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![user.id, user.username, user.password_hash, user.created_at],
    )?;
    Ok(())
}

/// Fetch user by username (used during login)
pub fn get_user_by_username(conn: &Connection, username: &str) -> Result<Option<User>> {
    conn.query_row(
        "SELECT * FROM users WHERE username = ?1",
        params![username],
        row_to_user,
    )
    .optional()
}

/// Fetch user by ID
pub fn get_user_by_id(conn: &Connection, id: &str) -> Result<Option<User>> {
    conn.query_row(
        "SELECT * FROM users WHERE id = ?1",
        params![id],
        row_to_user,
    )
    .optional()
}

/// Check if any user exists (used by server to decide whether to show setup wizard or login)
pub fn has_any_user(conn: &Connection) -> Result<bool> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
    Ok(count > 0)
}

/// Update user password hash
pub fn update_password(conn: &Connection, user_id: &str, new_password_hash: &str) -> Result<()> {
    conn.execute(
        "UPDATE users SET password_hash = ?1 WHERE id = ?2",
        params![new_password_hash, user_id],
    )?;
    Ok(())
}
