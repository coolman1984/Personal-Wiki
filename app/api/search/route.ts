import { getD1 } from "../../../server/data";
import { requireSession } from "../../../server/auth";

export const dynamic = "force-dynamic";

export async function GET(request: Request) {
  const result = await requireSession(); if ("response" in result) return result.response;
  const query = new URL(request.url).searchParams.get("q")?.trim() ?? "";
  if (!query) return Response.json({ ok: true, data: [] });
  const db = await getD1(); if (!db) return Response.json({ ok: true, data: [] });
  try {
    const rows = await db.prepare("SELECT i.id, i.title, i.description, i.syntax, i.tool_id FROM items i LEFT JOIN items_fts f ON f.item_id = i.id WHERE f.items_fts MATCH ?1 OR i.title LIKE ?2 OR i.description LIKE ?2 ORDER BY i.updated_at DESC LIMIT 50").bind(query.replace(/['\"]+/g, " "), `%${query}%`).all<Record<string, unknown>>();
    return Response.json({ ok: true, data: rows.results ?? [] });
  } catch { return Response.json({ ok: true, data: [] }); }
}
