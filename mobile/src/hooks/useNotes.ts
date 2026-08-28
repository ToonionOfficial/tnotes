import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import {
  createNote,
  deleteNotePermanently,
  getFolderNoteCounts,
  getNoteById,
  getNotes,
  type NoteFilters,
  restoreNote,
  searchNotes,
  togglePinNote,
  trashNote,
  updateNote,
} from "@/db/queries"

export const noteKeys = {
  all: ["notes"] as const,
  lists: () => [...noteKeys.all, "list"] as const,
  list: (filters?: NoteFilters) => [...noteKeys.lists(), filters] as const,
  details: () => [...noteKeys.all, "detail"] as const,
  detail: (id: string) => [...noteKeys.details(), id] as const,
  counts: () => [...noteKeys.all, "counts"] as const,
  search: (query: string, folderId?: string | null) =>
    [...noteKeys.all, "search", query, folderId] as const,
}

export function useNotes(filters?: NoteFilters) {
  return useQuery({
    queryKey: noteKeys.list(filters),
    queryFn: async () => getNotes(filters),
  })
}

export function useFolderNoteCounts() {
  return useQuery({
    queryKey: noteKeys.counts(),
    queryFn: async () => getFolderNoteCounts(),
  })
}

export function useNote(id: string | undefined | null) {
  return useQuery({
    queryKey: noteKeys.detail(id ?? ""),
    queryFn: async () => (id ? getNoteById(id) : null),
    enabled: Boolean(id && id !== "new"),
  })
}

export function useSearchNotes(
  query: string,
  filters?: { folderId?: string | null; trashed?: boolean; limit?: number },
) {
  return useQuery({
    queryKey: noteKeys.search(query, filters?.folderId),
    queryFn: async () => searchNotes(query, filters),
    enabled: query.trim().length > 0,
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
      queryClient.setQueryData(noteKeys.detail(newNote.id), newNote)
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
      queryClient.setQueryData(noteKeys.detail(updatedNote.id), updatedNote)
    },
  })
}

export function useTogglePinNote() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (id: string) => togglePinNote(id),
    onSuccess: (updatedNote) => {
      queryClient.invalidateQueries({ queryKey: noteKeys.lists() })
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
    },
  })
}
