import { useEffect, useRef } from 'react'
import { apiFetch } from '@/lib/api'

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

  const onSyncNotificationRef = useRef(onSyncNotification)
  onSyncNotificationRef.current = onSyncNotification

  const onStatusChangeRef = useRef(onStatusChange)
  onStatusChangeRef.current = onStatusChange

  useEffect(() => {
    if (!enabled || typeof window === 'undefined') {
      if (wsRef.current) {
        wsRef.current.close()
        wsRef.current = null
      }
      onStatusChangeRef.current?.('disconnected')
      return
    }

    let isDisposed = false

    const connect = async () => {
      if (isDisposed || wsRef.current) return

      try {
        onStatusChangeRef.current?.('connecting')

        let ticket: string | null = null
        try {
          console.log('[WEB_WS] Requesting WebSocket ticket from /api/ws/ticket...')
          const res = await apiFetch<{ ticket: string }>('/api/ws/ticket', {
            method: 'POST',
          })
          ticket = res?.ticket ?? null
          console.log('[WEB_WS] Ticket received:', ticket)
        } catch (ticketErr) {
          console.warn('[WEB_WS] Ticket request failed, falling back to cookie:', ticketErr)
        }

        if (isDisposed) return

        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
        const baseUrl = `${protocol}//${window.location.host}/ws/sync`
        const wsUrl = ticket ? `${baseUrl}?ticket=${encodeURIComponent(ticket)}` : baseUrl

        console.log('[WEB_WS] Opening WebSocket connection:', wsUrl)
        const ws = new WebSocket(wsUrl)
        wsRef.current = ws

        ws.onopen = () => {
          console.log('[WEB_WS] WebSocket connected successfully!')
          reconnectDelayRef.current = 1000
          onStatusChangeRef.current?.('connected')
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
            console.log('[WEB_WS] Received message:', rawData)
            const msg = JSON.parse(rawData) as {
              type?: string
              data?: WsSyncPayload
            }
            if (msg?.type === 'sync_notification' || msg?.type === 'sync_required') {
              console.log('[WEB_WS] Dispatching onSyncNotification event:', msg)
              onSyncNotificationRef.current?.(msg.data)
            }
          } catch (parseErr) {
            console.warn('[WEB_WS] Failed to parse message JSON:', parseErr)
          }
        }

        ws.onerror = (err) => {
          console.warn('[WEB_WS] WebSocket error:', err)
        }

        ws.onclose = (event) => {
          console.log(`[WEB_WS] WebSocket closed: code=${event.code}, reason='${event.reason}'`)
          wsRef.current = null
          onStatusChangeRef.current?.('disconnected')
          if (heartbeatIntervalRef.current) {
            clearInterval(heartbeatIntervalRef.current)
            heartbeatIntervalRef.current = null
          }

          if (!isDisposed && enabled) {
            const delay = reconnectDelayRef.current
            reconnectDelayRef.current = Math.min(delay * 1.5, 30000)
            if (reconnectTimeoutRef.current) clearTimeout(reconnectTimeoutRef.current)
            reconnectTimeoutRef.current = setTimeout(() => {
              void connect()
            }, delay)
          }
        }
      } catch {
        onStatusChangeRef.current?.('disconnected')
        if (!isDisposed) {
          if (reconnectTimeoutRef.current) clearTimeout(reconnectTimeoutRef.current)
          reconnectTimeoutRef.current = setTimeout(() => {
            void connect()
          }, 5000)
        }
      }
    }

    void connect()

    return () => {
      isDisposed = true
      if (reconnectTimeoutRef.current) clearTimeout(reconnectTimeoutRef.current)
      if (heartbeatIntervalRef.current) clearInterval(heartbeatIntervalRef.current)
      if (wsRef.current) {
        wsRef.current.close()
        wsRef.current = null
      }
      onStatusChangeRef.current?.('disconnected')
    }
  }, [enabled])
}
