import { computeChecksum } from "@/utils/crypto"
import { ulid } from "@/utils/id"
import { expo } from "../index"
import { getCurrentUserId, getOrCreateDeviceId } from "./sync"

export const BENCHMARK_NOTE_MARKER = "__tnotes_benchmark_note_v1__:"
export const BENCHMARK_FOLDER_MARKER = "__tnotes_benchmark_folder_v1__:"

export interface BenchmarkResult {
  noteCount: number
  folderCount: number
  elapsedMs: number
}

/** Generic helper for bulk DB work: split large id sets so each transaction
 * stays small and bind-param counts stay under SQLite limits. */
export function chunkArray<T>(items: T[], size: number): T[][] {
  if (size <= 0) throw new Error("chunk size must be > 0")
  const chunks: T[][] = []
  for (let index = 0; index < items.length; index += size) {
    chunks.push(items.slice(index, index + size))
  }
  return chunks
}

function toSqlString(value: string): string {
  return `'${value.replace(/'/g, "''")}'`
}

/**
 * Inserts benchmark data into SQLite tables, FTS index, and local_changes so benchmark data
 * syncs seamlessly to the server and connected devices.
 */
export async function createBenchmarkNotes(noteCount: number): Promise<BenchmarkResult> {
  const userId = getCurrentUserId()
  const deviceId = getOrCreateDeviceId()
  const batchId = ulid()
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
    const folderId = folderIds[index]
    const folderName = `${BENCHMARK_FOLDER_MARKER}${batchId}:${index + 1}`
    const folderPayload = {
      id: folderId,
      user_id: userId,
      parent_id: null,
      name: folderName,
      icon: "folder",
      sort_order: index,
      version: 1,
      updated_at: now,
      created_at: now,
      deleted_at: null,
      device_id: deviceId,
    }

    statements.push(`INSERT INTO folders (
      id, user_id, parent_id, name, icon, sort_order, version, updated_at, created_at, deleted_at, device_id
    ) VALUES (
      ${toSqlString(folderId)}, ${toSqlString(userId)}, NULL, ${toSqlString(folderName)}, 'folder',
      ${index}, 1, ${now}, ${now}, NULL, ${toSqlString(deviceId)}
    )`)

    statements.push(`INSERT INTO local_changes (
      entity_type, entity_id, version, updated_at, tombstone, payload, created_at
    ) VALUES (
      'folder', ${toSqlString(folderId)}, 1, ${now}, 0, ${toSqlString(JSON.stringify(folderPayload))}, ${now}
    )`)
  }

  for (let index = 0; index < noteCount; index++) {
    const noteId = ulid()
    const folderId = shuffledFolderIds[index % folderCount]
    const title = `Benchmark note ${index + 1}`
    const body = `${BENCHMARK_NOTE_MARKER}${batchId}:${index + 1}`
    const checksum = computeChecksum(body)

    const notePayload = {
      id: noteId,
      user_id: userId,
      folder_id: folderId,
      title,
      body,
      pinned: false,
      trashed: false,
      version: 1,
      updated_at: now,
      created_at: now,
      deleted_at: null,
      device_id: deviceId,
      checksum,
    }

    statements.push(`INSERT INTO notes (
      id, user_id, folder_id, title, body, pinned, trashed, version, updated_at, created_at, deleted_at, device_id, checksum
    ) VALUES (
      ${toSqlString(noteId)}, ${toSqlString(userId)}, ${toSqlString(folderId)},
      ${toSqlString(title)}, ${toSqlString(body)}, 0, 0, 1, ${now}, ${now}, NULL,
      ${toSqlString(deviceId)}, ${toSqlString(checksum)}
    )`)

    statements.push(`INSERT INTO local_changes (
      entity_type, entity_id, version, updated_at, tombstone, payload, created_at
    ) VALUES (
      'note', ${toSqlString(noteId)}, 1, ${now}, 0, ${toSqlString(JSON.stringify(notePayload))}, ${now}
    )`)
  }

  statements.push("COMMIT")

  const startedAt = Date.now()
  await expo.execAsync(`${statements.join(";\n")};`)

  return { noteCount, folderCount, elapsedMs: Date.now() - startedAt }
}

/** Permanently removes only rows created by createBenchmarkNotes, including its test folders, and logs tombstones to sync.
 *
 * Single set-based `execAsync` on purpose: one `BEGIN; 4 statements; COMMIT`
 * in a single native round-trip — the same pattern as `createBenchmarkNotes`
 * and `applyRemoteChangesAsync`, both proven at 1k+ rows. A previous revision
 * used one `withExclusiveTransactionAsync` per 200-row chunk and died
 * deterministically after the first chunk commit, so per-chunk exclusive
 * transactions are deliberately avoided here.
 */
export async function deleteBenchmarkNotes(): Promise<BenchmarkResult> {
  const startedAt = Date.now()
  const now = Date.now()

  const noteCountRow = await expo.getFirstAsync<{ count: number }>(
    `SELECT COUNT(*) AS count FROM notes WHERE substr(body, 1, ${BENCHMARK_NOTE_MARKER.length}) = ?`,
    BENCHMARK_NOTE_MARKER,
  )
  const folderCountRow = await expo.getFirstAsync<{ count: number }>(
    `SELECT COUNT(*) AS count FROM folders WHERE substr(name, 1, ${BENCHMARK_FOLDER_MARKER.length}) = ?`,
    BENCHMARK_FOLDER_MARKER,
  )
  const noteCount = Number(noteCountRow?.count ?? 0)
  const folderCount = Number(folderCountRow?.count ?? 0)

  if (noteCount === 0 && folderCount === 0) {
    return { noteCount, folderCount, elapsedMs: Date.now() - startedAt }
  }

  // Markers are compile-time constants and `now` is numeric: safe to inline,
  // following the same convention as createBenchmarkNotes above.
  // Notes first (FK: notes.folder_id ON DELETE SET NULL), then folders.
  await expo.execAsync(`BEGIN IMMEDIATE;
    INSERT INTO local_changes (entity_type, entity_id, version, updated_at, tombstone, payload, created_at)
      SELECT 'note', id, version + 1, ${now}, 1, '{}', ${now} FROM notes
      WHERE substr(body, 1, ${BENCHMARK_NOTE_MARKER.length}) = ${toSqlString(BENCHMARK_NOTE_MARKER)};
    DELETE FROM notes WHERE substr(body, 1, ${BENCHMARK_NOTE_MARKER.length}) = ${toSqlString(BENCHMARK_NOTE_MARKER)};
    INSERT INTO local_changes (entity_type, entity_id, version, updated_at, tombstone, payload, created_at)
      SELECT 'folder', id, version + 1, ${now}, 1, '{}', ${now} FROM folders
      WHERE substr(name, 1, ${BENCHMARK_FOLDER_MARKER.length}) = ${toSqlString(BENCHMARK_FOLDER_MARKER)};
    DELETE FROM folders WHERE substr(name, 1, ${BENCHMARK_FOLDER_MARKER.length}) = ${toSqlString(BENCHMARK_FOLDER_MARKER)};
    COMMIT;`)

  return { noteCount, folderCount, elapsedMs: Date.now() - startedAt }
}
