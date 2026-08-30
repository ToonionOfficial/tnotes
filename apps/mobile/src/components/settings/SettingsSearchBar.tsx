import { GlassView } from "expo-glass-effect"
import * as Haptics from "expo-haptics"
import { SymbolView } from "expo-symbols"
import { Search, X } from "lucide-react-native"
import { memo } from "react"
import { Platform, Pressable, TextInput, View } from "react-native"
import { useAppTheme } from "@/hooks/useAppTheme"

export interface SettingsSearchBarProps {
  value: string
  onChangeText: (text: string) => void
  placeholder?: string
}

export const SettingsSearchBar = memo(function SettingsSearchBar({
  value,
  onChangeText,
  placeholder = "Search settings",
}: SettingsSearchBarProps) {
  const { isDarkMode, colors } = useAppTheme()

  const handleClear = () => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    onChangeText("")
  }

  return (
    <GlassView
      isInteractive
      glassEffectStyle="regular"
      colorScheme={isDarkMode ? "dark" : "light"}
      style={{
        width: "100%",
        height: 48,
        borderRadius: 24,
        overflow: "hidden",
        borderWidth: 1,
        borderColor: isDarkMode ? "rgba(255, 255, 255, 0.12)" : "rgba(0, 0, 0, 0.08)",
        backgroundColor: Platform.select({
          ios: isDarkMode ? "rgba(32, 31, 36, 0.4)" : "rgba(255, 255, 255, 0.6)",
          default: isDarkMode ? "rgba(32, 31, 36, 0.88)" : "#FFFFFF",
        }),
        marginBottom: 20,
      }}
    >
      <View className="h-full flex-row items-center px-4">
        {Platform.OS === "ios" ? (
          <SymbolView
            name="magnifyingglass"
            size={17}
            tintColor={colors.mutedForeground}
            type="monochrome"
          />
        ) : (
          <Search size={18} color={colors.mutedForeground} />
        )}
        <TextInput
          value={value}
          onChangeText={onChangeText}
          placeholder={placeholder}
          placeholderTextColor={colors.mutedForeground}
          cursorColor={colors.primary}
          clearButtonMode="never"
          returnKeyType="search"
          className="h-full flex-1 px-3 text-[16px] text-foreground"
          style={{ paddingVertical: 0 }}
        />
        {value.length > 0 && (
          <Pressable onPress={handleClear} hitSlop={8} className="p-1 active:opacity-60">
            {Platform.OS === "ios" ? (
              <SymbolView
                name="xmark.circle.fill"
                size={16}
                tintColor={colors.mutedForeground}
                type="hierarchical"
              />
            ) : (
              <X size={16} color={colors.mutedForeground} />
            )}
          </Pressable>
        )}
      </View>
    </GlassView>
  )
})
