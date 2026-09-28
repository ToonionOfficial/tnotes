import { ulid } from "@/utils/id"
import type {
  AudioData,
  Block,
  BlockKind,
  Document,
  DrawingData,
  ImageData,
  ListItem,
  Marks,
  RichText,
  Span,
  SubList,
  TableData,
  TaskItem,
} from "./model"

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value)
}

function asString(value: unknown, fallback = ""): string {
  return typeof value === "string" ? value : fallback
}

function asNumber(value: unknown, fallback = 0): number {
  return typeof value === "number" && Number.isFinite(value) ? value : fallback
}

function asBoolean(value: unknown, fallback = false): boolean {
  return typeof value === "boolean" ? value : fallback
}

function asStringArray(value: unknown): string[] {
  if (!Array.isArray(value)) return []
  return value.filter((entry): entry is string => typeof entry === "string")
}

export function normalizeMarks(value: unknown): Marks {
  const source = isRecord(value) ? value : {}
  return {
    bold: asBoolean(source.bold),
    italic: asBoolean(source.italic),
    underline: asBoolean(source.underline),
    strike: asBoolean(source.strike),
    code: asBoolean(source.code),
  }
}

export function normalizeSpan(value: unknown): Span {
  const source = isRecord(value) ? value : {}
  const link = source.link
  return {
    text: asString(source.text),
    marks: normalizeMarks(source.marks),
    ...(typeof link === "string" ? { link } : {}),
  }
}

export function normalizeRichText(value: unknown): RichText {
  const source = isRecord(value) ? value : {}
  const spans = Array.isArray(source.spans) ? source.spans.map(normalizeSpan) : []
  return { spans }
}

function normalizeSubList(value: unknown): SubList | undefined {
  if (!isRecord(value)) return undefined
  if (Array.isArray(value.Bullet)) {
    return { Bullet: value.Bullet.map(normalizeListItem) }
  }
  if (Array.isArray(value.Ordered)) {
    return { Ordered: value.Ordered.map(normalizeListItem) }
  }
  return undefined
}

export function normalizeListItem(value: unknown): ListItem {
  const source = isRecord(value) ? value : {}
  const subList = normalizeSubList(source.sub_list)
  return {
    content: normalizeRichText(source.content),
    ...(subList ? { sub_list: subList } : {}),
  }
}

export function normalizeTaskItem(value: unknown): TaskItem {
  const source = isRecord(value) ? value : {}
  return {
    checked: asBoolean(source.checked),
    content: normalizeRichText(source.content),
  }
}

function normalizeTableData(value: unknown): TableData {
  const source = isRecord(value) ? value : {}
  const rows = Array.isArray(source.rows) ? source.rows.map((row) => asStringArray(row)) : []
  return {
    headers: asStringArray(source.headers),
    rows,
  }
}

function normalizeDrawingData(value: unknown): DrawingData {
  const source = isRecord(value) ? value : {}
  const assetId = source.asset_id
  const strokes = Array.isArray(source.strokes)
    ? source.strokes.filter(isRecord).map((stroke) => ({
        color: asString(stroke.color),
        width: asNumber(stroke.width),
        points: Array.isArray(stroke.points)
          ? stroke.points.filter(isRecord).map((point) => ({
              x: asNumber(point.x),
              y: asNumber(point.y),
            }))
          : [],
      }))
    : []
  return {
    ...(typeof assetId === "string" ? { asset_id: assetId } : {}),
    strokes,
  }
}

function normalizeImageData(value: unknown): ImageData {
  const source = isRecord(value) ? value : {}
  const alt = source.alt
  const width = source.width
  const height = source.height
  return {
    asset_id: asString(source.asset_id),
    ...(typeof alt === "string" ? { alt } : {}),
    ...(typeof width === "number" ? { width } : {}),
    ...(typeof height === "number" ? { height } : {}),
  }
}

function normalizeAudioData(value: unknown): AudioData {
  const source = isRecord(value) ? value : {}
  const waveform = source.waveform
  return {
    asset_id: asString(source.asset_id),
    duration_ms: asNumber(source.duration_ms),
    ...(Array.isArray(waveform) && waveform.every((entry) => typeof entry === "number")
      ? { waveform: waveform as number[] }
      : {}),
  }
}

function normalizeBlockKind(type: unknown, data: unknown): BlockKind | undefined {
  switch (type) {
    case "Paragraph":
      return { type: "Paragraph", data: normalizeRichText(data) }
    case "Heading": {
      const source = isRecord(data) ? data : {}
      return {
        type: "Heading",
        data: {
          level: asNumber(source.level),
          content: normalizeRichText(source.content),
        },
      }
    }
    case "BulletList":
      return {
        type: "BulletList",
        data: Array.isArray(data) ? data.map(normalizeListItem) : [],
      }
    case "OrderedList":
      return {
        type: "OrderedList",
        data: Array.isArray(data) ? data.map(normalizeListItem) : [],
      }
    case "TaskList":
      return {
        type: "TaskList",
        data: Array.isArray(data) ? data.map(normalizeTaskItem) : [],
      }
    case "Quote":
      return { type: "Quote", data: normalizeRichText(data) }
    case "CodeBlock": {
      const source = isRecord(data) ? data : {}
      const language = source.language
      return {
        type: "CodeBlock",
        data: {
          ...(typeof language === "string" ? { language } : {}),
          code: asString(source.code),
        },
      }
    }
    case "Divider":
      return { type: "Divider" }
    case "Table":
      return { type: "Table", data: normalizeTableData(data) }
    case "Drawing":
      return { type: "Drawing", data: normalizeDrawingData(data) }
    case "Image":
      return { type: "Image", data: normalizeImageData(data) }
    case "Audio":
      return { type: "Audio", data: normalizeAudioData(data) }
    default:
      return undefined
  }
}

export function normalizeBlock(value: unknown): Block | undefined {
  if (!isRecord(value)) return undefined
  const kind = normalizeBlockKind(value.type, value.data)
  if (!kind) return undefined
  const id = value.id
  return {
    id: typeof id === "string" && id.length > 0 ? id : ulid(),
    ...kind,
  } as Block
}

export function parseDocument(value: unknown): Document | undefined {
  if (typeof value === "string") {
    if (value.trim().length === 0) return { version: 1, blocks: [] }
    try {
      return parseDocument(JSON.parse(value))
    } catch {
      return undefined
    }
  }
  if (!isRecord(value)) return undefined
  const version = value.version
  const blocks = value.blocks
  if (!Array.isArray(blocks)) return undefined
  const normalized: Block[] = []
  for (const entry of blocks) {
    const block = normalizeBlock(entry)
    if (!block) return undefined
    normalized.push(block)
  }
  return {
    version: typeof version === "number" ? version : 1,
    blocks: normalized,
  }
}

export function documentToJson(doc: Document): string {
  return JSON.stringify({ version: doc.version, blocks: doc.blocks })
}

export function emptyDocument(): Document {
  return { version: 1, blocks: [] }
}

export function paragraphDocument(text: string): Document {
  return {
    version: 1,
    blocks: [
      {
        id: ulid(),
        type: "Paragraph",
        data: { spans: [{ text, marks: normalizeMarks(undefined) }] },
      },
    ],
  }
}
