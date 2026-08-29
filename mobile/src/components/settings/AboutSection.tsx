import { Code2 } from "lucide-react-native"
import { memo } from "react"
import { Text, View } from "react-native"
import { SettingsRow } from "./SettingsRow"
import { SettingsSection } from "./SettingsSection"

export interface AboutSectionProps {
  onPressGithub?: () => void
}

export const AboutSection = memo(function AboutSection({ onPressGithub }: AboutSectionProps) {
  return (
    <View>
      <SettingsSection title="About">
        <SettingsRow
          icon={<Code2 size={20} color="#E6E1E9" />}
          title="Source Code"
          onPress={onPressGithub}
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
