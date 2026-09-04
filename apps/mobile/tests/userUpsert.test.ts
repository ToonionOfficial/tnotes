import { DatabaseSync } from "node:sqlite"
import { eq } from "drizzle-orm"
import { drizzle } from "drizzle-orm/sqlite-proxy"
import { beforeEach, describe, expect, it, vi } from "vitest"
import { syncMeta, users } from "../src/db/schema"
import { dumpTable, fakeDb, resetFakeDb } from "./fakes/inMemoryDb"

vi.mock("../src/db/index", () => ({ db: fakeDb, expo: {} }))

import { pairWithServerAsync } from "../src/db/queries/pairing"
import { ensureUser, getSyncMeta, setSyncMeta, upsertUser } from "../src/db/queries/sync"

const proxyDb = drizzle(async () => ({ rows: [] }))

function exec(raw: DatabaseSync, sql: string, params: readonly unknown[]): void {
  raw.prepare(sql).run(...(params as (string | number | null)[]))
}

function memDb(): DatabaseSync {
  const raw = new DatabaseSync(":memory:")
  raw.exec(
    "CREATE TABLE users (id text PRIMARY KEY, username text NOT NULL UNIQUE, created_at integer NOT NULL)",
  )
  raw.exec("CREATE TABLE sync_meta (key text PRIMARY KEY, value text NOT NULL)")
  return raw
}

describe("pairing failure: UNIQUE(username) vs ON CONFLICT(id)", () => {
  it("reproduces the reported crash: stale same-username row + ON CONFLICT(id) insert", () => {
    const raw = memDb()
    // Orphaned row from an earlier pairing (e.g. server DB was reset and
    // re-issued ids while the device kept the old row).
    raw.prepare("INSERT INTO users VALUES (?, ?, ?)").run("OLD-ULID-111", "alh1", 111)

    const failed = proxyDb
      .insert(users)
      .values({ id: "01M1PE2D4GS48FMHVG", username: "alh1", createdAt: 1788533110067 })
      .onConflictDoUpdate({ target: users.id, set: { username: "alh1" } })
      .toSQL()

    expect(failed.sql).toContain('insert into "users"')
    expect(() => exec(raw, failed.sql, failed.params)).toThrow(
      /UNIQUE constraint failed: users\.username/,
    )
    raw.close()
  })

  it("stale-delete + plain insert (the fix) succeeds on real SQLite", () => {
    const raw = memDb()
    raw.prepare("INSERT INTO users VALUES (?, ?, ?)").run("OLD-ULID-111", "alh1", 111)

    const del = proxyDb.delete(users).where(eq(users.id, "OLD-ULID-111")).toSQL()
    exec(raw, del.sql, del.params)
    const insert = proxyDb
      .insert(users)
      .values({ id: "01M1PE2D4GS48FMHVG", username: "alh1", createdAt: 1788533110067 })
      .toSQL()
    exec(raw, insert.sql, insert.params)

    const rows = raw.prepare('SELECT "id", "username" FROM "users"').all()
    expect(rows).toEqual([{ id: "01M1PE2D4GS48FMHVG", username: "alh1" }])
    raw.close()
  })

  it("setSyncMeta statements (insert DO NOTHING + update) round-trip on real SQLite", () => {
    const raw = memDb()

    for (const value of ["http://192.168.1.50:8787", "http://10.0.0.9:8787"]) {
      const insert = proxyDb
        .insert(syncMeta)
        .values({ key: "server_url", value })
        .onConflictDoNothing()
        .toSQL()
      // No table-qualified conflict target: portable across SQLite versions.
      expect(insert.sql).toBe(
        'insert into "sync_meta" ("key", "value") values (?, ?) on conflict do nothing',
      )
      exec(raw, insert.sql, insert.params)
      const update = proxyDb
        .update(syncMeta)
        .set({ value })
        .where(eq(syncMeta.key, "server_url"))
        .toSQL()
      exec(raw, update.sql, update.params)
    }

    expect(
      raw.prepare('SELECT "value" FROM "sync_meta" WHERE "key" = ?').get("server_url"),
    ).toEqual({ value: "http://10.0.0.9:8787" })
    raw.close()
  })
})

