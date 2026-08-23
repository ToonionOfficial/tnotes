use rusqlite::{Connection, Result};
use std::path::Path;

const MIGRATIONS: &[(i64, &str)] = &[
    (1, include_str!("../migrations/001_init.sql")),
    (2, include_str!("../migrations/002_changes_table.sql")),
];

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
    let mut curr_version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    for &(version, sql) in MIGRATIONS {
        if version > curr_version {
            let tx = conn.transaction()?;
            tx.execute_batch(sql)?;
            tx.pragma_update(None, "user_version", version)?;
            tx.commit()?;
            curr_version = version;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_create_schema_v2() {
        let conn = open_in_memory().unwrap();
        let version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0)).unwrap();
        assert_eq!(version, 2);

        conn.execute("SELECT seq FROM changes LIMIT 0", []).unwrap();
        conn.execute("SELECT last_seq FROM device_cursors LIMIT 0", []).unwrap();
    }
}
