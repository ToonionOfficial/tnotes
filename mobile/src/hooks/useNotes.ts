import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import {
  batchDeleteNotesPermanently,
  batchTrashNotes,
  createBenchmarkNotes,
  createNote,
  deleteBenchmarkNotes,
  deleteNotePermanently,
  getFolderNoteCountsAsync,
  getNoteByIdAsync,
  getNotesPageAsync,
  type NoteFilters,
  restoreNote,
  togglePinNote,
  trashNote,
  updateNote,
} from "@/db/queries"
import { statsKeys } from "./useDatabaseStats"

export const noteKeys = {
  all: ["notes"] as const,
  lists: () => [...noteKeys.all, "list"] as const,
  list: (filters?: NoteFilters) => [...noteKeys.lists(), filters] as const,
  infinite: (filters?: NoteFilters) => [...noteKeys.all, "infinite", filters] as const,
  details: () => [...noteKeys.all, "detail"] as const,
  detail: (id: string) => [...noteKeys.details(), id] as const,
  counts: () => [...noteKeys.all, "counts"] as const,
  search: (query: string, folderId?: string | null) =>
    [...noteKeys.all, "search", query, folderId] as const,
}

const NOTES_PAGE_SIZE = 50
const NOTES_QUERY_OPTIONS = {
  staleTime: 1000 * 60 * 5,
  gcTime: 1000 * 60 * 5,
  refetchOnMount: false,
} as const

export function useNotes(filters?: NoteFilters, enabled = true) {
  return useInfiniteQuery({
    queryKey: noteKeys.infinite(filters),
    initialPageParam: 0,
    queryFn: async ({ pageParam }) => {
      const notes = await getNotesPageAsync({
        ...filters,
        limit: NOTES_PAGE_SIZE + 1,
        offset: pageParam,
      })
      return {
        notes: notes.slice(0, NOTES_PAGE_SIZE),
        nextOffset: notes.length > NOTES_PAGE_SIZE ? pageParam + NOTES_PAGE_SIZE : undefined,
      }
    },
    getNextPageParam: (lastPage) => lastPage.nextOffset,
    enabled,
    ...NOTES_QUERY_OPTIONS,
  })
}

export function useFolderNoteCounts() {
  return useQuery({
    queryKey: noteKeys.counts(),
    queryFn: getFolderNoteCountsAsync,
    staleTime: 1000 * 60 * 2,
    gcTime: 1000 * 60 * 5,
  })
}

export function useNote(id: string | undefined | null, enabled = true) {
  return useQuery({
    queryKey: noteKeys.detail(id ?? ""),
    queryFn: () => (id ? getNoteByIdAsync(id) : null),
    enabled: Boolean(id) && enabled,
    staleTime: 1000 * 60 * 5,
    gcTime: 1000 * 60 * 5,
    refetchOnMount: false,
  })
}

export function useSearchNotes(query: string, folderId?: string | null, pageSize = 20) {
  return useInfiniteQuery({
    queryKey: noteKeys.search(query, folderId),
    initialPageParam: 0,
    queryFn: async ({ pageParam }) => {
      const notes = await getNotesPageAsync({
        search: query,
        folderId,
        limit: pageSize + 1,
        offset: pageParam,
      })
      return {
        notes: notes.slice(0, pageSize),
        nextOffset: notes.length > pageSize ? pageParam + pageSize : undefined,
      }
    },
    getNextPageParam: (lastPage) => lastPage.nextOffset,
    enabled: query.trim().length > 0,
    ...NOTES_QUERY_OPTIONS,
  })
}

export function useCreateNote() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (input: {
      title?: string
      body?: string
      folderId?: string | null
      pinned?: boolean
    }) => createNote(input),
    onSuccess: (newNote) => {
      queryClient.invalidateQueries({ queryKey: noteKeys.all })
      queryClient.invalidateQueries({ queryKey: statsKeys.all })
      queryClient.setQueryData(noteKeys.detail(newNote.id), newNote)
    },
  })
}

export function useCreateBenchmarkNotes() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (count: number) => createBenchmarkNotes(count),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: noteKeys.all })
      queryClient.invalidateQueries({ queryKey: ["folders"] })
      queryClient.invalidateQueries({ queryKey: statsKeys.all })
    },
  })
}

export function useDeleteBenchmarkNotes() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: deleteBenchmarkNotes,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: noteKeys.all })
      queryClient.invalidateQueries({ queryKey: ["folders"] })
      queryClient.invalidateQueries({ queryKey: statsKeys.all })
    },
  })
}

export function useUpdateNote() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({
      id,
      input,
    }: {
      id: string
      input: {
        title?: string
        body?: string
        folderId?: string | null
        pinned?: boolean
      }
    }) => updateNote(id, input),
    onSuccess: (updatedNote) => {
      queryClient.invalidateQueries({ queryKey: noteKeys.all })
      queryClient.invalidateQueries({ queryKey: statsKeys.all })
      queryClient.setQueryData(noteKeys.detail(updatedNote.id), updatedNote)
    },
  })
}

export function useTogglePinNote() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (id: string) => togglePinNote(id),
    onSuccess: (updatedNote) => {
      queryClient.invalidateQueries({ queryKey: noteKeys.all })
      queryClient.setQueryData(noteKeys.detail(updatedNote.id), updatedNote)
    },
  })
}

export function useTrashNote() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (id: string) => {
      trashNote(id)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: noteKeys.all })
      queryClient.invalidateQueries({ queryKey: statsKeys.all })
    },
  })
}

export function useRestoreNote() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (id: string) => {
      restoreNote(id)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: noteKeys.all })
      queryClient.invalidateQueries({ queryKey: statsKeys.all })
    },
  })
}

export function useDeleteNotePermanently() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (id: string) => {
      deleteNotePermanently(id)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: noteKeys.all })
      queryClient.invalidateQueries({ queryKey: statsKeys.all })
    },
  })
}

export function useBatchTrashNotes() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (ids: string[]) => {
      batchTrashNotes(ids)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: noteKeys.all })
      queryClient.invalidateQueries({ queryKey: statsKeys.all })
    },
  })
}

export function useBatchDeleteNotesPermanently() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (ids: string[]) => {
      batchDeleteNotesPermanently(ids)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: noteKeys.all })
      queryClient.invalidateQueries({ queryKey: statsKeys.all })
    },
  })
}
