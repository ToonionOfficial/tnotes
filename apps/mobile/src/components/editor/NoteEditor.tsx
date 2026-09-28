import { useRef, useState } from "react"
import { View } from "react-native"
import {
  EnrichedMarkdownTextInput,
  type EnrichedMarkdownTextInputInstance,
  type StyleState,
} from "react-native-enriched-markdown"
import { useKeyboardState } from "react-native-keyboard-controller"
import { useAppTheme } from "@/hooks/useAppTheme"
import { EditorHeader } from "./EditorHeader"
import { EditorToolbar } from "./EditorToolbar"
import { FormatSheet, type FormatSheetRef } from "./FormatSheet"

interface NoteEditorProps {
  initialContent?: string
  autofocus?: boolean
  headerTitle?: string
  onBack?: () => void
  onDone?: () => void
  onSave?: (markdown: string) => void
}

export function NoteEditor({
  initialContent,
  autofocus = true,
  headerTitle = "Notes",
  onBack,
  onDone,
  onSave,
}: NoteEditorProps) {
  const { colors } = useAppTheme()
  const inputRef = useRef<EnrichedMarkdownTextInputInstance>(null)
  const formatSheetRef = useRef<FormatSheetRef>(null)
  const saveTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const [styleState, setStyleState] = useState<StyleState | null>(null)
  const keyboardVisible = useKeyboardState((state) => state.isVisible)

  const handleFlushSave = async () => {
    if (saveTimeoutRef.current) {
      clearTimeout(saveTimeoutRef.current)
      saveTimeoutRef.current = null
    }
    if (onSave) {
      try {
        const markdown = await inputRef.current?.getMarkdown()
        if (markdown !== undefined) onSave(markdown)
      } catch {}
    }
  }

  const handleChangeMarkdown = (markdown: string) => {
    if (!onSave) return
    if (saveTimeoutRef.current) {
      clearTimeout(saveTimeoutRef.current)
    }
    saveTimeoutRef.current = setTimeout(() => {
      onSave(markdown)
    }, 400)
  }

  const handleOpenFormat = () => {
    setTimeout(() => {
      formatSheetRef.current?.open()
    }, 300)
  }

  const handleBack = async () => {
    await handleFlushSave()
    onBack?.()
  }

  const handleDone = async () => {
    await handleFlushSave()
    onDone?.()
  }

  return (
    <View style={{ backgroundColor: colors.background }} className="flex-1">
      <EditorHeader title={headerTitle} onBack={handleBack} onDone={handleDone} />
      <View className="flex-1 px-6 pt-4">
        <EnrichedMarkdownTextInput
          ref={inputRef}
          defaultValue={initialContent ?? ""}
          autoFocus={autofocus}
          cursorColor={colors.primary}
          selectionColor={colors.primary}
          onChangeMarkdown={handleChangeMarkdown}
          onChangeState={setStyleState}
          markdownStyle={{
            strong: { color: colors.foreground },
            em: { color: colors.foreground },
            h1: { color: colors.foreground },
            h2: { color: colors.foreground },
            h3: { color: colors.foreground },
            h4: { color: colors.foreground },
            h5: { color: colors.foreground },
            h6: { color: colors.foreground },
          }}
          style={{
            flex: 1,
            color: colors.foreground,
            fontSize: 18,
            backgroundColor: colors.background,
          }}
        />
      </View>
      <EditorToolbar
        inputRef={inputRef}
        styleState={styleState}
        keyboardVisible={keyboardVisible}
        onOpenFormat={handleOpenFormat}
      />
      <FormatSheet ref={formatSheetRef} inputRef={inputRef} styleState={styleState} />
    </View>
  )
}
