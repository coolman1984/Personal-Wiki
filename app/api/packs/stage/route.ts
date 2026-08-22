import { getD1, id, nowIso } from "../../../../server/data";
import { requireSession } from "../../../../server/auth";

export async function POST(request: Request) {
  const result = await requireSession(); if ("response" in result) return result.response;
  const body = await request.json().catch(() => ({})) as { pack?: { items?: unknown[]; sources?: unknown[]; version?: unknown }; fileName?: string };
  const pack = body.pack;
  if (!pack || typeof pack !== "object" || (pack.items !== undefined && !Array.isArray(pack.items))) return Response.json({ ok: false, error: "حزمة غير صالحة" }, { status: 400 });
  const items = Array.isArray(pack.items) ? pack.items : [];
  if (JSON.stringify(pack).length > 1_500_000) return Response.json({ ok: false, error: "حجم الحزمة أكبر من الحد" }, { status: 413 });
  const db = await getD1(); const batchId = id("batch"); const timestamp = nowIso();
  if (db) {
    const statements = [db.prepare("INSERT INTO update_batches (id, owner_key, summary, status, created_at, updated_at) VALUES (?1, ?2, ?3, 'staged', ?4, ?4)").bind(batchId, result.session.ownerKey, body.fileName ? `استيراد ${body.fileName}` : "حزمة مستوردة", timestamp)];
    for (const value of items) {
      const item = value as Record<string, unknown>;
      const key = String(item.id ?? item.slug ?? id("item"));
      const type = item.changeType === "remove" ? "remove" : item.changeType === "update" ? "update" : "add";
      statements.push(db.prepare("INSERT OR REPLACE INTO staged_changes (id, batch_id, entity_type, entity_key, change_type, before_json, after_json, status, detected_at) VALUES (?1, ?2, 'item', ?3, ?4, NULL, ?5, 'pending', ?6)").bind(id("change"), batchId, key, type, JSON.stringify(item), timestamp));
    }
    await db.batch(statements);
  }
  return Response.json({ ok: true, data: { batchId, staged: items.length } }, { status: 201 });
}
