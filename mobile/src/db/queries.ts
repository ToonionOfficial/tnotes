import { and, desc, eq, isNotNull, isNull, sql } from "drizzle-orm"
import { computeChecksum } from "@/utils/crypto"
import { ulid } from "@/utils/id"
import { db, expo } from "./index"
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
const BENCHMARK_NOTE_MARKER = "__tnotes_benchmark_note_v1__:"
const BENCHMARK_FOLDER_MARKER = "__tnotes_benchmark_folder_v1__:"

export interface BenchmarkResult {
  noteCount: number
  folderCount: number
  elapsedMs: number
}

function toSqlString(value: string): string {
  return `'${value.replace(/'/g, "''")}'`
}

/**
 * Inserts benchmark data into the same SQLite tables and FTS index used by the app.
 * The data intentionally skips local_changes so a local performance test is never synced
 * to another device or server.
 */
export async function createBenchmarkNotes(noteCount: number): Promise<BenchmarkResult> {
  const userId = getCurrentUserId()
  const deviceId = getOrCreateDeviceId()
  const batchId = ulid()
  // About ten notes per folder reproduces a large library where individual folders stay small.
  const folderCount = Math.ceil(noteCount / 10)
  const now = Date.now()
  const folderIds = Array.from({ length: folderCount }, () => ulid())
  const shuffledFolderIds = [...folderIds]

  for (let index = shuffledFolderIds.length - 1; index > 0; index--) {
    const randomIndex = Math.floor(Math.random() * (index + 1))
    ;[shuffledFolderIds[index], shuffledFolderIds[randomIndex]] = [
      shuffledFolderIds[randomIndex],
      shuffledFolderIds[index],
    ]
  }

  const statements = ["BEGIN IMMEDIATE"]
  for (let index = 0; index < folderCount; index++) {
    const folderName = `${BENCHMARK_FOLDER_MARKER}${batchId}:${index + 1}`
    statements.push(`INSERT INTO folders (
      id, user_id, parent_id, name, icon, sort_order, version, updated_at, created_at, deleted_at, device_id
    ) VALUES (
      ${toSqlString(folderIds[index])}, ${toSqlString(userId)}, NULL, ${toSqlString(folderName)}, 'folder',
      ${index}, 1, ${now}, ${now}, NULL, ${toSqlString(deviceId)}
    )`)
  }

  for (let index = 0; index < noteCount; index++) {
    const body = `${BENCHMARK_NOTE_MARKER}${batchId}:${index + 1}`
    statements.push(`INSERT INTO notes (
      id, user_id, folder_id, title, body, pinned, trashed, version, updated_at, created_at, deleted_at, device_id, checksum
    ) VALUES (
      ${toSqlString(ulid())}, ${toSqlString(userId)}, ${toSqlString(shuffledFolderIds[index % folderCount])},
      ${toSqlString(`Benchmark note ${index + 1}`)}, ${toSqlString(body)}, 0, 0, 1, ${now}, ${now}, NULL,
      ${toSqlString(deviceId)}, ${toSqlString(computeChecksum(body))}
    )`)
  }
  statements.push("COMMIT")

  const startedAt = Date.now()
  await expo.execAsync(`${statements.join(";\n")};`)

  return { noteCount, folderCount, elapsedMs: Date.now() - startedAt }
}

