CREATE INDEX IF NOT EXISTS `idx_notes_folder_trashed_pinned_updated`
  ON `notes` (`folder_id`, `trashed`, `pinned` DESC, `updated_at` DESC);
