import * as Haptics from "expo-haptics"
import { Image } from "expo-image"
import { ChevronRight, Settings, Smartphone, Star, Trash2, User } from "lucide-react-native"
import { memo } from "react"
import { Pressable, ScrollView, Text, View } from "react-native"
import { useSafeAreaInsets } from "react-native-safe-area-context"

const APP_ICON = require("../../assets/images/icon.png")

interface SideDrawerContentProps {
  trashCount: number
  onPressTrash: () => void
  onPressFavorites?: () => void
  onPressProfile?: () => void
  onPressSettings?: () => void
}

export const SideDrawerContent = memo(function SideDrawerContent({
  trashCount,
  onPressTrash,
  onPressFavorites,
  onPressProfile,
  onPressSettings,
}: SideDrawerContentProps) {
  const insets = useSafeAreaInsets()

  return (
    <View
      className="flex-1 bg-[#141318] px-4"
      style={{
        paddingTop: insets.top + 16,
        paddingBottom: insets.bottom + 16,
      }}
    >
      {/* Pinned Top Brand Header */}
      <View className="mb-4 flex-row items-center gap-3 px-2">
        <Image
          source={APP_ICON}
          style={{ width: 36, height: 36, borderRadius: 10 }}
          contentFit="cover"
        />
        <Text className="text-[20px] font-bold tracking-tight text-white">TNotes</Text>
      </View>

      {/* Scrollable Middle Items */}
      <ScrollView
        className="flex-1"
        showsVerticalScrollIndicator={false}
        contentContainerStyle={{ paddingBottom: 16 }}
      >
        <View className="mb-6">
          <Text className="mb-2 px-2 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/60">
            Quick Access
          </Text>
          <View className="gap-1">
            {/* Favorites */}
            <Pressable
              onPress={() => {
                void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
                onPressFavorites?.()
              }}
              className="flex-row items-center justify-between rounded-3xl px-3.5 py-3 active:bg-white/[0.08]"
            >
              <View className="flex-row items-center gap-3">
                <View className="w-6 items-center justify-center">
                  <Star size={20} color="#FFC107" />
                </View>
                <Text className="text-[15px] font-medium text-white">Favorites</Text>
              </View>
              <ChevronRight size={16} color="#6E6D77" />
            </Pressable>

            {/* Trash */}
            <Pressable
              onPress={() => {
                void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
                onPressTrash()
              }}
              className="flex-row items-center justify-between rounded-3xl px-3.5 py-3 active:bg-white/[0.08]"
            >
              <View className="flex-row items-center gap-3">
                <View className="w-6 items-center justify-center">
                  <Trash2 size={20} color="#FF6B6B" />
                </View>
                <Text className="text-[15px] font-medium text-[#FF6B6B]">Trash</Text>
              </View>
              <Text className="text-[13px] font-medium text-[#FF6B6B]/70">{trashCount}</Text>
            </Pressable>
          </View>
        </View>
      </ScrollView>

      {/* Pinned Bottom Area: Unified Account & Settings Button */}
      <View className="pt-2">
        <Pressable
          onPress={() => {
            void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
            if (onPressSettings) {
              onPressSettings()
            } else {
              onPressProfile?.()
            }
          }}
          className="flex-row items-center justify-between rounded-3xl px-3.5 py-2.5 active:bg-white/[0.08]"
        >
          {/* Left: User info */}
          <View className="flex-1 flex-row items-center gap-3">
            <View className="w-6 items-center justify-center">
              <User size={22} color="#CABEFF" />
            </View>
            <View className="flex-1">
              <Text className="text-[15px] font-semibold text-white">Local Account</Text>
              <View className="flex-row items-center gap-1.5 pt-0.5">
                <Smartphone size={12} color="#8E8D94" />
                <Text className="text-[12px] text-muted-foreground">On-device storage</Text>
              </View>
            </View>
          </View>

          {/* Right: Settings Icon */}
          <View className="w-6 items-center justify-center">
            <Settings size={20} color="#E6E1E9" />
          </View>
        </Pressable>
      </View>
    </View>
  )
})
