import { BottomSheetModalProvider } from "@gorhom/bottom-sheet"
import * as Haptics from "expo-haptics"
import { Stack, useRouter } from "expo-router"
import { X } from "lucide-react-native"
import { useMemo, useState } from "react"
import { Alert, Platform, Pressable, ScrollView, Text, View } from "react-native"
import { type PairPayload, PairServerModal } from "@/components/scanner"
import {
  AboutSection,
  AppearanceSection,
  DataStorageSection,
  ENABLE_BENCHMARK,
  FlagsSection,
  ProfileSection,
  SettingsSearchBar,
  SyncServerSection,
} from "@/components/settings"
import { useAppTheme } from "@/hooks/useAppTheme"
import { useDatabaseStats } from "@/hooks/useDatabaseStats"
import {
  useAutoSyncQuery,
  usePairServerMutation,
  useSetAutoSyncMutation,
  useSyncNowMutation,
  useSyncState,
  useUnpairServerMutation,
} from "@/hooks/useSyncState"

export default function SettingsScreen() {
  const router = useRouter()
  const [searchQuery, setSearchQuery] = useState("")
  const [isPairModalOpen, setIsPairModalOpen] = useState(false)
  const { isDarkMode, toggleTheme, colors } = useAppTheme()

  const { data: syncStatus } = useSyncState()
  const { data: autoSyncEnabled } = useAutoSyncQuery()
  const { data: stats } = useDatabaseStats()
  const pairMutation = usePairServerMutation()
  const unpairMutation = useUnpairServerMutation()
  const syncNowMutation = useSyncNowMutation()
  const setAutoSyncMutation = useSetAutoSyncMutation()

  const normalizedQuery = searchQuery.trim().toLowerCase()

  const matches = useMemo(() => {
    return {
      profile: !normalizedQuery || "account profile user local offline".includes(normalizedQuery),
      sync:
        !normalizedQuery ||
        "sync server connect websocket cloud backend url disconnect".includes(normalizedQuery),
      appearance:
        !normalizedQuery || "theme appearance dark oled font typography".includes(normalizedQuery),
      data:
        !normalizedQuery ||
        "data storage database sqlite export backup markdown json".includes(normalizedQuery),
      about:
        !normalizedQuery ||
        "about version tnotes github repo license source".includes(normalizedQuery),
      flags:
        !normalizedQuery ||
        "flags benchmark dev developer performance test tools".includes(normalizedQuery),
    }
  }, [normalizedQuery])

  const hasAnyMatch = Object.values(matches).some(Boolean)

  const handlePairSuccess = async (payload: PairPayload) => {
    if (!payload.token) return
    setIsPairModalOpen(false)
    try {
      await pairMutation.mutateAsync(payload)
      void Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success)
    } catch (err) {
      const msg =
        err instanceof Error
          ? err.message
          : "Saved credentials, but could not immediately verify the server. Sync will retry when online."
      Alert.alert("Pairing Error", msg)
    }
  }

  const handleSyncNow = async () => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    try {
      const result = await syncNowMutation.mutateAsync()
      if (result.success) {
        void Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success)
      } else {
        Alert.alert("Sync Notice", result.error ?? "Failed to complete sync.")
      }
    } catch {
      Alert.alert("Sync Error", "Could not reach the sync server.")
    }
  }

  const handleDisconnect = () => {
    Alert.alert(
      "Disconnect Server",
      "Are you sure you want to disconnect from this sync server? Your local notes will remain on this device.",
      [
        { text: "Cancel", style: "cancel" },
        {
          text: "Disconnect",
          style: "destructive",
          onPress: () => {
            void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
            void unpairMutation.mutateAsync()
          },
        },
      ],
    )
  }

  const handleToggleAutoSync = (enabled: boolean) => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    void setAutoSyncMutation.mutateAsync(enabled)
  }

  return (
    <BottomSheetModalProvider>
      <View className="flex-1 bg-background">
        <Stack.Screen
          options={{
            title: "Settings",
            headerLargeTitle: true,
            unstable_headerRightItems: () => [
              {
                type: "button",
                label: "Close",
                icon: {
                  name: "xmark",
                  type: "sfSymbol",
                },
                tintColor: colors.foreground,
                onPress: () => {
                  void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
                  router.back()
                },
              },
            ],
            headerRight:
              Platform.OS !== "ios"
                ? () => (
                    <Pressable
                      onPress={() => {
                        void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
                        router.back()
                      }}
                      hitSlop={8}
                      className="p-1 active:opacity-60"
                    >
                      <X size={20} color={colors.foreground} strokeWidth={2} />
                    </Pressable>
                  )
                : undefined,
          }}
        />

        <ScrollView
          className="flex-1"
          contentInsetAdjustmentBehavior="automatic"
          keyboardShouldPersistTaps="handled"
          contentContainerStyle={{
            paddingHorizontal: 20,
            paddingTop: 12,
            paddingBottom: 60,
          }}
        >
          <SettingsSearchBar
            value={searchQuery}
            onChangeText={setSearchQuery}
            placeholder="Search settings"
          />

          {matches.profile && (
            <ProfileSection
              username={syncStatus?.username}
              isConnected={syncStatus?.isConnected}
              onPressProfile={() => router.push("/account")}
            />
          )}

          {matches.sync && (
            <SyncServerSection
              isConnected={syncStatus?.isConnected}
              serverUrl={syncStatus?.serverUrl}
              isSyncing={syncNowMutation.isPending}
              autoSync={autoSyncEnabled !== false}
              onToggleAutoSync={handleToggleAutoSync}
              onPressConnectServer={() => setIsPairModalOpen(true)}
              onPressSyncNow={handleSyncNow}
              onPressDisconnect={handleDisconnect}
            />
          )}

          {matches.appearance && (
            <AppearanceSection isDarkMode={isDarkMode} onToggleTheme={toggleTheme} />
          )}
          {matches.data && <DataStorageSection stats={stats} />}
          {ENABLE_BENCHMARK && matches.flags && <FlagsSection />}
          {matches.about && <AboutSection />}

          {!hasAnyMatch && (
            <View className="items-center justify-center py-16">
              <Text className="text-[17px] font-semibold text-foreground">No Results</Text>
              <Text className="pt-1 text-[14px] text-muted-foreground">
                No settings match &ldquo;{searchQuery}&rdquo;
              </Text>
            </View>
          )}
        </ScrollView>

        <PairServerModal
          visible={isPairModalOpen}
          onClose={() => setIsPairModalOpen(false)}
          onPairSuccess={handlePairSuccess}
        />
      </View>
    </BottomSheetModalProvider>
  )
}
