import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import {
  getOrCreateDeviceId,
  getSyncCredentials,
  getSyncMeta,
  setSyncCredentials,
  setSyncMeta,
} from "@/db/queries"

export const syncKeys = {
  all: ["sync"] as const,
  credentials: () => [...syncKeys.all, "credentials"] as const,
  deviceId: () => [...syncKeys.all, "deviceId"] as const,
  meta: (key: string) => [...syncKeys.all, "meta", key] as const,
}

export function useSyncCredentials() {
  return useQuery({
    queryKey: syncKeys.credentials(),
    queryFn: async () => getSyncCredentials(),
  })
}

export function useDeviceId() {
  return useQuery({
    queryKey: syncKeys.deviceId(),
    queryFn: async () => getOrCreateDeviceId(),
  })
}

export function useSyncMeta(key: string) {
  return useQuery({
    queryKey: syncKeys.meta(key),
    queryFn: async () => getSyncMeta(key),
  })
}

export function useSetSyncCredentials() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (creds: {
      serverUrl: string
      token: string
      deviceId: string
      userId: string
    }) => {
      setSyncCredentials(creds)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: syncKeys.all })
    },
  })
}

export function useSetSyncMeta() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({ key, value }: { key: string; value: string }) => {
      setSyncMeta(key, value)
    },
    onSuccess: (_, { key }) => {
      queryClient.invalidateQueries({ queryKey: syncKeys.meta(key) })
    },
  })
}
