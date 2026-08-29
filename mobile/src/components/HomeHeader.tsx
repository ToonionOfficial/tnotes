import { Button, Host, Icon } from "@expo/ui"
import {
  buttonBorderShape,
  buttonStyle,
  controlSize,
  foregroundStyle,
  tint,
} from "@expo/ui/swift-ui/modifiers"
import * as Haptics from "expo-haptics"
import { Menu, Plus } from "lucide-react-native"
import { memo } from "react"
import { Platform, Pressable, Text, View } from "react-native"
import Animated, {
  Easing,
  interpolate,
  useAnimatedStyle,
  useDerivedValue,
  withTiming,
} from "react-native-reanimated"
import { EditHeaderActions } from "./EditHeaderActions"

interface HomeHeaderProps {
  isEditing: boolean
  hasFolders: boolean
  selectedCount: number
  onPressMenu: () => void
  onToggleEdit: () => void
  onPressNewFolder: () => void
  onDeleteSelected: () => void
}

const MENU_ICON = Icon.select({
  ios: "line.3.horizontal",
  android: import("@expo/material-symbols/menu.xml"),
})

const PLUS_ICON = Icon.select({
  ios: "plus",
  android: import("@expo/material-symbols/add.xml"),
})

export const HomeHeader = memo(function HomeHeader({
  isEditing,
  hasFolders,
  selectedCount,
  onPressMenu,
  onToggleEdit,
  onPressNewFolder,
  onDeleteSelected,
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
      <View className="h-11 min-h-[44px] flex-row items-center justify-between">
        {/* Left: Menu Glass Button */}
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
              tint("#ffffff"),
              foregroundStyle("#ffffff"),
            ]}
          >
            <Icon name={MENU_ICON} color="#ffffff" size={20} />
          </Button>
        </Host>

        {/* Right: Cross-fading Actions Container */}
        <View className="relative h-11 items-center justify-center">
          {/* Normal Mode Actions [Edit, Plus] */}
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
                  tint("#ffffff"),
                  foregroundStyle("#ffffff"),
                ]}
              >
                <Icon name={PLUS_ICON} color="#ffffff" size={20} />
              </Button>
            </Host>
          </Animated.View>

          {/* Edit Mode Actions [Trash, Done] */}
          <Animated.View
            style={[editingActionsStyle, { position: "absolute", right: 0 }]}
            pointerEvents={isEditing ? "auto" : "none"}
          >
            <EditHeaderActions
              selectedCount={selectedCount}
              onDelete={onDeleteSelected}
              onDone={onToggleEdit}
            />
          </Animated.View>
        </View>
      </View>
    )
  }

  // Android Fallback
  return (
    <View className="h-11 min-h-[44px] flex-row items-center justify-between">
      {/* Left: Menu Button */}
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

      {/* Right: Actions Container */}
      <View className="relative h-11 items-center justify-center">
        {/* Normal Mode Actions */}
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
            <Plus size={22} color="#ffffff" />
          </Pressable>
        </Animated.View>

        {/* Edit Mode Actions */}
        <Animated.View
          style={[editingActionsStyle, { position: "absolute", right: 0 }]}
          pointerEvents={isEditing ? "auto" : "none"}
        >
          <EditHeaderActions
            selectedCount={selectedCount}
            onDelete={onDeleteSelected}
            onDone={onToggleEdit}
          />
        </Animated.View>
      </View>
    </View>
  )
})
