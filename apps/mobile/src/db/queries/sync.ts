import { and, eq, inArray, isNotNull } from "drizzle-orm"
import { ulid } from "@/utils/id"
import { db } from "../index"
import { type LocalChange, localChanges, notes, syncMeta, users } from "../schema"

const DEFAULT_USER_ID = "default_user"
const DEFAULT_USERNAME = "Local User"

/**
 * Inserts or updates a user row.
 *
 * `ON CONFLICT(id) DO UPDATE` alone cannot fix this: `users.username` is
 * UNIQUE, so inserting a new id for an already-known username (e.g. the
 * server database was reset and re-issued ids, leaving an orphaned row from
 * an earlier pairing) fails with `UNIQUE constraint failed: users.username`.
 * Resolve both shapes explicitly: same id -> refresh username, same username
 * under a stale id -> drop the stale row so the insert cannot conflict.
 */
export function upsertUser(userId: string, username: string): void {
  const byId = db.select().from(users).where(eq(users.id, userId)).get()
  if (byId) {
    if (byId.username !== username) {
      db.update(users).set({ username }).where(eq(users.id, userId)).run()
    }
    return
  }

  const byUsername = db.select().from(users).where(eq(users.username, username)).get()
  if (byUsername) {
    db.delete(users).where(eq(users.id, byUsername.id)).run()
  }

  try {
    db.insert(users).values({ id: userId, username, createdAt: Date.now() }).run()
  } catch {
    // Lost a race between the selects above and the insert: the row now
    // exists, so fall back to an update.
    db.update(users).set({ username }).where(eq(users.id, userId)).run()
  }
}

/**
 * Ensures a default user exists in the local database.
 */
export function ensureUser(userId = DEFAULT_USER_ID, username = DEFAULT_USERNAME): string {
  upsertUser(userId, username)

  // Ensure soft-deleted notes are properly flagged as trashed
  db.update(notes)
    .set({ trashed: true })
    .where(and(isNotNull(notes.deletedAt), eq(notes.trashed, false)))
    .run()

  return userId
}

/**
 * Retrieves the current active user ID.
 */
export function getCurrentUserId(): string {
  const stored = getSyncMeta("user_id")
  if (stored) return stored

  return ensureUser()
}

/**
 * Retrieves the device ID or generates a new ULID and stores it.
 */
export function getOrCreateDeviceId(): string {
  const stored = getSyncMeta("device_id")
  if (stored) return stored

  const newId = ulid()
  setSyncMeta("device_id", newId)
  return newId
}

// Sync metadata operations
export function getSyncMeta(key: string): string | null {
  const result = db.select().from(syncMeta).where(eq(syncMeta.key, key)).get()
  return result?.value ?? null
}

export function setSyncMeta(key: string, value: string): void {
  // Insert-then-update: avoids `ON CONFLICT(<table-qualified column>)`, which
  // older bundled SQLite versions reject in the conflict target.
  db.insert(syncMeta).values({ key, value }).onConflictDoNothing().run()
  db.update(syncMeta).set({ value }).where(eq(syncMeta.key, key)).run()
}

export function getSyncCredentials(): {
  serverUrl: string | null
  token: string | null
  deviceId: string | null
  userId: string | null
} {
  return {
    serverUrl: getSyncMeta("server_url"),
    token: getSyncMeta("auth_token"),
    deviceId: getSyncMeta("device_id"),
    userId: getSyncMeta("user_id"),
  }
}

export function setSyncCredentials(creds: {
  serverUrl: string
  token: string
  deviceId: string
  userId: string
}): void {
  setSyncMeta("server_url", creds.serverUrl)
  setSyncMeta("auth_token", creds.token)
  setSyncMeta("device_id", creds.deviceId)
  setSyncMeta("user_id", creds.userId)
}

export function recordLocalChange(
  entityType: "note" | "folder",
  entityId: string,
  version: number,
  updatedAt: number,
  tombstone: boolean,
  payload: object,
): void {
  db.insert(localChanges)
    .values({
      entityType,
      entityId,
      version,
      updatedAt,
      tombstone,
      payload: JSON.stringify(payload),
      createdAt: Date.now(),
    })
    .run()
}

export function getPendingLocalChanges(): LocalChange[] {
  return db.select().from(localChanges).orderBy(localChanges.id).all()
}

export function clearLocalChanges(changeIds: number[]): void {
  if (changeIds.length === 0) return
  db.delete(localChanges).where(inArray(localChanges.id, changeIds)).run()
}

const FREQUENT_FOLDER_ORDER_KEY = "frequent_folder_order"

/**
 * Device-local user arrangement of the "Frequently Used" home section.
 * Intentionally local-only (sync_meta, never local_changes): no migration,
 * no sync payload, no server changes.
 */
export function getFrequentFolderOrder(): string[] {
  const stored = getSyncMeta(FREQUENT_FOLDER_ORDER_KEY)
  if (!stored) return []
  try {
    const parsed: unknown = JSON.parse(stored)
    if (!Array.isArray(parsed)) return []
    return parsed.filter((id): id is string => typeof id === "string")
  } catch {
    return []
  }
}

export function setFrequentFolderOrder(ids: string[]): void {
  setSyncMeta(FREQUENT_FOLDER_ORDER_KEY, JSON.stringify(ids))
}
