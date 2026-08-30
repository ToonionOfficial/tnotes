import { GlassView } from "expo-glass-effect"
import * as Haptics from "expo-haptics"
import { SymbolView } from "expo-symbols"
import { Search, X } from "lucide-react-native"
import { memo } from "react"
import { Platform, Pressable, TextInput, View } from "react-native"

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
  const handleClear = () => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    onChangeText("")
  }

  return (
    <GlassView
      isInteractive
      glassEffectStyle="regular"
      colorScheme="dark"
      style={{
        width: "100%",
        height: 48,
        borderRadius: 24,
        overflow: "hidden",
        borderWidth: 1,
        borderColor: "rgba(255, 255, 255, 0.12)",
        backgroundColor: Platform.select({
          ios: "rgba(32, 31, 36, 0.4)",
          default: "rgba(32, 31, 36, 0.88)",
        }),
        marginBottom: 20,
      }}
    >
      <View className="h-full flex-row items-center px-4">
        {Platform.OS === "ios" ? (
          <SymbolView name="magnifyingglass" size={17} tintColor="#8E8D94" type="monochrome" />
        ) : (
          <Search size={18} color="#8E8D94" />
        )}
        <TextInput
          value={value}
          onChangeText={onChangeText}
          placeholder={placeholder}
          placeholderTextColor="#8E8D94"
          cursorColor="#CABEFF"
          clearButtonMode="never"
          returnKeyType="search"
          className="h-full flex-1 px-3 text-[16px] text-foreground"
          style={{ paddingVertical: 0 }}
        />
        {value.length > 0 && (
          <Pressable
            onPress={handleClear}
            hitSlop={8}
            className="size-6 items-center justify-center rounded-full bg-white/15 active:opacity-60"
          >
            {Platform.OS === "ios" ? (
              <SymbolView name="xmark" size={11} tintColor="#E6E1E9" type="monochrome" />
            ) : (
              <X size={12} color="#E6E1E9" />
            )}
          </Pressable>
        )}
      </View>
    </GlassView>
  )
})
