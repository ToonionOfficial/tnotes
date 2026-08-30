export type EntityType = "note" | "folder"

export interface SyncChange {
  entity_type: EntityType
  entity_id: string
  version: number
  updated_at: number
  tombstone: boolean
  payload: Record<string, unknown>
}

export interface SyncEnvelope {
  device_id: string
  last_seq: number
  last_sync_at: number
  changes: SyncChange[]
}

export interface SyncResponse {
  server_time: number
  cursor?: number
  has_more?: boolean
  changes: SyncChange[]
}

export interface SyncResult {
  success: boolean
  syncedUpCount: number
  syncedDownCount: number
  serverTime: number
  error?: string
}
