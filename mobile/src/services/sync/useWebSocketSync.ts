import { useQueryClient } from "@tanstack/react-query"
import { useCallback, useEffect, useRef } from "react"
import { AppState, type AppStateStatus } from "react-native"
import { getSyncMeta } from "@/db/queries"
import { statsKeys } from "@/hooks/useDatabaseStats"
import { deviceKeys } from "@/hooks/useDevices"
import { folderKeys } from "@/hooks/useFolders"
import { noteKeys } from "@/hooks/useNotes"
import { syncKeys, useAutoSyncQuery, useSyncState } from "@/hooks/useSyncState"
import { executeSyncAsync } from "./syncEngine"

export function buildWebSocketUrl(rawServerUrl: string, authToken: string): string {
  const cleanUrl = rawServerUrl.trim().replace(/\/+$/, "")
  const wsProtocol = cleanUrl.startsWith("https://") ? "wss://" : "ws://"
  const hostPath = cleanUrl.replace(/^https?:\/\//i, "")
  return `${wsProtocol}${hostPath}/ws/sync?token=${encodeURIComponent(authToken.trim())}`
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
    if (!serverUrl || !authToken) return

    let isDisposed = false

    const connect = () => {
      if (isDisposed || wsRef.current) return

      try {
        const wsUrl = buildWebSocketUrl(serverUrl, authToken)
        const ws = new WebSocket(wsUrl)
        wsRef.current = ws

        ws.onopen = () => {
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
            const msg = JSON.parse(rawData) as { type?: string }
            if (msg?.type === "sync_notification" || msg?.type === "sync_required") {
              void triggerSync()
            }
          } catch {}
        }

        ws.onerror = () => {
          // Socket will invoke onclose next
        }

        ws.onclose = () => {
          wsRef.current = null
          if (heartbeatIntervalRef.current) {
            clearInterval(heartbeatIntervalRef.current)
            heartbeatIntervalRef.current = null
          }

          if (!isDisposed && isConnected && isAutoSyncOn) {
            const delay = reconnectDelayRef.current
            reconnectDelayRef.current = Math.min(delay * 1.5, 30000)
            if (reconnectTimeoutRef.current) clearTimeout(reconnectTimeoutRef.current)
            reconnectTimeoutRef.current = setTimeout(connect, delay)
          }
        }
      } catch {
        if (!isDisposed) {
          if (reconnectTimeoutRef.current) clearTimeout(reconnectTimeoutRef.current)
          reconnectTimeoutRef.current = setTimeout(connect, 5000)
        }
      }
    }

    const handleAppStateChange = (nextAppState: AppStateStatus) => {
      if (nextAppState === "active") {
        if (!wsRef.current) {
          reconnectDelayRef.current = 1000
          connect()
        }
      } else if (nextAppState === "background") {
        if (wsRef.current) {
          wsRef.current.close()
          wsRef.current = null
        }
      }
    }

    const subscription = AppState.addEventListener("change", handleAppStateChange)
    connect()

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
