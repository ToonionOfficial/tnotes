import { Moon, Sparkles, Sun, Type } from "lucide-react-native"
import { memo } from "react"
import { View } from "react-native"
import { useAppTheme } from "@/hooks/useAppTheme"
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
  const { colors, preference } = useAppTheme()
  const accentColorBadge = <View className="size-3.5 rounded-full bg-primary" />

  const subtitle =
    preference === "system"
      ? `System (${isDarkMode ? "Dark" : "Light"})`
      : isDarkMode
        ? "Dark theme enabled"
        : "Light theme enabled"

  return (
    <SettingsSection title="Appearance">
      <SettingsRow
        icon={
          isDarkMode ? (
            <Moon size={20} color={colors.foreground} />
          ) : (
            <Sun size={20} color={colors.foreground} />
          )
        }
        title="Dark Mode"
        subtitle={subtitle}
        isSwitch
        switchValue={isDarkMode}
        onSwitchChange={onToggleTheme}
        showDivider
      />
      <SettingsRow
        icon={<Sparkles size={20} color={colors.foreground} />}
        title="Accent Color"
        value="Lavender"
        badge={accentColorBadge}
        onPress={onPressAccent}
        showDivider
      />
      <SettingsRow
        icon={<Type size={20} color={colors.foreground} />}
        title="Typography"
        value="System"
        onPress={onPressFont}
      />
    </SettingsSection>
  )
})
