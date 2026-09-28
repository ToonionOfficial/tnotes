// This file is required for Expo/React Native SQLite migrations - https://orm.drizzle.team/quick-sqlite/expo

import m0000 from "./20260828200809_needy_robin_chapel/migration.sql"
import m0001 from "./20260829120000_folder_note_navigation_index/migration.sql"
import m0002 from "./20260829130000_notes_fts/migration.sql"
import m0003 from "./20260928074602_searchable_text/migration.sql"
import m0004 from "./20260928074603_notes_fts_searchable/migration.sql"

export default {
  migrations: {
    "20260828200809_needy_robin_chapel": m0000,
    "20260829120000_folder_note_navigation_index": m0001,
    "20260829130000_notes_fts": m0002,
    "20260928074602_searchable_text": m0003,
    "20260928074603_notes_fts_searchable": m0004,
  },
}
