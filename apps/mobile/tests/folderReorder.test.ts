import { describe, expect, it } from "vitest"
import { moveIdInList } from "../src/db/queries/folders"

describe("moveIdInList", () => {
  const ids = ["a", "b", "c", "d", "e"]

  it("moves an item down the list", () => {
    expect(moveIdInList(ids, "b", 3)).toEqual(["a", "c", "d", "b", "e"])
  })

  it("moves an item up the list", () => {
    expect(moveIdInList(ids, "d", 1)).toEqual(["a", "d", "b", "c", "e"])
  })

  it("moves the first item to the last position", () => {
    expect(moveIdInList(ids, "a", 4)).toEqual(["b", "c", "d", "e", "a"])
  })

  it("moves the last item to the first position", () => {
    expect(moveIdInList(ids, "e", 0)).toEqual(["e", "a", "b", "c", "d"])
  })

  it("clamps out-of-range targets", () => {
    expect(moveIdInList(ids, "b", 99)).toEqual(["a", "c", "d", "e", "b"])
    expect(moveIdInList(ids, "d", -5)).toEqual(["d", "a", "b", "c", "e"])
  })

  it("returns the input unchanged for no-ops", () => {
    expect(moveIdInList(ids, "c", 2)).toBe(ids)
    expect(moveIdInList(ids, "missing", 0)).toBe(ids)
    expect(moveIdInList([], "a", 0)).toEqual([])
  })

  it("does not mutate the input array", () => {
    const input = [...ids]
    moveIdInList(input, "a", 4)
    expect(input).toEqual(ids)
  })

  it("models the 100s-of-folders drag: single move only rewrites the range", () => {
    const many = Array.from({ length: 300 }, (_, i) => `f-${i}`)
    const next = moveIdInList(many, "f-0", 299)
    expect(next[299]).toBe("f-0")
    expect(next[0]).toBe("f-1")
    // Outside the moved range nothing changes position identity.
    expect(next.length).toBe(300)
    expect(new Set(next).size).toBe(300)
  })
})
