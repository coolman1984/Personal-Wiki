import { getD1, nowIso } from "../../../../server/data";
import { requireSession } from "../../../../server/auth";

export async function POST(request: Request) {
  const result = await requireSession(); if ("response" in result) return result.response;
  const body = await request.json().catch(() => ({})) as { changeIds?: string[] };
  const ids = Array.isArray(body.changeIds) ? body.changeIds.filter(Boolean).slice(0, 200) : [];
  const db = await getD1(); if (db && ids.length) {
    const marks = ids.map(() => "?").join(",");
    await db.prepare(`UPDATE staged_changes SET status = 'approved', reviewed_at = ?1 WHERE id IN (${marks}) AND status = 'pending' AND batch_id IN (SELECT id FROM update_batches WHERE owner_key = ?${ids.length + 2})`).bind(nowIso(), ...ids, result.session.ownerKey).run();
  }
  return Response.json({ ok: true, approved: ids.length });
}
