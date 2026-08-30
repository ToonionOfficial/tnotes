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
import { useRouter } from "expo-router"
import { Check, ChevronLeft, Ellipsis, Share2, Undo2 } from "lucide-react-native"
import { Keyboard, Platform, Pressable, View } from "react-native"
import { useSafeAreaInsets } from "react-native-safe-area-context"
import { useAppTheme } from "@/hooks/useAppTheme"

interface EditorHeaderProps {
  editor: EditorBridge
  title?: string
  onBack?: () => void
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

export function EditorHeader({ editor, onBack, onShare, onMore, onDone }: EditorHeaderProps) {
  const router = useRouter()
  const insets = useSafeAreaInsets()
  const editorState = useBridgeState(editor)
  const { colors } = useAppTheme()

  const dismissEditorKeyboard = () => {
    try {
      editor.blur()
      editor.injectJS("if (document.activeElement) { document.activeElement.blur(); }")
    } catch {}
    Keyboard.dismiss()
  }

  const handleBack = () => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    dismissEditorKeyboard()
    if (onBack) {
      onBack()
    } else {
      router.back()
    }
  }

  const handleUndo = () => {
    if (!editorState.canUndo) return
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    editor.undo()
  }

  const handleShare = () => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    onShare?.()
  }

  const handleMore = () => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    dismissEditorKeyboard()
    onMore?.()
  }

  const handleDone = () => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    dismissEditorKeyboard()
    if (onDone) {
      onDone()
    } else {
      router.back()
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
              variant="filled"
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
                variant="filled"
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
                  variant="filled"
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
                  variant="filled"
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
                variant="filled"
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
                editorState.canUndo ? "opacity-100" : "opacity-35"
              }`}
            >
              <Undo2 size={20} color={colors.foreground} />
            </Pressable>

            {onShare && (
              <Pressable
                onPress={handleShare}
                hitSlop={8}
                className="size-11 items-center justify-center rounded-full bg-card border border-border/40 active:bg-accent"
              >
                <Share2 size={20} color={colors.foreground} />
              </Pressable>
            )}

            {onMore && (
              <Pressable
                onPress={handleMore}
                hitSlop={8}
                className="size-11 items-center justify-center rounded-full bg-card border border-border/40 active:bg-accent"
              >
                <Ellipsis size={20} color={colors.foreground} />
              </Pressable>
            )}

            <Pressable
              onPress={handleDone}
              hitSlop={8}
              className="size-11 items-center justify-center rounded-full bg-card border border-border/40 active:bg-accent"
            >
              <Check size={20} color={colors.foreground} strokeWidth={2.5} />
            </Pressable>
          </View>
        </View>
      )}
    </View>
  )
}
