import { getChatGPTUser, type ChatGPTUser } from "../app/chatgpt-auth";

export type Session = { user: ChatGPTUser; ownerKey: string };

export async function getSession(): Promise<Session | null> {
  const user = await getChatGPTUser();
  if (!user && process.env.NODE_ENV !== "production") return { user: { displayName: "Demo", email: "demo@example.com", fullName: "Demo" }, ownerKey: "demo@example.com" };
  if (!user) return null;
  const ownerEmail = process.env.OWNER_EMAIL?.trim().toLowerCase();
  if (ownerEmail && user.email.toLowerCase() !== ownerEmail) return null;
  return { user, ownerKey: user.email.toLowerCase() };
}

export async function requireSession(): Promise<{ session: Session } | { response: Response }> {
  const session = await getSession();
  if (!session) return { response: Response.json({ ok: false, error: "تسجيل الدخول مطلوب أو الحساب غير مسموح." }, { status: 401 }) };
  return { session };
}
