/**
 * Minimal typings for Node's built-in SQLite (`node:sqlite`), used only by
 * regression tests. Declared locally so `tsc --noEmit` passes without
 * `@types/node`.
 */
declare module "node:sqlite" {
  export class StatementSync {
    run(...params: unknown[]): { changes: number | bigint; lastInsertRowId: number | bigint }
    get(...params: unknown[]): Record<string, unknown> | undefined
    all(...params: unknown[]): Record<string, unknown>[]
  }

  export class DatabaseSync {
    constructor(path: string)
    exec(sql: string): void
    prepare(sql: string): StatementSync
    close(): void
  }
}
