import { getD1, id, nowIso } from "../../../server/data";
import { requireSession } from "../../../server/auth";

export async function POST(request: Request) {
  const result = await requireSession(); if ("response" in result) return result.response;
  const body = await request.json().catch(() => ({})) as { id?: string; title?: string; body?: string; itemId?: string };
  const title = body.title?.trim(); if (!title) return Response.json({ ok: false, error: "عنوان الملاحظة مطلوب" }, { status: 400 });
  const db = await getD1(); const timestamp = nowIso();
  if (db) await db.prepare("INSERT OR REPLACE INTO notes (id, owner_key, item_id, title, body, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, COALESCE((SELECT created_at FROM notes WHERE id = ?1), ?6), ?6)").bind(body.id ?? id("note"), result.session.ownerKey, body.itemId ?? null, title, body.body?.trim() ?? "", timestamp).run();
  return Response.json({ ok: true });
}
