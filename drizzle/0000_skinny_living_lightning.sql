CREATE TABLE `aliases` (
	`id` text PRIMARY KEY NOT NULL,
	`item_id` text NOT NULL,
	`alias` text NOT NULL,
	`normalized_alias` text NOT NULL
);
--> statement-breakpoint
CREATE UNIQUE INDEX `aliases_item_norm_uq` ON `aliases` (`item_id`,`normalized_alias`);--> statement-breakpoint
CREATE TABLE `audit_events` (
	`id` text PRIMARY KEY NOT NULL,
	`owner_key` text NOT NULL,
	`action` text NOT NULL,
	`entity_type` text NOT NULL,
	`entity_id` text,
	`details_json` text DEFAULT '{}' NOT NULL,
	`created_at` text NOT NULL
);
--> statement-breakpoint
CREATE TABLE `categories` (
	`id` text PRIMARY KEY NOT NULL,
	`slug` text NOT NULL,
	`name` text NOT NULL,
	`parent_id` text,
	`icon_key` text DEFAULT 'folder' NOT NULL,
	`accent_key` text DEFAULT 'blue' NOT NULL,
	`is_active` integer DEFAULT true NOT NULL,
	`sort_order` integer DEFAULT 0 NOT NULL,
	`created_at` text NOT NULL,
	`updated_at` text NOT NULL
);
--> statement-breakpoint
CREATE UNIQUE INDEX `categories_slug_uq` ON `categories` (`slug`);--> statement-breakpoint
CREATE INDEX `categories_parent_idx` ON `categories` (`parent_id`);--> statement-breakpoint
CREATE TABLE `examples` (
	`id` text PRIMARY KEY NOT NULL,
	`item_id` text NOT NULL,
	`title` text NOT NULL,
	`description` text DEFAULT '' NOT NULL,
	`code` text NOT NULL,
	`expected` text,
	`sort_order` integer DEFAULT 0 NOT NULL
);
--> statement-breakpoint
CREATE TABLE `favorites` (
	`owner_key` text NOT NULL,
	`item_id` text NOT NULL,
	`created_at` text NOT NULL,
	PRIMARY KEY(`owner_key`, `item_id`)
);
--> statement-breakpoint
CREATE TABLE `items` (
	`id` text PRIMARY KEY NOT NULL,
	`slug` text NOT NULL,
	`title` text NOT NULL,
	`subtitle` text DEFAULT '' NOT NULL,
	`item_type` text NOT NULL,
	`description` text DEFAULT '' NOT NULL,
	`full_description` text DEFAULT '' NOT NULL,
	`syntax` text,
	`when_to_use` text,
	`expected_result` text,
	`warning_text` text,
	`verification_status` text DEFAULT 'unverified' NOT NULL,
	`lifecycle_status` text DEFAULT 'active' NOT NULL,
	`tool_id` text,
	`category_id` text,
	`is_active` integer DEFAULT true NOT NULL,
	`sort_order` integer DEFAULT 0 NOT NULL,
	`created_at` text NOT NULL,
	`updated_at` text NOT NULL
);
--> statement-breakpoint
CREATE UNIQUE INDEX `items_slug_uq` ON `items` (`slug`);--> statement-breakpoint
CREATE INDEX `items_tool_active_idx` ON `items` (`tool_id`,`is_active`);--> statement-breakpoint
CREATE TABLE `notes` (
	`id` text PRIMARY KEY NOT NULL,
	`owner_key` text NOT NULL,
	`item_id` text,
	`title` text NOT NULL,
	`body` text DEFAULT '' NOT NULL,
	`created_at` text NOT NULL,
	`updated_at` text NOT NULL
);
--> statement-breakpoint
CREATE INDEX `notes_owner_updated_idx` ON `notes` (`owner_key`,`updated_at`);--> statement-breakpoint
CREATE TABLE `pinned_items` (
	`owner_key` text NOT NULL,
	`item_id` text NOT NULL,
	`created_at` text NOT NULL,
	PRIMARY KEY(`owner_key`, `item_id`)
);
--> statement-breakpoint
CREATE TABLE `saved_links` (
	`id` text PRIMARY KEY NOT NULL,
	`owner_key` text NOT NULL,
	`title` text NOT NULL,
	`url` text NOT NULL,
	`description` text DEFAULT '' NOT NULL,
	`tool_id` text,
	`is_favorite` integer DEFAULT false NOT NULL,
	`created_at` text NOT NULL,
	`updated_at` text NOT NULL
);
--> statement-breakpoint
CREATE TABLE `saved_searches` (
	`id` text PRIMARY KEY NOT NULL,
	`owner_key` text NOT NULL,
	`name` text NOT NULL,
	`query` text NOT NULL,
	`filters_json` text DEFAULT '{}' NOT NULL,
	`created_at` text NOT NULL,
	`updated_at` text NOT NULL
);
--> statement-breakpoint
CREATE TABLE `sources` (
	`id` text PRIMARY KEY NOT NULL,
	`name` text NOT NULL,
	`source_type` text NOT NULL,
	`url` text NOT NULL,
	`official` integer DEFAULT false NOT NULL,
	`health_state` text DEFAULT 'unchecked' NOT NULL,
	`is_active` integer DEFAULT true NOT NULL,
	`created_at` text NOT NULL,
	`updated_at` text NOT NULL
);
--> statement-breakpoint
CREATE TABLE `staged_changes` (
	`id` text PRIMARY KEY NOT NULL,
	`batch_id` text NOT NULL,
	`entity_type` text NOT NULL,
	`entity_key` text NOT NULL,
	`change_type` text NOT NULL,
	`before_json` text,
	`after_json` text,
	`status` text NOT NULL,
	`detected_at` text NOT NULL,
	`reviewed_at` text
);
--> statement-breakpoint
CREATE UNIQUE INDEX `changes_batch_entity_uq` ON `staged_changes` (`batch_id`,`entity_type`,`entity_key`);--> statement-breakpoint
CREATE INDEX `changes_batch_status_idx` ON `staged_changes` (`batch_id`,`status`);--> statement-breakpoint
CREATE TABLE `tasks` (
	`id` text PRIMARY KEY NOT NULL,
	`slug` text NOT NULL,
	`name` text NOT NULL,
	`description` text DEFAULT '' NOT NULL,
	`icon_key` text DEFAULT 'target' NOT NULL,
	`accent_key` text DEFAULT 'blue' NOT NULL,
	`is_active` integer DEFAULT true NOT NULL,
	`sort_order` integer DEFAULT 0 NOT NULL,
	`created_at` text NOT NULL,
	`updated_at` text NOT NULL
);
--> statement-breakpoint
CREATE UNIQUE INDEX `tasks_slug_uq` ON `tasks` (`slug`);--> statement-breakpoint
CREATE TABLE `tools` (
	`id` text PRIMARY KEY NOT NULL,
	`slug` text NOT NULL,
	`name` text NOT NULL,
	`description` text DEFAULT '' NOT NULL,
	`icon_key` text DEFAULT 'terminal' NOT NULL,
	`accent_key` text DEFAULT 'blue' NOT NULL,
	`is_active` integer DEFAULT true NOT NULL,
	`created_at` text NOT NULL,
	`updated_at` text NOT NULL
);
--> statement-breakpoint
CREATE UNIQUE INDEX `tools_slug_uq` ON `tools` (`slug`);--> statement-breakpoint
CREATE TABLE `update_batches` (
	`id` text PRIMARY KEY NOT NULL,
	`owner_key` text NOT NULL,
	`summary` text NOT NULL,
	`status` text NOT NULL,
	`snapshot_key` text,
	`created_at` text NOT NULL,
	`updated_at` text NOT NULL
);
