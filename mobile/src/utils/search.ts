/**
 * Sanitizes user search input and formats it for SQLite FTS5 prefix matching.
 * Strips syntax characters: ' " * ^ ( ) { } : ~ -
 */
export function buildFtsQuery(search: string): string | null {
  const tokens = search
    .trim()
    .replace(/['"*^(){}:~-]/g, " ")
    .split(/\s+/)
    .filter(Boolean)

  if (tokens.length === 0) return null

  return tokens.map((token) => `"${token}"*`).join(" ")
}