/** Permanently removes only rows created by createBenchmarkNotes, including its test folders. */
export async function deleteBenchmarkNotes(): Promise<BenchmarkResult> {
  const startedAt = Date.now()
  const noteCount = Number(
    (
      await expo.getFirstAsync<{ count: number }>(
        `SELECT COUNT(*) AS count FROM notes WHERE substr(body, 1, ${BENCHMARK_NOTE_MARKER.length}) = ?`,
        BENCHMARK_NOTE_MARKER,
      )
    )?.count ?? 0,
  )
  const folderCount = Number(
    (
      await expo.getFirstAsync<{ count: number }>(
        `SELECT COUNT(*) AS count FROM folders WHERE substr(name, 1, ${BENCHMARK_FOLDER_MARKER.length}) = ?`,
        BENCHMARK_FOLDER_MARKER,
      )
    )?.count ?? 0,
  )

  await expo.withExclusiveTransactionAsync(async (transaction) => {
    await transaction.runAsync(
      `DELETE FROM notes WHERE substr(body, 1, ${BENCHMARK_NOTE_MARKER.length}) = ?`,
      BENCHMARK_NOTE_MARKER,
    )
    await transaction.runAsync(
      `DELETE FROM folders WHERE substr(name, 1, ${BENCHMARK_FOLDER_MARKER.length}) = ?`,
      BENCHMARK_FOLDER_MARKER,
    )
  })

  return { noteCount, folderCount, elapsedMs: Date.now() - startedAt }
}

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

  // Ensure soft-deleted notes are properly flagged as trashed
  db.update(notes)
    .set({ trashed: true })
    .where(and(isNotNull(notes.deletedAt), eq(notes.trashed, false)))
    .run()

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
  limit?: number
  offset?: number
}

export interface SearchResult extends Note {
  snippet?: string
}

type NoteRow = Omit<Note, "folderId" | "pinned" | "trashed" | "deletedAt"> & {
  folderId: string | null
  pinned: number
  trashed: number
  deletedAt: number | null
  snippet?: string | null
}

function mapNoteRow(row: NoteRow): SearchResult {
  return {
    ...row,
    folderId: row.folderId,
    pinned: Boolean(row.pinned),
    trashed: Boolean(row.trashed),
    deletedAt: row.deletedAt,
    snippet: row.snippet ?? undefined,
  }
}

