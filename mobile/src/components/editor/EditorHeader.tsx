import type { EditorBridge } from "@10play/tentap-editor"
import { useBridgeState } from "@10play/tentap-editor"
import { useRouter } from "expo-router"
import { SymbolView } from "expo-symbols"
import { Check, ChevronLeft, Ellipsis, Share2, Undo2 } from "lucide-react-native"
import { Keyboard, Platform, Pressable, View } from "react-native"
import { useSafeAreaInsets } from "react-native-safe-area-context"

interface EditorHeaderProps {
  editor: EditorBridge
  title?: string
  onBack?: () => void
  onShare?: () => void
  onMore?: () => void
  onDone?: () => void
}

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
    dismissEditorKeyboard()
    if (onBack) {
      onBack()
    } else {
      router.back()
    }
  }

  const handleDone = () => {
    dismissEditorKeyboard()
    if (onDone) {
      onDone()
    }
  }

  return (
    <View
      style={{
        paddingTop: Math.max(insets.top, 8),
      }}
      className="z-10 flex-row items-center justify-between px-4 py-2"
    >
      <Pressable
        onPress={handleBack}
        hitSlop={8}
        className="size-10 items-center justify-center rounded-full bg-white/10 active:opacity-60"
      >
        {Platform.OS === "ios" ? (
          <SymbolView name="chevron.left" size={18} tintColor="#E6E1E9" type="monochrome" />
        ) : (
          <ChevronLeft size={20} color="#E6E1E9" />
        )}
      </Pressable>

      <View className="flex-row items-center gap-3">
        <View className="h-10 flex-row items-center rounded-full bg-white/10 px-1.5">
          <Pressable
            onPress={() => editor.undo()}
            disabled={!editorState.canUndo}
            hitSlop={6}
            className={`size-8 items-center justify-center rounded-full active:bg-white/10 ${
              editorState.canUndo ? "opacity-100" : "opacity-35"
            }`}
          >
            {Platform.OS === "ios" ? (
              <SymbolView
                name="arrow.uturn.backward"
                size={16}
                tintColor="#E6E1E9"
                type="monochrome"
              />
            ) : (
              <Undo2 size={17} color="#E6E1E9" />
            )}
          </Pressable>

          <Pressable
            onPress={onShare}
            hitSlop={6}
            className="size-8 items-center justify-center rounded-full active:bg-white/10"
          >
            {Platform.OS === "ios" ? (
              <SymbolView
                name="square.and.arrow.up"
                size={16}
                tintColor="#E6E1E9"
                type="monochrome"
              />
            ) : (
              <Share2 size={17} color="#E6E1E9" />
            )}
          </Pressable>

          <Pressable
            onPress={onMore}
            hitSlop={6}
            className="size-8 items-center justify-center rounded-full active:bg-white/10"
          >
            {Platform.OS === "ios" ? (
              <SymbolView name="ellipsis" size={16} tintColor="#E6E1E9" type="monochrome" />
            ) : (
              <Ellipsis size={17} color="#E6E1E9" />
            )}
          </Pressable>
        </View>

        <Pressable
          onPress={handleDone}
          hitSlop={8}
          className="size-10 items-center justify-center rounded-full bg-primary active:opacity-75"
        >
          {Platform.OS === "ios" ? (
            <SymbolView name="checkmark" size={16} tintColor="#32285F" type="monochrome" />
          ) : (
            <Check size={18} color="#32285F" strokeWidth={2.5} />
          )}
        </Pressable>
      </View>
    </View>
  )
}
