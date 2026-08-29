import * as Haptics from "expo-haptics"
import { FileText, Settings, Smartphone, Trash2, User } from "lucide-react-native"
import { memo } from "react"
import { Pressable, Text, View } from "react-native"
import { useSafeAreaInsets } from "react-native-safe-area-context"

interface SideDrawerContentProps {
  noteCount: number
  trashCount: number
  onPressAllNotes: () => void
  onPressTrash: () => void
  onPressSettings?: () => void
}

export const SideDrawerContent = memo(function SideDrawerContent({
  noteCount,
  trashCount,
  onPressAllNotes,
  onPressTrash,
  onPressSettings,
}: SideDrawerContentProps) {
  const insets = useSafeAreaInsets()

  return (
    <View
      className="flex-1 bg-[#141318] px-5"
      style={{
        paddingTop: insets.top + 16,
        paddingBottom: insets.bottom + 20,
      }}
    >
      {/* Profile & Device Header */}
      <View className="mb-6 flex-row items-center gap-3.5 rounded-2xl bg-white/[0.06] p-3.5">
        <View className="h-11 w-11 items-center justify-center rounded-xl bg-[#CABEFF]/15">
          <User size={22} color="#CABEFF" />
        </View>
        <View className="flex-1">
          <Text className="text-[16px] font-semibold text-white">Local Account</Text>
          <View className="flex-row items-center gap-1.5 pt-0.5">
            <Smartphone size={12} color="#8E8D94" />
            <Text className="text-[12px] text-muted-foreground">On-device storage</Text>
          </View>
        </View>
      </View>

      {/* Main Navigation Section */}
      <View className="mb-6">
        <Text className="mb-2 px-1 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/70">
          Core Folders
        </Text>
        <View className="overflow-hidden rounded-2xl bg-white/[0.06]">
          {/* All Notes */}
          <Pressable
            onPress={() => {
              void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
              onPressAllNotes()
            }}
            className="flex-row items-center justify-between p-3.5 active:bg-white/10"
          >
            <View className="flex-row items-center gap-3">
              <View className="h-8 w-8 items-center justify-center rounded-lg bg-[#CABEFF]/15">
                <FileText size={18} color="#CABEFF" />
              </View>
              <Text className="text-[15px] font-medium text-white">All Notes</Text>
            </View>
            <Text className="text-[13px] font-medium text-muted-foreground">{noteCount}</Text>
          </Pressable>

          <View className="h-px bg-white/5 ml-14" />

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

      {/* Preferences / System Section */}
      {onPressSettings && (
        <View className="mb-6">
          <Text className="mb-2 px-1 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/70">
            Preferences
          </Text>
          <View className="overflow-hidden rounded-2xl bg-white/[0.06]">
            <Pressable
              onPress={() => {
                void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
                onPressSettings()
              }}
              className="flex-row items-center justify-between p-3.5 active:bg-white/10"
            >
              <View className="flex-row items-center gap-3">
                <View className="h-8 w-8 items-center justify-center rounded-lg bg-white/10">
                  <Settings size={18} color="#e6e1e9" />
                </View>
                <Text className="text-[15px] font-medium text-white">Settings</Text>
              </View>
            </Pressable>
          </View>
        </View>
      )}

      {/* Footer Info */}
      <View className="mt-auto items-center py-2">
        <Text className="text-[12px] text-muted-foreground/50">TNotes v0.1.0</Text>
      </View>
    </View>
  )
})
