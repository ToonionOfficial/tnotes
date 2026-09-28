import { ulid } from "@/utils/id"
import type {
  Block,
  BlockKind,
  Document,
  ListItem,
  RichText,
  Span,
  SubList,
  TaskItem,
} from "./model"

function serializeSpan(span: Span): string {
  if (span.text.length === 0) return ""
  const trimmed = span.text.trim()
  if (trimmed.length === 0) return span.text
  const leading = span.text.slice(0, span.text.length - span.text.trimStart().length)
  const trailing = span.text.slice(span.text.trimEnd().length)
  const marks = span.marks
  let inner: string
  if (marks.code) {
    inner = `\`${trimmed}\``
  } else {
    let open = ""
    let close = ""
    if (marks.underline) {
      open += "<u>"
      close = `</u>${close}`
    }
    if (marks.strike) {
      open += "~~"
      close = `~~${close}`
    }
    if (marks.bold && marks.italic) {
      open += "***"
      close = `***${close}`
    } else if (marks.bold) {
      open += "**"
      close = `**${close}`
    } else if (marks.italic) {
      open += "*"
      close = `*${close}`
    }
    inner = `${open}${trimmed}${close}`
  }
  const linked = span.link ? `[${inner}](${span.link})` : inner
  return `${leading}${linked}${trailing}`
}

function serializeRichText(content: RichText): string {
  return content.spans.map(serializeSpan).join("")
}

function serializeListItem(
  item: ListItem,
  prefix: string,
  indentLevel: number,
  out: string[],
): void {
  out.push(`${"  ".repeat(indentLevel)}${prefix}${serializeRichText(item.content)}`)
  if (!item.sub_list) return
  const nested = "Bullet" in item.sub_list ? item.sub_list.Bullet : item.sub_list.Ordered
  const ordered = "Ordered" in item.sub_list
  nested.forEach((subItem, index) => {
    serializeListItem(subItem, ordered ? `${index + 1}. ` : "- ", indentLevel + 1, out)
  })
}

function serializeBlock(block: Block, out: string[]): void {
  switch (block.type) {
    case "Paragraph":
      out.push(serializeRichText(block.data))
      break
    case "Heading": {
      const level = Math.min(6, Math.max(1, block.data.level))
      out.push(`${"#".repeat(level)} ${serializeRichText(block.data.content)}`)
      break
    }
    case "Quote": {
      const text = serializeRichText(block.data)
      const lines = text.split("\n")
      if (lines.length === 0 || (lines.length === 1 && lines[0] === "")) {
        out.push(">")
      } else {
        for (const line of lines) out.push(`> ${line}`)
      }
      break
    }
    case "CodeBlock": {
      out.push(`\`\`\`${block.data.language ?? ""}`)
      out.push(block.data.code)
      out.push("```")
      break
    }
    case "Divider":
      out.push("---")
      break
    case "BulletList":
      for (const item of block.data) serializeListItem(item, "- ", 0, out)
      break
    case "OrderedList":
      block.data.forEach((item, index) => {
        serializeListItem(item, `${index + 1}. `, 0, out)
      })
      break
    case "TaskList":
      for (const item of block.data) {
        out.push(`- ${item.checked ? "[x]" : "[ ]"} ${serializeRichText(item.content)}`)
      }
      break
    case "Table": {
      const table = block.data
      const colCount = Math.max(table.headers.length, table.rows[0]?.length ?? 0)
      if (colCount === 0) break
      const cells = (row: string[]): string =>
        `|${Array.from({ length: colCount }, (_, i) => ` ${row[i] ?? ""} |`).join("")}`
      out.push(cells(table.headers))
      out.push(`|${Array.from({ length: colCount }, () => " --- |").join("")}`)
      for (const row of table.rows) out.push(cells(row))
      break
    }
    case "Image": {
      const url = block.data.asset_id.includes("://")
        ? block.data.asset_id
        : `asset://${block.data.asset_id}`
      out.push(`![${block.data.alt ?? ""}](${url})`)
      break
    }
    case "Drawing":
      out.push(
        block.data.asset_id
          ? `<!-- tnotes:drawing asset_id="${block.data.asset_id}" -->`
          : "<!-- tnotes:drawing -->",
      )
      break
    case "Audio":
      out.push(
        `<!-- tnotes:audio asset_id="${block.data.asset_id}" duration_ms="${block.data.duration_ms}" -->`,
      )
      break
  }
}

