import { expo } from "@/db"
import type { SyncChange } from "./types"

export async function applyRemoteChangesAsync(changes: SyncChange[]): Promise<number> {
  if (changes.length === 0) return 0

  let appliedCount = 0

  await expo.withExclusiveTransactionAsync(async (tx) => {
    for (const change of changes) {
      const { entity_type, entity_id, version, updated_at, tombstone, payload } = change

      if (entity_type === "folder") {
        const userId = String(payload.user_id ?? "default_user")
        const parentId = payload.parent_id ? String(payload.parent_id) : null
        const name = String(payload.name ?? "")
        const icon = String(payload.icon ?? "folder")
        const sortOrder = Number(payload.sort_order ?? 0)
        const createdAt = Number(payload.created_at ?? updated_at)
        const deletedAt =
          tombstone || payload.deleted_at ? Number(payload.deleted_at ?? updated_at) : null
        const deviceId = String(payload.device_id ?? "")

        await tx.runAsync(
          `INSERT INTO folders (id, user_id, parent_id, name, icon, sort_order, version, updated_at, created_at, deleted_at, device_id)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
           ON CONFLICT (id) DO UPDATE SET
             user_id = excluded.user_id,
             parent_id = excluded.parent_id,
             name = excluded.name,
             icon = excluded.icon,
             sort_order = excluded.sort_order,
             version = excluded.version,
             updated_at = excluded.updated_at,
             created_at = excluded.created_at,
             deleted_at = excluded.deleted_at,
             device_id = excluded.device_id
           WHERE excluded.updated_at >= folders.updated_at`,
          [
            entity_id,
            userId,
            parentId,
            name,
            icon,
            sortOrder,
            version,
            updated_at,
            createdAt,
            deletedAt,
            deviceId,
          ],
        )
        appliedCount++
      } else if (entity_type === "note") {
        const userId = String(payload.user_id ?? "default_user")
        const folderId = payload.folder_id ? String(payload.folder_id) : null
        const title = String(payload.title ?? "")
        const body = String(payload.body ?? "")
        const pinned = payload.pinned ? 1 : 0
        const trashed = tombstone || payload.trashed ? 1 : 0
        const createdAt = Number(payload.created_at ?? updated_at)
        const deletedAt =
          tombstone || payload.deleted_at ? Number(payload.deleted_at ?? updated_at) : null
        const deviceId = String(payload.device_id ?? "")
        const checksum = String(payload.checksum ?? "")

        await tx.runAsync(
          `INSERT INTO notes (id, user_id, folder_id, title, body, pinned, trashed, version, updated_at, created_at, deleted_at, device_id, checksum)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
           ON CONFLICT (id) DO UPDATE SET
             user_id = excluded.user_id,
             folder_id = excluded.folder_id,
             title = excluded.title,
             body = excluded.body,
             pinned = excluded.pinned,
             trashed = excluded.trashed,
             version = excluded.version,
             updated_at = excluded.updated_at,
             created_at = excluded.created_at,
             deleted_at = excluded.deleted_at,
             device_id = excluded.device_id,
             checksum = excluded.checksum
           WHERE excluded.updated_at >= notes.updated_at`,
          [
            entity_id,
            userId,
            folderId,
            title,
            body,
            pinned,
            trashed,
            version,
            updated_at,
            createdAt,
            deletedAt,
            deviceId,
            checksum,
          ],
        )
        appliedCount++
      }
    }
  })

  return appliedCount
}
