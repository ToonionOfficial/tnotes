import type { EditorBridge } from "@10play/tentap-editor"
import { useBridgeState } from "@10play/tentap-editor"
import { GlassView } from "expo-glass-effect"
import { Keyboard, Platform, ScrollView, View } from "react-native"
import { useReanimatedKeyboardAnimation } from "react-native-keyboard-controller"
import Animated, { interpolate, useAnimatedStyle } from "react-native-reanimated"
import { useSafeAreaInsets } from "react-native-safe-area-context"
import { ToolbarButton } from "./ToolbarButton"

interface EditorToolbarProps {
  editor: EditorBridge
  onOpenFormat?: () => void
  onInsertTable?: () => void
}

export function EditorToolbar({ editor, onOpenFormat, onInsertTable }: EditorToolbarProps) {
  const insets = useSafeAreaInsets()
  const editorState = useBridgeState(editor)

  const { height, progress } = useReanimatedKeyboardAnimation()

  const animatedContainerStyle = useAnimatedStyle(() => {
    const safeBottom = Math.max(insets.bottom - 6, 0)
    const offset = interpolate(progress.value, [0, 1], [0, safeBottom])
    const translateY = Math.min(0, height.value + offset)

    return {
      transform: [{ translateY }],
    }
  })

  const dismissContainerStyle = useAnimatedStyle(() => {
    return {
      opacity: progress.value,
      width: interpolate(progress.value, [0, 1], [0, 48]),
      transform: [{ scale: interpolate(progress.value, [0, 1], [0.6, 1]) }],
      overflow: "hidden" as const,
    }
  })

  const dismissAllKeyboards = () => {
    try {
      editor.blur()
      editor.injectJS(
        "if (document.activeElement) { document.activeElement.blur(); } window.getSelection()?.removeAllRanges();",
      )
    } catch {}
    Keyboard.dismiss()
  }

  const handleOpenFormat = () => {
    dismissAllKeyboards()
    if (onOpenFormat) {
      onOpenFormat()
    }
  }

  const handleDismissKeyboard = () => {
    dismissAllKeyboards()
  }

  const handleTablePress = () => {
    if (onInsertTable) {
      onInsertTable()
    }
  }

  const isHeadingOrStyleActive =
    Boolean(editorState.headingLevel) || editorState.isBlockquoteActive || editorState.isCodeActive

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
        colorScheme="dark"
        style={{
          width: "100%",
          maxWidth: 540,
          height: 52,
          borderRadius: 26,
          overflow: "hidden",
          borderWidth: 1,
          borderColor: "rgba(255, 255, 255, 0.15)",
          backgroundColor: Platform.select({
            ios: "rgba(32, 31, 36, 0.65)",
            default: "rgba(32, 31, 36, 0.94)",
          }),
        }}
      >
        <View className="h-full flex-row items-center justify-between px-2.5">
          <ScrollView
            horizontal
            showsHorizontalScrollIndicator={false}
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
              icon="checklist"
              size={21}
              isActive={editorState.isTaskListActive}
              isDisabled={!editorState.canToggleTaskList}
              onPress={() => editor.toggleTaskList()}
            />

            <ToolbarButton
              icon="bulletList"
              size={21}
              isActive={editorState.isBulletListActive}
              isDisabled={!editorState.canToggleBulletList}
              onPress={() => editor.toggleBulletList()}
            />

            <ToolbarButton icon="table" size={21} onPress={handleTablePress} />

            <ToolbarButton
              icon="bold"
              size={21}
              isActive={editorState.isBoldActive}
              isDisabled={!editorState.canToggleBold}
              onPress={() => editor.toggleBold()}
            />

            <ToolbarButton
              icon="italic"
              size={21}
              isActive={editorState.isItalicActive}
              isDisabled={!editorState.canToggleItalic}
              onPress={() => editor.toggleItalic()}
            />

            <ToolbarButton
              icon="underline"
              size={21}
              isActive={editorState.isUnderlineActive}
              isDisabled={!editorState.canToggleUnderline}
              onPress={() => editor.toggleUnderline()}
            />

            <ToolbarButton
              icon="strike"
              size={21}
              isActive={editorState.isStrikeActive}
              isDisabled={!editorState.canToggleStrike}
              onPress={() => editor.toggleStrike()}
            />

            <ToolbarButton
              icon="undo"
              size={21}
              isDisabled={!editorState.canUndo}
              onPress={() => editor.undo()}
            />

            <ToolbarButton
              icon="redo"
              size={21}
              isDisabled={!editorState.canRedo}
              onPress={() => editor.redo()}
            />
          </ScrollView>

          <Animated.View
            style={dismissContainerStyle}
            className="flex-row items-center justify-end"
          >
            <View className="border-l border-white/15 pl-2">
              <ToolbarButton icon="dismiss" size={21} onPress={handleDismissKeyboard} />
            </View>
          </Animated.View>
        </View>
      </GlassView>
    </Animated.View>
  )
}
