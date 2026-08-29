import path from "node:path"
import { defineConfig } from "vitest/config"

export default defineConfig({
  test: {
    globals: true,
    environment: "node",
    include: ["src/**/*.test.ts", "tests/**/*.test.ts"],
  },
  resolve: {
    alias: [
      {
        find: /^expo-sqlite(\/.*)?$/,
        replacement: path.resolve(
          import.meta.dirname,
          "./tests/mocks/expoSqlite.ts",
        ),
      },
      {
        find: "drizzle-orm/expo-sqlite",
        replacement: path.resolve(
          import.meta.dirname,
          "./tests/mocks/drizzleExpo.ts",
        ),
      },
      {
        find: "expo-crypto",
        replacement: path.resolve(
          import.meta.dirname,
          "./tests/mocks/expoCrypto.ts",
        ),
      },
      {
        find: "react-native",
        replacement: "react-native-web",
      },
      {
        find: "@",
        replacement: path.resolve(import.meta.dirname, "./src"),
      },
    ],
  },
})
