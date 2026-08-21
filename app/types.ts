export type Accent = "blue" | "green" | "purple" | "orange" | "teal" | "slate";
export type View = "home" | "tools" | "tasks" | "search" | "favorites" | "notes" | "links" | "updates" | "settings" | "diagnostics";

export type Tool = { id: string; slug: string; name: string; description: string; icon: string; accent: Accent; itemCount: number; exampleCount: number; updated: string };
export type Task = { id: string; slug: string; name: string; description: string; icon: string; accent: Accent; itemCount: number; toolCount: number };
export type Source = { id: string; name: string; host: string; url: string; type: string; official: boolean; status: "connected" | "warning" | "unchecked" };
export type Example = { id: string; title: string; description: string; code: string; expected?: string };
export type KnowledgeItem = {
  id: string; slug: string; title: string; subtitle: string; toolId: string; toolName: string; toolIcon: string;
  categoryName: string; type: string; description: string; whenToUse: string; syntax?: string; expected?: string;
  warning?: string; verification: "official" | "verified" | "community" | "unverified"; lifecycle: "active" | "experimental" | "deprecated" | "removed";
  tags: string[]; aliases: string[]; examples: Example[]; source?: Source; favorite: boolean; pinned: boolean;
};
export type Change = { id: string; batchId: string; type: "add" | "update" | "remove"; title: string; detail: string; time: string; status: "pending" | "approved" | "rejected" | "applied"; before?: Record<string, unknown>; after?: Record<string, unknown> };
export type Note = { id: string; title: string; body: string; itemId?: string; updated: string };
export type Link = { id: string; title: string; url: string; description: string; favorite: boolean; toolId?: string };
export type Dashboard = { tools: Tool[]; tasks: Task[]; items: KnowledgeItem[]; sources: Source[]; changes: Change[]; notes: Note[]; links: Link[]; activity: { id: string; title: string; detail: string; kind: "add" | "update" | "remove" | "sync"; time: string }[] };