export function documentToMarkdown(doc: Document): string {
  const out: string[] = []
  doc.blocks.forEach((block, index) => {
    if (index > 0) out.push("")
    serializeBlock(block, out)
  })
  return out.join("\n")
}

interface InlineMark {
  bold: boolean
  italic: boolean
  underline: boolean
  strike: boolean
  code: boolean
}

function blankMarks(): InlineMark {
  return { bold: false, italic: false, underline: false, strike: false, code: false }
}

function parseSpans(text: string): Span[] {
  const spans: Span[] = []
  const push = (spanText: string, marks: InlineMark, link?: string): void => {
    if (spanText.length === 0) return
    const last = spans[spans.length - 1]
    if (
      last &&
      last.link === link &&
      last.marks.bold === marks.bold &&
      last.marks.italic === marks.italic &&
      last.marks.underline === marks.underline &&
      last.marks.strike === marks.strike &&
      last.marks.code === marks.code
    ) {
      last.text += spanText
    } else {
      spans.push({ text: spanText, marks: { ...marks }, ...(link ? { link } : {}) })
    }
  }

  innerParse(text, blankMarks(), undefined, push)
  return spans
}

function innerParse(
  input: string,
  marks: InlineMark,
  link: string | undefined,
  push: (text: string, marks: InlineMark, link?: string) => void,
): void {
  let i = 0
  let plain = ""
  const flush = (): void => {
    if (plain.length > 0) {
      push(plain, marks, link)
      plain = ""
    }
  }
  while (i < input.length) {
    const rest = input.slice(i)
    if (!marks.code && rest.startsWith("`")) {
      const end = input.indexOf("`", i + 1)
      if (end !== -1) {
        flush()
        push(input.slice(i + 1, end), { ...marks, code: true }, link)
        i = end + 1
        continue
      }
    }
    if (!marks.code && rest.startsWith("<u>")) {
      const end = input.indexOf("</u>", i + 3)
      if (end !== -1) {
        flush()
        innerParse(input.slice(i + 3, end), { ...marks, underline: true }, link, push)
        i = end + 4
        continue
      }
    }
    if (!marks.code && rest.startsWith("~~")) {
      const end = input.indexOf("~~", i + 2)
      if (end !== -1) {
        flush()
        innerParse(input.slice(i + 2, end), { ...marks, strike: true }, link, push)
        i = end + 2
        continue
      }
    }
    if (
      !marks.code &&
      (rest.startsWith("***") || rest.startsWith("**") || rest[0] === "*" || rest[0] === "_")
    ) {
      const marker = rest.startsWith("***")
        ? "***"
        : rest.startsWith("**")
          ? "**"
          : (rest[0] as string)
      const end = input.indexOf(marker, i + marker.length)
      if (end !== -1) {
        flush()
        const next = { ...marks }
        if (marker === "***") {
          next.bold = true
          next.italic = true
        } else if (marker === "**") {
          next.bold = true
        } else {
          next.italic = true
        }
        innerParse(input.slice(i + marker.length, end), next, link, push)
        i = end + marker.length
        continue
      }
    }
    if (!marks.code && rest[0] === "[") {
      const linkMatch = /^\[([^\]]+)\]\(([^)\s]+)\)/.exec(rest)
      if (linkMatch && !link) {
        flush()
        innerParse(linkMatch[1] ?? "", marks, linkMatch[2], push)
        i += linkMatch[0].length
        continue
      }
    }
    plain += input[i]
    i += 1
  }
  flush()
}

function richText(text: string): RichText {
  return { spans: parseSpans(text) }
}

function makeBlock(kind: BlockKind): Block {
  return { id: ulid(), ...kind } as Block
}

