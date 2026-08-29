import {
  clearLocalChanges,
  getPendingLocalChanges,
  getSyncCredentials,
  getSyncMeta,
  setSyncMeta,
} from "@/db/queries"
import { applyRemoteChangesAsync } from "./applyRemoteChanges"
import type { SyncChange, SyncEnvelope, SyncResponse, SyncResult } from "./types"

type SyncListener = (isSyncing: boolean) => void
const syncListeners = new Set<SyncListener>()
let isCurrentlySyncing = false

export function subscribeIsSyncing(listener: SyncListener): () => void {
  syncListeners.add(listener)
  listener(isCurrentlySyncing)
  return () => {
    syncListeners.delete(listener)
  }
}

export function getGlobalIsSyncing(): boolean {
  return isCurrentlySyncing
}

export function setGlobalIsSyncing(syncing: boolean): void {
  if (isCurrentlySyncing === syncing) return
  isCurrentlySyncing = syncing
  for (const listener of syncListeners) {
    try {
      listener(syncing)
    } catch {}
  }
}

export function normalizePayloadForSync(
  entityType: string,
  raw: Record<string, unknown>,
): Record<string, unknown> {
  if (entityType === "note") {
    return {
      id: String(raw.id ?? ""),
      user_id: String(raw.user_id ?? raw.userId ?? ""),
      folder_id:
        raw.folder_id !== undefined
          ? raw.folder_id
            ? String(raw.folder_id)
            : null
          : raw.folderId
            ? String(raw.folderId)
            : null,
      title: String(raw.title ?? ""),
      body: String(raw.body ?? ""),
      pinned: Boolean(raw.pinned),
      trashed: Boolean(raw.trashed),
      version: Number(raw.version ?? 1),
      created_at: Number(raw.created_at ?? raw.createdAt ?? Date.now()),
      updated_at: Number(raw.updated_at ?? raw.updatedAt ?? Date.now()),
      deleted_at:
        raw.deleted_at !== undefined
          ? raw.deleted_at
            ? Number(raw.deleted_at)
            : null
          : raw.deletedAt
            ? Number(raw.deletedAt)
            : null,
      device_id: String(raw.device_id ?? raw.deviceId ?? ""),
      checksum: String(raw.checksum ?? ""),
    }
  }

  if (entityType === "folder") {
    return {
      id: String(raw.id ?? ""),
      user_id: String(raw.user_id ?? raw.userId ?? ""),
      parent_id:
        raw.parent_id !== undefined
          ? raw.parent_id
            ? String(raw.parent_id)
            : null
          : raw.parentId
            ? String(raw.parentId)
            : null,
      name: String(raw.name ?? ""),
      icon: String(raw.icon ?? "folder"),
      sort_order: Number(raw.sort_order ?? raw.sortOrder ?? 0),
      version: Number(raw.version ?? 1),
      created_at: Number(raw.created_at ?? raw.createdAt ?? Date.now()),
      updated_at: Number(raw.updated_at ?? raw.updatedAt ?? Date.now()),
      deleted_at:
        raw.deleted_at !== undefined
          ? raw.deleted_at
            ? Number(raw.deleted_at)
            : null
          : raw.deletedAt
            ? Number(raw.deletedAt)
            : null,
      device_id: String(raw.device_id ?? raw.deviceId ?? ""),
    }
  }

  return raw
}

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
  } finally {
    setGlobalIsSyncing(false)
  }
}
