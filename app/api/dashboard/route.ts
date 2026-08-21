import { dashboardForOwner } from "../../../server/data";
import { requireSession } from "../../../server/auth";

export const dynamic = "force-dynamic";

export async function GET() {
  const result = await requireSession();
  if ("response" in result) return result.response;
  return Response.json({ ok: true, data: await dashboardForOwner(result.session.ownerKey) });
}