function splitTableRow(line: string): string[] {
  let trimmed = line.trim()
  if (trimmed.startsWith("|")) trimmed = trimmed.slice(1)
  if (trimmed.endsWith("|")) trimmed = trimmed.slice(0, -1)
  return trimmed.split("|").map((cell) => cell.trim())
}

function isDividerLine(line: string): boolean {
  return /^(-{3,}|\*{3,}|_{3,})\s*$/.test(line.trim())
}

function parseDirective(line: string): Block | undefined {
  const match = /^<!--\s*tnotes:(\w+)(.*?)\s*-->$/.exec(line.trim())
  if (!match) return undefined
  const attrs = match[2] ?? ""
  const attr = (name: string): string | undefined => {
    const found = new RegExp(`${name}="([^"]*)"`).exec(attrs)
    return found?.[1]
  }
  if (match[1] === "drawing") {
    const assetId = attr("asset_id")
    return makeBlock({
      type: "Drawing",
      data: { ...(assetId ? { asset_id: assetId } : {}), strokes: [] },
    })
  }
  if (match[1] === "audio") {
    return makeBlock({
      type: "Audio",
      data: {
        asset_id: attr("asset_id") ?? "",
        duration_ms: Number(attr("duration_ms") ?? 0),
      },
    })
  }
  return undefined
}

interface ParsedListItem {
  content: string
  ordered: boolean
  taskChecked: boolean | undefined
  children: ParsedListItem[]
}

function parseListLine(
  line: string,
):
  | { indent: number; content: string; ordered: boolean; taskChecked: boolean | undefined }
  | undefined {
  const match = /^(\s*)([-*]|\d+\.)\s+(.*)$/.exec(line)
  if (!match) return undefined
  const indent = Math.floor((match[1] ?? "").replace(/\t/g, "  ").length / 2)
  const ordered = /^\d+\.$/.test(match[2] ?? "")
  let content = match[3] ?? ""
  let taskChecked: boolean | undefined
  const taskMatch = /^\[([ xX])\]\s+(.*)$/.exec(content)
  if (taskMatch && !ordered) {
    taskChecked = taskMatch[1]?.toLowerCase() === "x"
    content = taskMatch[2] ?? ""
  }
  return { indent, content, ordered, taskChecked }
}

function buildListItems(parsed: ParsedListItem[]): { items: ListItem[]; hasTask: boolean } {
  const items: ListItem[] = parsed.map((entry) => ({
    content: richText(entry.content),
  }))
  let hasTask = false
  parsed.forEach((entry, index) => {
    if (entry.taskChecked !== undefined) hasTask = true
    if (entry.children.length > 0) {
      const built = buildListItems(entry.children)
      if (built.hasTask) hasTask = true
      const target = items[index]
      if (target) {
        target.sub_list = (
          entry.children[0]?.ordered ? { Ordered: built.items } : { Bullet: built.items }
        ) as SubList
      }
    }
  })
  return { items, hasTask }
}

function nestListItems(flat: Array<{ indent: number; entry: ParsedListItem }>): ParsedListItem[] {
  const root: ParsedListItem[] = []
  const stack: Array<{ indent: number; entry: ParsedListItem }> = []
  for (const { indent, entry } of flat) {
    while (stack.length > 0 && (stack[stack.length - 1]?.indent ?? 0) >= indent) {
      stack.pop()
    }
    const parent = stack[stack.length - 1]?.entry
    if (parent && indent > 0) {
      parent.children.push(entry)
    } else {
      root.push(entry)
    }
    stack.push({ indent, entry })
  }
  return root
}

