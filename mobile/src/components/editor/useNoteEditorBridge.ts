import { PlaceholderBridge, TenTapStartKit, useEditorBridge } from "@10play/tentap-editor"
import { noteEditorTheme, ThemeBridge } from "./editorTheme"

export interface UseNoteEditorOptions {
  initialContent?: string
  placeholder?: string
  autofocus?: boolean
  editable?: boolean
  onChange?: () => void
}

export function useNoteEditor({
  initialContent,
  placeholder = "Start writing...",
  autofocus = true,
  editable = true,
  onChange,
}: UseNoteEditorOptions = {}) {
  const editor = useEditorBridge({
    bridgeExtensions: [
      ...TenTapStartKit,
      PlaceholderBridge.configureExtension({ placeholder }),
      ThemeBridge,
    ],
    initialContent,
    autofocus,
    avoidIosKeyboard: true,
    editable,
    theme: noteEditorTheme,
    onChange,
  })

  return editor
}
