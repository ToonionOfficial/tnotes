import { and, desc, eq, isNull, sql } from "drizzle-orm"
import { computeChecksum } from "@/utils/crypto"
import { ulid } from "@/utils/id"
import { db } from "./index"
import {
  type Folder,
  folders,
  type LocalChange,
  localChanges,
  type Note,
  notes,
  syncMeta,
  users,
} from "./schema"

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
      .run()
  }

  return userId
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

/**
 * Retrieves the current active user ID.
 */
export function getCurrentUserId(): string {
  const stored = getSyncMeta("user_id")
  if (stored) return stored

  return ensureUser()
}

// Sync stuff
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

function recordLocalChange(
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

export interface NoteFilters {
  folderId?: string | null
  trashed?: boolean
  pinned?: boolean
  search?: string
}

export interface SearchResult extends Note {
  snippet?: string
}

/**
 * High-performance full-text search across notes using SQLite FTS5 and BM25 relevance ranking.
 */
export function searchNotes(
  query: string,
  filters?: { folderId?: string | null; trashed?: boolean; limit?: number },
): SearchResult[] {
  const trimmed = query.trim()
  if (!trimmed) return []

  // Sanitize for FTS5 syntax & tokenize
  const tokens = trimmed
    .replace(/['"*^(){}:~-]/g, " ")
    .split(/\s+/)
    .filter((t) => t.length > 0)

  if (tokens.length === 0) return []

  // Prefix match each word: "rea nat" -> '"rea"* "nat"*'
  const matchQuery = tokens.map((t) => `"${t}"*`).join(" ")
  const trashedVal = filters?.trashed ? 1 : 0
  const limit = filters?.limit ?? 50

  let rows: Array<Record<string, unknown>> = []

  if (filters?.folderId !== undefined) {
    if (filters.folderId === null) {
      rows = db.all(sql`
        SELECT
          n.id,
          n.user_id AS userId,
          n.folder_id AS folderId,
          n.title,
          substr(n.body, 1, 300) AS body,
          n.pinned,
          n.trashed,
          n.version,
          n.created_at AS createdAt,
          n.updated_at AS updatedAt,
          n.deleted_at AS deletedAt,
          n.device_id AS deviceId,
          n.checksum,
          snippet(notes_fts, 1, '<mark>', '</mark>', '...', 20) AS snippet
        FROM notes n
        JOIN notes_fts f ON n.rowid = f.rowid
        WHERE notes_fts MATCH ${matchQuery}
          AND n.trashed = ${trashedVal}
          AND n.folder_id IS NULL
        ORDER BY f.rank
        LIMIT ${limit}
      `)
    } else {
      rows = db.all(sql`
        SELECT
          n.id,
          n.user_id AS userId,
          n.folder_id AS folderId,
          n.title,
          substr(n.body, 1, 300) AS body,
          n.pinned,
          n.trashed,
          n.version,
          n.created_at AS createdAt,
          n.updated_at AS updatedAt,
          n.deleted_at AS deletedAt,
          n.device_id AS deviceId,
          n.checksum,
          snippet(notes_fts, 1, '<mark>', '</mark>', '...', 20) AS snippet
        FROM notes n
        JOIN notes_fts f ON n.rowid = f.rowid
        WHERE notes_fts MATCH ${matchQuery}
          AND n.trashed = ${trashedVal}
          AND n.folder_id = ${filters.folderId}
        ORDER BY f.rank
        LIMIT ${limit}
      `)
    }
  } else {
    rows = db.all(sql`
      SELECT
        n.id,
        n.user_id AS userId,
        n.folder_id AS folderId,
        n.title,
        substr(n.body, 1, 300) AS body,
        n.pinned,
        n.trashed,
        n.version,
        n.created_at AS createdAt,
        n.updated_at AS updatedAt,
        n.deleted_at AS deletedAt,
        n.device_id AS deviceId,
        n.checksum,
        snippet(notes_fts, 1, '<mark>', '</mark>', '...', 20) AS snippet
      FROM notes n
      JOIN notes_fts f ON n.rowid = f.rowid
      WHERE notes_fts MATCH ${matchQuery}
        AND n.trashed = ${trashedVal}
      ORDER BY f.rank
      LIMIT ${limit}
    `)
  }

  return rows.map((row) => ({
    id: String(row.id),
    userId: String(row.userId),
    folderId: row.folderId ? String(row.folderId) : null,
    title: String(row.title ?? ""),
    body: String(row.body ?? ""),
    pinned: Boolean(row.pinned),
    trashed: Boolean(row.trashed),
    version: Number(row.version),
    createdAt: Number(row.createdAt),
    updatedAt: Number(row.updatedAt),
    deletedAt: row.deletedAt ? Number(row.deletedAt) : null,
    deviceId: String(row.deviceId),
    checksum: String(row.checksum ?? ""),
    snippet: row.snippet ? String(row.snippet) : undefined,
  }))
}

export interface FolderNoteCounts {
  total: number
  byFolder: Record<string, number>
  trash: number
}

/**
 * Returns aggregated note counts (total active, per folder, and trash) using fast SQLite index scans.
 */
export function getFolderNoteCounts(): FolderNoteCounts {
  const rows = db.all<{
    folderId: string | null
    activeCount: number
    trashCount: number
  }>(sql`
    SELECT
      folder_id AS folderId,
      COUNT(CASE WHEN trashed = 0 THEN 1 END) AS activeCount,
      COUNT(CASE WHEN trashed = 1 THEN 1 END) AS trashCount
    FROM notes
    GROUP BY folder_id
  `)

  let total = 0
  let trash = 0
  const byFolder: Record<string, number> = {}

  for (const row of rows) {
    const active = Number(row.activeCount)
    const trashed = Number(row.trashCount)

    total += active
    trash += trashed

    if (row.folderId) {
      byFolder[row.folderId] = active
    }
  }

  return {
    total,
    byFolder,
    trash,
  }
}

export function getNotes(filters?: NoteFilters): Note[] {
  // If search is requested, delegate to FTS5 for blazing speed
  if (filters?.search && filters.search.trim().length > 0) {
    return searchNotes(filters.search, {
      folderId: filters.folderId,
      trashed: filters.trashed,
    })
  }

  const conditions = []

  // Trashed condition (default: not trashed)
  if (filters?.trashed !== undefined) {
    conditions.push(eq(notes.trashed, filters.trashed))
  } else {
    conditions.push(eq(notes.trashed, false))
  }

  // Folder condition
  if (filters?.folderId !== undefined) {
    if (filters.folderId === null) {
      conditions.push(isNull(notes.folderId))
    } else {
      conditions.push(eq(notes.folderId, filters.folderId))
    }
  }

  // Pinned condition
  if (filters?.pinned !== undefined) {
    conditions.push(eq(notes.pinned, filters.pinned))
  }

  return db
    .select({
      id: notes.id,
      userId: notes.userId,
      folderId: notes.folderId,
      title: notes.title,
      body: sql<string>`substr(${notes.body}, 1, 300)`,
      pinned: notes.pinned,
      trashed: notes.trashed,
      version: notes.version,
      createdAt: notes.createdAt,
      updatedAt: notes.updatedAt,
      deletedAt: notes.deletedAt,
      deviceId: notes.deviceId,
      checksum: notes.checksum,
    })
    .from(notes)
    .where(conditions.length > 0 ? and(...conditions) : undefined)
    .orderBy(
      ...(filters?.trashed ? [desc(notes.updatedAt)] : [desc(notes.pinned), desc(notes.updatedAt)]),
    )
    .all()
}

export function getNoteById(id: string): Note | null {
  const result = db.select().from(notes).where(eq(notes.id, id)).get()
  return result ?? null
}

export function createNote(input: {
  title?: string
  body?: string
  folderId?: string | null
  pinned?: boolean
}): Note {
  const userId = getCurrentUserId()
  const deviceId = getOrCreateDeviceId()

  const now = Date.now()
  const id = ulid()
  const body = input.body ?? ""
  const title = input.title ?? ""
  const checksum = computeChecksum(body)

  const newNote: Note = {
    id,
    userId,
    folderId: input.folderId ?? null,
    title,
    body,
    pinned: input.pinned ?? false,
    trashed: false,
    version: 1,
    createdAt: now,
    updatedAt: now,
    deletedAt: null,
    deviceId,
    checksum,
  }

  db.insert(notes).values(newNote).run()
  recordLocalChange("note", id, 1, now, false, newNote)

  return newNote
}

export function updateNote(
  id: string,
  input: {
    title?: string
    body?: string
    folderId?: string | null
    pinned?: boolean
  },
): Note {
  const existing = getNoteById(id)
  if (!existing) throw new Error(`Note not found: ${id}`)

  const deviceId = getOrCreateDeviceId()
  const now = Date.now()
  const nextVersion = existing.version + 1
  const body = input.body !== undefined ? input.body : existing.body
  const checksum = input.body !== undefined ? computeChecksum(body) : existing.checksum

  const updatedNote: Note = {
    ...existing,
    title: input.title !== undefined ? input.title : existing.title,
    body,
    folderId: input.folderId !== undefined ? input.folderId : existing.folderId,
    pinned: input.pinned !== undefined ? input.pinned : existing.pinned,
    version: nextVersion,
    updatedAt: now,
    deviceId,
    checksum,
  }

  db.update(notes).set(updatedNote).where(eq(notes.id, id)).run()
  recordLocalChange("note", id, nextVersion, now, false, updatedNote)

  return updatedNote
}

export function togglePinNote(id: string): Note {
  const existing = getNoteById(id)
  if (!existing) throw new Error(`Note not found: ${id}`)

  return updateNote(id, { pinned: !existing.pinned })
}

export function trashNote(id: string): void {
  const existing = getNoteById(id)
  if (!existing) throw new Error(`Note not found: ${id}`)

  const deviceId = getOrCreateDeviceId()
  const now = Date.now()
  const nextVersion = existing.version + 1

  const updatedNote: Note = {
    ...existing,
    trashed: true,
    deletedAt: now,
    version: nextVersion,
    updatedAt: now,
    deviceId,
  }

  db.update(notes).set(updatedNote).where(eq(notes.id, id)).run()
  recordLocalChange("note", id, nextVersion, now, true, updatedNote)
}

export function restoreNote(id: string): void {
  const existing = getNoteById(id)
  if (!existing) throw new Error(`Note not found: ${id}`)

  const deviceId = getOrCreateDeviceId()
  const now = Date.now()
  const nextVersion = existing.version + 1

  const updatedNote: Note = {
    ...existing,
    trashed: false,
    deletedAt: null,
    version: nextVersion,
    updatedAt: now,
    deviceId,
  }

  db.update(notes).set(updatedNote).where(eq(notes.id, id)).run()
  recordLocalChange("note", id, nextVersion, now, false, updatedNote)
}

export function deleteNotePermanently(id: string): void {
  const existing = getNoteById(id)
  if (!existing) return

  const now = Date.now()
  const nextVersion = existing.version + 1

  db.delete(notes).where(eq(notes.id, id)).run()
  recordLocalChange("note", id, nextVersion, now, true, {
    ...existing,
    trashed: true,
    deletedAt: now,
    version: nextVersion,
    updatedAt: now,
  })
}

export interface FolderFilters {
  includeDeleted?: boolean
  parentId?: string | null
}

export function getFolders(filters?: FolderFilters): Folder[] {
  const conditions = []

  if (!filters?.includeDeleted) {
    conditions.push(isNull(folders.deletedAt))
  }

  if (filters?.parentId !== undefined) {
    if (filters.parentId === null) {
      conditions.push(isNull(folders.parentId))
    } else {
      conditions.push(eq(folders.parentId, filters.parentId))
    }
  }

  return db
    .select()
    .from(folders)
    .where(conditions.length > 0 ? and(...conditions) : undefined)
    .orderBy(folders.sortOrder, folders.name)
    .all()
}

export function getFolderById(id: string): Folder | null {
  const result = db.select().from(folders).where(eq(folders.id, id)).get()
  return result ?? null
}

export function createFolder(input: {
  name: string
  icon?: string
  parentId?: string | null
  sortOrder?: number
}): Folder {
  const userId = getCurrentUserId()
  const deviceId = getOrCreateDeviceId()

  const now = Date.now()
  const id = ulid()

  const newFolder: Folder = {
    id,
    userId,
    parentId: input.parentId ?? null,
    name: input.name,
    icon: input.icon ?? "📁",
    sortOrder: input.sortOrder ?? 0,
    version: 1,
    createdAt: now,
    updatedAt: now,
    deletedAt: null,
    deviceId,
  }

  db.insert(folders).values(newFolder).run()
  recordLocalChange("folder", id, 1, now, false, newFolder)

  return newFolder
}

export function updateFolder(
  id: string,
  input: {
    name?: string
    icon?: string
    parentId?: string | null
    sortOrder?: number
  },
): Folder {
  const existing = getFolderById(id)
  if (!existing) throw new Error(`Folder not found: ${id}`)

  const deviceId = getOrCreateDeviceId()
  const now = Date.now()
  const nextVersion = existing.version + 1

  const updatedFolder: Folder = {
    ...existing,
    name: input.name !== undefined ? input.name : existing.name,
    icon: input.icon !== undefined ? input.icon : existing.icon,
    parentId: input.parentId !== undefined ? input.parentId : existing.parentId,
    sortOrder: input.sortOrder !== undefined ? input.sortOrder : existing.sortOrder,
    version: nextVersion,
    updatedAt: now,
    deviceId,
  }

  db.update(folders).set(updatedFolder).where(eq(folders.id, id)).run()
  recordLocalChange("folder", id, nextVersion, now, false, updatedFolder)

  return updatedFolder
}

export function deleteFolder(id: string): void {
  const existing = getFolderById(id)
  if (!existing) throw new Error(`Folder not found: ${id}`)

  const deviceId = getOrCreateDeviceId()
  const now = Date.now()
  const nextVersion = existing.version + 1

  const updatedFolder: Folder = {
    ...existing,
    deletedAt: now,
    version: nextVersion,
    updatedAt: now,
    deviceId,
  }

  db.update(folders).set(updatedFolder).where(eq(folders.id, id)).run()
  recordLocalChange("folder", id, nextVersion, now, true, updatedFolder)

  // Soft-delete all active child notes in this folder
  const childNotes = db
    .select()
    .from(notes)
    .where(and(eq(notes.folderId, id), isNull(notes.deletedAt)))
    .all()

  for (const note of childNotes) {
    const nextNoteVersion = note.version + 1
    const updatedNote: Note = {
      ...note,
      deletedAt: now,
      version: nextNoteVersion,
      updatedAt: now,
      deviceId,
    }
    db.update(notes).set(updatedNote).where(eq(notes.id, note.id)).run()
    recordLocalChange("note", note.id, nextNoteVersion, now, true, updatedNote)
  }
}

export function restoreFolder(id: string): void {
  const existing = getFolderById(id)
  if (!existing) throw new Error(`Folder not found: ${id}`)

  const deviceId = getOrCreateDeviceId()
  const now = Date.now()
  const nextVersion = existing.version + 1

  const updatedFolder: Folder = {
    ...existing,
    deletedAt: null,
    version: nextVersion,
    updatedAt: now,
    deviceId,
  }

  db.update(folders).set(updatedFolder).where(eq(folders.id, id)).run()
  recordLocalChange("folder", id, nextVersion, now, false, updatedFolder)
}
