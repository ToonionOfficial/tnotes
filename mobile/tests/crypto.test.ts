import { describe, expect, it } from "vitest"
import { computeChecksum } from "../src/utils/crypto"

describe("Crypto Utilities", () => {
  describe("computeChecksum (Blake3)", () => {
    it("computes deterministic hex hash for empty string", () => {
      const hash1 = computeChecksum("")
      const hash2 = computeChecksum("")

      expect(hash1).toBe(hash2)
      expect(hash1.length).toBe(64)
      expect(hash1).toBe("af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262")
    })

    it("produces identical checksum for same content", () => {
      const text = "# My Note Title\n\nThis is the note content."
      expect(computeChecksum(text)).toBe(computeChecksum(text))
    })

    it("produces distinct checksums for differing content", () => {
      const text1 = "Hello World"
      const text2 = "Hello World!"
      expect(computeChecksum(text1)).not.toBe(computeChecksum(text2))
    })
  })
})
