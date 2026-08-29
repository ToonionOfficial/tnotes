export const expo = {
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
}

export const db = {
  select: () => ({
    from: () => ({
      where: () => ({
        get: () => null,
        all: () => [],
      }),
      orderBy: () => ({
        all: () => [],
      }),
      all: () => [],
      get: () => null,
    }),
  }),
  insert: () => ({
    values: () => ({
      run: () => ({}),
      onConflictDoUpdate: () => ({
        run: () => ({}),
      }),
    }),
  }),
  update: () => ({
    set: () => ({
      where: () => ({
        run: () => ({}),
      }),
    }),
  }),
  delete: () => ({
    where: () => ({
      run: () => ({}),
    }),
  }),
}
