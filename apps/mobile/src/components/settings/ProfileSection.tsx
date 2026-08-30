import * as Haptics from "expo-haptics"
import { ChevronRight, Smartphone, User } from "lucide-react-native"
import { memo } from "react"
import { Pressable, Text, View } from "react-native"
import { useAppTheme } from "@/hooks/useAppTheme"
import { SettingsSection } from "./SettingsSection"

export interface ProfileSectionProps {
  username?: string
  isConnected?: boolean
  onPressProfile?: () => void
}

export const ProfileSection = memo(function ProfileSection({
  username = "Local Account",
  isConnected = false,
  onPressProfile,
}: ProfileSectionProps) {
  const { colors } = useAppTheme()

  const handlePress = () => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    onPressProfile?.()
  }

  return (
    <SettingsSection title="Account">
      <Pressable
        onPress={handlePress}
        className="flex-row items-center justify-between p-4 active:bg-accent"
      >
        <View className="flex-row items-center gap-3.5">
          <View className="size-11 items-center justify-center rounded-2xl bg-primary/15">
            <User size={22} color={colors.primary} />
          </View>
          <View>
            <Text className="text-[16px] font-semibold text-foreground">{username}</Text>
            <View className="flex-row items-center gap-1.5 pt-0.5">
              <Smartphone size={12} color={colors.mutedForeground} />
              <Text className="text-[12px] text-muted-foreground">
                {isConnected ? "Synced with server" : "On-device"}
              </Text>
            </View>
          </View>
        </View>
        <ChevronRight size={16} color={colors.mutedForeground} />
      </Pressable>
    </SettingsSection>
  )
})
