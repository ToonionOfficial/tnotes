import { describe, expect, it } from "vitest"
import { BENCHMARK_FOLDER_MARKER, BENCHMARK_NOTE_MARKER } from "../src/db/queries/benchmark"
import { computeChecksum } from "../src/utils/crypto"

describe("Benchmark Markers and Payload Structure", () => {
  it("generates identifiable benchmark note and folder markers", () => {
    const batchId = "01M17TESTBATCHID"
    const folderName = `${BENCHMARK_FOLDER_MARKER}${batchId}:1`
    const noteBody = `${BENCHMARK_NOTE_MARKER}${batchId}:1`

    expect(folderName.startsWith(BENCHMARK_FOLDER_MARKER)).toBe(true)
    expect(noteBody.startsWith(BENCHMARK_NOTE_MARKER)).toBe(true)
    expect(folderName.slice(0, BENCHMARK_FOLDER_MARKER.length)).toBe(BENCHMARK_FOLDER_MARKER)
    expect(noteBody.slice(0, BENCHMARK_NOTE_MARKER.length)).toBe(BENCHMARK_NOTE_MARKER)
  })

  it("calculates deterministic SHA-256 checksums for benchmark notes", () => {
    const body1 = `${BENCHMARK_NOTE_MARKER}batchA:1`
    const body2 = `${BENCHMARK_NOTE_MARKER}batchA:1`
    const body3 = `${BENCHMARK_NOTE_MARKER}batchA:2`

    const checksum1 = computeChecksum(body1)
    const checksum2 = computeChecksum(body2)
    const checksum3 = computeChecksum(body3)

    expect(checksum1).toBe(checksum2)
    expect(checksum1).not.toBe(checksum3)
    expect(checksum1).toMatch(/^[a-f0-9]{64}$/)
  })

  it("structures valid folder sync payload for local_changes", () => {
    const folderPayload = {
      id: "folder_01",
      user_id: "usr_123",
      parent_id: null,
      name: `${BENCHMARK_FOLDER_MARKER}batch1:1`,
      icon: "folder",
      sort_order: 0,
      version: 1,
      updated_at: 1700000000000,
      created_at: 1700000000000,
      deleted_at: null,
      device_id: "dev_mobile",
    }

    const json = JSON.stringify(folderPayload)
    const parsed = JSON.parse(json)

    expect(parsed.id).toBe("folder_01")
    expect(parsed.name).toContain(BENCHMARK_FOLDER_MARKER)
    expect(parsed.sort_order).toBe(0)
    expect(parsed.version).toBe(1)
  })
})
