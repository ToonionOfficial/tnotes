import { describe, expect, it } from "vitest"
import { computeRecursiveCounts } from "../src/db/queries/notes"

describe("computeRecursiveCounts", () => {
  it("calculates direct note counts when no subfolders exist", () => {
    const folders = [
      { id: "f1", parentId: null },
      { id: "f2", parentId: null },
    ]
    const directCounts = { f1: 5, f2: 3 }

    const result = computeRecursiveCounts(folders, directCounts)
    expect(result).toEqual({ f1: 5, f2: 3 })
  })

  it("calculates recursive counts across nested subfolder trees", () => {
    // Tree:
    // Root (f1) [2 direct]
    // └── Work (f2) [3 direct]
    //     └── Projects (f3) [5 direct]
    //         └── Active (f4) [4 direct]
    const folders = [
      { id: "f1", parentId: null },
      { id: "f2", parentId: "f1" },
      { id: "f3", parentId: "f2" },
      { id: "f4", parentId: "f3" },
    ]
    const directCounts = { f1: 2, f2: 3, f3: 5, f4: 4 }

    const result = computeRecursiveCounts(folders, directCounts)
    expect(result.f4).toBe(4)
    expect(result.f3).toBe(5 + 4) // 9
    expect(result.f2).toBe(3 + 5 + 4) // 12
    expect(result.f1).toBe(2 + 3 + 5 + 4) // 14
  })

  it("handles parent folders with 0 direct notes", () => {
    const folders = [
      { id: "parent", parentId: null },
      { id: "child1", parentId: "parent" },
      { id: "child2", parentId: "parent" },
    ]
    const directCounts = { child1: 10, child2: 5 }

    const result = computeRecursiveCounts(folders, directCounts)
    expect(result.child1).toBe(10)
    expect(result.child2).toBe(5)
    expect(result.parent).toBe(15)
  })

  it("safely handles circular references without infinite recursion", () => {
    const folders = [
      { id: "f1", parentId: "f2" },
      { id: "f2", parentId: "f1" },
    ]
    const directCounts = { f1: 2, f2: 3 }

    const result = computeRecursiveCounts(folders, directCounts)
    expect(typeof result.f1).toBe("number")
    expect(typeof result.f2).toBe("number")
  })
})
