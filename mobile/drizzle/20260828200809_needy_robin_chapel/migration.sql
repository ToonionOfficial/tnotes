CREATE TABLE `folders` (
	`id` text PRIMARY KEY,
	`user_id` text NOT NULL,
	`parent_id` text,
	`name` text NOT NULL,
	`icon` text DEFAULT '📁' NOT NULL,
	`sort_order` integer DEFAULT 0 NOT NULL,
	`version` integer DEFAULT 1 NOT NULL,
	`updated_at` integer NOT NULL,
	`created_at` integer NOT NULL,
	`deleted_at` integer,
	`device_id` text NOT NULL,
	CONSTRAINT `fk_folders_user_id_users_id_fk` FOREIGN KEY (`user_id`) REFERENCES `users`(`id`) ON DELETE CASCADE
);
--> statement-breakpoint
CREATE TABLE `local_changes` (
	`id` integer PRIMARY KEY AUTOINCREMENT,
	`entity_type` text NOT NULL,
	`entity_id` text NOT NULL,
	`version` integer NOT NULL,
	`updated_at` integer NOT NULL,
	`tombstone` integer DEFAULT false NOT NULL,
	`payload` text NOT NULL,
	`created_at` integer NOT NULL
);
--> statement-breakpoint
CREATE TABLE `notes` (
	`id` text PRIMARY KEY,
	`user_id` text NOT NULL,
	`folder_id` text,
	`title` text DEFAULT '' NOT NULL,
	`body` text DEFAULT '' NOT NULL,
	`pinned` integer DEFAULT false NOT NULL,
	`trashed` integer DEFAULT false NOT NULL,
	`version` integer DEFAULT 1 NOT NULL,
	`updated_at` integer NOT NULL,
	`created_at` integer NOT NULL,
	`deleted_at` integer,
	`device_id` text NOT NULL,
	`checksum` text NOT NULL,
	CONSTRAINT `fk_notes_user_id_users_id_fk` FOREIGN KEY (`user_id`) REFERENCES `users`(`id`) ON DELETE CASCADE,
	CONSTRAINT `fk_notes_folder_id_folders_id_fk` FOREIGN KEY (`folder_id`) REFERENCES `folders`(`id`) ON DELETE SET NULL
);
--> statement-breakpoint
CREATE TABLE `sync_meta` (
	`key` text PRIMARY KEY,
	`value` text NOT NULL
);
--> statement-breakpoint
CREATE TABLE `users` (
	`id` text PRIMARY KEY,
	`username` text NOT NULL UNIQUE,
	`created_at` integer NOT NULL
);
--> statement-breakpoint
CREATE INDEX `idx_folders_user` ON `folders` (`user_id`);--> statement-breakpoint
CREATE INDEX `idx_folders_parent` ON `folders` (`parent_id`);--> statement-breakpoint
CREATE INDEX `idx_local_changes_entity` ON `local_changes` (`entity_id`,`entity_type`);--> statement-breakpoint
CREATE INDEX `idx_notes_user` ON `notes` (`user_id`);--> statement-breakpoint
CREATE INDEX `idx_notes_user_updated` ON `notes` (`user_id`,`updated_at`);--> statement-breakpoint
CREATE INDEX `idx_notes_folder` ON `notes` (`folder_id`);--> statement-breakpoint
CREATE INDEX `idx_notes_trashed` ON `notes` (`trashed`);