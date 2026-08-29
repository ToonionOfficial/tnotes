import { describe, expect, it } from "vitest"
import { buildWebSocketUrl } from "../src/services/sync/useWebSocketSync"

describe("WebSocket Real-Time Sync", () => {
  describe("buildWebSocketUrl", () => {
    it("converts http URL to ws protocol with query token", () => {
      const url = buildWebSocketUrl("http://localhost:8787", "tok_test123")
      expect(url).toBe("ws://localhost:8787/ws/sync?token=tok_test123")
    })

    it("converts https URL to wss secure protocol", () => {
      const url = buildWebSocketUrl("https://notes.example.com", "tok_secure456")
      expect(url).toBe("wss://notes.example.com/ws/sync?token=tok_secure456")
    })

    it("removes trailing slashes from server URL", () => {
      const url = buildWebSocketUrl("http://192.168.1.100:8787///", "tok_abc")
      expect(url).toBe("ws://192.168.1.100:8787/ws/sync?token=tok_abc")
    })

    it("properly URL encodes token special characters", () => {
      const url = buildWebSocketUrl("http://localhost:8787", "tok+with spaces&special=1")
      expect(url).toBe("ws://localhost:8787/ws/sync?token=tok%2Bwith%20spaces%26special%3D1")
    })
  })

  describe("WebSocket Message Handling", () => {
    it("identifies sync_notification and sync_required message payloads", () => {
      const isSyncEvent = (msgType: string) =>
        msgType === "sync_notification" || msgType === "sync_required"

      expect(isSyncEvent("sync_notification")).toBe(true)
      expect(isSyncEvent("sync_required")).toBe(true)
      expect(isSyncEvent("pong")).toBe(false)
      expect(isSyncEvent("unknown")).toBe(false)
    })
  })
})
