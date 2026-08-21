import { index, integer, primaryKey, sqliteTable, text, uniqueIndex } from "drizzle-orm/sqlite-core";

const timestamps = {
  createdAt: text("created_at").notNull(),
  updatedAt: text("updated_at").notNull(),
};

export const tools = sqliteTable("tools", {
  id: text("id").primaryKey(), slug: text("slug").notNull(), name: text("name").notNull(),
  description: text("description").notNull().default(""), iconKey: text("icon_key").notNull().default("terminal"),
  accentKey: text("accent_key").notNull().default("blue"), isActive: integer("is_active", { mode: "boolean" }).notNull().default(true),
  ...timestamps,
}, (table) => [uniqueIndex("tools_slug_uq").on(table.slug)]);

export const tasks = sqliteTable("tasks", {
  id: text("id").primaryKey(), slug: text("slug").notNull(), name: text("name").notNull(),
  description: text("description").notNull().default(""), iconKey: text("icon_key").notNull().default("target"),
  accentKey: text("accent_key").notNull().default("blue"), isActive: integer("is_active", { mode: "boolean" }).notNull().default(true),
  sortOrder: integer("sort_order").notNull().default(0), ...timestamps,
}, (table) => [uniqueIndex("tasks_slug_uq").on(table.slug)]);

export const categories = sqliteTable("categories", {
  id: text("id").primaryKey(), slug: text("slug").notNull(), name: text("name").notNull(),
  parentId: text("parent_id"), iconKey: text("icon_key").notNull().default("folder"),
  accentKey: text("accent_key").notNull().default("blue"), isActive: integer("is_active", { mode: "boolean" }).notNull().default(true),
  sortOrder: integer("sort_order").notNull().default(0), ...timestamps,
}, (table) => [uniqueIndex("categories_slug_uq").on(table.slug), index("categories_parent_idx").on(table.parentId)]);

export const items = sqliteTable("items", {
  id: text("id").primaryKey(), slug: text("slug").notNull(), title: text("title").notNull(),
  subtitle: text("subtitle").notNull().default(""), itemType: text("item_type").notNull(),
  description: text("description").notNull().default(""), fullDescription: text("full_description").notNull().default(""),
  syntax: text("syntax"), whenToUse: text("when_to_use"), expectedResult: text("expected_result"), warningText: text("warning_text"),
  verificationStatus: text("verification_status").notNull().default("unverified"), lifecycleStatus: text("lifecycle_status").notNull().default("active"),
  toolId: text("tool_id"), categoryId: text("category_id"), isActive: integer("is_active", { mode: "boolean" }).notNull().default(true),
  sortOrder: integer("sort_order").notNull().default(0), ...timestamps,
}, (table) => [uniqueIndex("items_slug_uq").on(table.slug), index("items_tool_active_idx").on(table.toolId, table.isActive)]);

export const aliases = sqliteTable("aliases", {
  id: text("id").primaryKey(), itemId: text("item_id").notNull(), alias: text("alias").notNull(), normalizedAlias: text("normalized_alias").notNull(),
}, (table) => [uniqueIndex("aliases_item_norm_uq").on(table.itemId, table.normalizedAlias)]);

export const examples = sqliteTable("examples", {
  id: text("id").primaryKey(), itemId: text("item_id").notNull(), title: text("title").notNull(), description: text("description").notNull().default(""), code: text("code").notNull(), expected: text("expected"), sortOrder: integer("sort_order").notNull().default(0),
});

export const sources = sqliteTable("sources", {
  id: text("id").primaryKey(), name: text("name").notNull(), sourceType: text("source_type").notNull(), url: text("url").notNull(), official: integer("official", { mode: "boolean" }).notNull().default(false), healthState: text("health_state").notNull().default("unchecked"), isActive: integer("is_active", { mode: "boolean" }).notNull().default(true), ...timestamps,
});

export const favorites = sqliteTable("favorites", {
  ownerKey: text("owner_key").notNull(), itemId: text("item_id").notNull(), createdAt: text("created_at").notNull(),
}, (table) => [primaryKey({ columns: [table.ownerKey, table.itemId] })]);

export const pins = sqliteTable("pinned_items", {
  ownerKey: text("owner_key").notNull(), itemId: text("item_id").notNull(), createdAt: text("created_at").notNull(),
}, (table) => [primaryKey({ columns: [table.ownerKey, table.itemId] })]);

export const notes = sqliteTable("notes", {
  id: text("id").primaryKey(), ownerKey: text("owner_key").notNull(), itemId: text("item_id"), title: text("title").notNull(), body: text("body").notNull().default(""), ...timestamps,
}, (table) => [index("notes_owner_updated_idx").on(table.ownerKey, table.updatedAt)]);

export const savedLinks = sqliteTable("saved_links", {
  id: text("id").primaryKey(), ownerKey: text("owner_key").notNull(), title: text("title").notNull(), url: text("url").notNull(), description: text("description").notNull().default(""), toolId: text("tool_id"), favorite: integer("is_favorite", { mode: "boolean" }).notNull().default(false), ...timestamps,
});

export const savedSearches = sqliteTable("saved_searches", {
  id: text("id").primaryKey(), ownerKey: text("owner_key").notNull(), name: text("name").notNull(), query: text("query").notNull(), filtersJson: text("filters_json").notNull().default("{}"), ...timestamps,
});

export const updateBatches = sqliteTable("update_batches", {
  id: text("id").primaryKey(), ownerKey: text("owner_key").notNull(), summary: text("summary").notNull(), status: text("status").notNull(), snapshotKey: text("snapshot_key"), ...timestamps,
});

export const stagedChanges = sqliteTable("staged_changes", {
  id: text("id").primaryKey(), batchId: text("batch_id").notNull(), entityType: text("entity_type").notNull(), entityKey: text("entity_key").notNull(), changeType: text("change_type").notNull(), beforeJson: text("before_json"), afterJson: text("after_json"), status: text("status").notNull(), detectedAt: text("detected_at").notNull(), reviewedAt: text("reviewed_at"),
}, (table) => [uniqueIndex("changes_batch_entity_uq").on(table.batchId, table.entityType, table.entityKey), index("changes_batch_status_idx").on(table.batchId, table.status)]);

export const auditEvents = sqliteTable("audit_events", {
  id: text("id").primaryKey(), ownerKey: text("owner_key").notNull(), action: text("action").notNull(), entityType: text("entity_type").notNull(), entityId: text("entity_id"), detailsJson: text("details_json").notNull().default("{}"), createdAt: text("created_at").notNull(),
});
