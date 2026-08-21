import { getD1, nowIso } from "../../../server/data";
import { requireSession } from "../../../server/auth";

export async function POST(request: Request) {
  const result = await requireSession(); if ("response" in result) return result.response;
  const body = await request.json().catch(() => ({})) as { itemId?: string; favorite?: boolean };
  if (!body.itemId) return Response.json({ ok: false, error: "itemId مطلوب" }, { status: 400 });
  const db = await getD1();
  if (db) { if (body.favorite === false) await db.prepare("DELETE FROM favorites WHERE owner_key = ?1 AND item_id = ?2").bind(result.session.ownerKey, body.itemId).run(); else await db.prepare("INSERT OR IGNORE INTO favorites (owner_key, item_id, created_at) VALUES (?1, ?2, ?3)").bind(result.session.ownerKey, body.itemId, nowIso()).run(); }
  return Response.json({ ok: true });
}
