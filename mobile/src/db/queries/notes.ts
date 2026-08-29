import { and, desc, eq, isNull, sql } from "drizzle-orm"
import { computeChecksum } from "@/utils/crypto"
import { ulid } from "@/utils/id"
import { db, expo } from "../index"
import { type Note, notes } from "../schema"
import { getFolderById } from "./folders"
import { getCurrentUserId, getOrCreateDeviceId, recordLocalChange } from "./sync"

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

export function computeRecursiveCounts(
  foldersList: Array<{ id: string; parentId: string | null }>,
  directCounts: Record<string, number>,
): Record<string, number> {
  const childrenMap = new Map<string, string[]>()
  for (const f of foldersList) {
    if (f.parentId) {
      const list = childrenMap.get(f.parentId) || []
      list.push(f.id)
      childrenMap.set(f.parentId, list)
    }
  }

  const memo = new Map<string, number>()
  const visited = new Set<string>()

  function getRecursiveCount(folderId: string): number {
    const cached = memo.get(folderId)
    if (cached !== undefined) return cached
    if (visited.has(folderId)) return 0
    visited.add(folderId)

    let count = directCounts[folderId] ?? 0
    const children = childrenMap.get(folderId) || []
    for (const childId of children) {
      count += getRecursiveCount(childId)
    }
    visited.delete(folderId)
    memo.set(folderId, count)
    return count
  }

  const result: Record<string, number> = {}
  for (const f of foldersList) {
    result[f.id] = getRecursiveCount(f.id)
  }
  return result
}

export async function getFolderNoteCountsAsync(): Promise<FolderNoteCounts> {
  const [noteRows, folderRows] = await Promise.all([
    expo.getAllAsync<{
      folderId: string | null
      activeCount: number
      trashCount: number
    }>(`SELECT folder_id AS folderId,
        COUNT(CASE WHEN trashed = 0 THEN 1 END) AS activeCount,
        COUNT(CASE WHEN trashed = 1 THEN 1 END) AS trashCount
      FROM notes GROUP BY folder_id`),
    expo.getAllAsync<{ id: string; parentId: string | null }>(
      `SELECT id, parent_id AS parentId FROM folders WHERE deleted_at IS NULL`,
    ),
  ])

  let total = 0
  let trash = 0
  const directCounts: Record<string, number> = {}
  for (const row of noteRows) {
    const active = Number(row.activeCount)
    const trashed = Number(row.trashCount)
    total += active
    trash += trashed
    if (row.folderId) directCounts[row.folderId] = active
  }

  const byFolder = computeRecursiveCounts(folderRows, directCounts)
  return { total, byFolder, trash }
}

/**
 * Returns aggregated note counts (total active, per folder recursively, and trash).
 */
export function getFolderNoteCounts(): FolderNoteCounts {
  const noteRows = db.all<{
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

  const folderRows = db.all<{ id: string; parentId: string | null }>(sql`
    SELECT id, parent_id AS parentId FROM folders WHERE deleted_at IS NULL
  `)

  let total = 0
  let trash = 0
  const directCounts: Record<string, number> = {}
  for (const row of noteRows) {
    const active = Number(row.activeCount)
    const trashed = Number(row.trashCount)
    total += active
    trash += trashed
    if (row.folderId) directCounts[row.folderId] = active
  }

  const byFolder = computeRecursiveCounts(folderRows, directCounts)
  return { total, byFolder, trash }
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

export function moveNote(id: string, folderId: string | null): Note {
  return updateNote(id, { folderId })
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

export function batchMoveNotes(ids: string[], folderId: string | null): void {
  for (const id of ids) {
    updateNote(id, { folderId })
  }
}
