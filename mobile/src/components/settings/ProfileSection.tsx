import * as Haptics from "expo-haptics"
import { ChevronRight, Smartphone, User } from "lucide-react-native"
import { memo } from "react"
import { Pressable, Text, View } from "react-native"
import { SettingsSection } from "./SettingsSection"

export interface ProfileSectionProps {
  onPressProfile?: () => void
}

export const ProfileSection = memo(function ProfileSection({
  onPressProfile,
}: ProfileSectionProps) {
  const handlePress = () => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    onPressProfile?.()
  }

  return (
    <SettingsSection title="Account">
      <Pressable
        onPress={handlePress}
        className="flex-row items-center justify-between p-4 active:bg-white/10"
      >
        <View className="flex-row items-center gap-3.5">
          <View className="size-11 items-center justify-center rounded-2xl bg-[#CABEFF]/15">
            <User size={22} color="#CABEFF" />
          </View>
          <View>
            <Text className="text-[16px] font-semibold text-white">Local Account</Text>
            <View className="flex-row items-center gap-1.5 pt-0.5">
              <Smartphone size={12} color="#8E8D94" />
              <Text className="text-[12px] text-muted-foreground">On-device</Text>
            </View>
          </View>
        </View>
        <ChevronRight size={16} color="#6E6D77" />
      </Pressable>
    </SettingsSection>
  )
})
