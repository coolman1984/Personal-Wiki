import { redirect } from "next/navigation";
import { chatGPTSignInPath, getChatGPTUser } from "./chatgpt-auth";
import WikiClient from "./wiki-client";

export const dynamic = "force-dynamic";

export default async function Home() {
  const user = await getChatGPTUser();
  if (!user && process.env.NODE_ENV === "production") redirect(chatGPTSignInPath("/"));
  return <WikiClient userEmail={user?.email ?? "demo@example.com"} />;
}
