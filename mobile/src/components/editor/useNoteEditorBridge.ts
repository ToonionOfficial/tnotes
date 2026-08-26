import { TenTapStartKit, useEditorBridge } from "@10play/tentap-editor"
import { noteEditorTheme, ThemeBridge } from "./editorTheme"

export interface UseNoteEditorOptions {
  initialContent?: string
  autofocus?: boolean
  editable?: boolean
  onChange?: () => void
}

export function useNoteEditor({
  initialContent,
  autofocus = true,
  editable = true,
  onChange,
}: UseNoteEditorOptions = {}) {
  const extensions = TenTapStartKit.filter((ext) => ext.name !== "placeholder")

  const editor = useEditorBridge({
    bridgeExtensions: [...extensions, ThemeBridge],
    initialContent,
    autofocus,
    avoidIosKeyboard: true,
    editable,
    theme: noteEditorTheme,
    onChange,
  })

  return editor
}
