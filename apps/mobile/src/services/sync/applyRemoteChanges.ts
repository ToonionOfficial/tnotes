import { expo } from "@/db"
import { isBenchmarkSyncPayload } from "@/db/queries/benchmark"
import type { SyncChange } from "./types"

function toSqlString(val: unknown): string {
  if (val === null || val === undefined) return "NULL"
  return `'${String(val).replace(/'/g, "''")}'`
}

function toSqlInt(val: unknown): string {
  if (val === null || val === undefined) return "NULL"
  if (typeof val === "boolean") return val ? "1" : "0"
  const num = Number(val)
  return Number.isNaN(num) ? "0" : String(num)
}

export async function applyRemoteChangesAsync(changes: SyncChange[]): Promise<number> {
  if (changes.length === 0) return 0

  const statements: string[] = ["BEGIN IMMEDIATE"]
  let appliedCount = 0

  for (const change of changes) {
    const { entity_type, entity_id, version, updated_at, tombstone, payload } = change

    // Benchmark rows are local-only: never materialize copies the server may
    // still hold from before they stopped syncing. Tombstones ('{}' payloads)
    // pass through so those server copies get deleted locally too.
    if (!tombstone && isBenchmarkSyncPayload(entity_type, payload)) continue

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

      statements.push(`INSERT INTO folders (
        id, user_id, parent_id, name, icon, sort_order, version, updated_at, created_at, deleted_at, device_id
      ) VALUES (
        ${toSqlString(entity_id)}, ${toSqlString(userId)}, ${toSqlString(parentId)}, ${toSqlString(name)},
        ${toSqlString(icon)}, ${toSqlInt(sortOrder)}, ${toSqlInt(version)}, ${toSqlInt(updated_at)},
        ${toSqlInt(createdAt)}, ${toSqlInt(deletedAt)}, ${toSqlString(deviceId)}
      )
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
      WHERE excluded.updated_at >= folders.updated_at`)
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

      statements.push(`INSERT INTO notes (
        id, user_id, folder_id, title, body, pinned, trashed, version, updated_at, created_at, deleted_at, device_id, checksum
      ) VALUES (
        ${toSqlString(entity_id)}, ${toSqlString(userId)}, ${toSqlString(folderId)}, ${toSqlString(title)},
        ${toSqlString(body)}, ${toSqlInt(pinned)}, ${toSqlInt(trashed)}, ${toSqlInt(version)},
        ${toSqlInt(updated_at)}, ${toSqlInt(createdAt)}, ${toSqlInt(deletedAt)}, ${toSqlString(deviceId)},
        ${toSqlString(checksum)}
      )
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
      WHERE excluded.updated_at >= notes.updated_at`)
      appliedCount++
    }
  }

  statements.push("COMMIT")
  await expo.execAsync(`${statements.join(";\n")};`)

  return appliedCount
}
