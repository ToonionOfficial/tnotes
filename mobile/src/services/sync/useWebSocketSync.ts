import { useQueryClient } from "@tanstack/react-query"
import { useCallback, useEffect, useRef } from "react"
import { AppState, type AppStateStatus } from "react-native"
import { getOrCreateDeviceId, getSyncMeta } from "@/db/queries"
import { statsKeys } from "@/hooks/useDatabaseStats"
import { deviceKeys } from "@/hooks/useDevices"
import { folderKeys } from "@/hooks/useFolders"
import { noteKeys } from "@/hooks/useNotes"
import { syncKeys, useAutoSyncQuery, useSyncState } from "@/hooks/useSyncState"
import { executeSyncAsync } from "./syncEngine"

export async function fetchWsTicketAsync(
  serverUrl: string,
  authToken: string,
  deviceId: string,
): Promise<string> {
  const cleanUrl = serverUrl.trim().replace(/\/+$/, "")
  const res = await fetch(`${cleanUrl}/api/ws/ticket`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${authToken.trim()}`,
      "X-Device-ID": deviceId,
    },
  })

  if (!res.ok) {
    throw new Error(`Failed to obtain WebSocket ticket: ${res.statusText}`)
  }

  const data = (await res.json()) as { ticket: string }
  return data.ticket
}

export function buildWebSocketUrl(rawServerUrl: string, ticket: string): string {
  const cleanUrl = rawServerUrl.trim().replace(/\/+$/, "")
  const wsProtocol = cleanUrl.startsWith("https://") ? "wss://" : "ws://"
  const hostPath = cleanUrl.replace(/^https?:\/\//i, "")
  return `${wsProtocol}${hostPath}/ws/sync?ticket=${encodeURIComponent(ticket.trim())}`
}

export function useWebSocketSync(): void {
  const queryClient = useQueryClient()
  const { data: syncStatus } = useSyncState()
  const { data: autoSyncEnabled } = useAutoSyncQuery()
  const wsRef = useRef<WebSocket | null>(null)
  const isSyncingRef = useRef(false)
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const heartbeatIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null)
  const reconnectDelayRef = useRef(1000)

  const isConnected = syncStatus?.isConnected ?? false
  const isAutoSyncOn = autoSyncEnabled !== false

  const triggerSync = useCallback(async () => {
    if (isSyncingRef.current) return
    isSyncingRef.current = true
    try {
      const result = await executeSyncAsync()
      if (result.success && result.syncedDownCount > 0) {
        void queryClient.invalidateQueries({ queryKey: noteKeys.all })
        void queryClient.invalidateQueries({ queryKey: folderKeys.all })
        void queryClient.invalidateQueries({ queryKey: statsKeys.all })
      }
      if (result.success) {
        void queryClient.invalidateQueries({ queryKey: syncKeys.all })
        void queryClient.invalidateQueries({ queryKey: deviceKeys.all })
      }
    } finally {
      isSyncingRef.current = false
    }
  }, [queryClient])

  useEffect(() => {
    if (!isConnected || !isAutoSyncOn) {
      if (wsRef.current) {
        wsRef.current.close()
        wsRef.current = null
      }
      return
    }

    const serverUrl = getSyncMeta("server_url")
    const authToken = getSyncMeta("auth_token")
    const deviceId = getOrCreateDeviceId()
    if (!serverUrl || !authToken) return

    let isDisposed = false

    const connect = async () => {
      if (isDisposed || wsRef.current) return

      try {
        console.log("[MOBILE_WS] Requesting WS ticket from server...")
        const ticket = await fetchWsTicketAsync(serverUrl, authToken, deviceId)
        if (isDisposed || wsRef.current) return

        const wsUrl = buildWebSocketUrl(serverUrl, ticket)
        console.log(`[MOBILE_WS] Connecting to WebSocket: ${wsUrl}`)
        const ws = new WebSocket(wsUrl)
        wsRef.current = ws

        ws.onopen = () => {
          console.log("[MOBILE_WS] WebSocket connected successfully!")
          reconnectDelayRef.current = 1000
          if (heartbeatIntervalRef.current) clearInterval(heartbeatIntervalRef.current)
          heartbeatIntervalRef.current = setInterval(() => {
            if (ws.readyState === WebSocket.OPEN) {
              try {
                ws.send(JSON.stringify({ type: "ping" }))
              } catch {}
            }
          }, 30000)
        }

        ws.onmessage = (event) => {
          try {
            const rawData = typeof event.data === "string" ? event.data : String(event.data)
            console.log(`[MOBILE_WS] Message received: ${rawData}`)
            const msg = JSON.parse(rawData) as { type?: string }
            if (msg?.type === "sync_notification" || msg?.type === "sync_required") {
              console.log(`[MOBILE_WS] Triggering instant sync for event: ${msg.type}`)
              void triggerSync()
            }
          } catch {}
        }

        ws.onerror = (err) => {
          console.warn("[MOBILE_WS] WebSocket error:", err)
        }

        ws.onclose = (event) => {
          console.log(
            `[MOBILE_WS] WebSocket closed (code: ${event.code}, reason: '${event.reason}')`,
          )
          wsRef.current = null
          if (heartbeatIntervalRef.current) {
            clearInterval(heartbeatIntervalRef.current)
            heartbeatIntervalRef.current = null
          }

          if (!isDisposed && isConnected && isAutoSyncOn) {
            const delay = reconnectDelayRef.current
            reconnectDelayRef.current = Math.min(delay * 1.5, 30000)
            if (reconnectTimeoutRef.current) clearTimeout(reconnectTimeoutRef.current)
            reconnectTimeoutRef.current = setTimeout(() => {
              void connect()
            }, delay)
          }
        }
      } catch (err) {
        console.warn("[MOBILE_WS] Ticket or connect error:", err)
        if (!isDisposed) {
          if (reconnectTimeoutRef.current) clearTimeout(reconnectTimeoutRef.current)
          reconnectTimeoutRef.current = setTimeout(() => {
            void connect()
          }, 5000)
        }
      }
    }

    const handleAppStateChange = (nextAppState: AppStateStatus) => {
      if (nextAppState === "active") {
        if (!wsRef.current) {
          reconnectDelayRef.current = 1000
          void connect()
        }
      } else if (nextAppState === "background") {
        if (wsRef.current) {
          wsRef.current.close()
          wsRef.current = null
        }
      }
    }

    const subscription = AppState.addEventListener("change", handleAppStateChange)
    void connect()

    return () => {
      isDisposed = true
      subscription.remove()
      if (reconnectTimeoutRef.current) clearTimeout(reconnectTimeoutRef.current)
      if (heartbeatIntervalRef.current) clearInterval(heartbeatIntervalRef.current)
      if (wsRef.current) {
        wsRef.current.close()
        wsRef.current = null
      }
    }
  }, [isConnected, isAutoSyncOn, triggerSync])
}
