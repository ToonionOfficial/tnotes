import { beforeEach, describe, expect, it, vi } from "vitest"
import {
  BENCHMARK_FOLDER_MARKER,
  BENCHMARK_NOTE_MARKER,
  isBenchmarkSyncPayload,
} from "../src/db/queries/benchmark"

vi.mock("../src/db/index", async () => {
  const fake = await import("./fakes/inMemoryDb")
  return { db: fake.fakeDb, expo: {} }
})

import { getPendingLocalChanges, recordLocalChange, setSyncMeta } from "../src/db/queries/sync"
import { executeSyncAsync } from "../src/services/sync/syncEngine"
import type { SyncEnvelope } from "../src/services/sync/types"
import { resetFakeDb } from "./fakes/inMemoryDb"

describe("isBenchmarkSyncPayload", () => {
  it("flags benchmark note bodies", () => {
    expect(isBenchmarkSyncPayload("note", { body: `${BENCHMARK_NOTE_MARKER}batch1:1` })).toBe(true)
  })

  it("flags benchmark folder names", () => {
    expect(isBenchmarkSyncPayload("folder", { name: `${BENCHMARK_FOLDER_MARKER}batch1:1` })).toBe(
      true,
    )
  })

  it("passes real notes, real folders, and tombstone '{}' payloads through", () => {
    expect(isBenchmarkSyncPayload("note", { body: "weekly shopping list" })).toBe(false)
    expect(isBenchmarkSyncPayload("folder", { name: "Projects" })).toBe(false)
    expect(isBenchmarkSyncPayload("note", {})).toBe(false)
    expect(isBenchmarkSyncPayload("folder", {})).toBe(false)
    expect(isBenchmarkSyncPayload("note", { body: 42 })).toBe(false)
  })
})

describe("executeSyncAsync benchmark filter", () => {
  beforeEach(() => {
    resetFakeDb()
    vi.unstubAllGlobals()
    setSyncMeta("server_url", "http://x:8787")
    setSyncMeta("auth_token", "tok_test")
    setSyncMeta("device_id", "dev_test")
    setSyncMeta("user_id", "uid_test")
  })

  function seedQueue(): void {
    recordLocalChange("note", "real-note-1", 1, 1000, false, {
      id: "real-note-1",
      user_id: "uid_test",
      title: "Real note",
      body: "hello",
    })
    recordLocalChange("note", "bench-note-1", 1, 1001, false, {
      id: "bench-note-1",
      user_id: "uid_test",
      title: "Benchmark note 1",
      body: `${BENCHMARK_NOTE_MARKER}batch1:1`,
    })
    recordLocalChange("folder", "bench-folder-1", 1, 1002, false, {
      id: "bench-folder-1",
      user_id: "uid_test",
      name: `${BENCHMARK_FOLDER_MARKER}batch1:1`,
    })
    // Benchmark delete tombstones carry '{}' and must still upload so server
    // copies synced before the local-only rule get removed.
    recordLocalChange("note", "bench-note-2", 2, 1003, true, {})
  }

  it("uploads real changes + tombstones but never benchmark creates, and drains the queue", async () => {
    seedQueue()
    expect(getPendingLocalChanges()).toHaveLength(4)

    const sentEnvelopes: SyncEnvelope[] = []
    vi.stubGlobal(
      "fetch",
      vi.fn(async (_url: unknown, init?: { body?: string }) => {
        sentEnvelopes.push(JSON.parse(String(init?.body)) as SyncEnvelope)
        return {
          ok: true,
          json: async () => ({ changes: [], server_time: 999 }),
          text: async () => "",
        }
      }),
    )

    const result = await executeSyncAsync()

    expect(result.success).toBe(true)
    expect(result.syncedUpCount).toBe(2)
    expect(sentEnvelopes).toHaveLength(1)
    expect(sentEnvelopes[0]?.changes.map((change) => change.entity_id).sort()).toEqual([
      "bench-note-2",
      "real-note-1",
    ])
    // Skipped benchmark rows are dropped from the queue, not retried forever.
    expect(getPendingLocalChanges()).toHaveLength(0)
  })
})
