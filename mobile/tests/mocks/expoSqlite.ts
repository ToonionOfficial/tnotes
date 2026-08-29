export const openDatabaseSync = () => ({
  execSync: () => {},
  runSync: () => ({ changes: 0, lastInsertRowId: 0 }),
  getAllSync: () => [],
  getFirstSync: () => null,
  getAllAsync: async () => [],
  getFirstAsync: async () => null,
  runAsync: async () => ({ changes: 0, lastInsertRowId: 0 }),
  execAsync: async () => {},
  withExclusiveTransactionAsync: async (cb: (tx: unknown) => Promise<unknown>) =>
    cb({
      runAsync: async () => {},
      getAllAsync: async () => [],
    }),
})

export const SQLiteDatabase = class {}
