import { Code2, Heart } from "lucide-react-native"
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
          showDivider
        />
        <SettingsRow
          icon={<Heart size={19} color="#E6E1E9" />}
          title="Made with React Native & Rust"
          showChevron={false}
        />
      </SettingsSection>

      <View className="items-center pb-8 pt-2">
        <Text className="text-[12px] text-muted-foreground/40">TNotes v0.1.0</Text>
      </View>
    </View>
  )
})
