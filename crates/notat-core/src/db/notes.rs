use crate::models::note::Note;
use rusqlite::{Connection, OptionalExtension, Result, Row, params};

pub fn row_to_note(row: &Row) -> Result<Note> {
    Ok(Note {
        id: row.get("id")?,
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
            id, folder_id, title, body, pinned, trashed,
            version, updated_at, created_at, deleted_at,
            device_id, checksum
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12
        )",
        params![
            note.id,
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
        "
            INSERT INTO notes(
                id, folder_id, title, body, pinned, trashed,
                version, updated_at, created_at, deleted_at,
                device_id, checksum
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12
            )
            ON CONFLICT(id) DO UPDATE SET
                folder_id = excluded.folder_id,
                title = excluded.title,
                body = excluded.body,
                pinned = excluded.pinned,
                trashed = excluded.trashed,
                version = excluded.version,
                updated_at = excluded.updated_at,
                deleted_at = excluded.deleted_at,
                device_id = excluded.device_id,
                checksum = excluded.checksum
        ",
        params![
            note.id,
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

pub fn list_active_notes(conn: &Connection) -> Result<Vec<Note>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM notes
         WHERE trashed = 0
         ORDER BY pinned DESC, updated_at DESC",
    )?;
    let notes = stmt
        .query_map([], row_to_note)?
        .collect::<Result<Vec<_>>>()?;
    Ok(notes)
}
