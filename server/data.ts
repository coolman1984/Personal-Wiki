import { mockDashboard } from "../app/mock-data";
import type { Dashboard, KnowledgeItem } from "../app/types";
import { ensureSeeded, type D1Binding } from "./seed";

export async function getD1(): Promise<D1Binding | null> {
  try {
    const runtime = await import("cloudflare:workers");
    return ((runtime.env as unknown as { DB?: D1Binding }).DB ?? null);
  } catch { return null; }
}

function cloneDashboard(): Dashboard { return JSON.parse(JSON.stringify(mockDashboard)) as Dashboard; }

export async function dashboardForOwner(ownerKey: string): Promise<Dashboard> {
  const db = await getD1();
  if (!db) return cloneDashboard();
  try {
    await ensureSeeded(db);
    const result = cloneDashboard();
    const itemRows = await db.prepare("SELECT id, slug, title, subtitle, item_type, description, when_to_use, syntax, expected_result, verification_status, lifecycle_status, tool_id, category_id FROM items WHERE is_active = 1 ORDER BY sort_order, title").all<Record<string, unknown>>();
    const toolRows = await db.prepare("SELECT id, name, description, icon_key, accent_key FROM tools WHERE is_active = 1 ORDER BY rowid").all<Record<string, unknown>>();
    const sourceRows = await db.prepare("SELECT id, name, source_type, url, official, health_state FROM sources WHERE is_active = 1 ORDER BY rowid").all<Record<string, unknown>>();
    const favoriteRows = await db.prepare("SELECT item_id FROM favorites WHERE owner_key = ?1").bind(ownerKey).all<{ item_id: string }>();
    const pinRows = await db.prepare("SELECT item_id FROM pinned_items WHERE owner_key = ?1").bind(ownerKey).all<{ item_id: string }>();
    const noteRows = await db.prepare("SELECT id, title, body, item_id, updated_at FROM notes WHERE owner_key = ?1 ORDER BY updated_at DESC LIMIT 50").bind(ownerKey).all<Record<string, unknown>>();
    const linkRows = await db.prepare("SELECT id, title, url, description, tool_id, is_favorite FROM saved_links WHERE owner_key = ?1 ORDER BY updated_at DESC LIMIT 50").bind(ownerKey).all<Record<string, unknown>>();
    const changeRows = await db.prepare("SELECT id, batch_id, change_type, entity_key, status, after_json, before_json, detected_at FROM staged_changes WHERE batch_id IN (SELECT id FROM update_batches WHERE owner_key = ?1) ORDER BY detected_at DESC LIMIT 100").bind(ownerKey).all<Record<string, unknown>>();
    const toolById = new Map((toolRows.results ?? []).map((row) => [String(row.id), row]));
    const sourceById = new Map((sourceRows.results ?? []).map((row) => [String(row.id), row]));
    const favorites = new Set((favoriteRows.results ?? []).map((row) => row.item_id));
    const pins = new Set((pinRows.results ?? []).map((row) => row.item_id));
    if (itemRows.results?.length) {
      result.items = itemRows.results.map((row) => {
        const tool = toolById.get(String(row.tool_id));
        const source = sourceById.get(String(row.source_id));
        const fallback = result.items.find((item) => item.id === row.id);
        return {
          id: String(row.id), slug: String(row.slug), title: String(row.title), subtitle: String(row.subtitle ?? ""),
          toolId: String(row.tool_id ?? ""), toolName: String(tool?.name ?? fallback?.toolName ?? ""), toolIcon: String(tool?.icon_key ?? fallback?.toolIcon ?? "terminal"),
          categoryName: fallback?.categoryName ?? "أوامر عامة", type: String(row.item_type ?? "command"), description: String(row.description ?? ""),
          whenToUse: String(row.when_to_use ?? ""), syntax: row.syntax ? String(row.syntax) : undefined, expected: row.expected_result ? String(row.expected_result) : undefined,
          verification: (row.verification_status as KnowledgeItem["verification"]) ?? "unverified", lifecycle: (row.lifecycle_status as KnowledgeItem["lifecycle"]) ?? "active",
          tags: fallback?.tags ?? [], aliases: fallback?.aliases ?? [], examples: fallback?.examples ?? [], source: source ? { id: String(source.id), name: String(source.name), host: new URL(String(source.url)).host, url: String(source.url), type: String(source.source_type), official: Boolean(source.official), status: source.health_state === "healthy" ? "connected" : "warning" } : fallback?.source,
          favorite: favorites.has(String(row.id)), pinned: pins.has(String(row.id)),
        } satisfies KnowledgeItem;
      });
    }
    if (sourceRows.results?.length) result.sources = sourceRows.results.map((row) => ({ id: String(row.id), name: String(row.name), host: new URL(String(row.url)).host, url: String(row.url), type: String(row.source_type), official: Boolean(row.official), status: row.health_state === "healthy" ? "connected" : "warning" }));
    if (noteRows.results) result.notes = noteRows.results.map((row) => ({ id: String(row.id), title: String(row.title), body: String(row.body ?? ""), itemId: row.item_id ? String(row.item_id) : undefined, updated: String(row.updated_at) }));
    if (linkRows.results) result.links = linkRows.results.map((row) => ({ id: String(row.id), title: String(row.title), url: String(row.url), description: String(row.description ?? ""), toolId: row.tool_id ? String(row.tool_id) : undefined, favorite: Boolean(row.is_favorite) }));
    if (changeRows.results) result.changes = changeRows.results.map((row) => ({ id: String(row.id), batchId: String(row.batch_id), type: (row.change_type as "add" | "update" | "remove") ?? "update", title: String(row.entity_key), detail: "تغيير مستورد بانتظار المراجعة", time: String(row.detected_at), status: (row.status as "pending" | "approved" | "rejected" | "applied") ?? "pending", before: parseJson(row.before_json), after: parseJson(row.after_json) }));
    return result;
  } catch { return cloneDashboard(); }
}

export function parseJson(value: unknown): Record<string, unknown> | undefined { if (!value) return undefined; try { return JSON.parse(String(value)) as Record<string, unknown>; } catch { return undefined; } }

export function nowIso() { return new Date().toISOString(); }
export function id(prefix: string) { return `${prefix}-${crypto.randomUUID()}`; }
