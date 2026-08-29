import { computeChecksum } from "@/utils/crypto"
import { ulid } from "@/utils/id"
import { expo } from "../index"
import { getCurrentUserId, getOrCreateDeviceId } from "./sync"

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
