import { describe, expect, it } from "vitest"
import { normalizePayloadForSync } from "../src/services/sync/syncEngine"

describe("Sync Payload Normalization", () => {
  it("converts camelCase Note fields to snake_case", () => {
    const rawNote = {
      id: "01HXYZ",
      userId: "user_123",
      folderId: "folder_456",
      title: "Meeting Notes",
      body: "Discuss Q3 goals",
      pinned: true,
      trashed: false,
      version: 2,
      createdAt: 1700000000000,
      updatedAt: 1700000050000,
      deletedAt: null,
      deviceId: "dev_iphone",
      checksum: "abc123hash",
    }

    const normalized = normalizePayloadForSync("note", rawNote)

    expect(normalized).toEqual({
      id: "01HXYZ",
      user_id: "user_123",
      folder_id: "folder_456",
      title: "Meeting Notes",
      body: "Discuss Q3 goals",
      pinned: true,
      trashed: false,
      version: 2,
      created_at: 1700000000000,
      updated_at: 1700000050000,
      deleted_at: null,
      device_id: "dev_iphone",
      checksum: "abc123hash",
    })
  })

  it("handles Note with null folder and soft delete", () => {
    const rawNote = {
      id: "01HXYZ2",
      userId: "user_123",
      folderId: null,
      title: "Deleted Note",
      body: "",
      pinned: false,
      trashed: true,
      version: 3,
      createdAt: 1700000000000,
      updatedAt: 1700000090000,
      deletedAt: 1700000090000,
      deviceId: "dev_iphone",
      checksum: "e3b0c442",
    }

    const normalized = normalizePayloadForSync("note", rawNote)

    expect(normalized.folder_id).toBeNull()
    expect(normalized.trashed).toBe(true)
    expect(normalized.deleted_at).toBe(1700000090000)
    expect(normalized.user_id).toBe("user_123")
  })

  it("converts camelCase Folder fields to snake_case", () => {
    const rawFolder = {
      id: "folder_01",
      userId: "user_123",
      parentId: "folder_root",
      name: "Work",
      icon: "briefcase",
      sortOrder: 5,
      version: 1,
      createdAt: 1700000000000,
      updatedAt: 1700000010000,
      deletedAt: null,
      deviceId: "dev_iphone",
    }

    const normalized = normalizePayloadForSync("folder", rawFolder)

    expect(normalized).toEqual({
      id: "folder_01",
      user_id: "user_123",
      parent_id: "folder_root",
      name: "Work",
      icon: "briefcase",
      sort_order: 5,
      version: 1,
      created_at: 1700000000000,
      updated_at: 1700000010000,
      deleted_at: null,
      device_id: "dev_iphone",
    })
  })
})
