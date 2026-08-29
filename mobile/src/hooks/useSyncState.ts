import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import {
  getSyncStatusAsync,
  pairWithServerAsync,
  type QrPairPayload,
  unpairServerAsync,
} from "@/db/queries"

export const syncKeys = {
  all: ["syncStatus"] as const,
}

export function useSyncState() {
  return useQuery({
    queryKey: syncKeys.all,
    queryFn: getSyncStatusAsync,
  })
}

export function usePairServerMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (payload: QrPairPayload) => pairWithServerAsync(payload),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: syncKeys.all })
    },
  })
}

export function useUnpairServerMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => unpairServerAsync(),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: syncKeys.all })
    },
  })
}
