import { describe, expect, it } from "vitest"
import { applyFrequentOrder, orderFrequentFolders } from "../src/db/queries/folders"
import type { Folder } from "../src/db/schema"

function makeFolder(id: string): Folder {
  return {
    id,
    userId: "u1",
    parentId: null,
    name: id,
    icon: "folder",
    sortOrder: 0,
    version: 1,
    updatedAt: 1,
    createdAt: 1,
    deletedAt: null,
    deviceId: "d1",
  }
}

describe("applyFrequentOrder", () => {
  it("keeps stored arrangement for ids that still exist", () => {
    expect(applyFrequentOrder(["a", "b", "c"], ["c", "a", "b"])).toEqual(["c", "a", "b"])
  })

  it("surfaces never-arranged ids first so just-used folders land on top", () => {
    expect(applyFrequentOrder(["a", "b", "c", "d"], ["c", "a"])).toEqual(["b", "d", "c", "a"])
  })

  it("stays stable once every id has an arrangement (no refetch flapping)", () => {
    const arranged = ["c", "a", "b"]
    expect(applyFrequentOrder(["a", "b", "c"], arranged)).toEqual(arranged)
  })

  it("drops stored ids that no longer exist", () => {
    expect(applyFrequentOrder(["a", "b"], ["z", "b", "a", "y"])).toEqual(["b", "a"])
  })

  it("falls back to activity order with no stored arrangement", () => {
    expect(applyFrequentOrder(["a", "b", "c"], [])).toEqual(["a", "b", "c"])
  })

  it("dedupes repeat stored ids", () => {
    expect(applyFrequentOrder(["a", "b"], ["b", "b", "a"])).toEqual(["b", "a"])
  })
})

describe("orderFrequentFolders", () => {
  it("prepends unarranged rows, then follows the stored arrangement", () => {
    const folders = [makeFolder("a"), makeFolder("b"), makeFolder("c")]
    expect(orderFrequentFolders(folders, ["c", "a"]).map((f) => f.id)).toEqual(["b", "c", "a"])
  })

  it("skips stored ids with no matching row", () => {
    const folders = [makeFolder("a")]
    expect(orderFrequentFolders(folders, ["gone", "a"]).map((f) => f.id)).toEqual(["a"])
  })
})
