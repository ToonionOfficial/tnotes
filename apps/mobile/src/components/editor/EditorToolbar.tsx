import { GlassView } from "expo-glass-effect"
import type { RefObject } from "react"
import { Keyboard, Platform, ScrollView, View } from "react-native"
import type { EnrichedMarkdownTextInputInstance, StyleState } from "react-native-enriched-markdown"
import { useReanimatedKeyboardAnimation } from "react-native-keyboard-controller"
import Animated, { interpolate, useAnimatedStyle } from "react-native-reanimated"
import { useSafeAreaInsets } from "react-native-safe-area-context"
import { useAppTheme } from "@/hooks/useAppTheme"
import { ToolbarButton } from "./ToolbarButton"

interface EditorToolbarProps {
  inputRef: RefObject<EnrichedMarkdownTextInputInstance | null>
  styleState: StyleState | null
  keyboardVisible: boolean
  onOpenFormat?: () => void
}

export function EditorToolbar({
  inputRef,
  styleState,
  keyboardVisible,
  onOpenFormat,
}: EditorToolbarProps) {
  const insets = useSafeAreaInsets()
  const { isDarkMode } = useAppTheme()

  const { height, progress } = useReanimatedKeyboardAnimation()

  const animatedContainerStyle = useAnimatedStyle(() => {
    const safeBottom = Math.max(insets.bottom - 6, 0)
    const offset = interpolate(progress.value, [0, 1], [0, safeBottom])
    const translateY = Math.min(0, height.value + offset)

    return {
      transform: [{ translateY }],
    }
  })

  const handleKeyboardToggle = () => {
    try {
      if (keyboardVisible) {
        inputRef.current?.blur()
      } else {
        inputRef.current?.focus()
      }
    } catch {}
    if (keyboardVisible) {
      Keyboard.dismiss()
    }
  }

  const handleOpenFormat = () => {
    if (keyboardVisible) {
      try {
        inputRef.current?.blur()
      } catch {}
      Keyboard.dismiss()
    }
    if (onOpenFormat) {
      onOpenFormat()
    }
  }

  const isHeadingOrStyleActive = Boolean(styleState?.heading.isActive)

  return (
    <Animated.View
      style={[
        {
          paddingBottom: Math.max(insets.bottom, 8),
        },
        animatedContainerStyle,
      ]}
      className="absolute bottom-0 left-0 right-0 items-center px-3"
      pointerEvents="box-none"
    >
      <GlassView
        isInteractive
        glassEffectStyle="regular"
        colorScheme={isDarkMode ? "dark" : "light"}
        style={{
          width: "100%",
          maxWidth: 540,
          height: 52,
          borderRadius: 26,
          overflow: "hidden",
          borderWidth: 1,
          borderColor: isDarkMode ? "rgba(255, 255, 255, 0.15)" : "rgba(0, 0, 0, 0.08)",
          backgroundColor: Platform.select({
            ios: isDarkMode ? "rgba(32, 31, 36, 0.65)" : "rgba(255, 255, 255, 0.75)",
            default: isDarkMode ? "rgba(32, 31, 36, 0.94)" : "#FFFFFF",
          }),
        }}
      >
        <View className="h-full flex-row items-center justify-between px-2.5">
          <ScrollView
            horizontal
            showsHorizontalScrollIndicator={false}
            keyboardShouldPersistTaps="handled"
            contentContainerStyle={{
              alignItems: "center",
              gap: 7,
              paddingRight: 8,
            }}
            className="flex-1"
          >
            <ToolbarButton
              icon="format"
              variant="pill"
              size={22}
              isActive={isHeadingOrStyleActive}
              onPress={handleOpenFormat}
            />

            <ToolbarButton
              icon="bulletList"
              size={21}
              isActive={styleState?.unorderedList.isActive ?? false}
              onPress={() => inputRef.current?.toggleUnorderedList()}
            />

            <ToolbarButton
              icon="bold"
              size={21}
              isActive={styleState?.bold.isActive ?? false}
              onPress={() => inputRef.current?.toggleBold()}
            />

            <ToolbarButton
              icon="italic"
              size={21}
              isActive={styleState?.italic.isActive ?? false}
              onPress={() => inputRef.current?.toggleItalic()}
            />

            <ToolbarButton
              icon="underline"
              size={21}
              isActive={styleState?.underline.isActive ?? false}
              onPress={() => inputRef.current?.toggleUnderline()}
            />

            <ToolbarButton
              icon="strike"
              size={21}
              isActive={styleState?.strikethrough.isActive ?? false}
              onPress={() => inputRef.current?.toggleStrikethrough()}
            />
          </ScrollView>

          <View className="h-6 w-px bg-white/10" />

          <ToolbarButton icon="dismiss" size={22} onPress={handleKeyboardToggle} />
        </View>
      </GlassView>
    </Animated.View>
  )
}
