import { useEffect, useRef } from 'react'

export interface WsSyncPayload {
  sender_device_id?: string
  count?: number
  reason?: string
}

export interface WebSocketSyncOptions {
  enabled?: boolean
  onSyncNotification?: (payload?: WsSyncPayload) => void
  onStatusChange?: (status: 'connected' | 'connecting' | 'disconnected') => void
}

export function useWebSocketSync({
  enabled = true,
  onSyncNotification,
  onStatusChange,
}: WebSocketSyncOptions = {}) {
  const wsRef = useRef<WebSocket | null>(null)
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const heartbeatIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null)
  const reconnectDelayRef = useRef(1000)

  useEffect(() => {
    if (!enabled || typeof window === 'undefined') {
      if (wsRef.current) {
        wsRef.current.close()
        wsRef.current = null
      }
      onStatusChange?.('disconnected')
      return
    }

    let isDisposed = false

    const connect = () => {
      if (isDisposed || wsRef.current) return

      try {
        onStatusChange?.('connecting')
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
        const wsUrl = `${protocol}//${window.location.host}/ws/sync`
        const ws = new WebSocket(wsUrl)
        wsRef.current = ws

        ws.onopen = () => {
          reconnectDelayRef.current = 1000
          onStatusChange?.('connected')
          if (heartbeatIntervalRef.current) clearInterval(heartbeatIntervalRef.current)
          heartbeatIntervalRef.current = setInterval(() => {
            if (ws.readyState === WebSocket.OPEN) {
              try {
                ws.send(JSON.stringify({ type: 'ping' }))
              } catch {}
            }
          }, 30000)
        }

        ws.onmessage = (event) => {
          try {
            const rawData = typeof event.data === 'string' ? event.data : String(event.data)
            const msg = JSON.parse(rawData) as {
              type?: string
              data?: WsSyncPayload
            }
            if (msg?.type === 'sync_notification' || msg?.type === 'sync_required') {
              onSyncNotification?.(msg.data)
            }
          } catch {}
        }

        ws.onerror = () => {
          // Handled in onclose
        }

        ws.onclose = () => {
          wsRef.current = null
          onStatusChange?.('disconnected')
          if (heartbeatIntervalRef.current) {
            clearInterval(heartbeatIntervalRef.current)
            heartbeatIntervalRef.current = null
          }

          if (!isDisposed && enabled) {
            const delay = reconnectDelayRef.current
            reconnectDelayRef.current = Math.min(delay * 1.5, 30000)
            if (reconnectTimeoutRef.current) clearTimeout(reconnectTimeoutRef.current)
            reconnectTimeoutRef.current = setTimeout(connect, delay)
          }
        }
      } catch {
        onStatusChange?.('disconnected')
        if (!isDisposed) {
          if (reconnectTimeoutRef.current) clearTimeout(reconnectTimeoutRef.current)
          reconnectTimeoutRef.current = setTimeout(connect, 5000)
        }
      }
    }

    connect()

    return () => {
      isDisposed = true
      if (reconnectTimeoutRef.current) clearTimeout(reconnectTimeoutRef.current)
      if (heartbeatIntervalRef.current) clearInterval(heartbeatIntervalRef.current)
      if (wsRef.current) {
        wsRef.current.close()
        wsRef.current = null
      }
      onStatusChange?.('disconnected')
    }
  }, [enabled, onSyncNotification, onStatusChange])
}
