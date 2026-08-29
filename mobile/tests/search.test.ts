import { describe, expect, it } from "vitest"
import { buildFtsQuery } from "../src/utils/search"

describe("FTS5 Search Query Sanitizer", () => {
  it("formats single word into prefix match token", () => {
    expect(buildFtsQuery("meeting")).toBe('"meeting"*')
  })

  it("formats multi-word search into prefix match tokens", () => {
    expect(buildFtsQuery("react native")).toBe('"react"* "native"*')
  })

  it("strips special SQLite FTS syntax characters", () => {
    expect(buildFtsQuery('react* (native): "fast" -bug ~test ^start {end}')).toBe(
      '"react"* "native"* "fast"* "bug"* "test"* "start"* "end"*',
    )
  })

  it("returns null for empty or whitespace-only queries", () => {
    expect(buildFtsQuery("")).toBeNull()
    expect(buildFtsQuery("   ")).toBeNull()
  })

  it("returns null when search consists only of special characters", () => {
    expect(buildFtsQuery('*** """ ((( - :::')).toBeNull()
  })
})
