import {
  clearLocalChanges,
  getPendingLocalChanges,
  getSyncCredentials,
  getSyncMeta,
  setSyncMeta,
} from "@/db/queries"
import { applyRemoteChangesAsync } from "./applyRemoteChanges"
import type { SyncChange, SyncEnvelope, SyncResponse, SyncResult } from "./types"

export async function executeSyncAsync(): Promise<SyncResult> {
  const creds = getSyncCredentials()
  if (!creds.serverUrl || !creds.token || !creds.deviceId) {
    return {
      success: false,
      syncedUpCount: 0,
      syncedDownCount: 0,
      serverTime: 0,
      error: "Not paired with a sync server",
    }
  }

  const lastSyncAtStr = getSyncMeta("last_sync_at")
  const lastSyncAt = lastSyncAtStr ? Number(lastSyncAtStr) : 0

  const pendingRows = getPendingLocalChanges()
  const changeIds: number[] = []
  const changes: SyncChange[] = []

  for (const row of pendingRows) {
    changeIds.push(row.id)
    let payload: Record<string, unknown> = {}
    try {
      payload = JSON.parse(row.payload)
    } catch {}

    changes.push({
      entity_type: row.entityType as "note" | "folder",
      entity_id: row.entityId,
      version: row.version,
      updated_at: row.updatedAt,
      tombstone: Boolean(row.tombstone),
      payload,
    })
  }

  const envelope: SyncEnvelope = {
    device_id: creds.deviceId,
    last_seq: 0,
    last_sync_at: lastSyncAt,
    changes,
  }

  try {
    const controller = new AbortController()
    const timeoutId = setTimeout(() => controller.abort(), 10000)

    const response = await fetch(`${creds.serverUrl}/api/sync`, {
      method: "POST",
      signal: controller.signal,
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${creds.token}`,
        "X-Device-ID": creds.deviceId,
      },
      body: JSON.stringify(envelope),
    })

    clearTimeout(timeoutId)

    if (!response.ok) {
      const errorText = await response.text()
      return {
        success: false,
        syncedUpCount: 0,
        syncedDownCount: 0,
        serverTime: 0,
        error: `Server responded with ${response.status}: ${errorText}`,
      }
    }

    const data = (await response.json()) as SyncResponse

    const syncedDownCount = await applyRemoteChangesAsync(data.changes ?? [])

    clearLocalChanges(changeIds)

    setSyncMeta("last_sync_at", String(data.server_time))
    setSyncMeta("last_synced_at", String(Date.now()))

    return {
      success: true,
      syncedUpCount: changes.length,
      syncedDownCount,
      serverTime: data.server_time,
    }
  } catch (err) {
    const message = err instanceof Error ? err.message : "Network error"
    return {
      success: false,
      syncedUpCount: 0,
      syncedDownCount: 0,
      serverTime: 0,
      error: message,
    }
  }
}
