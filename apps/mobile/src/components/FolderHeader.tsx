import { Button, Host, Icon } from "@expo/ui"
import {
  buttonBorderShape,
  buttonStyle,
  controlSize,
  foregroundStyle,
} from "@expo/ui/swift-ui/modifiers"
import * as Haptics from "expo-haptics"
import { ChevronLeft, FolderPlus } from "lucide-react-native"
import { memo } from "react"
import { Platform, Pressable, Text, View } from "react-native"
import Animated, {
  Easing,
  interpolate,
  useAnimatedStyle,
  useDerivedValue,
  withTiming,
} from "react-native-reanimated"
import { useSafeAreaInsets } from "react-native-safe-area-context"
import { useAppTheme } from "@/hooks/useAppTheme"

interface FolderHeaderProps {
  title: string
  folderId?: string
  hasItems: boolean
  isEditing: boolean
  onBack: () => void
  onToggleEdit: () => void
  onPressNewFolder?: () => void
}

const BACK_ICON = Icon.select({
  ios: "chevron.left",
  android: import("@expo/material-symbols/arrow_back.xml"),
})

const NEW_FOLDER_ICON = Icon.select({
  ios: "folder.badge.plus",
  android: import("@expo/material-symbols/create_new_folder.xml"),
})

export const FolderHeader = memo(function FolderHeader({
  title,
  folderId,
  hasItems,
  isEditing,
  onBack,
  onToggleEdit,
  onPressNewFolder,
}: FolderHeaderProps) {
  const insets = useSafeAreaInsets()
  const { colors } = useAppTheme()

  const editProgress = useDerivedValue(() => {
    return withTiming(isEditing ? 1 : 0, {
      duration: 250,
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

  const handleBack = () => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    onBack()
  }

  return (
    <View style={{ paddingTop: insets.top + 8 }} className="px-5">
      <View className="h-11 min-h-11 flex-row items-center justify-between">
        {/* Left: Back Button */}
        {Platform.OS === "ios" ? (
          <Host matchContents ignoreSafeArea="all">
            <Button
              variant="text"
              onPress={handleBack}
              modifiers={[
                buttonStyle("glass"),
                buttonBorderShape("circle"),
                controlSize("large"),
                foregroundStyle(colors.foreground),
              ]}
            >
              <Icon name={BACK_ICON} color={colors.foreground} size={20} />
            </Button>
          </Host>
        ) : (
          <Pressable
            onPress={handleBack}
            hitSlop={8}
            className="size-11 items-center justify-center rounded-full bg-card border border-border/40 active:bg-accent"
          >
            <ChevronLeft size={22} color={colors.foreground} />
          </Pressable>
        )}

        {/* Right Actions */}
        <View className="relative h-11 items-center justify-center">
          <Animated.View
            style={normalActionsStyle}
            pointerEvents={isEditing ? "none" : "auto"}
            className="flex-row items-center gap-2.5"
          >
            {hasItems &&
              (Platform.OS === "ios" ? (
                <Host matchContents ignoreSafeArea="all">
                  <Button
                    variant="text"
                    label="Edit"
                    onPress={() => {
                      void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
                      onToggleEdit()
                    }}
                    modifiers={[
                      buttonStyle("glass"),
                      buttonBorderShape("capsule"),
                      controlSize("large"),
                      foregroundStyle(colors.foreground),
                    ]}
                  />
                </Host>
              ) : (
                <Pressable
                  onPress={() => {
                    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
                    onToggleEdit()
                  }}
                  hitSlop={8}
                  className="h-11 items-center justify-center rounded-full bg-card border border-border/40 px-4 active:bg-accent"
                >
                  <Text className="text-[15px] font-medium text-foreground">Edit</Text>
                </Pressable>
              ))}

            {folderId &&
              onPressNewFolder &&
              (Platform.OS === "ios" ? (
                <Host matchContents ignoreSafeArea="all">
                  <Button
                    variant="text"
                    onPress={() => {
                      void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
                      onPressNewFolder()
                    }}
                    modifiers={[
                      buttonStyle("glass"),
                      buttonBorderShape("circle"),
                      controlSize("large"),
                      foregroundStyle(colors.foreground),
                    ]}
                  >
                    <Icon name={NEW_FOLDER_ICON} color={colors.foreground} size={20} />
                  </Button>
                </Host>
              ) : (
                <Pressable
                  onPress={() => {
                    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium)
                    onPressNewFolder()
                  }}
                  hitSlop={8}
                  className="size-11 items-center justify-center rounded-full bg-card border border-border/40 active:bg-accent"
                >
                  <FolderPlus size={22} color={colors.foreground} />
                </Pressable>
              ))}
          </Animated.View>

          <Animated.View
            style={[editingActionsStyle, { position: "absolute", right: 0 }]}
            pointerEvents={isEditing ? "auto" : "none"}
          >
            {Platform.OS === "ios" ? (
              <Host matchContents ignoreSafeArea="all">
                <Button
                  variant="text"
                  label="Done"
                  onPress={() => {
                    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
                    onToggleEdit()
                  }}
                  modifiers={[
                    buttonStyle("glass"),
                    buttonBorderShape("capsule"),
                    controlSize("large"),
                    foregroundStyle(colors.foreground),
                  ]}
                />
              </Host>
            ) : (
              <Pressable
                onPress={() => {
                  void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
                  onToggleEdit()
                }}
                hitSlop={8}
                className="h-11 items-center justify-center rounded-full bg-card border border-border/40 px-4 active:bg-accent"
              >
                <Text className="text-[15px] font-semibold text-foreground">Done</Text>
              </Pressable>
            )}
          </Animated.View>
        </View>
      </View>

      {/* Large Title */}
      <Text className="mb-1 mt-2 text-[34px] font-bold tracking-tight text-foreground">
        {title}
      </Text>
    </View>
  )
})
