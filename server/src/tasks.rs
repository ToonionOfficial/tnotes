use std::sync::Arc;
use tnotes_core::{
    Connection,
    db::{notes::delete_trashed_older_than, sessions::cleanup_expired_sessions},
    models::current_time_ms,
};
use tokio::sync::Mutex;

pub fn run_housekeeping(conn: &Connection) -> (usize, usize) {
    let now = current_time_ms();
    let thirty_days_ago = now - (30 * 24 * 60 * 60 * 1000);

    let expired_sessions = cleanup_expired_sessions(conn, now).unwrap_or(0);
    let purged_notes = delete_trashed_older_than(conn, thirty_days_ago).unwrap_or(0);

    if expired_sessions > 0 {
        tracing::info!(
            "Housekeeping: cleaned up {} expired sessions",
            expired_sessions
        );
    }
    if purged_notes > 0 {
        tracing::info!(
            "Housekeeping: purged {} trashed notes older than 30 days",
            purged_notes
        );
    }

    (expired_sessions, purged_notes)
}

pub fn start_housekeeping_task(
    db: Arc<Mutex<Connection>>,
    interval_secs: u64,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
        loop {
            interval.tick().await;
            let conn = db.lock().await;
            run_housekeeping(&conn);
        }
    })
}
