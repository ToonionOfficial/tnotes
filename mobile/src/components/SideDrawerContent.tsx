import * as Haptics from "expo-haptics"
import { ChevronRight, Settings, Smartphone, Star, Trash2, User } from "lucide-react-native"
import { memo } from "react"
import { Pressable, Text, View } from "react-native"
import { useSafeAreaInsets } from "react-native-safe-area-context"

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
      className="flex-1 bg-[#141318] px-5"
      style={{
        paddingTop: insets.top + 16,
        paddingBottom: insets.bottom + 16,
      }}
    >
      {/* App Brand Header */}
      <View className="mb-6 flex-row items-center gap-3 px-1">
        <View className="h-9 w-9 items-center justify-center rounded-xl bg-[#CABEFF]">
          <Text className="text-[18px] font-bold text-[#141318]">T</Text>
        </View>
        <Text className="text-[20px] font-bold tracking-tight text-white">TNotes</Text>
      </View>

      {/* Main System Items */}
      <View className="mb-6">
        <Text className="mb-2 px-1 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/70">
          Quick Access
        </Text>
        <View className="overflow-hidden rounded-2xl bg-white/[0.06]">
          {/* Favorites */}
          <Pressable
            onPress={() => {
              void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
              onPressFavorites?.()
            }}
            className="flex-row items-center justify-between p-3.5 active:bg-white/10"
          >
            <View className="flex-row items-center gap-3">
              <View className="h-8 w-8 items-center justify-center rounded-lg bg-[#FFC107]/15">
                <Star size={18} color="#FFC107" />
              </View>
              <Text className="text-[15px] font-medium text-white">Favorites</Text>
            </View>
            <ChevronRight size={16} color="#6E6D77" />
          </Pressable>

          <View className="ml-14 h-px bg-white/5" />

          {/* Trash */}
          <Pressable
            onPress={() => {
              void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
              onPressTrash()
            }}
            className="flex-row items-center justify-between p-3.5 active:bg-white/10"
          >
            <View className="flex-row items-center gap-3">
              <View className="h-8 w-8 items-center justify-center rounded-lg bg-[#FF6B6B]/15">
                <Trash2 size={18} color="#FF6B6B" />
              </View>
              <Text className="text-[15px] font-medium text-[#FF6B6B]">Trash</Text>
            </View>
            <Text className="text-[13px] font-medium text-[#FF6B6B]/70">{trashCount}</Text>
          </Pressable>
        </View>
      </View>

      {/* Bottom Area: Profile & Settings */}
      <View className="mt-auto gap-3">
        {/* Profile Card */}
        <Pressable
          onPress={() => {
            void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
            onPressProfile?.()
          }}
          className="flex-row items-center gap-3.5 rounded-2xl bg-white/[0.06] p-3.5 active:bg-white/10"
        >
          <View className="h-11 w-11 items-center justify-center rounded-xl bg-[#CABEFF]/15">
            <User size={22} color="#CABEFF" />
          </View>
          <View className="flex-1">
            <Text className="text-[15px] font-semibold text-white">Local Account</Text>
            <View className="flex-row items-center gap-1.5 pt-0.5">
              <Smartphone size={12} color="#8E8D94" />
              <Text className="text-[12px] text-muted-foreground">On-device storage</Text>
            </View>
          </View>
          <ChevronRight size={16} color="#6E6D77" />
        </Pressable>

        {/* Settings Button */}
        <Pressable
          onPress={() => {
            void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
            onPressSettings?.()
          }}
          className="flex-row items-center justify-between rounded-2xl bg-white/[0.06] p-3.5 active:bg-white/10"
        >
          <View className="flex-row items-center gap-3">
            <View className="h-8 w-8 items-center justify-center rounded-lg bg-white/10">
              <Settings size={18} color="#e6e1e9" />
            </View>
            <Text className="text-[15px] font-medium text-white">Settings</Text>
          </View>
          <ChevronRight size={16} color="#6E6D77" />
        </Pressable>
      </View>
    </View>
  )
})
