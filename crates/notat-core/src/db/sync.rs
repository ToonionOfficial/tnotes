use rusqlite::{params, Connection, OptionalExtension, Result};
use crate::db::{folders::row_to_folder, notes::row_to_note, themes::row_to_theme};
use crate::sync::envelope::{Change, EntityType};

/// Collect all entity changes (notes, folders, themes) that have been modified since `since_timestamp`.
/// Optionally excludes changes that originated from `exclude_device_id`.
pub fn get_changes_since(
    conn: &Connection,
    since_timestamp: i64,
    exclude_device_id: Option<&str>,
) -> Result<Vec<Change>> {
    let mut changes = Vec::new();

    // 1. Collect Note changes
    let notes = match exclude_device_id {
        Some(dev_id) => {
            let mut stmt = conn.prepare(
                "SELECT * FROM notes WHERE updated_at > ?1 AND device_id != ?2 ORDER BY updated_at ASC",
            )?;
            stmt.query_map(params![since_timestamp, dev_id], row_to_note)?
                .collect::<Result<Vec<_>>>()?
        }
        None => {
            let mut stmt = conn.prepare(
                "SELECT * FROM notes WHERE updated_at > ?1 ORDER BY updated_at ASC",
            )?;
            stmt.query_map(params![since_timestamp], row_to_note)?
                .collect::<Result<Vec<_>>>()?
        }
    };

    for note in notes {
        let payload = serde_json::to_value(&note).map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(e))
        })?;

        changes.push(Change {
            entity_type: EntityType::Note,
            entity_id: note.id,
            version: note.version,
            updated_at: note.updated_at,
            tombstone: note.trashed || note.deleted_at.is_some(),
            payload,
        });
    }

    // 2. Collect Folder changes
    let folders = match exclude_device_id {
        Some(dev_id) => {
            let mut stmt = conn.prepare(
                "SELECT * FROM folders WHERE updated_at > ?1 AND device_id != ?2 ORDER BY updated_at ASC",
            )?;
            stmt.query_map(params![since_timestamp, dev_id], row_to_folder)?
                .collect::<Result<Vec<_>>>()?
        }
        None => {
            let mut stmt = conn.prepare(
                "SELECT * FROM folders WHERE updated_at > ?1 ORDER BY updated_at ASC",
            )?;
            stmt.query_map(params![since_timestamp], row_to_folder)?
                .collect::<Result<Vec<_>>>()?
        }
    };

    for folder in folders {
        let payload = serde_json::to_value(&folder).map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(e))
        })?;

        changes.push(Change {
            entity_type: EntityType::Folder,
            entity_id: folder.id,
            version: folder.version,
            updated_at: folder.updated_at,
            tombstone: folder.deleted_at.is_some(),
            payload,
        });
    }

    // 3. Collect Theme changes (custom synced themes)
    let themes = match exclude_device_id {
        Some(dev_id) => {
            let mut stmt = conn.prepare(
                "SELECT * FROM themes WHERE updated_at > ?1 AND device_id != ?2 AND builtin = 0 ORDER BY updated_at ASC",
            )?;
            stmt.query_map(params![since_timestamp, dev_id], row_to_theme)?
                .collect::<Result<Vec<_>>>()?
        }
        None => {
            let mut stmt = conn.prepare(
                "SELECT * FROM themes WHERE updated_at > ?1 AND builtin = 0 ORDER BY updated_at ASC",
            )?;
            stmt.query_map(params![since_timestamp], row_to_theme)?
                .collect::<Result<Vec<_>>>()?
        }
    };

    for theme in themes {
        let payload = serde_json::to_value(&theme).map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(e))
        })?;

        changes.push(Change {
            entity_type: EntityType::Theme,
            entity_id: theme.id,
            version: theme.version,
            updated_at: theme.updated_at,
            tombstone: false,
            payload,
        });
    }

    // Sort all changes chronologically so clients apply them in the correct order
    changes.sort_by_key(|c| c.updated_at);

    Ok(changes)
}

// ─── Sync Metadata Helpers (Client-side cursor storage) ──────────────────────

/// Retrieve a value from `sync_meta`
pub fn get_sync_meta(conn: &Connection, key: &str) -> Result<Option<String>> {
    conn.query_row(
        "SELECT value FROM sync_meta WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .optional()
}

/// Store or update a key-value pair in `sync_meta`
pub fn set_sync_meta(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO sync_meta (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

/// Get the `last_sync_at` timestamp cursor (defaults to 0 if never synced)
pub fn get_last_sync_at(conn: &Connection) -> Result<i64> {
    match get_sync_meta(conn, "last_sync_at")? {
        Some(val) => Ok(val.parse::<i64>().unwrap_or(0)),
        None => Ok(0),
    }
}

/// Update the `last_sync_at` timestamp cursor
pub fn set_last_sync_at(conn: &Connection, timestamp: i64) -> Result<()> {
    set_sync_meta(conn, "last_sync_at", &timestamp.to_string())
}
