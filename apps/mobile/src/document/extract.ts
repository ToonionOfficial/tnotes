import type { Block, Document, ListItem, RichText } from "./model"

function pushRichText(content: RichText, buffer: string[]): void {
  const text = content.spans
    .map((span) => span.text)
    .join("")
    .trim()
  if (text.length > 0) {
    buffer.push(text)
  }
}

function pushListItem(item: ListItem, buffer: string[]): void {
  pushRichText(item.content, buffer)
  const subList = item.sub_list
  if (!subList) return
  const nested = "Bullet" in subList ? subList.Bullet : subList.Ordered
  for (const nestedItem of nested) {
    pushListItem(nestedItem, buffer)
  }
}

export function extractText(doc: Document): string {
  const buffer: string[] = []
  for (const block of doc.blocks) {
    pushBlock(block, buffer)
  }
  return buffer.join(" ")
}

export function firstContentText(doc: Document): string {
  for (const block of doc.blocks) {
    const buffer: string[] = []
    pushBlock(block, buffer)
    const first = buffer.find((text) => text.length > 0)
    if (first !== undefined) return first
  }
  return ""
}

function pushBlock(block: Block, buffer: string[]): void {
  switch (block.type) {
    case "Paragraph":
    case "Quote":
      pushRichText(block.data, buffer)
      break
    case "Heading":
      pushRichText(block.data.content, buffer)
      break
    case "BulletList":
    case "OrderedList":
      for (const item of block.data) {
        pushListItem(item, buffer)
      }
      break
    case "TaskList":
      for (const item of block.data) {
        pushRichText(item.content, buffer)
      }
      break
    case "CodeBlock": {
      const code = block.data.code.trim()
      if (code.length > 0) {
        buffer.push(code)
      }
      break
    }
    case "Table":
      for (const header of block.data.headers) {
        const cell = header.trim()
        if (cell.length > 0) {
          buffer.push(cell)
        }
      }
      for (const row of block.data.rows) {
        for (const cell of row) {
          const trimmed = cell.trim()
          if (trimmed.length > 0) {
            buffer.push(trimmed)
          }
        }
      }
      break
    case "Image": {
      const alt = block.data.alt?.trim()
      if (alt && alt.length > 0) {
        buffer.push(alt)
      }
      break
    }
    case "Divider":
    case "Drawing":
    case "Audio":
      break
  }
}
