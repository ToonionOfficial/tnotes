import { extractText, parseDocument } from "@/document"

/**
 * Strips HTML tags from rich text to produce clean preview strings.
 */
export function stripHtml(html: string): string {
  return html
    .replace(/<[^>]*>/g, " ")
    .replace(/\s+/g, " ")
    .trim()
}

/**
 * Extracts a concise title from plain text.
 */
export function extractTitle(plainText: string): string {
  const firstLine = plainText
    .replace(/\r/g, "")
    .split("\n")
    .map((l) => l.trim())
    .find((l) => l.length > 0)
  if (!firstLine) return "Untitled Note"
  return firstLine.replace(/\s+/g, " ").slice(0, 60)
}

export function firstLineTitle(title: string): string {
  const firstLine = title.replace(/\r/g, "").split("\n")[0] ?? ""
  return firstLine.replace(/\s+/g, " ").trim()
}

export function displayTitle(title: string): string {
  const firstLine = firstLineTitle(title)
  if (!firstLine) return "Untitled Note"
  return firstLine
}

export function notePreviewText(note: {
  snippet?: string | null
  searchableText?: string
  body: string
}): string {
  if (note.snippet) return stripHtml(note.snippet)
  if (note.searchableText?.trim()) return note.searchableText.trim()
  const doc = parseDocument(note.body)
  if (doc) return extractText(doc)
  return stripHtml(note.body)
}
