import { getD1, nowIso, parseJson } from "../../../../server/data";
import { requireSession } from "../../../../server/auth";
import { getR2 } from "../../../../server/storage";

export async function POST(request: Request) {
  const result = await requireSession(); if ("response" in result) return result.response;
  const body = await request.json().catch(() => ({})) as { key?: string };
  if (!body.key || !body.key.startsWith(`backups/${result.session.ownerKey}/`)) return Response.json({ ok: false, error: "نسخة احتياطية غير مسموحة" }, { status: 400 });
  const bucket = await getR2(); const db = await getD1(); if (!bucket || !db) return Response.json({ ok: false, error: "التخزين غير متاح" }, { status: 503 });
  const object = await bucket.get(body.key); if (!object) return Response.json({ ok: false, error: "النسخة غير موجودة" }, { status: 404 });
  const payload = parseJson(await object.text()); const items = Array.isArray(payload?.items) ? payload.items : [];
  if (!items.length) return Response.json({ ok: false, error: "لا توجد عناصر قابلة للاستعادة" }, { status: 400 });
  const timestamp = nowIso();
  const statements = items.slice(0, 500).map((value) => { const item = value as Record<string, unknown>; return db.prepare("UPDATE items SET title = COALESCE(?1, title), description = COALESCE(?2, description), syntax = ?3, lifecycle_status = COALESCE(?4, lifecycle_status), updated_at = ?5 WHERE id = ?6").bind(item.title ?? null, item.description ?? null, item.syntax ?? null, item.lifecycle_status ?? null, timestamp, String(item.id)); });
  if (statements.length) await db.batch(statements);
  return Response.json({ ok: true, restored: statements.length });
}
