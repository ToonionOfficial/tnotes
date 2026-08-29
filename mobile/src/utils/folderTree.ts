import type { Folder } from "@/db/schema"

export interface TreeFolderItem {
  folder: Folder
  depth: number
  hasChildren: boolean
  isCollapsed: boolean
}

/**
 * Builds a depth-first pre-order flattened folder tree with depth levels and collapsible subtree support.
 */
export function buildFolderTree(
  folders: Folder[],
  collapsedFolderIds?: Set<string>,
): TreeFolderItem[] {
  const childrenMap = new Map<string | null, Folder[]>()
  const folderIdSet = new Set(folders.map((f) => f.id))

  for (const f of folders) {
    const parentKey = f.parentId && folderIdSet.has(f.parentId) ? f.parentId : null
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

      const directChildren = childrenMap.get(child.id) || []
      const hasChildren = directChildren.length > 0
      const isCollapsed = Boolean(collapsedFolderIds?.has(child.id))

      result.push({
        folder: child,
        depth,
        hasChildren,
        isCollapsed,
      })

      // Only traverse children if this folder is not collapsed
      if (!isCollapsed) {
        traverse(child.id, depth + 1)
      }
    }
  }

  traverse(null, 0)
  return result
}
