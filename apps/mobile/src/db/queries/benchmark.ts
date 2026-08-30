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

/** Permanently removes only rows created by createBenchmarkNotes, including its test folders, and logs tombstones to sync. */
export async function deleteBenchmarkNotes(): Promise<BenchmarkResult> {
  const startedAt = Date.now()
  const now = Date.now()

  const noteRows =
    (await expo.getAllAsync<{ id: string; version: number }>(
      `SELECT id, version FROM notes WHERE substr(body, 1, ${BENCHMARK_NOTE_MARKER.length}) = ?`,
      BENCHMARK_NOTE_MARKER,
    )) ?? []

  const folderRows =
    (await expo.getAllAsync<{ id: string; version: number }>(
      `SELECT id, version FROM folders WHERE substr(name, 1, ${BENCHMARK_FOLDER_MARKER.length}) = ?`,
      BENCHMARK_FOLDER_MARKER,
    )) ?? []

  const noteCount = noteRows.length
  const folderCount = folderRows.length

  await expo.withExclusiveTransactionAsync(async (transaction) => {
    for (const note of noteRows) {
      await transaction.runAsync(
        "INSERT INTO local_changes (entity_type, entity_id, version, updated_at, tombstone, payload, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        ["note", note.id, Number(note.version) + 1, now, 1, "{}", now],
      )
    }

    for (const folder of folderRows) {
      await transaction.runAsync(
        "INSERT INTO local_changes (entity_type, entity_id, version, updated_at, tombstone, payload, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        ["folder", folder.id, Number(folder.version) + 1, now, 1, "{}", now],
      )
    }

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
