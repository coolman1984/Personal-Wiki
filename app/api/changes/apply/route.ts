import { getD1, id, nowIso, parseJson } from "../../../../server/data";
import { requireSession } from "../../../../server/auth";
import { getR2 } from "../../../../server/storage";

export async function POST(request: Request) {
  const result = await requireSession(); if ("response" in result) return result.response;
  const body = await request.json().catch(() => ({})) as { batchId?: string };
  const db = await getD1(); if (!db) return Response.json({ ok: false, error: "قاعدة البيانات غير متاحة" }, { status: 503 });
  const batchId = body.batchId;
  if (!batchId) return Response.json({ ok: false, error: "batchId مطلوب" }, { status: 400 });
  const batch = await db.prepare("SELECT id, status FROM update_batches WHERE id = ?1 AND owner_key = ?2").bind(batchId, result.session.ownerKey).first<{ id: string; status: string }>();
  if (!batch) return Response.json({ ok: false, error: "دفعة غير موجودة" }, { status: 404 });
  const changes = await db.prepare("SELECT id, entity_type, entity_key, change_type, before_json, after_json, status FROM staged_changes WHERE batch_id = ?1 ORDER BY rowid").bind(batchId).all<Record<string, unknown>>();
  const rows = changes.results ?? [];
  if (rows.some((row) => row.status === "pending")) return Response.json({ ok: false, error: "راجع كل عناصر الدفعة قبل التطبيق" }, { status: 409 });
  const approved = rows.filter((row) => row.status === "approved");
  if (!approved.length) return Response.json({ ok: false, error: "لا توجد تغييرات موافق عليها" }, { status: 409 });
  const timestamp = nowIso(); const snapshotKey = `backups/${result.session.ownerKey}/${batchId}-${Date.now()}.json`;
  const snapshot = { batchId, createdAt: timestamp, changes: rows.map((row) => ({ ...row, before: parseJson(row.before_json), after: parseJson(row.after_json) })) };
  const bucket = await getR2(); if (bucket) await bucket.put(snapshotKey, JSON.stringify(snapshot), { httpMetadata: { contentType: "application/json" } });
  const statements = [] as ReturnType<typeof db.prepare>[];
  for (const row of approved) {
    const after = parseJson(row.after_json) ?? {};
    if (row.entity_type === "item") {
      const removed = row.change_type === "remove";
      const itemId = String(row.entity_key);
      const existing = await db.prepare("SELECT slug, title, subtitle, item_type, description, full_description, syntax, when_to_use, expected_result, verification_status, lifecycle_status, tool_id, category_id, sort_order, created_at FROM items WHERE id = ?1").bind(itemId).first<Record<string, unknown>>();
      const slug = String(after.slug ?? existing?.slug ?? itemId);
      statements.push(db.prepare("INSERT OR REPLACE INTO items (id, slug, title, subtitle, item_type, description, full_description, syntax, when_to_use, expected_result, verification_status, lifecycle_status, tool_id, category_id, is_active, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, COALESCE(?17, ?18), ?18)").bind(itemId, slug, String(after.title ?? existing?.title ?? itemId), String(after.subtitle ?? existing?.subtitle ?? ""), String(after.itemType ?? after.type ?? existing?.item_type ?? "command"), String(after.description ?? existing?.description ?? ""), String(after.fullDescription ?? existing?.full_description ?? after.description ?? ""), after.syntax ?? existing?.syntax ?? null, after.whenToUse ?? existing?.when_to_use ?? null, after.expected ?? after.expectedResult ?? existing?.expected_result ?? null, String(after.verification ?? existing?.verification_status ?? "unverified"), removed ? "removed" : String(after.lifecycle ?? existing?.lifecycle_status ?? "active"), after.toolId ?? existing?.tool_id ?? null, after.categoryId ?? existing?.category_id ?? null, removed ? 0 : 1, Number(after.sortOrder ?? existing?.sort_order ?? 0), existing?.created_at ?? null, timestamp));
      statements.push(db.prepare("INSERT INTO audit_events (id, owner_key, action, entity_type, entity_id, details_json, created_at) VALUES (?1, ?2, ?3, 'item', ?4, ?5, ?6)").bind(id("audit"), result.session.ownerKey, row.change_type === "add" ? "create" : removed ? "remove" : "update", itemId, JSON.stringify(after), timestamp));
    }
    statements.push(db.prepare("UPDATE staged_changes SET status = 'applied', reviewed_at = ?1 WHERE id = ?2").bind(timestamp, row.id));
  }
  statements.push(db.prepare("UPDATE update_batches SET status = 'applied', snapshot_key = ?1, updated_at = ?2 WHERE id = ?3 AND owner_key = ?4").bind(snapshotKey, timestamp, batchId, result.session.ownerKey));
  await db.batch(statements);
  try {
    for (const row of approved) {
      const after = parseJson(row.after_json) ?? {};
      if (row.entity_type === "item") await db.prepare("INSERT OR REPLACE INTO items_fts (item_id, title, aliases, description, syntax, tool, category) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)").bind(String(row.entity_key), String(after.title ?? row.entity_key), Array.isArray(after.aliases) ? after.aliases.join(" ") : "", String(after.description ?? ""), String(after.syntax ?? ""), String(after.toolName ?? ""), String(after.categoryName ?? "")).run();
    }
  } catch { /* Search index can be rebuilt without blocking a committed content change. */ }
  return Response.json({ ok: true, applied: approved.length, snapshotKey: bucket ? snapshotKey : null });
}
