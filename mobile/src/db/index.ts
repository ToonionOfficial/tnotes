import { defineRelations } from "drizzle-orm"
import { drizzle } from "drizzle-orm/expo-sqlite"
import * as SQLite from "expo-sqlite"
import * as schema from "./schema"

export const expo = SQLite.openDatabaseSync("tnotes.db", {
  enableChangeListener: true,
})

expo.execSync(`
  PRAGMA journal_mode = WAL;
  PRAGMA foreign_keys = ON;

  CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
    title,
    body,
    content='notes',
    content_rowid='rowid'
  );

  CREATE TRIGGER IF NOT EXISTS notes_ai AFTER INSERT ON notes BEGIN
    INSERT INTO notes_fts(rowid, title, body) VALUES (new.rowid, new.title, new.body);
  END;

  CREATE TRIGGER IF NOT EXISTS notes_ad AFTER DELETE ON notes BEGIN
    INSERT INTO notes_fts(notes_fts, rowid, title, body) VALUES('delete', old.rowid, old.title, old.body);
  END;

  CREATE TRIGGER IF NOT EXISTS notes_au AFTER UPDATE ON notes BEGIN
    INSERT INTO notes_fts(notes_fts, rowid, title, body) VALUES('delete', old.rowid, old.title, old.body);
    INSERT INTO notes_fts(rowid, title, body) VALUES (new.rowid, new.title, new.body);
  END;
`)

const relations = defineRelations(schema, ({ one, many, notes, folders, users }) => ({
  notes: {
    folder: one.folders({
      from: notes.folderId,
      to: folders.id,
    }),
    user: one.users({
      from: notes.userId,
      to: users.id,
    }),
  },
  folders: {
    notes: many.notes({
      from: folders.id,
      to: notes.folderId,
    }),
    parent: one.folders({
      from: folders.parentId,
      to: folders.id,
    }),
    children: many.folders({
      from: folders.id,
      to: folders.parentId,
    }),
    user: one.users({
      from: folders.userId,
      to: users.id,
    }),
  },
  users: {
    notes: many.notes({
      from: users.id,
      to: notes.userId,
    }),
    folders: many.folders({
      from: users.id,
      to: folders.userId,
    }),
  },
}))

export const db = drizzle(expo, { relations })
export type Database = typeof db
