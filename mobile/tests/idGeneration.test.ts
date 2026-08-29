import { describe, expect, it } from "vitest"
import { ulid } from "../src/utils/id"

describe("ULID Generator", () => {
  it("generates valid 26-character Crockford Base32 strings", () => {
    const id = ulid()
    expect(typeof id).toBe("string")
    expect(id.length).toBe(26)
    expect(id).toMatch(/^[0-9A-HJKMNP-TV-Z]{26}$/)
  })

  it("generates monotonically sortable unique IDs", () => {
    const id1 = ulid()
    const id2 = ulid()
    const id3 = ulid()

    expect(id1).not.toBe(id2)
    expect(id2).not.toBe(id3)
    expect(id1 < id2).toBe(true)
    expect(id2 < id3).toBe(true)
  })
})
