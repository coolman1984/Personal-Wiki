import { getD1 } from "../../../server/data";
import { requireSession } from "../../../server/auth";
import { getR2 } from "../../../server/storage";

export async function GET() {
  const result = await requireSession(); if ("response" in result) return result.response;
  const db = await getD1(); const bucket = await getR2(); let tables = false; let fts = false;
  if (db) { try { await db.prepare("SELECT 1 FROM items LIMIT 1").first(); tables = true; } catch { /* migration pending */ } try { await db.prepare("SELECT 1 FROM items_fts LIMIT 1").first(); fts = true; } catch { /* optional search index */ } }
  return Response.json({ ok: true, data: { auth: true, d1: Boolean(db) && tables, r2: Boolean(bucket), fts, owner: result.session.user.email } });
}
