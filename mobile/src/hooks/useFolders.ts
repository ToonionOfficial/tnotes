import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import {
  batchDeleteFolders,
  createFolder,
  deleteFolder,
  type FolderFilters,
  getFolderById,
  getFolders,
  reorderFolders,
  restoreFolder,
  updateFolder,
} from "@/db/queries"
import type { Folder } from "@/db/schema"
import { noteKeys } from "./useNotes"

export const folderKeys = {
  all: ["folders"] as const,
  lists: () => [...folderKeys.all, "list"] as const,
  list: (filters?: FolderFilters) => [...folderKeys.lists(), filters] as const,
  details: () => [...folderKeys.all, "detail"] as const,
  detail: (id: string) => [...folderKeys.details(), id] as const,
}

export function useFolders(filters?: FolderFilters) {
  return useQuery({
    queryKey: folderKeys.list(filters),
    queryFn: async () => getFolders(filters),
  })
}

export function useFolder(id: string | undefined | null) {
  return useQuery({
    queryKey: folderKeys.detail(id ?? ""),
    queryFn: async () => (id ? getFolderById(id) : null),
    enabled: Boolean(id),
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
      queryClient.invalidateQueries({ queryKey: folderKeys.lists() })
      queryClient.setQueryData(folderKeys.detail(newFolder.id), newFolder)
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
      queryClient.invalidateQueries({ queryKey: folderKeys.lists() })
      queryClient.setQueryData(folderKeys.detail(updatedFolder.id), updatedFolder)
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
      queryClient.invalidateQueries({ queryKey: folderKeys.lists() })
    },
  })
}
