DROP TRIGGER IF EXISTS `notes_ai`;--> statement-breakpoint
DROP TRIGGER IF EXISTS `notes_ad`;--> statement-breakpoint
DROP TRIGGER IF EXISTS `notes_au`;--> statement-breakpoint
DROP TABLE IF EXISTS `notes_fts`;--> statement-breakpoint
CREATE VIRTUAL TABLE IF NOT EXISTS `notes_fts` USING fts5(
  `title`,
  `searchable_text`,
  content='notes',
  content_rowid='rowid'
);--> statement-breakpoint
CREATE TRIGGER IF NOT EXISTS `notes_ai` AFTER INSERT ON `notes` BEGIN
  INSERT INTO `notes_fts` (`rowid`, `title`, `searchable_text`) VALUES (new.`rowid`, new.`title`, new.`searchable_text`);
END;--> statement-breakpoint
CREATE TRIGGER IF NOT EXISTS `notes_ad` AFTER DELETE ON `notes` BEGIN
  INSERT INTO `notes_fts` (`notes_fts`, `rowid`, `title`, `searchable_text`) VALUES ('delete', old.`rowid`, old.`title`, old.`searchable_text`);
END;--> statement-breakpoint
CREATE TRIGGER IF NOT EXISTS `notes_au` AFTER UPDATE ON `notes` BEGIN
  INSERT INTO `notes_fts` (`notes_fts`, `rowid`, `title`, `searchable_text`) VALUES ('delete', old.`rowid`, old.`title`, old.`searchable_text`);
  INSERT INTO `notes_fts` (`rowid`, `title`, `searchable_text`) VALUES (new.`rowid`, new.`title`, new.`searchable_text`);
END;--> statement-breakpoint
INSERT INTO `notes_fts` (`notes_fts`) VALUES ('rebuild');
