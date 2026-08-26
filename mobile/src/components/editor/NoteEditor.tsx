import { RichText } from "@10play/tentap-editor"
import { useEffect, useState } from "react"
import { Keyboard, Platform, View } from "react-native"
import { EditorHeader } from "./EditorHeader"
import { EditorToolbar } from "./EditorToolbar"
import { FormatSheet } from "./FormatSheet"
import { useNoteEditor } from "./useNoteEditorBridge"

interface NoteEditorProps {
  initialContent?: string
  placeholder?: string
  autofocus?: boolean
  headerTitle?: string
  onBack?: () => void
  onChange?: () => void
}

export function NoteEditor({
  initialContent,
  placeholder = "Start writing...",
  autofocus = true,
  headerTitle = "Notes",
  onBack,
  onChange,
}: NoteEditorProps) {
  const [isFormatOpen, setIsFormatOpen] = useState(false)

  const editor = useNoteEditor({
    initialContent,
    placeholder,
    autofocus,
    onChange,
  })

  useEffect(() => {
    const showSub = Keyboard.addListener(
      Platform.OS === "ios" ? "keyboardWillShow" : "keyboardDidShow",
      () => {
        if (isFormatOpen) {
          setIsFormatOpen(false)
        }
      },
    )
    return () => showSub.remove()
  }, [isFormatOpen])

  return (
    <View className="flex-1 bg-background">
      <EditorHeader editor={editor} title={headerTitle} onBack={onBack} />

      <View className="flex-1">
        <RichText editor={editor} />
      </View>

      <EditorToolbar editor={editor} onOpenFormat={() => setIsFormatOpen(true)} />

      <FormatSheet editor={editor} isOpen={isFormatOpen} onClose={() => setIsFormatOpen(false)} />
    </View>
  )
}
