import { eq } from "drizzle-orm"
import { db } from "../index"
import { syncMeta, users } from "../schema"
import { ensureUser, getOrCreateDeviceId, getSyncMeta, setSyncMeta } from "./sync"

export interface QrPairPayload {
  v?: number
  url: string
  token: string
  device_id?: string
  deviceId?: string
  user_id?: string
  userId?: string
  username?: string
  pairing_code?: string
  expires_at?: number
}

export interface SyncStatus {
  isConnected: boolean
  serverUrl: string | null
  username: string
  userId: string
  deviceId: string
  lastSyncedAt: string | null
}

export function normalizeServerUrl(rawUrl: string): string {
  let url = rawUrl.trim()
  while (url.endsWith("/")) {
    url = url.slice(0, -1)
  }
  return url
}

export async function getSyncStatusAsync(): Promise<SyncStatus> {
  const serverUrl = getSyncMeta("server_url")
  const authToken = getSyncMeta("auth_token")
  const userId = getSyncMeta("user_id") ?? "default_user"
  const username = getSyncMeta("username") ?? (serverUrl ? "Synced User" : "Local User")
  const deviceId = getOrCreateDeviceId()
  const lastSyncedAt = getSyncMeta("last_synced_at")

  const isConnected = Boolean(serverUrl && authToken)

  return {
    isConnected,
    serverUrl,
    username,
    userId,
    deviceId,
    lastSyncedAt,
  }
}

export async function pairWithServerAsync(payload: QrPairPayload): Promise<SyncStatus> {
  const serverUrl = normalizeServerUrl(payload.url)
  const token = payload.token.trim()
  const deviceId = payload.device_id || payload.deviceId || getOrCreateDeviceId()
  const userId = payload.user_id || payload.userId || "synced_user"
  const username = payload.username?.trim() || "Synced User"

  setSyncMeta("server_url", serverUrl)
  setSyncMeta("auth_token", token)
  setSyncMeta("device_id", deviceId)
  setSyncMeta("user_id", userId)
  setSyncMeta("username", username)
  setSyncMeta("paired_at", String(Date.now()))

  const existingUser = db.select().from(users).where(eq(users.id, userId)).get()
  if (!existingUser) {
    db.insert(users)
      .values({
        id: userId,
        username,
        createdAt: Date.now(),
      })
      .run()
  } else {
    db.update(users).set({ username }).where(eq(users.id, userId)).run()
  }

  try {
    const controller = new AbortController()
    const timeoutId = setTimeout(() => controller.abort(), 4000)
    await fetch(`${serverUrl}/api/health`, {
      signal: controller.signal,
      headers: {
        Authorization: `Bearer ${token}`,
        "X-Device-ID": deviceId,
      },
    })
    clearTimeout(timeoutId)
  } catch {}

  return getSyncStatusAsync()
}

export async function unpairServerAsync(): Promise<SyncStatus> {
  db.delete(syncMeta).where(eq(syncMeta.key, "server_url")).run()
  db.delete(syncMeta).where(eq(syncMeta.key, "auth_token")).run()
  db.delete(syncMeta).where(eq(syncMeta.key, "paired_at")).run()
  db.delete(syncMeta).where(eq(syncMeta.key, "last_synced_at")).run()

  ensureUser()
  setSyncMeta("user_id", "default_user")
  setSyncMeta("username", "Local User")

  return getSyncStatusAsync()
}