export function markdownToDocument(markdown: string): Document {
  const blocks: Block[] = []
  const lines = markdown.replace(/\r\n/g, "\n").split("\n")
  let i = 0
  let paragraph: string[] = []
  const flushParagraph = (): void => {
    const text = paragraph.join("\n").trim()
    paragraph = []
    if (text.length > 0) {
      blocks.push(makeBlock({ type: "Paragraph", data: richText(text) }))
    }
  }

  while (i < lines.length) {
    const line = lines[i] ?? ""
    const trimmed = line.trim()

    if (trimmed.length === 0) {
      flushParagraph()
      i += 1
      continue
    }

    if (trimmed.startsWith("```")) {
      flushParagraph()
      const language = trimmed.slice(3).trim() || undefined
      const code: string[] = []
      i += 1
      while (i < lines.length && !(lines[i] ?? "").trim().startsWith("```")) {
        code.push(lines[i] ?? "")
        i += 1
      }
      i += 1
      blocks.push(
        makeBlock({
          type: "CodeBlock",
          data: { ...(language ? { language } : {}), code: code.join("\n") },
        }),
      )
      continue
    }

    const directive = parseDirective(line)
    if (directive) {
      flushParagraph()
      blocks.push(directive)
      i += 1
      continue
    }

    if (isDividerLine(line)) {
      flushParagraph()
      blocks.push(makeBlock({ type: "Divider" }))
      i += 1
      continue
    }

    const headingMatch = /^(#{1,6})\s+(.*)$/.exec(trimmed)
    if (headingMatch) {
      flushParagraph()
      blocks.push(
        makeBlock({
          type: "Heading",
          data: {
            level: (headingMatch[1] ?? "#").length,
            content: richText(headingMatch[2] ?? ""),
          },
        }),
      )
      i += 1
      continue
    }

    if (trimmed.startsWith(">")) {
      flushParagraph()
      const quote: string[] = []
      while (i < lines.length && (lines[i] ?? "").trim().startsWith(">")) {
        quote.push((lines[i] ?? "").trim().replace(/^>\s?/, ""))
        i += 1
      }
      blocks.push(makeBlock({ type: "Quote", data: richText(quote.join("\n")) }))
      continue
    }

    if (trimmed.startsWith("|") && i + 1 < lines.length) {
      const delimiter = (lines[i + 1] ?? "").trim()
      if (/^\|?[\s:|-]+\|?[\s:|-]*$/.test(delimiter) && delimiter.includes("-")) {
        flushParagraph()
        const headers = splitTableRow(trimmed)
        const rows: string[][] = []
        i += 2
        while (i < lines.length && (lines[i] ?? "").trim().includes("|")) {
          rows.push(splitTableRow((lines[i] ?? "").trim()))
          i += 1
        }
        blocks.push(makeBlock({ type: "Table", data: { headers, rows } }))
        continue
      }
    }

    const imageMatch = /^!\[([^\]]*)\]\(([^)\s]+)\)\s*$/.exec(trimmed)
    if (imageMatch) {
      flushParagraph()
      const url = imageMatch[2] ?? ""
      blocks.push(
        makeBlock({
          type: "Image",
          data: {
            asset_id: url.startsWith("asset://") ? url.slice("asset://".length) : url,
            ...(imageMatch[1] ? { alt: imageMatch[1] } : {}),
          },
        }),
      )
      i += 1
      continue
    }

    const listLine = parseListLine(line)
    if (listLine) {
      flushParagraph()
      const flat: Array<{ indent: number; entry: ParsedListItem }> = []
      while (i < lines.length) {
        const next = parseListLine(lines[i] ?? "")
        if (!next) break
        flat.push({
          indent: next.indent,
          entry: {
            content: next.content,
            ordered: next.ordered,
            taskChecked: next.taskChecked,
            children: [],
          },
        })
        i += 1
      }
      const nested = nestListItems(flat)
      const { items, hasTask } = buildListItems(nested)
      if (hasTask) {
        const tasks: TaskItem[] = []
        const collect = (entries: ParsedListItem[]): void => {
          for (const entry of entries) {
            tasks.push({ checked: entry.taskChecked ?? false, content: richText(entry.content) })
            collect(entry.children)
          }
        }
        collect(nested)
        blocks.push(makeBlock({ type: "TaskList", data: tasks }))
      } else if (nested[0]?.ordered) {
        blocks.push(makeBlock({ type: "OrderedList", data: items }))
      } else {
        blocks.push(makeBlock({ type: "BulletList", data: items }))
      }
      continue
    }

    paragraph.push(line)
    i += 1
  }
  flushParagraph()

  return { version: 1, blocks }
}
