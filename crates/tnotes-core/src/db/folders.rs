use crate::models::folder::Folder;
use rusqlite::{Connection, OptionalExtension, Result, Row, params};
use serde::{Deserialize, Serialize};

/// Represents a folder with its hierarchical depth and computed path in the tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderNode {
    pub folder: Folder,
    pub depth: i64,
    pub path: String,
}

pub fn row_to_folder(row: &Row) -> Result<Folder> {
    Ok(Folder {
        id: row.get("id")?,
        user_id: row.get("user_id")?,
        parent_id: row.get("parent_id")?,
        name: row.get("name")?,
        icon: row.get("icon")?,
        sort_order: row.get("sort_order")?,
        version: row.get::<_, i64>("version")? as u64,
        updated_at: row.get("updated_at")?,
        created_at: row.get("created_at")?,
        deleted_at: row.get("deleted_at")?,
        device_id: row.get("device_id")?,
    })
}

/// Insert a new folder
pub fn insert_folder(conn: &Connection, folder: &Folder) -> Result<()> {
    conn.execute(
        "INSERT INTO folders (
            id, user_id, parent_id, name, icon, sort_order,
            version, updated_at, created_at, deleted_at, device_id
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11
        )",
        params![
            folder.id,
            folder.user_id,
            folder.parent_id,
            folder.name,
            folder.icon,
            folder.sort_order,
            folder.version as i64,
            folder.updated_at,
            folder.created_at,
            folder.deleted_at,
            folder.device_id,
        ],
    )?;
    Ok(())
}

/// Upsert a folder (used by save operations and sync engine)
pub fn upsert_folder(conn: &Connection, folder: &Folder) -> Result<()> {
    conn.execute(
        "INSERT INTO folders (
            id, user_id, parent_id, name, icon, sort_order,
            version, updated_at, created_at, deleted_at, device_id
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11
        )
        ON CONFLICT(id) DO UPDATE SET
            user_id = excluded.user_id,
            parent_id = excluded.parent_id,
            name = excluded.name,
            icon = excluded.icon,
            sort_order = excluded.sort_order,
            version = excluded.version,
            updated_at = excluded.updated_at,
            deleted_at = excluded.deleted_at,
            device_id = excluded.device_id",
        params![
            folder.id,
            folder.user_id,
            folder.parent_id,
            folder.name,
            folder.icon,
            folder.sort_order,
            folder.version as i64,
            folder.updated_at,
            folder.created_at,
            folder.deleted_at,
            folder.device_id,
        ],
    )?;
    Ok(())
}

/// Get a single folder by ID
pub fn get_folder_by_id(conn: &Connection, id: &str) -> Result<Option<Folder>> {
    conn.query_row(
        "SELECT * FROM folders WHERE id = ?1",
        params![id],
        row_to_folder,
    )
    .optional()
}

/// List all active (non-deleted) folders for a specific user
pub fn list_all_folders(conn: &Connection, user_id: &str) -> Result<Vec<Folder>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM folders
         WHERE user_id = ?1 AND deleted_at IS NULL
         ORDER BY sort_order ASC, name ASC",
    )?;
    let folders = stmt
        .query_map(params![user_id], row_to_folder)?
        .collect::<Result<Vec<_>>>()?;
    Ok(folders)
}

/// List direct subfolders of a parent for a specific user (or root folders if `parent_id` is None)
pub fn list_subfolders(
    conn: &Connection,
    user_id: &str,
    parent_id: Option<&str>,
) -> Result<Vec<Folder>> {
    match parent_id {
        Some(pid) => {
            let mut stmt = conn.prepare(
                "SELECT * FROM folders
                 WHERE user_id = ?1 AND parent_id = ?2 AND deleted_at IS NULL
                 ORDER BY sort_order ASC, name ASC",
            )?;
            let folders = stmt
                .query_map(params![user_id, pid], row_to_folder)?
                .collect::<Result<Vec<_>>>()?;
            Ok(folders)
        }
        None => {
            let mut stmt = conn.prepare(
                "SELECT * FROM folders
                 WHERE user_id = ?1 AND parent_id IS NULL AND deleted_at IS NULL
                 ORDER BY sort_order ASC, name ASC",
            )?;
            let folders = stmt
                .query_map(params![user_id], row_to_folder)?
                .collect::<Result<Vec<_>>>()?;
            Ok(folders)
        }
    }
}

/// Permanently delete a folder (hard delete)
pub fn delete_folder_permanently(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM folders WHERE id = ?1", params![id])?;
    Ok(())
}

/// Fetch the complete hierarchical folder tree for a specific user using a recursive CTE query
pub fn get_folder_tree(conn: &Connection, user_id: &str) -> Result<Vec<FolderNode>> {
    let mut stmt = conn.prepare(
        "WITH RECURSIVE folder_tree AS (
            SELECT 
                id, user_id, parent_id, name, icon, sort_order, version,
                updated_at, created_at, deleted_at, device_id,
                0 AS depth,
                name AS path
            FROM folders
            WHERE user_id = ?1 AND parent_id IS NULL AND deleted_at IS NULL

            UNION ALL

            SELECT 
                f.id, f.user_id, f.parent_id, f.name, f.icon, f.sort_order, f.version,
                f.updated_at, f.created_at, f.deleted_at, f.device_id,
                ft.depth + 1 AS depth,
                ft.path || ' / ' || f.name AS path
            FROM folders f
            JOIN folder_tree ft ON f.parent_id = ft.id
            WHERE f.user_id = ?1 AND f.deleted_at IS NULL
        )
        SELECT * FROM folder_tree ORDER BY path ASC, sort_order ASC",
    )?;

    let nodes = stmt
        .query_map(params![user_id], |row| {
            Ok(FolderNode {
                folder: row_to_folder(row)?,
                depth: row.get("depth")?,
                path: row.get("path")?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(nodes)
}

/// Returns total non-deleted folders count for a user
pub fn count_active_folders_for_user(conn: &Connection, user_id: &str) -> Result<usize> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM folders WHERE user_id = ?1 AND deleted_at IS NULL",
        params![user_id],
        |row| row.get(0),
    )?;
    Ok(count as usize)
}
