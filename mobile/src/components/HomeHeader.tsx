import { Button, Host, Icon } from "@expo/ui"
import {
  buttonBorderShape,
  buttonStyle,
  controlSize,
  foregroundStyle,
} from "@expo/ui/swift-ui/modifiers"
import * as Haptics from "expo-haptics"
import { FolderPlus, Menu } from "lucide-react-native"
import { memo } from "react"
import { Platform, Pressable, Text, View } from "react-native"
import Animated, {
  Easing,
  interpolate,
  useAnimatedStyle,
  useDerivedValue,
  withTiming,
} from "react-native-reanimated"

interface HomeHeaderProps {
  isEditing: boolean
  hasFolders: boolean
  onPressMenu: () => void
  onToggleEdit: () => void
  onPressNewFolder: () => void
}

const MENU_ICON = Icon.select({
  ios: "line.3.horizontal",
  android: import("@expo/material-symbols/menu.xml"),
})

const NEW_FOLDER_ICON = Icon.select({
  ios: "folder.badge.plus",
  android: import("@expo/material-symbols/create_new_folder.xml"),
})

export const HomeHeader = memo(function HomeHeader({
  isEditing,
  hasFolders,
  onPressMenu,
  onToggleEdit,
  onPressNewFolder,
}: HomeHeaderProps) {
  const editProgress = useDerivedValue(() => {
    return withTiming(isEditing ? 1 : 0, {
      duration: 220,
      easing: Easing.bezier(0.25, 0.1, 0.25, 1),
    })
  }, [isEditing])

  const normalActionsStyle = useAnimatedStyle(() => ({
    opacity: 1 - editProgress.value,
    transform: [{ scale: interpolate(editProgress.value, [0, 1], [1, 0.88]) }],
  }))

  const editingActionsStyle = useAnimatedStyle(() => ({
    opacity: editProgress.value,
    transform: [{ scale: interpolate(editProgress.value, [0, 1], [0.88, 1]) }],
  }))

  if (Platform.OS === "ios") {
    return (
      <View className="h-11 min-h-11 flex-row items-center justify-between">
        <Host matchContents ignoreSafeArea="all">
          <Button
            variant="filled"
            onPress={() => {
              void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
              onPressMenu()
            }}
            modifiers={[
              buttonStyle("glass"),
              buttonBorderShape("circle"),
              controlSize("large"),
              foregroundStyle("#ffffff"),
            ]}
          >
            <Icon name={MENU_ICON} color="#ffffff" size={20} />
          </Button>
        </Host>

        <View className="relative h-11 items-center justify-center">
          <Animated.View
            style={normalActionsStyle}
            pointerEvents={isEditing ? "none" : "auto"}
            className="flex-row items-center gap-2.5"
          >
            {hasFolders && (
              <Host matchContents ignoreSafeArea="all">
                <Button
                  variant="filled"
                  label="Edit"
                  onPress={() => {
                    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
                    onToggleEdit()
                  }}
                  modifiers={[
                    buttonStyle("glass"),
                    buttonBorderShape("capsule"),
                    controlSize("large"),
                    foregroundStyle("#ffffff"),
                  ]}
                />
              </Host>
            )}
            <Host matchContents ignoreSafeArea="all">
              <Button
                variant="filled"
                onPress={() => {
                  void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
                  onPressNewFolder()
                }}
                modifiers={[
                  buttonStyle("glass"),
                  buttonBorderShape("circle"),
                  controlSize("large"),
                  foregroundStyle("#ffffff"),
                ]}
              >
                <Icon name={NEW_FOLDER_ICON} color="#ffffff" size={20} />
              </Button>
            </Host>
          </Animated.View>

          <Animated.View
            style={[editingActionsStyle, { position: "absolute", right: 0 }]}
            pointerEvents={isEditing ? "auto" : "none"}
          >
            <Host matchContents ignoreSafeArea="all">
              <Button
                variant="filled"
                label="Done"
                onPress={() => {
                  void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
                  onToggleEdit()
                }}
                modifiers={[
                  buttonStyle("glass"),
                  buttonBorderShape("capsule"),
                  controlSize("large"),
                  foregroundStyle("#ffffff"),
                ]}
              />
            </Host>
          </Animated.View>
        </View>
      </View>
    )
  }

  return (
    <View className="h-11 min-h-11 flex-row items-center justify-between">
      <Pressable
        onPress={() => {
          void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
          onPressMenu()
        }}
        hitSlop={8}
        className="size-11 items-center justify-center rounded-full bg-white/[0.07] active:opacity-60"
      >
        <Menu size={22} color="#ffffff" />
      </Pressable>

      <View className="relative h-11 items-center justify-center">
        <Animated.View
          style={normalActionsStyle}
          pointerEvents={isEditing ? "none" : "auto"}
          className="flex-row items-center gap-2.5"
        >
          {hasFolders && (
            <Pressable
              onPress={() => {
                void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
                onToggleEdit()
              }}
              hitSlop={8}
              className="h-11 items-center justify-center rounded-full bg-white/[0.07] px-4 active:opacity-60"
            >
              <Text className="text-[15px] font-medium text-white">Edit</Text>
            </Pressable>
          )}
          <Pressable
            onPress={() => {
              void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
              onPressNewFolder()
            }}
            hitSlop={8}
            className="size-11 items-center justify-center rounded-full bg-white/[0.07] active:opacity-60"
          >
            <FolderPlus size={22} color="#ffffff" />
          </Pressable>
        </Animated.View>

        <Animated.View
          style={[editingActionsStyle, { position: "absolute", right: 0 }]}
          pointerEvents={isEditing ? "auto" : "none"}
        >
          <Pressable
            onPress={() => {
              void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
              onToggleEdit()
            }}
            hitSlop={8}
            className="h-11 items-center justify-center rounded-full bg-white/[0.07] px-4 active:opacity-60"
          >
            <Text className="text-[15px] font-semibold text-white">Done</Text>
          </Pressable>
        </Animated.View>
      </View>
    </View>
  )
})
