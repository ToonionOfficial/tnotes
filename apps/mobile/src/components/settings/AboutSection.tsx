import * as Haptics from "expo-haptics"
import * as Linking from "expo-linking"
import { Code2 } from "lucide-react-native"
import { memo } from "react"
import { Text, View } from "react-native"
import { useAppTheme } from "@/hooks/useAppTheme"
import { SettingsRow } from "./SettingsRow"
import { SettingsSection } from "./SettingsSection"

const GITHUB_REPO_URL = "https://github.com/ToonionOfficial/tnotes"

export interface AboutSectionProps {
  onPressGithub?: () => void
}

export const AboutSection = memo(function AboutSection({ onPressGithub }: AboutSectionProps) {
  const { colors } = useAppTheme()

  const handlePressGithub = () => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    if (onPressGithub) {
      onPressGithub()
    } else {
      void Linking.openURL(GITHUB_REPO_URL)
    }
  }

  return (
    <View>
      <SettingsSection title="About">
        <SettingsRow
          icon={<Code2 size={20} color={colors.foreground} />}
          title="Source Code"
          subtitle="github.com/ToonionOfficial/tnotes"
          onPress={handlePressGithub}
        />
      </SettingsSection>

      <View className="items-center pb-8 pt-4 gap-1">
        <Text className="text-[12px] font-medium text-muted-foreground/60">
          Made with React Native & Rust
        </Text>
        <Text className="text-[11px] text-muted-foreground/40">TNotes v0.1.0</Text>
      </View>
    </View>
  )
})
