import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import {
  batchDeleteFolders,
  createFolder,
  deleteFolder,
  type FolderFilters,
  getFolderByIdAsync,
  getFoldersPageAsync,
  reorderFolders,
  restoreFolder,
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
  details: () => [...folderKeys.all, "detail"] as const,
  detail: (id: string) => [...folderKeys.details(), id] as const,
}

const FOLDERS_PAGE_SIZE = 50
const FOLDERS_QUERY_OPTIONS = {
  staleTime: 1000 * 60,
  gcTime: 1000 * 60 * 5,
  refetchOnMount: false,
} as const

export function useInfiniteFolders(filters?: FolderFilters) {
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

export function useReorderFolders() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (folderIds: string[]) => {
      reorderFolders(folderIds)
    },
    onMutate: async (folderIds: string[]) => {
      await queryClient.cancelQueries({ queryKey: folderKeys.lists() })
      const prevFolders = queryClient.getQueryData<Folder[]>(folderKeys.list())
      if (prevFolders) {
        const folderMap = new Map(prevFolders.map((f) => [f.id, f]))
        const reordered: Folder[] = []
        folderIds.forEach((id, index) => {
          const f = folderMap.get(id)
          if (f) {
            reordered.push({ ...f, sortOrder: index })
          }
        })
        queryClient.setQueryData(folderKeys.list(), reordered)
      }
      return { prevFolders }
    },
    onError: (_err, _folderIds, context) => {
      if (context?.prevFolders) {
        queryClient.setQueryData(folderKeys.list(), context.prevFolders)
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: folderKeys.all })
      queryClient.invalidateQueries({ queryKey: statsKeys.all })
      triggerBackgroundSyncIfConnected()
    },
  })
}
