import { getD1, nowIso } from "../../../server/data";
import { requireSession } from "../../../server/auth";
import { getR2 } from "../../../server/storage";

export async function GET() {
  const result = await requireSession(); if ("response" in result) return result.response;
  const bucket = await getR2(); if (!bucket) return Response.json({ ok: true, data: [] });
  const listing = await bucket.list({ prefix: `backups/${result.session.ownerKey}/` });
  return Response.json({ ok: true, data: listing.objects ?? [] });
}

export async function POST() {
  const result = await requireSession(); if ("response" in result) return result.response;
  const db = await getD1(); const bucket = await getR2(); if (!db) return Response.json({ ok: false, error: "قاعدة البيانات غير متاحة" }, { status: 503 });
  const rows = await db.prepare("SELECT id, slug, title, description, syntax, lifecycle_status, updated_at FROM items ORDER BY rowid").all<Record<string, unknown>>();
  const key = `backups/${result.session.ownerKey}/manual-${Date.now()}.json`; if (bucket) await bucket.put(key, JSON.stringify({ createdAt: nowIso(), items: rows.results ?? [] }), { httpMetadata: { contentType: "application/json" } });
  return Response.json({ ok: true, key: bucket ? key : null });
}
