import { RichText } from "@10play/tentap-editor"
import { useRef } from "react"
import { View } from "react-native"
import { EditorHeader } from "./EditorHeader"
import { EditorToolbar } from "./EditorToolbar"
import { FormatSheet, type FormatSheetRef } from "./FormatSheet"
import { useNoteEditor } from "./useNoteEditorBridge"

interface NoteEditorProps {
  initialContent?: string
  autofocus?: boolean
  headerTitle?: string
  onBack?: () => void
  onDone?: () => void
  onSave?: (html: string, text: string) => void
}

export function NoteEditor({
  initialContent,
  autofocus = true,
  headerTitle = "Notes",
  onBack,
  onDone,
  onSave,
}: NoteEditorProps) {
  const formatSheetRef = useRef<FormatSheetRef>(null)
  const saveTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  const handleFlushSave = async () => {
    if (saveTimeoutRef.current) {
      clearTimeout(saveTimeoutRef.current)
      saveTimeoutRef.current = null
    }
    if (onSave) {
      try {
        const html = await editor.getHTML()
        const text = await editor.getText()
        onSave(html, text)
      } catch {}
    }
  }

  const handleChange = () => {
    if (!onSave) return
    if (saveTimeoutRef.current) {
      clearTimeout(saveTimeoutRef.current)
    }
    saveTimeoutRef.current = setTimeout(async () => {
      try {
        const html = await editor.getHTML()
        const text = await editor.getText()
        onSave(html, text)
      } catch {}
    }, 400)
  }

  const editor = useNoteEditor({
    initialContent,
    autofocus,
    onChange: handleChange,
  })

  const handleBack = async () => {
    await handleFlushSave()
    onBack?.()
  }

  const handleDone = async () => {
    await handleFlushSave()
    onDone?.()
  }

  return (
    <View className="flex-1 bg-background">
      <EditorHeader editor={editor} title={headerTitle} onBack={handleBack} onDone={handleDone} />
      <View className="flex-1">
        <RichText editor={editor} />
      </View>
      <EditorToolbar editor={editor} onOpenFormat={() => formatSheetRef.current?.open()} />
      <FormatSheet ref={formatSheetRef} editor={editor} />
    </View>
  )
}
