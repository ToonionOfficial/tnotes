use rusqlite::{Connection, Result};
use std::path::Path;

const MIGRATION_001: &str = include_str!("../migrations/001_init.sql");

pub fn open_connection(path: impl AsRef<Path>) -> Result<Connection> {
    let mut conn = Connection::open(path)?;
    config_pragmas(&mut conn)?;
    run_migrations(&mut conn)?;
    Ok(conn)
}

pub fn open_in_memory() -> Result<Connection> {
    let mut conn = Connection::open_in_memory()?;
    config_pragmas(&mut conn)?;
    run_migrations(&mut conn)?;
    Ok(conn)
}

fn config_pragmas(conn: &mut Connection) -> Result<()> {
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA busy_timeout = 5000;
        ",
    )?;
    Ok(())
}

pub fn run_migrations(conn: &mut Connection) -> Result<()> {
    let curr_version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    // Migration 1
    if curr_version < 1 {
        let tx = conn.transaction()?;
        tx.execute_batch(MIGRATION_001)?;
        tx.pragma_update(None, "user_version", 1)?;
        tx.commit()?
    }

    Ok(())
}
