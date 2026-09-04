import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import {
  batchDeleteFolders,
  createFolder,
  deleteFolder,
  type FolderFilters,
  FREQUENT_FOLDER_LIMIT,
  getFolderByIdAsync,
  getFoldersPageAsync,
  getFrequentFolderOrder,
  getFrequentFoldersAsync,
  moveFolderToIndex,
  moveIdInList,
  orderFrequentFolders,
  restoreFolder,
  setFrequentFolderOrder,
  updateFolder,
} from "@/db/queries"
import type { Folder } from "@/db/schema"
import { triggerBackgroundSyncIfConnected } from "@/services/sync"
import { statsKeys } from "./useDatabaseStats"
import { noteKeys } from "./useNotes"

export const folderKeys = {
  all: ["folders"] as const,
  lists: () => [...folderKeys.all, "list"] as const,
  list: (filters?: FolderFilters) => [...folderKeys.lists(), filters] as const,
  infinite: (filters?: FolderFilters) => [...folderKeys.all, "infinite", filters] as const,
  frequent: () => [...folderKeys.all, "frequent"] as const,
  details: () => [...folderKeys.all, "detail"] as const,
  detail: (id: string) => [...folderKeys.details(), id] as const,
}

const FOLDERS_PAGE_SIZE = 50
const FOLDERS_QUERY_OPTIONS = {
  staleTime: 1000 * 60,
  gcTime: 1000 * 60 * 5,
  refetchOnMount: false,
} as const

export function useInfiniteFolders(filters?: FolderFilters, enabled = true) {
  return useInfiniteQuery({
    queryKey: folderKeys.infinite(filters),
    initialPageParam: 0,
    queryFn: async ({ pageParam }) => {
      const folders = await getFoldersPageAsync({
        ...filters,
        limit: FOLDERS_PAGE_SIZE + 1,
        offset: pageParam,
      })
      return {
        folders: folders.slice(0, FOLDERS_PAGE_SIZE),
        nextOffset: folders.length > FOLDERS_PAGE_SIZE ? pageParam + FOLDERS_PAGE_SIZE : undefined,
      }
    },
    getNextPageParam: (lastPage) => lastPage.nextOffset,
    enabled,
    ...FOLDERS_QUERY_OPTIONS,
  })
}

export function useFolder(id: string | undefined | null, enabled = true) {
  return useQuery({
    queryKey: folderKeys.detail(id ?? ""),
    queryFn: () => (id ? getFolderByIdAsync(id) : null),
    enabled: Boolean(id) && enabled,
    staleTime: 1000 * 60 * 5,
    gcTime: 1000 * 60 * 5,
    refetchOnMount: false,
  })
}

export function useCreateFolder() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (input: {
      name: string
      icon?: string
      parentId?: string | null
      sortOrder?: number
    }) => createFolder(input),
    onSuccess: (newFolder) => {
      queryClient.invalidateQueries({ queryKey: folderKeys.all })
      queryClient.invalidateQueries({ queryKey: statsKeys.all })
      queryClient.setQueryData(folderKeys.detail(newFolder.id), newFolder)
      triggerBackgroundSyncIfConnected()
    },
  })
}

export function useUpdateFolder() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({
      id,
      input,
    }: {
      id: string
      input: {
        name?: string
        icon?: string
        parentId?: string | null
        sortOrder?: number
      }
    }) => updateFolder(id, input),
    onSuccess: (updatedFolder) => {
      queryClient.invalidateQueries({ queryKey: folderKeys.all })
      queryClient.invalidateQueries({ queryKey: statsKeys.all })
      queryClient.setQueryData(folderKeys.detail(updatedFolder.id), updatedFolder)
      triggerBackgroundSyncIfConnected()
    },
  })
}

export function useDeleteFolder() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (id: string) => {
      deleteFolder(id)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: folderKeys.all })
      queryClient.invalidateQueries({ queryKey: noteKeys.all })
      queryClient.invalidateQueries({ queryKey: statsKeys.all })
      triggerBackgroundSyncIfConnected()
    },
  })
}

export function useBatchDeleteFolders() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (ids: string[]) => {
      batchDeleteFolders(ids)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: folderKeys.all })
      queryClient.invalidateQueries({ queryKey: noteKeys.all })
      queryClient.invalidateQueries({ queryKey: statsKeys.all })
      triggerBackgroundSyncIfConnected()
    },
  })
}

export function useRestoreFolder() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (id: string) => {
      restoreFolder(id)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: folderKeys.all })
      queryClient.invalidateQueries({ queryKey: noteKeys.all })
      queryClient.invalidateQueries({ queryKey: statsKeys.all })
      triggerBackgroundSyncIfConnected()
    },
  })
}

