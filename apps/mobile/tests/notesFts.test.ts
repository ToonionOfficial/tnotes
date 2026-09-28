import { readFileSync } from "node:fs"
import { dirname, join } from "node:path"
import { DatabaseSync } from "node:sqlite"
import { fileURLToPath } from "node:url"
import { describe, expect, it } from "vitest"

const drizzleDir = join(dirname(fileURLToPath(import.meta.url)), "..", "drizzle")

function migrationStatements(name: string): string[] {
  const sql = readFileSync(join(drizzleDir, name, "migration.sql"), "utf8")
  return sql
    .split("--> statement-breakpoint")
    .map((statement) => statement.trim().replace(/;$/, ""))
    .filter((statement) => statement.length > 0)
}

function hits(db: DatabaseSync, term: string): number {
  const rows = db
    .prepare("SELECT COUNT(*) AS count FROM notes_fts WHERE notes_fts MATCH ?")
    .all(term)
  return Number((rows[0] as { count: number })?.count ?? 0)
}

function searchableDb(): DatabaseSync {
  const db = new DatabaseSync(":memory:")
  db.exec(
    `CREATE TABLE notes (
      id TEXT PRIMARY KEY,
      user_id TEXT NOT NULL,
      folder_id TEXT,
      title TEXT NOT NULL DEFAULT '',
      body TEXT NOT NULL DEFAULT '',
      searchable_text TEXT NOT NULL DEFAULT '',
      pinned INTEGER NOT NULL DEFAULT 0,
      trashed INTEGER NOT NULL DEFAULT 0,
      version INTEGER NOT NULL DEFAULT 1,
      updated_at INTEGER NOT NULL,
      created_at INTEGER NOT NULL,
      deleted_at INTEGER,
      device_id TEXT NOT NULL,
      checksum TEXT NOT NULL
    )`,
  )
  for (const statement of migrationStatements("20260928074603_notes_fts_searchable")) {
    db.exec(statement)
  }
  return db
}

const CANONICAL_BODY = JSON.stringify({
  version: 1,
  blocks: [
    {
      id: "01J0000000000000000000001",
      type: "Paragraph",
      data: { spans: [{ text: "Discussed roadmap." }] },
    },
  ],
})

describe("searchable_text column migration", () => {
  it("adds the column with an empty default to pre-migration rows", () => {
    const db = new DatabaseSync(":memory:")
    db.exec(
      "CREATE TABLE notes (id TEXT PRIMARY KEY, title TEXT NOT NULL DEFAULT '', body TEXT NOT NULL DEFAULT '')",
    )
    db.prepare("INSERT INTO notes (id, title, body) VALUES (?, ?, ?)").run(
      "n1",
      "Old",
      "<p>legacy</p>",
    )
    for (const statement of migrationStatements("20260928074602_searchable_text")) {
      db.exec(statement)
    }
    const row = db.prepare("SELECT searchable_text FROM notes WHERE id = ?").get("n1") as {
      searchable_text: string
    }
    expect(row.searchable_text).toBe("")
    db.close()
  })
})

describe("notes_fts isolation on (title, searchable_text)", () => {
  it("finds visible text but not canonical JSON keys", () => {
    const db = searchableDb()
    db.prepare(
      "INSERT INTO notes (id, user_id, title, body, searchable_text, updated_at, created_at, device_id, checksum) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    ).run("n1", "u1", "Meeting Notes", CANONICAL_BODY, "Discussed roadmap", 1, 1, "d1", "c1")

    expect(hits(db, '"Discussed"*')).toBe(1)
    expect(hits(db, '"roadmap"*')).toBe(1)
    expect(hits(db, '"Meeting"*')).toBe(1)
    for (const token of ["Paragraph", "version", "blocks", "spans", "type", "data"]) {
      expect(hits(db, `"${token}"`)).toBe(0)
    }
    db.close()
  })

  it("does not index raw body HTML noise", () => {
    const db = searchableDb()
    db.prepare(
      "INSERT INTO notes (id, user_id, title, body, searchable_text, updated_at, created_at, device_id, checksum) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    ).run("n1", "u1", "Legacy", "<p>hello</p>", "hello", 1, 1, "d1", "c1")

    expect(hits(db, '"hello"*')).toBe(1)
    expect(hits(db, '"Legacy"*')).toBe(1)
    db.close()
  })

  it("keeps the index in sync on update and delete", () => {
    const db = searchableDb()
    db.prepare(
      "INSERT INTO notes (id, user_id, title, body, searchable_text, updated_at, created_at, device_id, checksum) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    ).run("n1", "u1", "Draft", "{}", "first text", 1, 1, "d1", "c1")
    expect(hits(db, '"first"*')).toBe(1)

    db.prepare("UPDATE notes SET searchable_text = ? WHERE id = ?").run("second text", "n1")
    expect(hits(db, '"first"*')).toBe(0)
    expect(hits(db, '"second"*')).toBe(1)

    db.prepare("DELETE FROM notes WHERE id = ?").run("n1")
    expect(hits(db, '"second"*')).toBe(0)
    db.close()
  })

  it("snippets render from the searchable_text column", () => {
    const db = searchableDb()
    db.prepare(
      "INSERT INTO notes (id, user_id, title, body, searchable_text, updated_at, created_at, device_id, checksum) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    ).run("n1", "u1", "T", "{}", "Discussed roadmap today", 1, 1, "d1", "c1")
    const rows = db
      .prepare(
        "SELECT snippet(notes_fts, 1, '<mark>', '</mark>', '...', 20) AS snippet FROM notes_fts WHERE notes_fts MATCH ?",
      )
      .all('"roadmap"*') as Array<{ snippet: string }>
    expect(rows).toHaveLength(1)
    expect(rows[0]?.snippet).toContain("<mark>roadmap</mark>")
    db.close()
  })
})
