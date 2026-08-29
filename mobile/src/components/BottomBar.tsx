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
    const translateY = Math.min(0, height.value + offset)

    return {
      transform: [{ translateY }],
    }
  })

  const handleNewNote = () => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
    if (onPressNewNote) {
      onPressNewNote()
    } else {
      router.push("/notes/new")
    }
  }

  const handleRightButtonPress = () => {
    if (isKeyboardVisible) {
      inputRef.current?.blur()
      Keyboard.dismiss()
    } else {
      handleNewNote()
    }
  }

  const handleClear = () => {
    onSearchChange("")
    inputRef.current?.focus()
  }

  const handleFocusSearch = () => {
    inputRef.current?.focus()
  }

  return (
    <Animated.View
      style={[
        {
          paddingBottom: Math.max(insets.bottom, 12),
        },
        animatedContainerStyle,
      ]}
      className="absolute bottom-0 left-0 right-0 items-center px-5"
      pointerEvents="box-none"
    >
      <View className="w-full max-w-110 flex-row items-center gap-3.5">
        {/* Search */}
        <GlassView
          isInteractive
          glassEffectStyle="regular"
          colorScheme="dark"
          style={{
            flex: 1,
            height: 48,
            borderRadius: 24,
            overflow: "hidden",
            borderWidth: 1,
            borderColor: "rgba(255, 255, 255, 0.12)",
            backgroundColor: Platform.select({
              ios: "rgba(32, 31, 36, 0.4)",
              default: "rgba(32, 31, 36, 0.88)",
            }),
          }}
        >
          <View className="h-full flex-1 flex-row items-center px-4">
            <Pressable
              onPress={handleFocusSearch}
              hitSlop={8}
              className="items-center justify-center"
            >
              <SymbolView name="magnifyingglass" size={18} tintColor="#C6C2CD" type="monochrome" />
            </Pressable>

            <TextInput
              ref={inputRef}
              value={searchValue}
              onChangeText={onSearchChange}
              placeholder="Search"
              placeholderTextColor="#8E8C99"
              cursorColor="#CABEFF"
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
                <SymbolView name="xmark" size={11} tintColor="#E6E1E9" type="monochrome" />
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
                      foregroundStyle("#E6E1E9"),
                    ]
                  : [
                      buttonStyle("glassProminent"),
                      controlSize("large"),
                      buttonBorderShape("circle"),
                      tint("#CABEFF"),
                      foregroundStyle("#32285F"),
                    ]
                : undefined
            }
          >
            <Icon
              name={isKeyboardVisible ? CLOSE_ICON : NEW_NOTE_ICON}
              size={isKeyboardVisible ? 18 : 22}
              color={isKeyboardVisible ? "#E6E1E9" : "#32285F"}
            />
          </Button>
        </Host>
      </View>
    </Animated.View>
  )
}
