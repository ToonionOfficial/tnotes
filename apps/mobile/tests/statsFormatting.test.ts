import { describe, expect, it } from "vitest"
import { formatBytes } from "../src/db/queries/stats"

describe("Storage Stats Formatting", () => {
  it("formats 0 bytes properly", () => {
    expect(formatBytes(0)).toBe("0 B")
    expect(formatBytes(-100)).toBe("0 B")
  })

  it("formats Bytes correctly", () => {
    expect(formatBytes(512)).toBe("512 B")
    expect(formatBytes(1)).toBe("1 B")
    expect(formatBytes(1023)).toBe("1023 B")
  })

  it("formats Kilobytes correctly", () => {
    expect(formatBytes(1024)).toBe("1 KB")
    expect(formatBytes(1536)).toBe("1.5 KB")
    expect(formatBytes(84 * 1024)).toBe("84 KB")
  })

  it("formats Megabytes correctly", () => {
    expect(formatBytes(1024 * 1024)).toBe("1 MB")
    expect(formatBytes(1024 * 1024 * 4.5)).toBe("4.5 MB")
  })

  it("formats Gigabytes correctly", () => {
    expect(formatBytes(1024 * 1024 * 1024 * 2.25)).toBe("2.3 GB")
  })
})
