import { describe, expect, it } from "vitest"
import { displayTitle, firstLineTitle } from "../src/utils/text"

describe("firstLineTitle", () => {
  it("keeps single-line titles trimmed", () => {
    expect(firstLineTitle("Shopping")).toBe("Shopping")
    expect(firstLineTitle("  Shopping  ")).toBe("Shopping")
    expect(firstLineTitle("")).toBe("")
  })

  it("cuts multi-line titles at the first newline", () => {
    expect(firstLineTitle("First line\nSecond line")).toBe("First line")
    expect(firstLineTitle("CRLF title\r\nMore")).toBe("CRLF title")
  })

  it("collapses inner whitespace", () => {
    expect(firstLineTitle("Buy   milk\tand eggs")).toBe("Buy milk and eggs")
  })
})

describe("displayTitle", () => {
  it("falls back for empty titles", () => {
    expect(displayTitle("")).toBe("Untitled Note")
    expect(displayTitle("   ")).toBe("Untitled Note")
  })

  it("shows only the first line", () => {
    expect(displayTitle("First line\nSecond line")).toBe("First line")
  })
})
