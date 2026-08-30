import { Moon, Sparkles, Sun, Type } from "lucide-react-native"
import { memo } from "react"
import { View } from "react-native"
import { SettingsRow } from "./SettingsRow"
import { SettingsSection } from "./SettingsSection"

export interface AppearanceSectionProps {
  isDarkMode?: boolean
  onToggleTheme?: (isDark: boolean) => void
  onPressAccent?: () => void
  onPressFont?: () => void
}

export const AppearanceSection = memo(function AppearanceSection({
  isDarkMode = true,
  onToggleTheme,
  onPressAccent,
  onPressFont,
}: AppearanceSectionProps) {
  const accentColorBadge = <View className="size-3.5 rounded-full bg-[#CABEFF]" />

  return (
    <SettingsSection title="Appearance">
      <SettingsRow
        icon={isDarkMode ? <Moon size={20} color="#E6E1E9" /> : <Sun size={20} color="#E6E1E9" />}
        title="Dark Mode"
        subtitle={isDarkMode ? "Dark theme enabled" : "Light theme enabled"}
        isSwitch
        switchValue={isDarkMode}
        onSwitchChange={onToggleTheme}
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
