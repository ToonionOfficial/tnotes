import type { EditorBridge } from "@10play/tentap-editor"
import { useBridgeState } from "@10play/tentap-editor"
import { Button, Host, Icon } from "@expo/ui"
import {
  buttonBorderShape,
  buttonStyle,
  controlSize,
  disabled as disabledModifier,
  foregroundStyle,
  tint,
} from "@expo/ui/swift-ui/modifiers"
import * as Haptics from "expo-haptics"
import { useRouter } from "expo-router"
import { ChevronLeft, Ellipsis, Share2, Undo2 } from "lucide-react-native"
import { Keyboard, Platform, Pressable, Text, View } from "react-native"
import { useSafeAreaInsets } from "react-native-safe-area-context"

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

export function EditorHeader({ editor, onBack, onShare, onMore, onDone }: EditorHeaderProps) {
  const router = useRouter()
  const insets = useSafeAreaInsets()
  const editorState = useBridgeState(editor)

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

  const handleDone = () => {
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    dismissEditorKeyboard()
    if (onDone) {
      onDone()
    }
  }

  const handleUndo = () => {
    if (!editorState.canUndo) return
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    editor.undo()
  }

  const handleShare = () => {
    if (!onShare) return
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    onShare()
  }

  const handleMore = () => {
    if (!onMore) return
    void Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light)
    onMore()
  }

  if (Platform.OS === "ios") {
    return (
      <View
        style={{
          paddingTop: Math.max(insets.top, 8),
        }}
        className="z-10 h-11 min-h-11 flex-row items-center justify-between px-4 py-2"
      >
        <Host matchContents ignoreSafeArea="all">
          <Button
            variant="filled"
            onPress={handleBack}
            modifiers={[
              buttonStyle("glass"),
              buttonBorderShape("circle"),
              controlSize("large"),
              tint("#ffffff"),
              foregroundStyle("#ffffff"),
            ]}
          >
            <Icon name={BACK_ICON} color="#ffffff" size={20} />
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
                tint("#ffffff"),
                foregroundStyle("#ffffff"),
                disabledModifier(!editorState.canUndo),
              ]}
            >
              <Icon name={UNDO_ICON} color="#ffffff" size={18} />
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
                  tint("#ffffff"),
                  foregroundStyle("#ffffff"),
                ]}
              >
                <Icon name={SHARE_ICON} color="#ffffff" size={18} />
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
                  tint("#ffffff"),
                  foregroundStyle("#ffffff"),
                ]}
              >
                <Icon name={MORE_ICON} color="#ffffff" size={18} />
              </Button>
            </Host>
          )}

          <Host matchContents ignoreSafeArea="all">
            <Button
              variant="filled"
              label="Done"
              onPress={handleDone}
              modifiers={[
                buttonStyle("glass"),
                buttonBorderShape("capsule"),
                controlSize("large"),
                foregroundStyle("#ffffff"),
              ]}
            />
          </Host>
        </View>
      </View>
    )
  }

  return (
    <View
      style={{
        paddingTop: Math.max(insets.top, 8),
      }}
      className="z-10 h-11 min-h-11 flex-row items-center justify-between px-4 py-2"
    >
      <Pressable
        onPress={handleBack}
        hitSlop={8}
        className="size-11 items-center justify-center rounded-full bg-white/[0.07] active:opacity-60"
      >
        <ChevronLeft size={22} color="#ffffff" />
      </Pressable>

      <View className="flex-row items-center gap-2.5">
        <Pressable
          onPress={handleUndo}
          disabled={!editorState.canUndo}
          hitSlop={8}
          className={`size-11 items-center justify-center rounded-full bg-white/[0.07] active:opacity-60 ${
            editorState.canUndo ? "opacity-100" : "opacity-35"
          }`}
        >
          <Undo2 size={20} color="#ffffff" />
        </Pressable>

        {onShare && (
          <Pressable
            onPress={handleShare}
            hitSlop={8}
            className="size-11 items-center justify-center rounded-full bg-white/[0.07] active:opacity-60"
          >
            <Share2 size={20} color="#ffffff" />
          </Pressable>
        )}

        {onMore && (
          <Pressable
            onPress={handleMore}
            hitSlop={8}
            className="size-11 items-center justify-center rounded-full bg-white/[0.07] active:opacity-60"
          >
            <Ellipsis size={20} color="#ffffff" />
          </Pressable>
        )}

        <Pressable
          onPress={handleDone}
          hitSlop={8}
          className="h-11 items-center justify-center rounded-full bg-white/[0.07] px-4 active:opacity-60"
        >
          <Text className="text-[15px] font-semibold text-white">Done</Text>
        </Pressable>
      </View>
    </View>
  )
}
