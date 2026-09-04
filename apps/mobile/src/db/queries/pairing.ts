import { eq } from "drizzle-orm"
import { db } from "../index"
import { folders, localChanges, notes, syncMeta } from "../schema"
import { getOrCreateDeviceId, getSyncMeta, setSyncMeta, upsertUser } from "./sync"

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
  autoSyncEnabled: boolean
}

export function getAutoSyncEnabled(): boolean {
  const val = getSyncMeta("auto_sync")
  return val !== "false"
}

export function setAutoSyncEnabled(enabled: boolean): void {
  setSyncMeta("auto_sync", String(enabled))
}

export function normalizeServerUrl(rawUrl: string): string {
  let url = rawUrl.trim()
  while (url.endsWith("/")) {
    url = url.slice(0, -1)
  }
  return url
}

export function formatDisplayServerUrl(rawUrl?: string | null): string {
  if (!rawUrl) return ""
  return rawUrl
    .trim()
    .replace(/^https?:\/\//i, "")
    .replace(/\/+$/, "")
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
    autoSyncEnabled: getAutoSyncEnabled(),
  }
}

export async function pairWithServerAsync(payload: QrPairPayload): Promise<SyncStatus> {
  const serverUrl = normalizeServerUrl(payload.url)
  const inputCodeOrToken = payload.token.trim() || payload.pairing_code?.trim() || ""
  const is6DigitCode = /^\d{6}$/.test(inputCodeOrToken)

  let activeToken = inputCodeOrToken
  let activeDeviceId = payload.device_id || payload.deviceId || getOrCreateDeviceId()
  let activeUserId = payload.user_id || payload.userId || "synced_user"
  let activeUsername = payload.username?.trim() || "Synced User"

  const controller = new AbortController()
  const timeoutId = setTimeout(() => controller.abort(), 6000)

  try {
    if (is6DigitCode) {
      const claimRes = await fetch(`${serverUrl}/api/pair/claim`, {
        method: "POST",
        signal: controller.signal,
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          code: inputCodeOrToken,
          device_name: "Mobile App",
          platform: "mobile",
        }),
      })

      if (!claimRes.ok) {
        const errorText = await claimRes.text()
        throw new Error(errorText || "Invalid or expired pairing code")
      }

      const claimData = (await claimRes.json()) as {
        token: string
        user_id: string
        username: string
        device_id: string
      }

      activeToken = claimData.token
      activeUserId = claimData.user_id
      activeUsername = claimData.username
      activeDeviceId = claimData.device_id
    } else {
      const meRes = await fetch(`${serverUrl}/api/me`, {
        signal: controller.signal,
        headers: {
          Authorization: `Bearer ${activeToken}`,
          "X-Device-ID": activeDeviceId,
        },
      })

      if (meRes.ok) {
        const meData = (await meRes.json()) as {
          user_id: string
          username: string
        }
        if (meData.username) activeUsername = meData.username
        if (meData.user_id) activeUserId = meData.user_id
      }
    }
  } finally {
    clearTimeout(timeoutId)
  }

  const previousUserId = getSyncMeta("user_id")

  setSyncMeta("server_url", serverUrl)
  setSyncMeta("auth_token", activeToken)
  setSyncMeta("device_id", activeDeviceId)
  setSyncMeta("user_id", activeUserId)
  setSyncMeta("username", activeUsername)
  setSyncMeta("paired_at", String(Date.now()))

  upsertUser(activeUserId, activeUsername)

  // If pairing to a DIFFERENT authenticated user account, wipe local cache so new account starts clean
  if (previousUserId && previousUserId !== "default_user" && previousUserId !== activeUserId) {
    db.delete(notes).run()
    db.delete(folders).run()
    db.delete(localChanges).run()
    db.delete(syncMeta).where(eq(syncMeta.key, "last_synced_at")).run()
  } else if (previousUserId === "default_user" || !previousUserId) {
    // Migrate local notes and folders created under default_user to the authenticated user ID
    db.update(notes).set({ userId: activeUserId }).where(eq(notes.userId, "default_user")).run()
    db.update(folders).set({ userId: activeUserId }).where(eq(folders.userId, "default_user")).run()
  }

  return getSyncStatusAsync()
}

export async function unpairServerAsync(): Promise<SyncStatus> {
  db.delete(syncMeta).where(eq(syncMeta.key, "server_url")).run()
  db.delete(syncMeta).where(eq(syncMeta.key, "auth_token")).run()
  db.delete(syncMeta).where(eq(syncMeta.key, "paired_at")).run()
  db.delete(syncMeta).where(eq(syncMeta.key, "last_synced_at")).run()

  return getSyncStatusAsync()
}
