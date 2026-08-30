import { describe, expect, it } from "vitest"

describe("Auto-Sync Setting Logic", () => {
  const isAutoSyncEnabled = (rawVal: string | null): boolean => rawVal !== "false"

  it("defaults auto-sync to true when unset", () => {
    expect(isAutoSyncEnabled(null)).toBe(true)
  })

  it("evaluates auto-sync as true when set to true string", () => {
    expect(isAutoSyncEnabled("true")).toBe(true)
  })

  it("evaluates auto-sync as false when explicitly set to false string", () => {
    expect(isAutoSyncEnabled("false")).toBe(false)
  })

  it("determines sync eligibility based on connection and autoSync state", () => {
    const shouldSync = (connected: boolean, autoSync: boolean) => connected && autoSync

    expect(shouldSync(true, true)).toBe(true)
    expect(shouldSync(true, false)).toBe(false)
    expect(shouldSync(false, true)).toBe(false)
    expect(shouldSync(false, false)).toBe(false)
  })
})