export interface FolderMoveInput {
  folderId: string
  fromIndex: number
  toIndex: number
  parentId?: string | null
}

interface InfiniteFoldersData {
  pages: Array<{ folders: Folder[]; nextOffset?: number }>
  pageParams: unknown[]
}

export function useReorderFolders() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (input: FolderMoveInput) => {
      moveFolderToIndex({
        folderId: input.folderId,
        toIndex: input.toIndex,
        parentId: input.parentId,
      })
    },
    onMutate: async (input: FolderMoveInput) => {
      await queryClient.cancelQueries({ queryKey: folderKeys.all })
      // Optimistically patch every cached infinite folder list containing
      // the moved id. Cached pages may hold a subset of the full sibling
      // order (pagination), so the target is clamped to what is loaded —
      // onSettled refetches the authoritative order from SQLite.
      const previous = queryClient.getQueriesData({
        queryKey: folderKeys.all,
        predicate: (query) => query.queryKey[1] === "infinite",
      })
      for (const [key, data] of previous) {
        const pages = (data as InfiniteFoldersData | undefined)?.pages
        if (!pages) continue
        const flat = pages.flatMap((page) => page.folders)
        if (!flat.some((folder) => folder.id === input.folderId)) continue
        const movedIds = moveIdInList(
          flat.map((folder) => folder.id),
          input.folderId,
          input.toIndex,
        )
        const byId = new Map(flat.map((folder) => [folder.id, folder]))
        const reordered: Folder[] = []
        movedIds.forEach((id, index) => {
          const folder = byId.get(id)
          if (folder) reordered.push({ ...folder, sortOrder: index })
        })
        let offset = 0
        const nextPages = pages.map((page) => {
          const slice = reordered.slice(offset, offset + page.folders.length)
          offset += page.folders.length
          return { ...page, folders: slice }
        })
        queryClient.setQueryData(key, { ...(data as object), pages: nextPages })
      }
      return { previous }
    },
    onError: (_err, _input, context) => {
      context?.previous.forEach(([key, data]) => {
        queryClient.setQueryData(key, data)
      })
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: folderKeys.all })
      queryClient.invalidateQueries({ queryKey: statsKeys.all })
      triggerBackgroundSyncIfConnected()
    },
  })
}

/**
 * "Frequently Used" home section: top folders by recent activity with the
 * user's local arrangement applied. Refreshed by note/folder invalidations
 * plus a home-entry refetch.
 *
 * The stored id list means "order last displayed" (not "last dragged"): it
 * is written back whenever a recompute differs, so evicted ids drop out and
 * re-entering hot folders prepend instead of fossilizing. Ids continuously
 * displayed keep their sequence, so edits never reshuffle and drags stick.
 * Compare-and-write keeps idle refetches write-free; the write targets
 * sync_meta (never the query cache), so it cannot loop invalidations.
 */
export function useFrequentFolders(limit: number = FREQUENT_FOLDER_LIMIT, enabled = true) {
  return useQuery({
    queryKey: folderKeys.frequent(),
    queryFn: async () => {
      const folders = await getFrequentFoldersAsync(limit)
      const stored = getFrequentFolderOrder()
      const ordered = orderFrequentFolders(folders, stored)
      const orderedIds = ordered.map((folder) => folder.id)
      if (orderedIds.join() !== stored.join()) {
        setFrequentFolderOrder(orderedIds)
      }
      return ordered
    },
    enabled,
    staleTime: 1000 * 60,
    gcTime: 1000 * 60 * 5,
    refetchOnMount: true,
    refetchOnWindowFocus: false,
  })
}

/**
 * Persists a drag-and-drop move inside the frequent section. Device-local
 * only: rewrites the sync_meta id list — no folder row writes, no
 * local_changes, and deliberately no background sync trigger.
 */
export function useReorderFrequentFolders() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (orderedIds: string[]) => {
      setFrequentFolderOrder(orderedIds)
    },
    onMutate: async (orderedIds: string[]) => {
      await queryClient.cancelQueries({ queryKey: folderKeys.frequent() })
      const previous = queryClient.getQueryData<Folder[]>(folderKeys.frequent())
      if (previous) {
        const byId = new Map(previous.map((folder) => [folder.id, folder]))
        const next: Folder[] = []
        for (const id of orderedIds) {
          const folder = byId.get(id)
          if (folder) next.push(folder)
        }
        for (const folder of previous) {
          if (!orderedIds.includes(folder.id)) next.push(folder)
        }
        queryClient.setQueryData(folderKeys.frequent(), next)
      }
      return { previous }
    },
    onError: (_err, _orderedIds, context) => {
      if (context?.previous) {
        queryClient.setQueryData(folderKeys.frequent(), context.previous)
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: folderKeys.frequent() })
    },
  })
}
