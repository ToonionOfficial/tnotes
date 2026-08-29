import { describe, expect, it } from "vitest"
import { formatDisplayServerUrl, normalizeServerUrl } from "../src/db/queries/pairing"

describe("Pairing Utilities", () => {
  describe("normalizeServerUrl", () => {
    it("removes single trailing slash", () => {
      expect(normalizeServerUrl("http://localhost:8787/")).toBe("http://localhost:8787")
    })

    it("removes multiple trailing slashes", () => {
      expect(normalizeServerUrl("http://192.168.2.4:8787///")).toBe("http://192.168.2.4:8787")
    })

    it("trims surrounding whitespace", () => {
      expect(normalizeServerUrl("   https://notes.example.com/   ")).toBe(
        "https://notes.example.com",
      )
    })

    it("leaves URL without trailing slash unchanged", () => {
      expect(normalizeServerUrl("https://sync.myfamily.org:8443")).toBe(
        "https://sync.myfamily.org:8443",
      )
    })
  })

  describe("formatDisplayServerUrl", () => {
    it("strips http:// prefix and trailing slashes", () => {
      expect(formatDisplayServerUrl("http://192.168.1.50:8787/")).toBe("192.168.1.50:8787")
    })

    it("strips https:// prefix", () => {
      expect(formatDisplayServerUrl("https://notes.example.com")).toBe("notes.example.com")
    })

    it("handles empty or null URL cleanly", () => {
      expect(formatDisplayServerUrl("")).toBe("")
      expect(formatDisplayServerUrl(null)).toBe("")
    })
  })

  describe("6-Digit Code Regex Detection", () => {
    const is6DigitCode = (str: string) => /^\d{6}$/.test(str.trim())

    it("matches standard 6-digit numeric OTP codes", () => {
      expect(is6DigitCode("842190")).toBe(true)
      expect(is6DigitCode("000123")).toBe(true)
      expect(is6DigitCode("999999")).toBe(true)
      expect(is6DigitCode(" 123456 ")).toBe(true)
    })

    it("rejects non-6-digit tokens or alphanumeric strings", () => {
      expect(is6DigitCode("12345")).toBe(false)
      expect(is6DigitCode("1234567")).toBe(false)
      expect(is6DigitCode("tok_abcdef123456")).toBe(false)
      expect(is6DigitCode("abc123")).toBe(false)
      expect(is6DigitCode("")).toBe(false)
    })
  })

  describe("Account Switching Decision Logic", () => {
    type SwitchAction = "wipe_and_resync" | "migrate_offline" | "resume_sync"

    const determineSwitchAction = (
      previousUserId: string | null,
      newUserId: string,
    ): SwitchAction => {
      if (previousUserId && previousUserId !== "default_user" && previousUserId !== newUserId) {
        return "wipe_and_resync"
      }
      if (previousUserId === "default_user" || !previousUserId) {
        return "migrate_offline"
      }
      return "resume_sync"
    }

    it("resumes sync when reconnecting to the same account", () => {
      expect(determineSwitchAction("user_alice", "user_alice")).toBe("resume_sync")
    })

    it("migrates offline notes on initial pairing from default_user", () => {
      expect(determineSwitchAction("default_user", "user_alice")).toBe("migrate_offline")
      expect(determineSwitchAction(null, "user_alice")).toBe("migrate_offline")
    })

    it("wipes previous user local cache when switching to a different account", () => {
      expect(determineSwitchAction("user_alice", "user_bob")).toBe("wipe_and_resync")
    })
  })
})
