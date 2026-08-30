import { Button, Host, Icon } from "@expo/ui"
import {
  buttonBorderShape,
  buttonStyle,
  controlSize,
  foregroundStyle,
  tint,
} from "@expo/ui/swift-ui/modifiers"
import { GlassView } from "expo-glass-effect"
import * as Haptics from "expo-haptics"
import { useRouter } from "expo-router"
import { SymbolView } from "expo-symbols"
import { useEffect, useRef, useState } from "react"
import { Keyboard, Platform, Pressable, TextInput, View } from "react-native"
import { useReanimatedKeyboardAnimation } from "react-native-keyboard-controller"
import Animated, { interpolate, useAnimatedStyle } from "react-native-reanimated"
import { useSafeAreaInsets } from "react-native-safe-area-context"
import { useAppTheme } from "@/hooks/useAppTheme"

interface BottomBarProps {
  onPressNewNote?: () => void
  searchValue: string
  onSearchChange: (value: string) => void
}

const NEW_NOTE_ICON = Icon.select({
  ios: "square.and.pencil",
  android: import("@expo/material-symbols/edit_note.xml"),
})

const CLOSE_ICON = Icon.select({
  ios: "xmark",
  android: import("@expo/material-symbols/close.xml"),
})

export function BottomBar({ onPressNewNote, searchValue, onSearchChange }: BottomBarProps) {
  const router = useRouter()
  const insets = useSafeAreaInsets()
  const inputRef = useRef<TextInput>(null)
  const [isKeyboardVisible, setIsKeyboardVisible] = useState(false)
  const { isDarkMode, colors } = useAppTheme()

  const { height, progress } = useReanimatedKeyboardAnimation()

  useEffect(() => {
    const showSub = Keyboard.addListener(
      Platform.OS === "ios" ? "keyboardWillShow" : "keyboardDidShow",
      () => setIsKeyboardVisible(true),
    )
    const hideSub = Keyboard.addListener(
      Platform.OS === "ios" ? "keyboardWillHide" : "keyboardDidHide",
      () => setIsKeyboardVisible(false),
    )
    return () => {
      showSub.remove()
      hideSub.remove()
    }
  }, [])

  const animatedContainerStyle = useAnimatedStyle(() => {
    const safeBottom = Math.max(insets.bottom - 8, 0)
    const offset = interpolate(progress.value, [0, 1], [0, safeBottom])
    return {
      transform: [{ translateY: height.value + offset }],
    }
  })

  const handleClear = () => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    onSearchChange("")
  }

  const handleRightButtonPress = () => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    if (isKeyboardVisible) {
      Keyboard.dismiss()
    } else if (onPressNewNote) {
      onPressNewNote()
    } else {
      router.push("/notes/new")
    }
  }

  const handleFocusSearch = () => {
    inputRef.current?.focus()
  }

  return (
    <Animated.View
      style={[
        {
          position: "absolute",
          left: 0,
          right: 0,
          bottom: insets.bottom + 12,
          paddingHorizontal: 20,
          alignItems: "center",
          zIndex: 40,
        },
        animatedContainerStyle,
      ]}
      pointerEvents="box-none"
    >
      <View className="w-full max-w-110 flex-row items-center gap-3.5">
        {/* Search */}
        <GlassView
          isInteractive
          glassEffectStyle="regular"
          colorScheme={isDarkMode ? "dark" : "light"}
          style={{
            flex: 1,
            height: 48,
            borderRadius: 24,
            overflow: "hidden",
            borderWidth: 1,
            borderColor: isDarkMode ? "rgba(255, 255, 255, 0.12)" : "rgba(0, 0, 0, 0.08)",
            backgroundColor: Platform.select({
              ios: isDarkMode ? "rgba(32, 31, 36, 0.4)" : "rgba(255, 255, 255, 0.6)",
              default: isDarkMode ? "rgba(32, 31, 36, 0.88)" : "#FFFFFF",
            }),
          }}
        >
          <View className="h-full flex-1 flex-row items-center px-4">
            <Pressable
              onPress={handleFocusSearch}
              hitSlop={8}
              className="items-center justify-center"
            >
              <SymbolView
                name="magnifyingglass"
                size={18}
                tintColor={colors.mutedForeground}
                type="monochrome"
              />
            </Pressable>

            <TextInput
              ref={inputRef}
              value={searchValue}
              onChangeText={onSearchChange}
              placeholder="Search"
              placeholderTextColor={colors.mutedForeground}
              cursorColor={colors.primary}
              returnKeyType="search"
              className="h-full flex-1 px-3 text-[16px] text-foreground"
              style={{
                paddingVertical: 0,
              }}
            />

            {searchValue.length > 0 && (
              <Pressable
                onPress={handleClear}
                hitSlop={10}
                className="size-6 items-center justify-center rounded-full bg-white/15 active:opacity-60"
              >
                <SymbolView
                  name="xmark"
                  size={11}
                  tintColor={colors.foreground}
                  type="monochrome"
                />
              </Pressable>
            )}
          </View>
        </GlassView>

        {/* New note / Dismiss button */}
        <Host matchContents ignoreSafeArea="all">
          <Button
            variant="filled"
            onPress={handleRightButtonPress}
            modifiers={
              Platform.OS === "ios"
                ? isKeyboardVisible
                  ? [
                      buttonStyle("glass"),
                      controlSize("large"),
                      buttonBorderShape("circle"),
                      foregroundStyle(colors.foreground),
                    ]
                  : [
                      buttonStyle("glassProminent"),
                      controlSize("large"),
                      buttonBorderShape("circle"),
                      tint(colors.primary),
                      foregroundStyle(isDarkMode ? "#32285F" : "#FFFFFF"),
                    ]
                : undefined
            }
          >
            <Icon
              name={isKeyboardVisible ? CLOSE_ICON : NEW_NOTE_ICON}
              size={isKeyboardVisible ? 18 : 22}
              color={isKeyboardVisible ? colors.foreground : isDarkMode ? "#32285F" : "#FFFFFF"}
            />
          </Button>
        </Host>
      </View>
    </Animated.View>
  )
}
