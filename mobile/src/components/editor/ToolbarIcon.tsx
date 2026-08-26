import { SymbolView, type SymbolViewProps } from "expo-symbols"
import {
  Bold,
  CheckSquare,
  Code,
  Heading1,
  Heading2,
  Heading3,
  HelpCircle,
  Indent,
  Italic,
  Keyboard as KeyboardIcon,
  List,
  ListOrdered,
  Outdent,
  Quote,
  Redo2,
  Strikethrough,
  Table,
  Type,
  Underline,
  Undo2,
  X,
} from "lucide-react-native"
import { Platform } from "react-native"

export type ToolbarIconName =
  | "format"
  | "checklist"
  | "bulletList"
  | "orderedList"
  | "table"
  | "bold"
  | "italic"
  | "underline"
  | "strike"
  | "code"
  | "quote"
  | "h1"
  | "h2"
  | "h3"
  | "body"
  | "indent"
  | "outdent"
  | "undo"
  | "redo"
  | "dismiss"
  | "close"

interface ToolbarIconProps {
  name: ToolbarIconName
  size?: number
  color?: string
}

const IOS_SYMBOLS: Record<ToolbarIconName, SymbolViewProps["name"]> = {
  format: "textformat.alt",
  checklist: "checklist",
  bulletList: "list.bullet",
  orderedList: "list.number",
  table: "tablecells",
  bold: "bold",
  italic: "italic",
  underline: "underline",
  strike: "strikethrough",
  code: "chevron.left.forwardslash.chevron.right",
  quote: "text.quote",
  h1: "character.textbox",
  h2: "character.textbox",
  h3: "character.textbox",
  body: "paragraph",
  indent: "increase.indent",
  outdent: "decrease.indent",
  undo: "arrow.uturn.backward",
  redo: "arrow.uturn.forward",
  dismiss: "keyboard.chevron.compact.down",
  close: "xmark",
}

export function ToolbarIcon({ name, size = 21, color = "#E6E1E9" }: ToolbarIconProps) {
  if (Platform.OS === "ios") {
    const symbolName = IOS_SYMBOLS[name]
    if (symbolName) {
      return <SymbolView name={symbolName} size={size} tintColor={color} type="monochrome" />
    }
  }

  // Android / Web / Fallback Lucide icons
  switch (name) {
    case "format":
      return <Type size={size} color={color} strokeWidth={2.2} />
    case "checklist":
      return <CheckSquare size={size} color={color} strokeWidth={2.2} />
    case "bulletList":
      return <List size={size} color={color} strokeWidth={2.2} />
    case "orderedList":
      return <ListOrdered size={size} color={color} strokeWidth={2.2} />
    case "table":
      return <Table size={size} color={color} strokeWidth={2.2} />
    case "bold":
      return <Bold size={size} color={color} strokeWidth={2.5} />
    case "italic":
      return <Italic size={size} color={color} strokeWidth={2.2} />
    case "underline":
      return <Underline size={size} color={color} strokeWidth={2.2} />
    case "strike":
      return <Strikethrough size={size} color={color} strokeWidth={2.2} />
    case "code":
      return <Code size={size} color={color} strokeWidth={2.2} />
    case "quote":
      return <Quote size={size} color={color} strokeWidth={2.2} />
    case "h1":
      return <Heading1 size={size} color={color} strokeWidth={2.2} />
    case "h2":
      return <Heading2 size={size} color={color} strokeWidth={2.2} />
    case "h3":
      return <Heading3 size={size} color={color} strokeWidth={2.2} />
    case "body":
      return <Type size={size} color={color} strokeWidth={2.2} />
    case "indent":
      return <Indent size={size} color={color} strokeWidth={2.2} />
    case "outdent":
      return <Outdent size={size} color={color} strokeWidth={2.2} />
    case "undo":
      return <Undo2 size={size} color={color} strokeWidth={2.2} />
    case "redo":
      return <Redo2 size={size} color={color} strokeWidth={2.2} />
    case "dismiss":
      return <KeyboardIcon size={size} color={color} strokeWidth={2.2} />
    case "close":
      return <X size={size} color={color} strokeWidth={2.2} />
    default:
      return <HelpCircle size={size} color={color} strokeWidth={2.2} />
  }
}
