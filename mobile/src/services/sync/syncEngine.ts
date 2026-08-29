import {
  clearLocalChanges,
  getPendingLocalChanges,
  getSyncCredentials,
  getSyncMeta,
  setSyncMeta,
} from "@/db/queries"
import { computeChecksum } from "@/utils/crypto"
import { applyRemoteChangesAsync } from "./applyRemoteChanges"
import type { SyncChange, SyncEnvelope, SyncResponse, SyncResult } from "./types"

let globalIsSyncing = false
const syncingListeners = new Set<(isSyncing: boolean) => void>()

export function getGlobalIsSyncing(): boolean {
  return globalIsSyncing
}

export function subscribeIsSyncing(listener: (isSyncing: boolean) => void): () => void {
  syncingListeners.add(listener)
  return () => {
    syncingListeners.delete(listener)
  }
}

function setGlobalIsSyncing(syncing: boolean): void {
  if (globalIsSyncing !== syncing) {
    globalIsSyncing = syncing
    for (const listener of syncingListeners) {
      try {
        listener(syncing)
      } catch {}
    }
  }
}

export function normalizePayloadForSync(
  entityType: string,
  raw: Record<string, unknown>,
): Record<string, unknown> {
  if (entityType === "note") {
    return {
      id: raw.id,
      user_id: raw.userId ?? raw.user_id,
      folder_id: raw.folderId !== undefined ? raw.folderId : (raw.folder_id ?? null),
      title: raw.title ?? "",
      body: raw.body ?? "",
      pinned: Boolean(raw.pinned),
      trashed: Boolean(raw.trashed),
      version: Number(raw.version ?? 1),
      created_at: Number(raw.createdAt ?? raw.created_at ?? Date.now()),
      updated_at: Number(raw.updatedAt ?? raw.updated_at ?? Date.now()),
      deleted_at:
        raw.deletedAt !== undefined
          ? raw.deletedAt
          : (raw.deleted_at ?? (raw.trashed ? Date.now() : null)),
      device_id: raw.deviceId ?? raw.device_id,
      checksum: raw.checksum ?? computeChecksum(String(raw.body ?? "")),
    }
  }

  if (entityType === "folder") {
    return {
      id: raw.id,
      user_id: raw.userId ?? raw.user_id,
      parent_id: raw.parentId !== undefined ? raw.parentId : (raw.parent_id ?? null),
      name: raw.name ?? "",
      icon: raw.icon ?? "folder",
      sort_order: Number(raw.sortOrder ?? raw.sort_order ?? 0),
      version: Number(raw.version ?? 1),
      created_at: Number(raw.createdAt ?? raw.created_at ?? Date.now()),
      updated_at: Number(raw.updatedAt ?? raw.updated_at ?? Date.now()),
      deleted_at: raw.deletedAt !== undefined ? raw.deletedAt : (raw.deleted_at ?? null),
      device_id: raw.deviceId ?? raw.device_id,
    }
  }

  return raw
}

const SYNC_BATCH_SIZE = 100
const SYNC_TIMEOUT_MS = 30000 // 30s per batch

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

  setGlobalIsSyncing(true)

  let totalSyncedUp = 0
  let totalSyncedDown = 0
  let latestServerTime = 0

  try {
    while (true) {
      const pendingRows = getPendingLocalChanges()
      const currentBatch = pendingRows.slice(0, SYNC_BATCH_SIZE)
      const changeIds: number[] = []
      const changes: SyncChange[] = []

      for (const row of currentBatch) {
        changeIds.push(row.id)
        let payload: Record<string, unknown> = {}
        try {
          payload = JSON.parse(row.payload)
        } catch {}

        const normalizedPayload = normalizePayloadForSync(row.entityType, payload)

        changes.push({
          entity_type: row.entityType as "note" | "folder",
          entity_id: row.entityId,
          version: row.version,
          updated_at: row.updatedAt,
          tombstone: Boolean(row.tombstone),
          payload: normalizedPayload,
        })
      }

      const lastSyncAtStr = getSyncMeta("last_sync_at")
      const lastSyncAt = lastSyncAtStr ? Number(lastSyncAtStr) : 0

      const envelope: SyncEnvelope = {
        device_id: creds.deviceId,
        last_seq: 0,
        last_sync_at: lastSyncAt,
        changes,
      }

      const controller = new AbortController()
      const timeoutId = setTimeout(() => controller.abort(), SYNC_TIMEOUT_MS)

      let response: Response
      try {
        response = await fetch(`${creds.serverUrl}/api/sync`, {
          method: "POST",
          signal: controller.signal,
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${creds.token}`,
            "X-Device-ID": creds.deviceId,
          },
          body: JSON.stringify(envelope),
        })
      } finally {
        clearTimeout(timeoutId)
      }

      if (!response.ok) {
        const errorText = await response.text()
        return {
          success: false,
          syncedUpCount: totalSyncedUp,
          syncedDownCount: totalSyncedDown,
          serverTime: latestServerTime,
          error: `Server responded with ${response.status}: ${errorText}`,
        }
      }

      const data = (await response.json()) as SyncResponse
      const syncedDownCount = await applyRemoteChangesAsync(data.changes ?? [])

      clearLocalChanges(changeIds)

      totalSyncedUp += changes.length
      totalSyncedDown += syncedDownCount
      latestServerTime = data.server_time

      setSyncMeta("last_sync_at", String(data.server_time))
      setSyncMeta("last_synced_at", String(Date.now()))

      const hasMoreRemote = Boolean(data.has_more)
      const hasMoreLocal = pendingRows.length > SYNC_BATCH_SIZE

      // If no more remote pages and no more local changes to upload, complete sync
      if (!hasMoreRemote && !hasMoreLocal) {
        break
      }
    }

    return {
      success: true,
      syncedUpCount: totalSyncedUp,
      syncedDownCount: totalSyncedDown,
      serverTime: latestServerTime,
    }
  } catch (err) {
    const message = err instanceof Error ? err.message : "Network error"
    return {
      success: false,
      syncedUpCount: totalSyncedUp,
      syncedDownCount: totalSyncedDown,
      serverTime: latestServerTime,
      error: message,
    }
  } finally {
    setGlobalIsSyncing(false)
  }
}
