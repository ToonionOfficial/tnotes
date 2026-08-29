CREATE VIRTUAL TABLE IF NOT EXISTS `notes_fts` USING fts5(
  `title`,
  `body`,
  content='notes',
  content_rowid='rowid'
);--> statement-breakpoint
CREATE TRIGGER IF NOT EXISTS `notes_ai` AFTER INSERT ON `notes` BEGIN
  INSERT INTO `notes_fts` (`rowid`, `title`, `body`) VALUES (new.`rowid`, new.`title`, new.`body`);
END;--> statement-breakpoint
CREATE TRIGGER IF NOT EXISTS `notes_ad` AFTER DELETE ON `notes` BEGIN
  INSERT INTO `notes_fts` (`notes_fts`, `rowid`, `title`, `body`) VALUES ('delete', old.`rowid`, old.`title`, old.`body`);
END;--> statement-breakpoint
CREATE TRIGGER IF NOT EXISTS `notes_au` AFTER UPDATE ON `notes` BEGIN
  INSERT INTO `notes_fts` (`notes_fts`, `rowid`, `title`, `body`) VALUES ('delete', old.`rowid`, old.`title`, old.`body`);
  INSERT INTO `notes_fts` (`rowid`, `title`, `body`) VALUES (new.`rowid`, new.`title`, new.`body`);
END;--> statement-breakpoint
INSERT INTO `notes_fts` (`notes_fts`) VALUES ('rebuild');
