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
    .split("\n")
    .map((l) => l.trim())
    .find((l) => l.length > 0)
  if (!firstLine) return "Untitled Note"
  return firstLine.slice(0, 60)
}
