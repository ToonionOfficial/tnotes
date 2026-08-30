import { and, desc, eq, isNotNull, isNull, min } from "drizzle-orm"
import { ulid } from "@/utils/id"
import { db, expo } from "../index"
import { type Folder, folders, type Note, notes } from "../schema"
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
