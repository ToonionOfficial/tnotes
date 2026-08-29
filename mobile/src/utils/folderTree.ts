import type { Folder } from "@/db/schema"

export interface TreeFolderItem {
  folder: Folder
  depth: number
}

/**
 * Builds a depth-first pre-order flattened folder tree with depth levels for hierarchy views.
 */
export function buildFolderTree(folders: Folder[]): TreeFolderItem[] {
  const childrenMap = new Map<string | null, Folder[]>()

  for (const f of folders) {
    const parentKey = f.parentId ?? null
    const list = childrenMap.get(parentKey) || []
    list.push(f)
    childrenMap.set(parentKey, list)
  }

  const result: TreeFolderItem[] = []
  const visited = new Set<string>()

  function traverse(parentId: string | null, depth: number) {
    const children = childrenMap.get(parentId) || []
    for (const child of children) {
      if (visited.has(child.id)) continue
      visited.add(child.id)
      result.push({ folder: child, depth })
      traverse(child.id, depth + 1)
    }
  }

  traverse(null, 0)

  // Append any disconnected / orphaned folders at depth 0
  for (const f of folders) {
    if (!visited.has(f.id)) {
      result.push({ folder: f, depth: 0 })
    }
  }

  return result
}