/** Async, paged reads keep SQLite work off the JavaScript thread during navigation. */
export async function getNotesPageAsync(filters: NoteFilters = {}): Promise<SearchResult[]> {
  const limit = filters.limit ?? 50
  const offset = filters.offset ?? 0

  if (filters.search?.trim()) {
    const tokens = filters.search
      .trim()
      .replace(/['"*^(){}:~-]/g, " ")
      .split(/\s+/)
      .filter(Boolean)
    if (tokens.length === 0) return []

    const conditions = ["notes_fts MATCH ?", "n.trashed = ?"]
    const params: Array<string | number> = [
      tokens.map((token) => `"${token}"*`).join(" "),
      filters.trashed ? 1 : 0,
    ]
    if (filters.folderId === null) {
      conditions.push("n.folder_id IS NULL")
    } else if (filters.folderId !== undefined) {
      conditions.push("n.folder_id = ?")
      params.push(filters.folderId)
    }
    params.push(limit, offset)
    const rows = await expo.getAllAsync<NoteRow>(
      `SELECT n.id, n.user_id AS userId, n.folder_id AS folderId, n.title,
        substr(n.body, 1, 300) AS body, n.pinned, n.trashed, n.version,
        n.created_at AS createdAt, n.updated_at AS updatedAt, n.deleted_at AS deletedAt,
        n.device_id AS deviceId, n.checksum,
        snippet(notes_fts, 1, '<mark>', '</mark>', '...', 20) AS snippet
      FROM notes n JOIN notes_fts f ON n.rowid = f.rowid
      WHERE ${conditions.join(" AND ")} ORDER BY f.rank LIMIT ? OFFSET ?`,
      params,
    )
    return rows.map(mapNoteRow)
  }

  const conditions = ["trashed = ?"]
  const params: Array<string | number> = [filters.trashed ? 1 : 0]
  if (filters.folderId === null) {
    conditions.push("folder_id IS NULL")
  } else if (filters.folderId !== undefined) {
    conditions.push("folder_id = ?")
    params.push(filters.folderId)
  }
  if (filters.pinned !== undefined) {
    conditions.push("pinned = ?")
    params.push(filters.pinned ? 1 : 0)
  }
  params.push(limit, offset)
  const orderBy = filters.trashed
    ? "updated_at DESC, id DESC"
    : "pinned DESC, updated_at DESC, id DESC"
  const rows = await expo.getAllAsync<NoteRow>(
    `SELECT id, user_id AS userId, folder_id AS folderId, title, substr(body, 1, 300) AS body,
      pinned, trashed, version, created_at AS createdAt, updated_at AS updatedAt,
      deleted_at AS deletedAt, device_id AS deviceId, checksum
    FROM notes WHERE ${conditions.join(" AND ")} ORDER BY ${orderBy} LIMIT ? OFFSET ?`,
    params,
  )
  return rows.map(mapNoteRow)
}

/**
 * High-performance full-text search across notes using SQLite FTS5 and BM25 relevance ranking.
 */
export function searchNotes(
  query: string,
  filters?: { folderId?: string | null; trashed?: boolean; limit?: number; offset?: number },
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
  const offset = filters?.offset ?? 0

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
        OFFSET ${offset}
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
        OFFSET ${offset}
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
      OFFSET ${offset}
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

export async function getFolderNoteCountsAsync(): Promise<FolderNoteCounts> {
  const rows = await expo.getAllAsync<{
    folderId: string | null
    activeCount: number
    trashCount: number
  }>(`SELECT folder_id AS folderId,
      COUNT(CASE WHEN trashed = 0 THEN 1 END) AS activeCount,
      COUNT(CASE WHEN trashed = 1 THEN 1 END) AS trashCount
    FROM notes GROUP BY folder_id`)
  return folderNoteCountsFromRows(rows)
}

function folderNoteCountsFromRows(
  rows: Array<{ folderId: string | null; activeCount: number; trashCount: number }>,
): FolderNoteCounts {
  let total = 0
  let trash = 0
  const byFolder: Record<string, number> = {}
  for (const row of rows) {
    const active = Number(row.activeCount)
    const trashed = Number(row.trashCount)
    total += active
    trash += trashed
    if (row.folderId) byFolder[row.folderId] = active
  }
  return { total, byFolder, trash }
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

  return folderNoteCountsFromRows(rows)
}

export function getNotes(filters?: NoteFilters): Note[] {
  // If search is requested, delegate to FTS5 for blazing speed
  if (filters?.search && filters.search.trim().length > 0) {
    return searchNotes(filters.search, {
      folderId: filters.folderId,
      trashed: filters.trashed,
      limit: filters.limit,
      offset: filters.offset,
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

  const query = db
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

  if (filters?.limit !== undefined) {
    return query
      .limit(filters.limit)
      .offset(filters.offset ?? 0)
      .all()
  }

  return query.all()
}

export function getNoteById(id: string): Note | null {
  const result = db.select().from(notes).where(eq(notes.id, id)).get()
  return result ?? null
}

export async function getNoteByIdAsync(id: string): Promise<Note | null> {
  const row = await expo.getFirstAsync<NoteRow>(
    `SELECT id, user_id AS userId, folder_id AS folderId, title, body, pinned, trashed, version,
      created_at AS createdAt, updated_at AS updatedAt, deleted_at AS deletedAt,
      device_id AS deviceId, checksum FROM notes WHERE id = ?`,
    id,
  )
  return row ? mapNoteRow(row) : null
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

  // If the parent folder no longer exists or was deleted, restore to root (All Notes)
  let targetFolderId = existing.folderId
  if (targetFolderId) {
    const parentFolder = getFolderById(targetFolderId)
    if (!parentFolder || parentFolder.deletedAt !== null) {
      targetFolderId = null
    }
  }

  const updatedNote: Note = {
    ...existing,
    folderId: targetFolderId,
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

export function batchTrashNotes(ids: string[]): void {
  for (const id of ids) {
    trashNote(id)
  }
}

export function batchDeleteNotesPermanently(ids: string[]): void {
  for (const id of ids) {
    deleteNotePermanently(id)
  }
}

export interface FolderFilters {
  includeDeleted?: boolean
  parentId?: string | null
  limit?: number
  offset?: number
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

  const query = db
    .select()
    .from(folders)
    .where(conditions.length > 0 ? and(...conditions) : undefined)
    .orderBy(folders.sortOrder, folders.name)

  if (filters?.limit !== undefined) {
    return query
      .limit(filters.limit)
      .offset(filters.offset ?? 0)
      .all()
  }

  return query.all()
}

export function getFolderById(id: string): Folder | null {
  const result = db.select().from(folders).where(eq(folders.id, id)).get()
  return result ?? null
}

export async function getFolderByIdAsync(id: string): Promise<Folder | null> {
  const row = await expo.getFirstAsync<Folder>(
    `SELECT id, user_id AS userId, parent_id AS parentId, name, icon, sort_order AS sortOrder,
      version, updated_at AS updatedAt, created_at AS createdAt, deleted_at AS deletedAt, device_id AS deviceId
    FROM folders WHERE id = ?`,
    id,
  )
  return row ?? null
}

export async function getFoldersPageAsync(filters: FolderFilters = {}): Promise<Folder[]> {
  const conditions = filters.includeDeleted ? [] : ["deleted_at IS NULL"]
  const params: Array<string | number> = []
  if (filters.parentId === null) {
    conditions.push("parent_id IS NULL")
  } else if (filters.parentId !== undefined) {
    conditions.push("parent_id = ?")
    params.push(filters.parentId)
  }
  params.push(filters.limit ?? 50, filters.offset ?? 0)
  return expo.getAllAsync<Folder>(
    `SELECT id, user_id AS userId, parent_id AS parentId, name, icon, sort_order AS sortOrder,
      version, updated_at AS updatedAt, created_at AS createdAt, deleted_at AS deletedAt, device_id AS deviceId
    FROM folders${conditions.length ? ` WHERE ${conditions.join(" AND ")}` : ""}
    ORDER BY sort_order, name, id LIMIT ? OFFSET ?`,
    params,
  )
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

export function reorderFolders(folderIds: string[]): void {
  const deviceId = getOrCreateDeviceId()
  const now = Date.now()

  folderIds.forEach((id, index) => {
    const existing = getFolderById(id)
    if (!existing || existing.sortOrder === index) return

    const nextVersion = existing.version + 1
    const updatedFolder: Folder = {
      ...existing,
      sortOrder: index,
      version: nextVersion,
      updatedAt: now,
      deviceId,
    }

    db.update(folders)
      .set({
        sortOrder: index,
        version: nextVersion,
        updatedAt: now,
        deviceId,
      })
      .where(eq(folders.id, id))
      .run()

    recordLocalChange("folder", id, nextVersion, now, false, updatedFolder)
  })
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

  // Soft-delete and trash all active child notes in this folder
  const childNotes = db
    .select()
    .from(notes)
    .where(and(eq(notes.folderId, id), eq(notes.trashed, false)))
    .all()

  for (const note of childNotes) {
    const nextNoteVersion = note.version + 1
    const updatedNote: Note = {
      ...note,
      trashed: true,
      deletedAt: now,
      version: nextNoteVersion,
      updatedAt: now,
      deviceId,
    }
    db.update(notes).set(updatedNote).where(eq(notes.id, note.id)).run()
    recordLocalChange("note", note.id, nextNoteVersion, now, true, updatedNote)
  }

  // Ensure any orphaned soft-deleted notes have trashed = true
  db.update(notes)
    .set({ trashed: true })
    .where(and(isNotNull(notes.deletedAt), eq(notes.trashed, false)))
    .run()
}

export function batchDeleteFolders(ids: string[]): void {
  for (const id of ids) {
    deleteFolder(id)
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
