import { useQueryClient } from "@tanstack/react-query"
import { useCallback, useEffect, useRef } from "react"
import { AppState, type AppStateStatus } from "react-native"
import { getAutoSyncEnabled, getSyncMeta } from "@/db/queries"
import { statsKeys } from "@/hooks/useDatabaseStats"
import { folderKeys } from "@/hooks/useFolders"
import { noteKeys } from "@/hooks/useNotes"
import { syncKeys, useAutoSyncQuery, useSyncState } from "@/hooks/useSyncState"
import { executeSyncAsync } from "./syncEngine"

const AUTO_SYNC_INTERVAL_MS = 60 * 1000 // 60s

let globalSyncTrigger: (() => Promise<void>) | null = null

export function triggerBackgroundSyncIfConnected(): void {
  if (globalSyncTrigger) {
    void globalSyncTrigger()
    return
  }
  const isAutoSync = getAutoSyncEnabled()
  const isConnected = Boolean(getSyncMeta("server_url") && getSyncMeta("auth_token"))
  if (isConnected && isAutoSync) {
    void executeSyncAsync()
  }
}

export function useAutoSyncRunner(): void {
  const queryClient = useQueryClient()
  const { data: syncStatus } = useSyncState()
  const { data: autoSyncEnabled } = useAutoSyncQuery()
  const isSyncingRef = useRef(false)

  const isConnected = syncStatus?.isConnected ?? false
  const isAutoSyncOn = autoSyncEnabled !== false

  const triggerSync = useCallback(async () => {
    if (isSyncingRef.current) return
    if (!isConnected || !isAutoSyncOn) return

    isSyncingRef.current = true
    try {
      const result = await executeSyncAsync()
      if (result.success && result.syncedDownCount > 0) {
        void queryClient.invalidateQueries({ queryKey: noteKeys.all })
        void queryClient.invalidateQueries({ queryKey: folderKeys.all })
        void queryClient.invalidateQueries({ queryKey: statsKeys.all })
      }
      if (result.success) {
        void queryClient.invalidateQueries({ queryKey: syncKeys.all })
      }
    } finally {
      isSyncingRef.current = false
    }
  }, [isConnected, isAutoSyncOn, queryClient])

  useEffect(() => {
    globalSyncTrigger = triggerSync
    return () => {
      if (globalSyncTrigger === triggerSync) {
        globalSyncTrigger = null
      }
    }
  }, [triggerSync])

  // Sync on app foreground / focus
  useEffect(() => {
    if (!isConnected || !isAutoSyncOn) return

    const handleAppStateChange = (nextAppState: AppStateStatus) => {
      if (nextAppState === "active") {
        void triggerSync()
      }
    }

    const subscription = AppState.addEventListener("change", handleAppStateChange)
    void triggerSync()

    return () => {
      subscription.remove()
    }
  }, [isConnected, isAutoSyncOn, triggerSync])

  // Periodic interval sync
  useEffect(() => {
    if (!isConnected || !isAutoSyncOn) return

    const interval = setInterval(() => {
      void triggerSync()
    }, AUTO_SYNC_INTERVAL_MS)

    return () => clearInterval(interval)
  }, [isConnected, isAutoSyncOn, triggerSync])
}
