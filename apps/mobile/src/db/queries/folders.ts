import { and, asc, desc, eq, isNotNull, isNull, min } from "drizzle-orm"
import { ulid } from "@/utils/id"
import { db, expo } from "../index"
import { type Folder, folders, localChanges, type Note, notes } from "../schema"
import { BENCHMARK_FOLDER_MARKER } from "./benchmark"
import { getCurrentUserId, getOrCreateDeviceId, recordLocalChange } from "./sync"

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
    .orderBy(folders.sortOrder, desc(folders.createdAt), desc(folders.id))

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
  }
  if (filters.parentId !== undefined && filters.parentId !== null) {
    params.push(filters.parentId)
  }
  params.push(filters.limit ?? 50, filters.offset ?? 0)
  return expo.getAllAsync<Folder>(
    `SELECT id, user_id AS userId, parent_id AS parentId, name, icon, sort_order AS sortOrder,
      version, updated_at AS updatedAt, created_at AS createdAt, deleted_at AS deletedAt, device_id AS deviceId
    FROM folders${conditions.length ? ` WHERE ${conditions.join(" AND ")}` : ""}
    ORDER BY sort_order ASC, created_at DESC, id DESC LIMIT ? OFFSET ?`,
    params,
  )
}

export const FREQUENT_FOLDER_LIMIT = 5

interface FrequentFolderRow {
  id: string
  userId: string
  parentId: string | null
  name: string
  icon: string
  sortOrder: number
  version: number
  updatedAt: number
  createdAt: number
  deletedAt: number | null
  deviceId: string
  activity: number
}

/**
 * Top-level folders ranked by recent activity (latest note update, falling
 * back to the folder itself so empty folders still rank). Powers the
 * "Frequently Used" home section. Benchmark/dev folders are excluded.
 */
export async function getFrequentFoldersAsync(
  limit: number = FREQUENT_FOLDER_LIMIT,
): Promise<Folder[]> {
  const rows = await expo.getAllAsync<FrequentFolderRow>(
    `SELECT f.id, f.user_id AS userId, f.parent_id AS parentId, f.name, f.icon,
      f.sort_order AS sortOrder, f.version, f.updated_at AS updatedAt,
      f.created_at AS createdAt, f.deleted_at AS deletedAt, f.device_id AS deviceId,
      MAX(COALESCE(n.updated_at, 0), f.updated_at, f.created_at) AS activity
    FROM folders f LEFT JOIN notes n ON n.folder_id = f.id AND n.trashed = 0
    WHERE f.deleted_at IS NULL
      AND f.parent_id IS NULL
      AND substr(f.name, 1, ${BENCHMARK_FOLDER_MARKER.length}) != ?
    GROUP BY f.id
    ORDER BY activity DESC, f.updated_at DESC, f.id DESC
    LIMIT ?`,
    [BENCHMARK_FOLDER_MARKER, limit],
  )
  return rows.map((row) => ({
    id: row.id,
    userId: row.userId,
    parentId: row.parentId,
    name: row.name,
    icon: row.icon,
    sortOrder: row.sortOrder,
    version: row.version,
    updatedAt: row.updatedAt,
    createdAt: row.createdAt,
    deletedAt: row.deletedAt,
    deviceId: row.deviceId,
  }))
}

/**
 * Applies a user-arranged order (persisted id list) to freshly computed
 * frequent ids: never-arranged ids surface FIRST in activity order (so a
 * just-used folder appears on top, not buried at the bottom), then stored
 * ids that still exist keep their positions, and deleted ids drop out.
 * Deterministic per input — pure, unit tested.
 */
export function applyFrequentOrder(currentIds: string[], storedIds: string[]): string[] {
  const current = new Set(currentIds)
  const stored = new Set(storedIds)
  const ordered: string[] = []
  for (const id of currentIds) {
    if (!stored.has(id)) ordered.push(id)
  }
  for (const id of storedIds) {
    if (current.has(id) && !ordered.includes(id)) ordered.push(id)
  }
  return ordered
}