describe("upsertUser / ensureUser / setSyncMeta", () => {
  beforeEach(() => {
    resetFakeDb()
  })

  it("ensureUser inserts the default user when missing", () => {
    expect(ensureUser()).toBe("default_user")
    expect(dumpTable(users)).toHaveLength(1)
    expect(dumpTable(users)[0]).toMatchObject({ id: "default_user", username: "Local User" })
  })

  it("ensureUser refreshes the username for a known id instead of inserting", () => {
    ensureUser("u1", "Ada")
    ensureUser("u1", "Grace")
    expect(dumpTable(users)).toHaveLength(1)
    expect(dumpTable(users)[0]).toMatchObject({ id: "u1", username: "Grace" })
  })

  it("upsertUser replaces a stale same-username row instead of crashing", () => {
    upsertUser("stale-ulid", "alh1")
    expect(() => upsertUser("01M1PE2D4GS48FMHVG", "alh1")).not.toThrow()
    expect(dumpTable(users)).toEqual([
      expect.objectContaining({ id: "01M1PE2D4GS48FMHVG", username: "alh1" }),
    ])
  })

  it("the fake enforces UNIQUE(username) like real SQLite (pins test fidelity)", () => {
    upsertUser("stale-ulid", "alh1")
    expect(() =>
      fakeDb
        .insert(users)
        .values({ id: "01M1PE2D4GS48FMHVG", username: "alh1", createdAt: Date.now() })
        .run(),
    ).toThrow(/UNIQUE constraint failed/)
  })

  it("setSyncMeta inserts then overwrites the same key", () => {
    setSyncMeta("server_url", "http://a")
    expect(getSyncMeta("server_url")).toBe("http://a")
    setSyncMeta("server_url", "http://b")
    expect(getSyncMeta("server_url")).toBe("http://b")
    expect(dumpTable(syncMeta)).toHaveLength(1)
  })
})

describe("pairWithServerAsync", () => {
  const meResponse = (user_id: string, username: string) =>
    ({
      ok: true,
      json: async () => ({ user_id, username }),
      text: async () => "",
    }) as unknown as Response

  beforeEach(() => {
    resetFakeDb()
    vi.unstubAllGlobals()
  })

  it("pairs via /api/me and persists user + sync metadata", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => meResponse("01M1PE2D4GS48FMHVG", "alh1")),
    )

    const status = await pairWithServerAsync({ url: "http://192.168.1.50:8787/", token: "tok_abc" })

    expect(status.isConnected).toBe(true)
    expect(status.serverUrl).toBe("http://192.168.1.50:8787")
    expect(status.userId).toBe("01M1PE2D4GS48FMHVG")
    expect(dumpTable(users)).toEqual([
      expect.objectContaining({ id: "01M1PE2D4GS48FMHVG", username: "alh1" }),
    ])
    expect(getSyncMeta("auth_token")).toBe("tok_abc")
  })

  it("re-pairing the same account updates the username without duplicating rows", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => meResponse("uid-1", "alh1")),
    )
    await pairWithServerAsync({ url: "http://x:8787", token: "tok_1" })

    vi.stubGlobal(
      "fetch",
      vi.fn(async () => meResponse("uid-1", "renamed")),
    )
    await pairWithServerAsync({ url: "http://x:8787", token: "tok_2" })

    expect(dumpTable(users)).toEqual([
      expect.objectContaining({ id: "uid-1", username: "renamed" }),
    ])
  })

  it("pairs over a stale same-username row (the reported QR pairing error)", async () => {
    upsertUser("OLD-ULID-111", "alh1")

    vi.stubGlobal(
      "fetch",
      vi.fn(async () => meResponse("01M1PE2D4GS48FMHVG", "alh1")),
    )
    const status = await pairWithServerAsync({ url: "http://x:8787", token: "tok_new" })

    expect(status.userId).toBe("01M1PE2D4GS48FMHVG")
    expect(dumpTable(users)).toEqual([
      expect.objectContaining({ id: "01M1PE2D4GS48FMHVG", username: "alh1" }),
    ])
  })
})
