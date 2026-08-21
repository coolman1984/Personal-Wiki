import { mockDashboard } from "../app/mock-data";

export type Prepared = { bind: (...values: unknown[]) => Prepared; run: () => Promise<unknown>; first: <T = unknown>() => Promise<T | null>; all: <T = Record<string, unknown>>() => Promise<{ results: T[] }> };
export type D1Binding = { prepare: (statement: string) => Prepared; batch: (statements: Prepared[]) => Promise<unknown[]> };

export async function ensureSeeded(db: D1Binding) {
  try {
    await db.prepare("CREATE TABLE IF NOT EXISTS _wiki_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL)").run();
    const existing = await db.prepare("SELECT value FROM _wiki_meta WHERE key = 'seed-v1'").first<{ value: string }>();
    if (existing) return;
    const now = new Date().toISOString();
    const statements: Prepared[] = [
      db.prepare("INSERT OR IGNORE INTO _wiki_meta (key, value) VALUES ('seed-v1', ?1)").bind(now),
      ...mockDashboard.tools.map((tool) => db.prepare("INSERT OR IGNORE INTO tools (id, slug, name, description, icon_key, accent_key, is_active, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?7)").bind(tool.id, tool.slug, tool.name, tool.description, tool.icon, tool.accent, now)),
      ...mockDashboard.tasks.map((task, index) => db.prepare("INSERT OR IGNORE INTO tasks (id, slug, name, description, icon_key, accent_key, is_active, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8, ?8)").bind(task.id, task.slug, task.name, task.description, task.icon, task.accent, index, now)),
      ...mockDashboard.sources.map((source) => db.prepare("INSERT OR IGNORE INTO sources (id, name, source_type, url, official, health_state, is_active, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, 'healthy', 1, ?6, ?6)").bind(source.id, source.name, source.type, source.url, source.official ? 1 : 0, now)),
      ...mockDashboard.items.map((item, index) => db.prepare("INSERT OR IGNORE INTO items (id, slug, title, subtitle, item_type, description, full_description, syntax, when_to_use, expected_result, verification_status, lifecycle_status, tool_id, is_active, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 1, ?13, ?14, ?14)").bind(item.id, item.slug, item.title, item.subtitle, item.type, item.description, item.syntax ?? null, item.whenToUse, item.expected ?? null, item.verification, item.lifecycle, item.toolId, index, now)),
      ...mockDashboard.items.flatMap((item) => item.aliases.map((alias, index) => db.prepare("INSERT OR IGNORE INTO aliases (id, item_id, alias, normalized_alias) VALUES (?1, ?2, ?3, ?4)").bind(`${item.id}-alias-${index}`, item.id, alias, alias.toLocaleLowerCase("ar")))),
      ...mockDashboard.items.flatMap((item) => item.examples.map((example, index) => db.prepare("INSERT OR IGNORE INTO examples (id, item_id, title, description, code, expected, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)").bind(example.id, item.id, example.title, example.description, example.code, example.expected ?? null, index))),
    ];
    await db.batch(statements);
    try {
      await db.batch(mockDashboard.items.map((item) => db.prepare("INSERT OR REPLACE INTO items_fts (item_id, title, aliases, description, syntax, tool, category) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)").bind(item.id, item.title, item.aliases.join(" "), item.description, item.syntax ?? "", item.toolName, item.categoryName)));
    } catch { /* FTS migration may be applied on the next checkpoint. */ }
  } catch { /* The dashboard still has a safe read-only fallback while migrations are pending. */ }
}
