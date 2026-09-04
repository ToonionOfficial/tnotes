import { Column, getTableColumns, getTableName, Param, type SQL } from "drizzle-orm"

export type Row = Record<string, unknown>

const tables = new Map<object, Row[]>()

export function resetFakeDb(): void {
  tables.clear()
}

function getRows(table: object): Row[] {
  let rows = tables.get(table)
  if (!rows) {
    rows = []
    tables.set(table, rows)
  }
  return rows
}

/** Test-only read access to a table's rows. */
export function dumpTable(table: object): Row[] {
  return getRows(table).map((row) => ({ ...row }))
}

function jsKeyFor(table: object, column: Column): string | null {
  const columns = getTableColumns(table as never) as Record<string, Column>
  for (const [jsKey, col] of Object.entries(columns)) {
    if (col === column || col.name === column.name) return jsKey
  }
  return null
}

/**
 * Evaluates the simple `eq(column, literal)` / `inArray(column, literals)`
 * conditions our query layer uses against `users` / `sync_meta` /
 * `local_changes`. Anything else matches everything; other tables are stubbed
 * empty by the builders below.
 */
function matches(table: object, cond: unknown, row: Row): boolean {
  const chunks = (cond as SQL | null)?.queryChunks
  if (!Array.isArray(chunks)) return true
  const column = chunks.find((chunk): chunk is Column => chunk instanceof Column)
  const values = chunks
    .filter((chunk): chunk is Param => chunk instanceof Param)
    .map((param) => (param as unknown as { value: unknown }).value)
  if (!column || values.length === 0) return true
  const jsKey = jsKeyFor(table, column)
  if (!jsKey) return false
  return values.includes(row[jsKey])
}

/**
 * Mirrors `apps/mobile/drizzle/*_*.sql` (the actual on-device constraints).
 * NOTE: the drizzle schema object does not carry `.unique()` for
 * `users.username`, but the migrated table has `username TEXT NOT NULL
 * UNIQUE` — SQLite enforces it regardless, so the fake must too.
 */
const PRIMARY_BY_TABLE: Record<string, string[]> = {
  users: ["id"],
  sync_meta: ["key"],
}
const UNIQUE_BY_TABLE: Record<string, string[][]> = {
  users: [["username"]],
}

function constraintColumns(table: object): { primary: string[]; unique: string[][] } {
  const name = getTableName(table as never)
  return {
    primary: PRIMARY_BY_TABLE[name] ?? [],
    unique: UNIQUE_BY_TABLE[name] ?? [],
  }
}

function conflictsWith(table: object, existing: Row, incoming: Row): boolean {
  const { primary, unique } = constraintColumns(table)
  if (primary.length > 0 && primary.every((key) => existing[key] === incoming[key])) {
    return true
  }
  return unique.some(
    (keys) => keys.length > 0 && keys.every((key) => existing[key] === incoming[key]),
  )
}

/**
 * Mirrors SQLite: throws on PRIMARY KEY / UNIQUE violations unless the
 * statement used `ON CONFLICT DO NOTHING`.
 */
function insertRow(table: object, values: Row, ignoreConflicts: boolean): void {
  const rows = getRows(table)
  if (rows.some((row) => conflictsWith(table, row, values))) {
    if (ignoreConflicts) return
    throw new Error(`UNIQUE constraint failed: ${getTableName(table as never)}`)
  }
  const stored = { ...values }
  if (stored.id === undefined) {
    // Mirrors INTEGER PRIMARY KEY AUTOINCREMENT (local_changes.id).
    stored.id = rows.reduce((max, row) => Math.max(max, Number(row.id ?? 0)), 0) + 1
  }
  rows.push(stored)
}

function updateRows(table: object, patch: Row, cond: unknown): void {
  for (const row of getRows(table)) {
    if (matches(table, cond, row)) Object.assign(row, patch)
  }
}

function deleteRows(table: object, cond: unknown | null): void {
  const rows = getRows(table)
  if (cond === null) {
    rows.length = 0
    return
  }
  for (let i = rows.length - 1; i >= 0; i--) {
    if (matches(table, cond, rows[i] as Row)) rows.splice(i, 1)
  }
}

/**
 * In-memory stand-in for the expo-sqlite drizzle `db`. Supports exactly the
 * sync call shapes used by `src/db/queries/*`.
 *
 * `onConflictDoUpdate` deliberately throws: the table-qualified conflict
 * target it generates is fragile across bundled SQLite versions and cannot
 * cover `UNIQUE(username)` clashes — use `upsertUser` / `setSyncMeta`.
 */
export const fakeDb = {
  select: () => ({
    from: (table: object) => ({
      where: (cond: unknown) => ({
        get: () => getRows(table).find((row) => matches(table, cond, row)) ?? null,
        all: () => getRows(table).filter((row) => matches(table, cond, row)),
      }),
      orderBy: () => ({ all: () => [...getRows(table)] }),
      all: () => [...getRows(table)],
      get: () => getRows(table)[0] ?? null,
    }),
  }),
  insert: (table: object) => ({
    values: (values: Row) => ({
      run: () => {
        insertRow(table, values, false)
      },
      onConflictDoNothing: () => ({
        run: () => {
          insertRow(table, values, true)
        },
      }),
      onConflictDoUpdate: () => {
        throw new Error(
          "onConflictDoUpdate is banned in tests: use upsertUser() / setSyncMeta() instead",
        )
      },
    }),
  }),
  update: (table: object) => ({
    set: (patch: Row) => ({
      where: (cond: unknown) => ({
        run: () => {
          updateRows(table, patch, cond)
        },
      }),
    }),
  }),
  delete: (table: object) => ({
    where: (cond: unknown) => ({
      run: () => {
        deleteRows(table, cond)
      },
    }),
    run: () => {
      deleteRows(table, null)
    },
  }),
}
