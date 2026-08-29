import * as Haptics from "expo-haptics"
import { Stack, useRouter } from "expo-router"
import { useMemo, useState } from "react"
import { Pressable, ScrollView, Text, View } from "react-native"
import {
  AboutSection,
  AppearanceSection,
  DataStorageSection,
  ProfileSection,
  SettingsSearchBar,
  SyncServerSection,
} from "@/components/settings"
import { useDatabaseStats } from "@/hooks/useDatabaseStats"

export default function SettingsScreen() {
  const router = useRouter()
  const [searchQuery, setSearchQuery] = useState("")
  const { data: stats } = useDatabaseStats()

  const normalizedQuery = searchQuery.trim().toLowerCase()

  const matches = useMemo(() => {
    return {
      profile: !normalizedQuery || "account profile user local offline".includes(normalizedQuery),
      sync:
        !normalizedQuery ||
        "sync server connect websocket cloud backend url".includes(normalizedQuery),
      appearance:
        !normalizedQuery ||
        "theme appearance dark oled accent color font typography".includes(normalizedQuery),
      data:
        !normalizedQuery ||
        "data storage database sqlite export backup markdown json".includes(normalizedQuery),
      about:
        !normalizedQuery ||
        "about version tnotes github repo license source".includes(normalizedQuery),
    }
  }, [normalizedQuery])

  const hasAnyMatch = Object.values(matches).some(Boolean)

  return (
    <View className="flex-1 bg-background">
      <Stack.Screen
        options={{
          title: "Settings",
          headerLargeTitle: true,
          headerRight: () => (
            <Pressable
              onPress={() => {
                void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
                router.back()
              }}
              hitSlop={8}
              className="px-2 py-1 active:opacity-60"
            >
              <Text className="text-[17px] font-semibold text-[#CABEFF]">Done</Text>
            </Pressable>
          ),
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

        {matches.profile && <ProfileSection />}
        {matches.sync && <SyncServerSection isConnected={false} />}
        {matches.appearance && <AppearanceSection />}
        {matches.data && <DataStorageSection stats={stats} />}
        {matches.about && <AboutSection />}

        {!hasAnyMatch && (
          <View className="items-center justify-center py-16">
            <Text className="text-[17px] font-semibold text-white">No Results</Text>
            <Text className="pt-1 text-[14px] text-muted-foreground">
              No settings match &ldquo;{searchQuery}&rdquo;
            </Text>
          </View>
        )}
      </ScrollView>
    </View>
  )
}
