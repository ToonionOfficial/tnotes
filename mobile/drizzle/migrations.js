// This file is required for Expo/React Native SQLite migrations - https://orm.drizzle.team/quick-sqlite/expo

import m0000 from "./20260828200809_needy_robin_chapel/migration.sql"
import m0001 from "./20260829120000_folder_note_navigation_index/migration.sql"
import m0002 from "./20260829130000_notes_fts/migration.sql"

export default {
  migrations: {
    "20260828200809_needy_robin_chapel": m0000,
    "20260829120000_folder_note_navigation_index": m0001,
    "20260829130000_notes_fts": m0002,
  },
}
