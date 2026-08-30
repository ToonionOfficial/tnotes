import { index, integer, sqliteTable, text } from "drizzle-orm/sqlite-core"

export const users = sqliteTable("users", {
  id: text("id").primaryKey(),
  username: text("username").notNull(),
  createdAt: integer("created_at").notNull(),
})

export type User = typeof users.$inferSelect
export type NewUser = typeof users.$inferInsert

export const folders = sqliteTable(
  "folders",
  {
    id: text("id").primaryKey(),
    userId: text("user_id")
      .notNull()
      .references(() => users.id, { onDelete: "cascade" }),
    parentId: text("parent_id"),
    name: text("name").notNull(),
    icon: text("icon").notNull().default("folder"),
    sortOrder: integer("sort_order").notNull().default(0),
    version: integer("version").notNull().default(1),
    updatedAt: integer("updated_at").notNull(),
    createdAt: integer("created_at").notNull(),
    deletedAt: integer("deleted_at"),
    deviceId: text("device_id").notNull(),
  },
  (table) => [
    index("idx_folders_user").on(table.userId),
    index("idx_folders_parent").on(table.parentId),
  ],
)

export type Folder = typeof folders.$inferSelect
export type NewFolder = typeof folders.$inferInsert

export const notes = sqliteTable(
  "notes",
  {
    id: text("id").primaryKey(),
    userId: text("user_id")
      .notNull()
      .references(() => users.id, { onDelete: "cascade" }),
    folderId: text("folder_id").references(() => folders.id, {
      onDelete: "set null",
    }),
    title: text("title").notNull().default(""),
    body: text("body").notNull().default(""),
    pinned: integer("pinned", { mode: "boolean" }).notNull().default(false),
    trashed: integer("trashed", { mode: "boolean" }).notNull().default(false),
    version: integer("version").notNull().default(1),
    updatedAt: integer("updated_at").notNull(),
    createdAt: integer("created_at").notNull(),
    deletedAt: integer("deleted_at"),
    deviceId: text("device_id").notNull(),
    checksum: text("checksum").notNull(),
  },
  (table) => [
    index("idx_notes_user").on(table.userId),
    index("idx_notes_user_updated").on(table.userId, table.updatedAt),
    index("idx_notes_folder").on(table.folderId),
    index("idx_notes_trashed").on(table.trashed),
    index("idx_notes_folder_trashed_pinned_updated").on(
      table.folderId,
      table.trashed,
      table.pinned,
      table.updatedAt,
    ),
  ],
)

export type Note = typeof notes.$inferSelect
export type NewNote = typeof notes.$inferInsert

export const syncMeta = sqliteTable("sync_meta", {
  key: text("key").primaryKey(),
  value: text("value").notNull(),
})

export type SyncMeta = typeof syncMeta.$inferSelect
export type NewSyncMeta = typeof syncMeta.$inferInsert

export const localChanges = sqliteTable(
  "local_changes",
  {
    id: integer("id").primaryKey({ autoIncrement: true }),
    entityType: text("entity_type").notNull(), // 'note' | 'folder'
    entityId: text("entity_id").notNull(),
    version: integer("version").notNull(),
    updatedAt: integer("updated_at").notNull(),
    tombstone: integer("tombstone", { mode: "boolean" }).notNull().default(false),
    payload: text("payload").notNull(), // Full JSON entity payload matching sync envelope
    createdAt: integer("created_at").notNull(),
  },
  (table) => [index("idx_local_changes_entity").on(table.entityId, table.entityType)],
)

export type LocalChange = typeof localChanges.$inferSelect
export type NewLocalChange = typeof localChanges.$inferInsert
