import { and, eq, isNotNull, sql } from "drizzle-orm"
import { ulid } from "@/utils/id"
import { db } from "../index"
import { type LocalChange, localChanges, notes, syncMeta, users } from "../schema"

const DEFAULT_USER_ID = "default_user"
const DEFAULT_USERNAME = "Local User"

/**
 * Ensures a default user exists in the local database.
 */
export function ensureUser(userId = DEFAULT_USER_ID, username = DEFAULT_USERNAME): string {
  const existing = db.select().from(users).where(eq(users.id, userId)).get()

  if (!existing) {
    db.insert(users)
      .values({
        id: userId,
        username,
        createdAt: Date.now(),
      })
      .onConflictDoUpdate({
        target: users.id,
        set: { username },
      })
      .run()
  }

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
  db.insert(syncMeta)
    .values({ key, value })
    .onConflictDoUpdate({
      target: syncMeta.key,
      set: { value },
    })
    .run()
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
  db.delete(localChanges).where(sql`${localChanges.id} IN ${changeIds}`).run()
}
