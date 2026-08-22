import { requireSession } from "../../../../server/auth";

export async function POST(request: Request) {
  const result = await requireSession(); if ("response" in result) return result.response;
  const body = await request.json().catch(() => ({})) as { pack?: unknown };
  const pack = body.pack as { items?: unknown; sources?: unknown; version?: unknown } | null;
  const errors: string[] = [];
  if (!pack || typeof pack !== "object") errors.push("الحزمة يجب أن تكون كائن JSON");
  if (pack && pack.items !== undefined && !Array.isArray(pack.items)) errors.push("items يجب أن تكون مصفوفة");
  if (pack && pack.sources !== undefined && !Array.isArray(pack.sources)) errors.push("sources يجب أن تكون مصفوفة");
  for (const item of Array.isArray(pack?.items) ? pack.items : []) {
    const candidate = item as { id?: unknown; slug?: unknown; title?: unknown; sourceUrl?: unknown };
    if (!candidate.id || !candidate.slug || !candidate.title) errors.push("كل عنصر يحتاج id وslug وtitle");
    if (candidate.sourceUrl && (!String(candidate.sourceUrl).startsWith("https://"))) errors.push("مصادر العناصر يجب أن تستخدم HTTPS");
  }
  return Response.json({ ok: errors.length === 0, errors, data: { version: pack?.version ?? 1, items: Array.isArray(pack?.items) ? pack.items.length : 0, sources: Array.isArray(pack?.sources) ? pack.sources.length : 0 } }, { status: errors.length ? 400 : 200 });
}
