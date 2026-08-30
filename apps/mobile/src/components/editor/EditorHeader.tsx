import type { EditorBridge } from "@10play/tentap-editor"
import { useBridgeState } from "@10play/tentap-editor"
import { Button, Host, Icon } from "@expo/ui"
import {
  buttonBorderShape,
  buttonStyle,
  controlSize,
  disabled as disabledModifier,
  foregroundStyle,
} from "@expo/ui/swift-ui/modifiers"
import * as Haptics from "expo-haptics"
import { Check, ChevronLeft, MoreHorizontal, Share, Undo } from "lucide-react-native"
import { memo } from "react"
import { Platform, Pressable, View } from "react-native"
import { useSafeAreaInsets } from "react-native-safe-area-context"
import { useAppTheme } from "@/hooks/useAppTheme"

interface EditorHeaderProps {
  editor: EditorBridge
  title?: string
  onBack: () => void
  onShare?: () => void
  onMore?: () => void
  onDone?: () => void
}

const BACK_ICON = Icon.select({
  ios: "chevron.left",
  android: import("@expo/material-symbols/arrow_back.xml"),
})

const UNDO_ICON = Icon.select({
  ios: "arrow.uturn.backward",
  android: import("@expo/material-symbols/undo.xml"),
})

const SHARE_ICON = Icon.select({
  ios: "square.and.arrow.up",
  android: import("@expo/material-symbols/share.xml"),
})

const MORE_ICON = Icon.select({
  ios: "ellipsis",
  android: import("@expo/material-symbols/more_horiz.xml"),
})

const CHECK_ICON = Icon.select({
  ios: "checkmark",
  android: import("@expo/material-symbols/check.xml"),
})

export const EditorHeader = memo(function EditorHeader({
  editor,
  onBack,
  onShare,
  onMore,
  onDone,
}: EditorHeaderProps) {
  const insets = useSafeAreaInsets()
  const editorState = useBridgeState(editor)
  const { colors } = useAppTheme()

  const handleBack = () => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    onBack()
  }

  const handleUndo = () => {
    if (editorState.canUndo) {
      void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
      editor.undo()
    }
  }

  const handleShare = () => {
    if (onShare) {
      void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
      onShare()
    }
  }

  const handleMore = () => {
    if (onMore) {
      void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
      onMore()
    }
  }

  const handleDone = () => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    if (onDone) {
      onDone()
    } else {
      onBack()
    }
  }

  return (
    <View
      style={{
        paddingTop: insets.top + 6,
        paddingBottom: 8,
      }}
      className="px-4"
    >
      {Platform.OS === "ios" ? (
        <View className="h-11 min-h-11 flex-row items-center justify-between">
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

          <View className="flex-row items-center gap-2.5">
            <Host matchContents ignoreSafeArea="all">
              <Button
                variant="text"
                onPress={handleUndo}
                modifiers={[
                  buttonStyle("glass"),
                  buttonBorderShape("circle"),
                  controlSize("large"),
                  foregroundStyle(colors.foreground),
                  disabledModifier(!editorState.canUndo),
                ]}
              >
                <Icon name={UNDO_ICON} color={colors.foreground} size={18} />
              </Button>
            </Host>

            {onShare && (
              <Host matchContents ignoreSafeArea="all">
                <Button
                  variant="text"
                  onPress={handleShare}
                  modifiers={[
                    buttonStyle("glass"),
                    buttonBorderShape("circle"),
                    controlSize("large"),
                    foregroundStyle(colors.foreground),
                  ]}
                >
                  <Icon name={SHARE_ICON} color={colors.foreground} size={18} />
                </Button>
              </Host>
            )}

            {onMore && (
              <Host matchContents ignoreSafeArea="all">
                <Button
                  variant="text"
                  onPress={handleMore}
                  modifiers={[
                    buttonStyle("glass"),
                    buttonBorderShape("circle"),
                    controlSize("large"),
                    foregroundStyle(colors.foreground),
                  ]}
                >
                  <Icon name={MORE_ICON} color={colors.foreground} size={18} />
                </Button>
              </Host>
            )}

            <Host matchContents ignoreSafeArea="all">
              <Button
                variant="text"
                onPress={handleDone}
                modifiers={[
                  buttonStyle("glass"),
                  buttonBorderShape("circle"),
                  controlSize("large"),
                  foregroundStyle(colors.foreground),
                ]}
              >
                <Icon name={CHECK_ICON} color={colors.foreground} size={18} />
              </Button>
            </Host>
          </View>
        </View>
      ) : (
        <View className="h-11 min-h-11 flex-row items-center justify-between">
          <Pressable
            onPress={handleBack}
            hitSlop={8}
            className="size-11 items-center justify-center rounded-full bg-card border border-border/40 active:bg-accent"
          >
            <ChevronLeft size={22} color={colors.foreground} />
          </Pressable>

          <View className="flex-row items-center gap-2.5">
            <Pressable
              onPress={handleUndo}
              disabled={!editorState.canUndo}
              hitSlop={8}
              className={`size-11 items-center justify-center rounded-full bg-card border border-border/40 active:bg-accent ${
                !editorState.canUndo ? "opacity-35" : ""
              }`}
            >
              <Undo size={19} color={colors.foreground} />
            </Pressable>

            {onShare && (
              <Pressable
                onPress={handleShare}
                hitSlop={8}
                className="size-11 items-center justify-center rounded-full bg-card border border-border/40 active:bg-accent"
              >
                <Share size={19} color={colors.foreground} />
              </Pressable>
            )}

            {onMore && (
              <Pressable
                onPress={handleMore}
                hitSlop={8}
                className="size-11 items-center justify-center rounded-full bg-card border border-border/40 active:bg-accent"
              >
                <MoreHorizontal size={20} color={colors.foreground} />
              </Pressable>
            )}

            <Pressable
              onPress={handleDone}
              hitSlop={8}
              className="size-11 items-center justify-center rounded-full bg-card border border-border/40 active:bg-accent"
            >
              <Check size={20} color={colors.foreground} />
            </Pressable>
          </View>
        </View>
      )}
    </View>
  )
})
