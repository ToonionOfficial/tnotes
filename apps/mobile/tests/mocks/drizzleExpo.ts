export const drizzle = () => ({
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
})