/** Same as applyFrequentOrder but over Folder rows. */
export function orderFrequentFolders(foldersList: Folder[], storedIds: string[]): Folder[] {
  const byId = new Map(foldersList.map((folder) => [folder.id, folder]))
  const ordered: Folder[] = []
  for (const id of applyFrequentOrder(
    foldersList.map((folder) => folder.id),
    storedIds,
  )) {
    const folder = byId.get(id)
    if (folder) ordered.push(folder)
  }
  return ordered
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

  let sortOrder = input.sortOrder
  if (sortOrder === undefined) {
    const parentCondition =
      input.parentId === undefined || input.parentId === null
        ? isNull(folders.parentId)
        : eq(folders.parentId, input.parentId)

    const minResult = db
      .select({ minOrder: min(folders.sortOrder) })
      .from(folders)
      .where(and(isNull(folders.deletedAt), parentCondition))
      .get()

    sortOrder =
      minResult?.minOrder !== null && minResult?.minOrder !== undefined ? minResult.minOrder - 1 : 0
  }

  const newFolder: Folder = {
    id,
    userId,
    parentId: input.parentId ?? null,
    name: input.name,
    icon: input.icon ?? "📁",
    sortOrder,
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

/**
 * Pure list surgery for folder drag-and-drop: moves one id within an ordered
 * id list. Clamps out-of-range targets and returns the input unchanged when
 * the id is missing or already at the target. Shared by the DB persist path
 * and the React Query optimistic patch so both compute identical orders.
 */
export function moveIdInList(ids: string[], folderId: string, toIndex: number): string[] {
  const fromIndex = ids.indexOf(folderId)
  if (fromIndex === -1) return ids
  const clamped = Math.max(0, Math.min(ids.length - 1, toIndex))
  if (clamped === fromIndex) return ids
  const next = [...ids]
  const [moved] = next.splice(fromIndex, 1)
  if (moved === undefined) return ids
  next.splice(clamped, 0, moved)
  return next
}

export interface FolderMoveResult {
  moved: boolean
  updatedCount: number
}

/**
 * Persists a drag-and-drop move of one folder within its sibling list.
 * Loads the FULL sibling order from SQLite in one query (never a paginated
 * subset, which would corrupt sortOrders), then rewrites only the rows whose
 * position actually changed — a single move touches just the range between
 * the old and new index — inside one synchronous drizzle transaction.
 */
export function moveFolderToIndex(input: {
  folderId: string
  toIndex: number
  parentId?: string | null
}): FolderMoveResult {
  const parentCondition =
    input.parentId === undefined || input.parentId === null
      ? isNull(folders.parentId)
      : eq(folders.parentId, input.parentId)

  // Same ordering as getFolders/getFoldersPageAsync so splice indices match
  // what the UI rendered.
  const siblings = db
    .select()
    .from(folders)
    .where(and(isNull(folders.deletedAt), parentCondition))
    .orderBy(asc(folders.sortOrder), desc(folders.createdAt), desc(folders.id))
    .all()

  const currentIds = siblings.map((sibling) => sibling.id)
  const nextIds = moveIdInList(currentIds, input.folderId, input.toIndex)
  if (nextIds === currentIds) return { moved: false, updatedCount: 0 }

  const deviceId = getOrCreateDeviceId()
  const now = Date.now()
  const byId = new Map(siblings.map((sibling) => [sibling.id, sibling]))

  const updates: Array<{ id: string; sortOrder: number; version: number; snapshot: Folder }> = []
  nextIds.forEach((id, index) => {
    const existing = byId.get(id)
    if (!existing || existing.sortOrder === index) return
    const nextVersion = existing.version + 1
    updates.push({
      id,
      sortOrder: index,
      version: nextVersion,
      snapshot: { ...existing, sortOrder: index, version: nextVersion, updatedAt: now, deviceId },
    })
  })

  if (updates.length === 0) return { moved: false, updatedCount: 0 }

  db.transaction((tx) => {
    for (const update of updates) {
      tx.update(folders)
        .set({ sortOrder: update.sortOrder, version: update.version, updatedAt: now, deviceId })
        .where(eq(folders.id, update.id))
        .run()
      tx.insert(localChanges)
        .values({
          entityType: "folder",
          entityId: update.id,
          version: update.version,
          updatedAt: now,
          tombstone: false,
          payload: JSON.stringify(update.snapshot),
          createdAt: now,
        })
        .run()
    }
  })

  return { moved: true, updatedCount: updates.length }
}

export function deleteFolder(id: string): void {
  const existing = getFolderById(id)
  if (!existing) throw new Error(`Folder not found: ${id}`)

  const deviceId = getOrCreateDeviceId()
  const now = Date.now()

  // Collect all descendant folder IDs recursively
  const allFolderIds: string[] = [id]
  const queue: string[] = [id]
  while (queue.length > 0) {
    const currentId = queue.shift()
    if (!currentId) break
    const children = db
      .select()
      .from(folders)
      .where(and(eq(folders.parentId, currentId), isNull(folders.deletedAt)))
      .all()
    for (const child of children) {
      allFolderIds.push(child.id)
      queue.push(child.id)
    }
  }

  for (const folderId of allFolderIds) {
    const folder = getFolderById(folderId)
    if (!folder || folder.deletedAt !== null) continue

    const nextVersion = folder.version + 1
    const updatedFolder: Folder = {
      ...folder,
      deletedAt: now,
      version: nextVersion,
      updatedAt: now,
      deviceId,
    }

    db.update(folders).set(updatedFolder).where(eq(folders.id, folderId)).run()
    recordLocalChange("folder", folderId, nextVersion, now, true, updatedFolder)

    // Soft-delete and trash all active child notes in this folder
    const childNotes = db
      .select()
      .from(notes)
      .where(and(eq(notes.folderId, folderId), eq(notes.trashed, false)))
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
