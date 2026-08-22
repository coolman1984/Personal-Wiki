export type R2Object = { text: () => Promise<string>; arrayBuffer: () => Promise<ArrayBuffer> };
export type R2Binding = { put: (key: string, value: string, options?: unknown) => Promise<unknown>; get: (key: string) => Promise<R2Object | null>; list: (options?: unknown) => Promise<{ objects: { key: string; uploaded?: Date; size?: number }[] }> };

export async function getR2(): Promise<R2Binding | null> { try { const runtime = await import("cloudflare:workers"); return ((runtime.env as unknown as { BUCKET?: R2Binding }).BUCKET ?? null); } catch { return null; } }
