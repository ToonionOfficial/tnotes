import { expo } from "../index"

export interface DatabaseStats {
  sizeBytes: number
  formattedSize: string
  notesCount: number
  foldersCount: number
}

export function formatBytes(bytes: number, decimals = 1): string {
  if (bytes <= 0) return "0 B"
  const k = 1024
  const dm = decimals < 0 ? 0 : decimals
  const sizes = ["B", "KB", "MB", "GB"]
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${parseFloat((bytes / k ** i).toFixed(dm))} ${sizes[i]}`
}

export async function getDatabaseStatsAsync(): Promise<DatabaseStats> {
  const pageCountResult = await expo.getFirstAsync<{ page_count: number }>("PRAGMA page_count;")
  const pageSizeResult = await expo.getFirstAsync<{ page_size: number }>("PRAGMA page_size;")

  const pageCount = pageCountResult?.page_count ?? 0
  const pageSize = pageSizeResult?.page_size ?? 0
  const sizeBytes = pageCount * pageSize

  const notesResult = await expo.getFirstAsync<{ count: number }>(
    "SELECT COUNT(*) as count FROM notes WHERE trashed = 0 AND deleted_at IS NULL;",
  )
  const foldersResult = await expo.getFirstAsync<{ count: number }>(
    "SELECT COUNT(*) as count FROM folders WHERE deleted_at IS NULL;",
  )

  return {
    sizeBytes,
    formattedSize: formatBytes(sizeBytes),
    notesCount: notesResult?.count ?? 0,
    foldersCount: foldersResult?.count ?? 0,
  }
}
