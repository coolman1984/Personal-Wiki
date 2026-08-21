import { getD1 } from "../../../server/data";
import { requireSession } from "../../../server/auth";

export const dynamic = "force-dynamic";

export async function GET() {
  const result = await requireSession(); if ("response" in result) return result.response;
  const db = await getD1(); if (!db) return Response.json({ ok: true, data: [] });
  const rows = await db.prepare("SELECT c.id, c.batch_id, c.entity_type, c.entity_key, c.change_type, c.before_json, c.after_json, c.status, c.detected_at FROM staged_changes c JOIN update_batches b ON b.id = c.batch_id WHERE b.owner_key = ?1 ORDER BY c.detected_at DESC LIMIT 100").bind(result.session.ownerKey).all<Record<string, unknown>>();
  return Response.json({ ok: true, data: rows.results ?? [] });
}
