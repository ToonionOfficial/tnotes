import { Moon, Sparkles, Type } from "lucide-react-native"
import { memo } from "react"
import { View } from "react-native"
import { SettingsRow } from "./SettingsRow"
import { SettingsSection } from "./SettingsSection"

export interface AppearanceSectionProps {
  onPressTheme?: () => void
  onPressAccent?: () => void
  onPressFont?: () => void
}

export const AppearanceSection = memo(function AppearanceSection({
  onPressTheme,
  onPressAccent,
  onPressFont,
}: AppearanceSectionProps) {
  const accentColorBadge = <View className="size-3.5 rounded-full bg-[#CABEFF]" />

  return (
    <SettingsSection title="Appearance">
      <SettingsRow
        icon={<Moon size={20} color="#E6E1E9" />}
        title="Theme"
        value="Dark"
        onPress={onPressTheme}
        showDivider
      />
      <SettingsRow
        icon={<Sparkles size={20} color="#E6E1E9" />}
        title="Accent Color"
        value="Lavender"
        badge={accentColorBadge}
        onPress={onPressAccent}
        showDivider
      />
      <SettingsRow
        icon={<Type size={20} color="#E6E1E9" />}
        title="Typography"
        value="System"
        onPress={onPressFont}
      />
    </SettingsSection>
  )
})
