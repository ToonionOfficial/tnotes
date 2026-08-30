import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useEffect, useState } from "react"
import {
  getAutoSyncEnabled,
  getSyncStatusAsync,
  pairWithServerAsync,
  type QrPairPayload,
  setAutoSyncEnabled,
  unpairServerAsync,
} from "@/db/queries"
import { statsKeys } from "@/hooks/useDatabaseStats"
import { folderKeys } from "@/hooks/useFolders"
import { noteKeys } from "@/hooks/useNotes"
import { executeSyncAsync, getGlobalIsSyncing, subscribeIsSyncing } from "@/services/sync"

export const syncKeys = {
  all: ["syncStatus"] as const,
  autoSync: ["autoSyncSetting"] as const,
}

export function useIsSyncing(): boolean {
  const [isSyncing, setIsSyncing] = useState(() => getGlobalIsSyncing())

  useEffect(() => {
    return subscribeIsSyncing(setIsSyncing)
  }, [])

  return isSyncing
}

export function useSyncState() {
  return useQuery({
    queryKey: syncKeys.all,
    queryFn: getSyncStatusAsync,
  })
}

export function useAutoSyncQuery() {
  return useQuery({
    queryKey: syncKeys.autoSync,
    queryFn: () => getAutoSyncEnabled(),
  })
}

export function useSetAutoSyncMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (enabled: boolean) => {
      setAutoSyncEnabled(enabled)
      return enabled
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: syncKeys.autoSync })
      void queryClient.invalidateQueries({ queryKey: syncKeys.all })
    },
  })
}

export function usePairServerMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (payload: QrPairPayload) => {
      const status = await pairWithServerAsync(payload)
      try {
        await executeSyncAsync()
      } catch {}
      return status
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: syncKeys.all })
      void queryClient.invalidateQueries({ queryKey: syncKeys.autoSync })
      void queryClient.invalidateQueries({ queryKey: noteKeys.all })
      void queryClient.invalidateQueries({ queryKey: folderKeys.all })
      void queryClient.invalidateQueries({ queryKey: statsKeys.all })
    },
  })
}

export function useUnpairServerMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => unpairServerAsync(),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: syncKeys.all })
      void queryClient.invalidateQueries({ queryKey: syncKeys.autoSync })
      void queryClient.invalidateQueries({ queryKey: noteKeys.all })
      void queryClient.invalidateQueries({ queryKey: folderKeys.all })
      void queryClient.invalidateQueries({ queryKey: statsKeys.all })
    },
  })
}

export function useSyncNowMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => executeSyncAsync(),
    onSuccess: (result) => {
      if (result.success) {
        void queryClient.invalidateQueries({ queryKey: syncKeys.all })
        void queryClient.invalidateQueries({ queryKey: noteKeys.all })
        void queryClient.invalidateQueries({ queryKey: folderKeys.all })
        void queryClient.invalidateQueries({ queryKey: statsKeys.all })
      }
    },
  })
}
