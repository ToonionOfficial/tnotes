use crate::models::theme::{Theme, ThemeSchema};
use rusqlite::{Connection, OptionalExtension, Result, Row, params};

pub fn row_to_theme(row: &Row) -> Result<Theme> {
    let raw_schema: String = row.get("schema")?;
    let schema: ThemeSchema = serde_json::from_str(&raw_schema).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    Ok(Theme {
        id: row.get("id")?,
        user_id: row.get("user_id")?,
        name: row.get("name")?,
        builtin: row.get::<_, i64>("builtin")? != 0,
        schema,
        version: row.get::<_, i64>("version")? as u64,
        updated_at: row.get("updated_at")?,
        created_at: row.get("created_at")?,
        device_id: row.get("device_id")?,
    })
}

/// Insert a new theme
pub fn insert_theme(conn: &Connection, theme: &Theme) -> Result<()> {
    let schema_json = serde_json::to_string(&theme.schema)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

    conn.execute(
        "INSERT INTO themes (id, user_id, name, builtin, schema, version, updated_at, created_at, device_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            theme.id,
            theme.user_id,
            theme.name,
            theme.builtin as i64,
            schema_json,
            theme.version as i64,
            theme.updated_at,
            theme.created_at,
            theme.device_id,
        ],
    )?;
    Ok(())
}

/// Upsert a theme (used by sync engine)
pub fn upsert_theme(conn: &Connection, theme: &Theme) -> Result<()> {
    let schema_json = serde_json::to_string(&theme.schema)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

    conn.execute(
        "INSERT INTO themes (id, user_id, name, builtin, schema, version, updated_at, created_at, device_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(id) DO UPDATE SET
             user_id = excluded.user_id,
             name = excluded.name,
             builtin = excluded.builtin,
             schema = excluded.schema,
             version = excluded.version,
             updated_at = excluded.updated_at,
             device_id = excluded.device_id",
        params![
            theme.id,
            theme.user_id,
            theme.name,
            theme.builtin as i64,
            schema_json,
            theme.version as i64,
            theme.updated_at,
            theme.created_at,
            theme.device_id,
        ],
    )?;
    Ok(())
}

/// Get a theme by ID
pub fn get_theme_by_id(conn: &Connection, id: &str) -> Result<Option<Theme>> {
    conn.query_row(
        "SELECT * FROM themes WHERE id = ?1",
        params![id],
        row_to_theme,
    )
    .optional()
}

/// List themes accessible to a user (all built-in themes plus custom themes created by this user)
pub fn list_themes_for_user(conn: &Connection, user_id: Option<&str>) -> Result<Vec<Theme>> {
    match user_id {
        Some(uid) => {
            let mut stmt = conn.prepare(
                "SELECT * FROM themes WHERE builtin = 1 OR user_id = ?1 ORDER BY builtin DESC, name ASC",
            )?;
            let themes = stmt
                .query_map(params![uid], row_to_theme)?
                .collect::<Result<Vec<_>>>()?;
            Ok(themes)
        }
        None => {
            let mut stmt =
                conn.prepare("SELECT * FROM themes WHERE builtin = 1 ORDER BY name ASC")?;
            let themes = stmt
                .query_map([], row_to_theme)?
                .collect::<Result<Vec<_>>>()?;
            Ok(themes)
        }
    }
}

/// List all themes (built-in first, then custom themes by name)
pub fn list_themes(conn: &Connection) -> Result<Vec<Theme>> {
    let mut stmt = conn.prepare("SELECT * FROM themes ORDER BY builtin DESC, name ASC")?;
    let themes = stmt
        .query_map([], row_to_theme)?
        .collect::<Result<Vec<_>>>()?;
    Ok(themes)
}

/// Delete a theme (cannot delete built-in themes)
pub fn delete_theme(conn: &Connection, id: &str) -> Result<bool> {
    let affected = conn.execute(
        "DELETE FROM themes WHERE id = ?1 AND builtin = 0",
        params![id],
    )?;
    Ok(affected > 0)
}

/// Seed the built-in Light and Dark themes if they don't exist in the DB
pub fn seed_default_themes(conn: &Connection) -> Result<()> {
    let dark = Theme::builtin_dark();
    let light = Theme::builtin_light();

    upsert_theme(conn, &dark)?;
    upsert_theme(conn, &light)?;

    Ok(())
}
