use crate::models::device::Device;
use rusqlite::{params, Connection, OptionalExtension, Result, Row};

pub fn row_to_device(row: &Row) -> Result<Device> {
    Ok(Device {
        id: row.get("id")?,
        name: row.get("name")?,
        platform: row.get("platform")?,
        last_seen_at: row.get("last_seen_at")?,
        created_at: row.get("created_at")?,
    })
}

/// Register or update a device
pub fn upsert_device(conn: &Connection, device: &Device) -> Result<()> {
    conn.execute(
        "INSERT INTO devices (id, name, platform, last_seen_at, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(id) DO UPDATE SET
             name = excluded.name,
             platform = excluded.platform,
             last_seen_at = excluded.last_seen_at",
        params![
            device.id,
            device.name,
            device.platform,
            device.last_seen_at,
            device.created_at,
        ],
    )?;
    Ok(())
}

/// Get a device by ID
pub fn get_device_by_id(conn: &Connection, id: &str) -> Result<Option<Device>> {
    conn.query_row(
        "SELECT * FROM devices WHERE id = ?1",
        params![id],
        row_to_device,
    )
    .optional()
}

/// Update the `last_seen_at` timestamp of a device
pub fn touch_device(conn: &Connection, id: &str, last_seen_at: i64) -> Result<()> {
    conn.execute(
        "UPDATE devices SET last_seen_at = ?1 WHERE id = ?2",
        params![last_seen_at, id],
    )?;
    Ok(())
}

/// List all registered devices, ordered by most recently active
pub fn list_devices(conn: &Connection) -> Result<Vec<Device>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM devices ORDER BY last_seen_at DESC",
    )?;
    let devices = stmt
        .query_map([], row_to_device)?
        .collect::<Result<Vec<_>>>()?;
    Ok(devices)
}

/// Delete a device
pub fn delete_device(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM devices WHERE id = ?1", params![id])?;
    Ok(())
}
