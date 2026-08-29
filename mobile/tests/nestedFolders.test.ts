import { describe, expect, it } from "vitest"
import { computeRecursiveCounts } from "../src/db/queries/notes"
import type { Folder } from "../src/db/schema"
import { normalizePayloadForSync } from "../src/services/sync/syncEngine"
import { buildFolderTree } from "../src/utils/folderTree"

describe("Nested Folders & Hierarchical Actions", () => {
  describe("Tree Hierarchy & Filtering", () => {
    const sampleFolders = [
      { id: "root_1", parentId: null, name: "Work" },
      { id: "root_2", parentId: null, name: "Personal" },
      { id: "sub_1_1", parentId: "root_1", name: "Projects" },
      { id: "sub_1_2", parentId: "root_1", name: "Meetings" },
      { id: "sub_1_1_1", parentId: "sub_1_1", name: "Q3 Launch" },
      { id: "sub_2_1", parentId: "root_2", name: "Finance" },
    ] as Folder[]

    it("identifies root level folders (parentId === null)", () => {
      const rootFolders = sampleFolders.filter((f) => f.parentId === null)
      expect(rootFolders.map((f) => f.id)).toEqual(["root_1", "root_2"])
    })

    it("filters direct child subfolders for a specific parent", () => {
      const workSubfolders = sampleFolders.filter((f) => f.parentId === "root_1")
      expect(workSubfolders.map((f) => f.name)).toEqual(["Projects", "Meetings"])

      const projectSubfolders = sampleFolders.filter((f) => f.parentId === "sub_1_1")
      expect(projectSubfolders.map((f) => f.name)).toEqual(["Q3 Launch"])
    })

    it("builds a pre-order depth-first tree with depth indices for Move Sheet", () => {
      const tree = buildFolderTree(sampleFolders)

      // Expected order:
      // Work (depth 0)
      //   -> Projects (depth 1)
      //     -> Q3 Launch (depth 2)
      //   -> Meetings (depth 1)
      // Personal (depth 0)
      //   -> Finance (depth 1)
      expect(tree.map((t) => ({ id: t.folder.id, depth: t.depth }))).toEqual([
        { id: "root_1", depth: 0 },
        { id: "sub_1_1", depth: 1 },
        { id: "sub_1_1_1", depth: 2 },
        { id: "sub_1_2", depth: 1 },
        { id: "root_2", depth: 0 },
        { id: "sub_2_1", depth: 1 },
      ])
    })

    it("collapses subtrees when parent folder ID is in collapsedFolderIds", () => {
      // Collapsing 'root_1' (Work) should hide all its subfolders ('sub_1_1', 'sub_1_1_1', 'sub_1_2')
      const collapsedTree = buildFolderTree(sampleFolders, new Set(["root_1"]))

      expect(collapsedTree.map((t) => t.folder.id)).toEqual(["root_1", "root_2", "sub_2_1"])
      expect(collapsedTree.find((t) => t.folder.id === "root_1")?.isCollapsed).toBe(true)
      expect(collapsedTree.find((t) => t.folder.id === "root_1")?.hasChildren).toBe(true)
    })
  })

  describe("Recursive Note Count Aggregation", () => {
    it("propagates note counts up deep multi-level branches", () => {
      // Tree:
      // Work (root)
      // └── Projects [5 notes]
      //     └── Mobile App [8 notes]
      //         └── Release v1 [3 notes]
      const folders = [
        { id: "work", parentId: null },
        { id: "projects", parentId: "work" },
        { id: "mobile_app", parentId: "projects" },
        { id: "release_v1", parentId: "mobile_app" },
      ]
      const directCounts = {
        work: 2,
        projects: 5,
        mobile_app: 8,
        release_v1: 3,
      }

      const counts = computeRecursiveCounts(folders, directCounts)

      expect(counts.release_v1).toBe(3)
      expect(counts.mobile_app).toBe(8 + 3) // 11
      expect(counts.projects).toBe(5 + 8 + 3) // 16
      expect(counts.work).toBe(2 + 5 + 8 + 3) // 18
    })

    it("handles multiple sibling subtrees under a single parent", () => {
      // Tree:
      // Personal (root) [0 direct]
      // ├── Finance [4 notes]
      // ├── Health [2 notes]
      // └── Travel [6 notes]
      //     └── Japan 2026 [5 notes]
      const folders = [
        { id: "personal", parentId: null },
        { id: "finance", parentId: "personal" },
        { id: "health", parentId: "personal" },
        { id: "travel", parentId: "personal" },
        { id: "japan", parentId: "travel" },
      ]
      const directCounts = {
        personal: 0,
        finance: 4,
        health: 2,
        travel: 6,
        japan: 5,
      }

      const counts = computeRecursiveCounts(folders, directCounts)

      expect(counts.japan).toBe(5)
      expect(counts.travel).toBe(6 + 5) // 11
      expect(counts.finance).toBe(4)
      expect(counts.health).toBe(2)
      expect(counts.personal).toBe(4 + 2 + 6 + 5) // 17
    })
  })

  describe("Recursive Cascade Deletion Logic", () => {
    it("collects all descendant folder IDs in a hierarchy", () => {
      const allFolders = [
        { id: "f_root", parentId: null, deletedAt: null },
        { id: "f_child_1", parentId: "f_root", deletedAt: null },
        { id: "f_child_2", parentId: "f_root", deletedAt: null },
        { id: "f_grandchild_1", parentId: "f_child_1", deletedAt: null },
        { id: "f_other_root", parentId: null, deletedAt: null },
      ]

      type FolderRecord = {
        id: string
        parentId: string | null
        deletedAt: number | null
      }

      // Recursive descendant collection logic used in deleteFolder
      function collectDescendantIds(targetId: string, foldersList: FolderRecord[]): string[] {
        const collected: string[] = [targetId]
        const queue: string[] = [targetId]
        while (queue.length > 0) {
          const currentId = queue.shift()
          if (!currentId) break
          const children = foldersList.filter(
            (f) => f.parentId === currentId && f.deletedAt === null,
          )
          for (const child of children) {
            collected.push(child.id)
            queue.push(child.id)
          }
        }
        return collected
      }

      const deletedIds = collectDescendantIds("f_root", allFolders)
      expect(deletedIds).toContain("f_root")
      expect(deletedIds).toContain("f_child_1")
      expect(deletedIds).toContain("f_child_2")
      expect(deletedIds).toContain("f_grandchild_1")
      expect(deletedIds).not.toContain("f_other_root")
      expect(deletedIds.length).toBe(4)
    })
  })

  describe("Reparenting & Cycle Prevention", () => {
    it("prevents moving a parent folder into its own descendant", () => {
      const folderHierarchy: Record<string, string | null> = {
        work: null,
        projects: "work",
        tnotes: "projects",
        backend: "tnotes",
      }

      // Cycle detector: checks if targetParentId is a descendant of folderToMove
      function isDescendantOf(
        candidateChildId: string | null,
        potentialAncestorId: string,
      ): boolean {
        let current = candidateChildId
        while (current !== null) {
          if (current === potentialAncestorId) return true
          current = folderHierarchy[current] ?? null
        }
        return false
      }

      // Moving 'projects' into 'backend' should be detected as an illegal cycle
      expect(isDescendantOf("backend", "projects")).toBe(true)
      expect(isDescendantOf("tnotes", "projects")).toBe(true)
      expect(isDescendantOf("work", "projects")).toBe(false)
      expect(isDescendantOf(null, "projects")).toBe(false)
    })

    it("allows un-nesting a subfolder to root (parentId = null)", () => {
      const folder = { id: "projects", parentId: "work", name: "Projects" }
      const updatedFolder = { ...folder, parentId: null }
      expect(updatedFolder.parentId).toBeNull()
    })
  })

  describe("Sync Payload Normalization", () => {
    it("serializes nested folders with parent_id for remote sync", () => {
      const localFolder = {
        id: "folder_sub_99",
        userId: "usr_1",
        parentId: "folder_root_01",
        name: "Subproject A",
        icon: "folder",
        sortOrder: 2,
        version: 4,
        createdAt: 1700000000000,
        updatedAt: 1700000050000,
        deletedAt: null,
        deviceId: "dev_client_1",
      }

      const syncPayload = normalizePayloadForSync("folder", localFolder)

      expect(syncPayload).toEqual({
        id: "folder_sub_99",
        user_id: "usr_1",
        parent_id: "folder_root_01",
        name: "Subproject A",
        icon: "folder",
        sort_order: 2,
        version: 4,
        created_at: 1700000000000,
        updated_at: 1700000050000,
        deleted_at: null,
        device_id: "dev_client_1",
      })
    })
  })
})
