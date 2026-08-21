import { getD1, id, nowIso } from "../../../server/data";
import { requireSession } from "../../../server/auth";

export async function POST(request: Request) {
  const result = await requireSession(); if ("response" in result) return result.response;
  const body = await request.json().catch(() => ({})) as { id?: string; title?: string; url?: string; description?: string; toolId?: string; favorite?: boolean };
  const title = body.title?.trim(); const url = body.url?.trim();
  if (!title || !url || !/^https:\/\//i.test(url)) return Response.json({ ok: false, error: "رابط HTTPS صالح وعنوان مطلوبان" }, { status: 400 });
  const db = await getD1(); const timestamp = nowIso();
  if (db) await db.prepare("INSERT OR REPLACE INTO saved_links (id, owner_key, title, url, description, tool_id, is_favorite, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, COALESCE(?7, 0), COALESCE((SELECT created_at FROM saved_links WHERE id = ?1), ?8), ?8)").bind(body.id ?? id("link"), result.session.ownerKey, title, url, body.description?.trim() ?? "", body.toolId ?? null, body.favorite ? 1 : 0, timestamp).run();
  return Response.json({ ok: true });
}
