use crate::models::note::Note;
use rusqlite::{params, Connection, OptionalExtension, Result, Row};

pub fn row_to_note(row: &Row) -> Result<Note> {
    Ok(Note {
        id: row.get("id")?,
        user_id: row.get("user_id")?,
        folder_id: row.get("folder_id")?,
        title: row.get("title")?,
        body: row.get("body")?,
        pinned: row.get::<_, i64>("pinned")? != 0,
        trashed: row.get::<_, i64>("trashed")? != 0,
        version: row.get::<_, i64>("version")? as u64,
        updated_at: row.get("updated_at")?,
        created_at: row.get("created_at")?,
        deleted_at: row.get("deleted_at")?,
        device_id: row.get("device_id")?,
        checksum: row.get("checksum")?,
    })
}

pub fn insert_note(conn: &Connection, note: &Note) -> Result<()> {
    conn.execute(
        "INSERT INTO notes (
            id, user_id, folder_id, title, body, pinned, trashed,
            version, updated_at, created_at, deleted_at,
            device_id, checksum
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13
        )",
        params![
            note.id,
            note.user_id,
            note.folder_id,
            note.title,
            note.body,
            note.pinned as i64,
            note.trashed as i64,
            note.version as i64,
            note.updated_at,
            note.created_at,
            note.deleted_at,
            note.device_id,
            note.checksum,
        ],
    )?;
    Ok(())
}

pub fn upsert_note(conn: &Connection, note: &Note) -> Result<()> {
    conn.execute(
        "INSERT INTO notes (
            id, user_id, folder_id, title, body, pinned, trashed,
            version, updated_at, created_at, deleted_at,
            device_id, checksum
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13
        )
        ON CONFLICT(id) DO UPDATE SET
            user_id = excluded.user_id,
            folder_id = excluded.folder_id,
            title = excluded.title,
            body = excluded.body,
            pinned = excluded.pinned,
            trashed = excluded.trashed,
            version = excluded.version,
            updated_at = excluded.updated_at,
            deleted_at = excluded.deleted_at,
            device_id = excluded.device_id,
            checksum = excluded.checksum",
        params![
            note.id,
            note.user_id,
            note.folder_id,
            note.title,
            note.body,
            note.pinned as i64,
            note.trashed as i64,
            note.version as i64,
            note.updated_at,
            note.created_at,
            note.deleted_at,
            note.device_id,
            note.checksum,
        ],
    )?;
    Ok(())
}

pub fn get_note_by_id(conn: &Connection, id: &str) -> Result<Option<Note>> {
    conn.query_row(
        "SELECT * FROM notes WHERE id = ?1",
        params![id],
        row_to_note,
    )
    .optional()
}

pub fn list_active_notes(conn: &Connection, user_id: &str) -> Result<Vec<Note>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM notes
         WHERE user_id = ?1 AND trashed = 0
         ORDER BY pinned DESC, updated_at DESC",
    )?;
    let notes = stmt
        .query_map(params![user_id], row_to_note)?
        .collect::<Result<Vec<_>>>()?;
    Ok(notes)
}

/// List active notes inside a specific folder (or root notes if `folder_id` is None)
pub fn list_notes_by_folder(conn: &Connection, user_id: &str, folder_id: Option<&str>) -> Result<Vec<Note>> {
    match folder_id {
        Some(fid) => {
            let mut stmt = conn.prepare(
                "SELECT * FROM notes
                 WHERE user_id = ?1 AND folder_id = ?2 AND trashed = 0
                 ORDER BY pinned DESC, updated_at DESC",
            )?;
            let notes = stmt.query_map(params![user_id, fid], row_to_note)?.collect::<Result<Vec<_>>>()?;
            Ok(notes)
        }
        None => {
            let mut stmt = conn.prepare(
                "SELECT * FROM notes
                 WHERE user_id = ?1 AND folder_id IS NULL AND trashed = 0
                 ORDER BY pinned DESC, updated_at DESC",
            )?;
            let notes = stmt.query_map(params![user_id], row_to_note)?.collect::<Result<Vec<_>>>()?;
            Ok(notes)
        }
    }
}

/// List all trashed notes for a specific user
pub fn list_trashed_notes(conn: &Connection, user_id: &str) -> Result<Vec<Note>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM notes
         WHERE user_id = ?1 AND trashed = 1
         ORDER BY deleted_at DESC",
    )?;
    let notes = stmt
        .query_map(params![user_id], row_to_note)?
        .collect::<Result<Vec<_>>>()?;
    Ok(notes)
}

/// Permanently delete a note (hard delete)
pub fn delete_note_permanently(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
    Ok(())
}

/// Permanently delete all trashed notes older than `threshold_ms` (e.g. 30 days old)
pub fn delete_trashed_older_than(conn: &Connection, threshold_ms: i64) -> Result<usize> {
    let count = conn.execute(
        "DELETE FROM notes WHERE trashed = 1 AND deleted_at IS NOT NULL AND deleted_at < ?1",
        params![threshold_ms],
    )?;
    Ok(count)
}

/// Full-Text Search across notes for a specific user using SQLite FTS5 index
pub fn search_notes(conn: &Connection, user_id: &str, search_term: &str) -> Result<Vec<Note>> {
    let trimmed = search_term.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    // Safely tokenize and quote each word as a prefix match (e.g. "rust"* "metal"*)
    // to prevent FTS5 syntax errors on special characters or operators
    let tokens: Vec<String> = trimmed
        .split_whitespace()
        .map(|word| format!("\"{}\"*", word.replace('"', "\"\"")))
        .collect();

    let formatted_query = tokens.join(" ");

    let mut stmt = conn.prepare(
        "SELECT n.* FROM notes n
         JOIN notes_fts fts ON fts.rowid = n.rowid
         WHERE notes_fts MATCH ?1 AND n.user_id = ?2 AND n.trashed = 0
         ORDER BY bm25(notes_fts) ASC",
    )?;

    let notes = stmt
        .query_map(params![formatted_query, user_id], row_to_note)?
        .collect::<Result<Vec<_>>>()?;
    Ok(notes)
}
