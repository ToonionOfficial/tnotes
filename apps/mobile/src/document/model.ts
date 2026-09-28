export interface Marks {
  bold: boolean
  italic: boolean
  underline: boolean
  strike: boolean
  code: boolean
}

export interface Span {
  text: string
  marks: Marks
  link?: string
}

export interface RichText {
  spans: Span[]
}

export type SubList = { Bullet: ListItem[] } | { Ordered: ListItem[] }

export interface ListItem {
  content: RichText
  sub_list?: SubList
}

export interface TaskItem {
  checked: boolean
  content: RichText
}

export interface TableData {
  headers: string[]
  rows: string[][]
}

export interface Point {
  x: number
  y: number
}

export interface Stroke {
  color: string
  width: number
  points: Point[]
}

export interface DrawingData {
  asset_id?: string
  strokes: Stroke[]
}

export interface ImageData {
  asset_id: string
  alt?: string
  width?: number
  height?: number
}

export interface AudioData {
  asset_id: string
  duration_ms: number
  waveform?: number[]
}

export type BlockKind =
  | { type: "Paragraph"; data: RichText }
  | { type: "Heading"; data: { level: number; content: RichText } }
  | { type: "BulletList"; data: ListItem[] }
  | { type: "OrderedList"; data: ListItem[] }
  | { type: "TaskList"; data: TaskItem[] }
  | { type: "Quote"; data: RichText }
  | { type: "CodeBlock"; data: { language?: string; code: string } }
  | { type: "Divider" }
  | { type: "Table"; data: TableData }
  | { type: "Drawing"; data: DrawingData }
  | { type: "Image"; data: ImageData }
  | { type: "Audio"; data: AudioData }

export type Block = { id: string } & BlockKind

export interface Document {
  version: number
  blocks: Block[]
}
