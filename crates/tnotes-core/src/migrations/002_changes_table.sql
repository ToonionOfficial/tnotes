CREATE TABLE IF NOT EXISTS changes (
    seq               INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id           TEXT    NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    entity_type       TEXT    NOT NULL,
    entity_id         TEXT    NOT NULL,
    device_id         TEXT    NOT NULL,
    entity_version    INTEGER NOT NULL,
    entity_updated_at INTEGER NOT NULL,
    is_tombstone      INTEGER NOT NULL DEFAULT 0,
    payload           TEXT    NOT NULL,
    created_at        INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_changes_user_seq ON changes(user_id, seq);
CREATE INDEX IF NOT EXISTS idx_changes_entity ON changes(entity_id, entity_type);

CREATE TABLE IF NOT EXISTS device_cursors (
    device_id    TEXT PRIMARY KEY REFERENCES devices(id) ON DELETE CASCADE,
    last_seq     INTEGER NOT NULL DEFAULT 0,
    last_sync_at INTEGER NOT NULL DEFAULT 0
);
