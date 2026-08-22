/**
 * The single door between the UI and the trusted database.
 *
 * The frontend never opens SQLite and never mutates trusted rows directly: it
 * calls typed Tauri commands, which delegate to the shared Rust core.
 *
 * In a plain browser (Vite dev, Playwright) there is no Tauri runtime, so calls
 * are served by a fixture recorded from the real core — see tests/e2e/fixtures.
 * That path is test scaffolding only; it never ships inside the desktop app.
 */

type Invoke = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;

let invokeImpl: Invoke | null = null;

function hasTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export function isDesktop(): boolean {
  return hasTauri();
}

/** Test harnesses install their own transport here. */
export function setInvoke(fn: Invoke | null): void {
  invokeImpl = fn;
}

async function resolveInvoke(): Promise<Invoke> {
  if (invokeImpl) return invokeImpl;
  if (hasTauri()) {
    const { invoke } = await import('@tauri-apps/api/core');
    invokeImpl = invoke as Invoke;
    return invokeImpl;
  }
  const browserFixture = (window as unknown as { __PW_FIXTURE_INVOKE__?: Invoke }).__PW_FIXTURE_INVOKE__;
  if (browserFixture) {
    invokeImpl = browserFixture;
    return invokeImpl;
  }
  throw new Error(
    'Personal Wiki is not connected to its knowledge database. Open the desktop application.'
  );
}

export class IpcError extends Error {
  readonly command: string;
  constructor(command: string, message: string) {
    super(message);
    this.name = 'IpcError';
    this.command = command;
  }
}

export async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const invoke = await resolveInvoke();
  try {
    return (await invoke(command, args)) as T;
  } catch (error) {
    const message =
      typeof error === 'string' ? error : error instanceof Error ? error.message : String(error);
    throw new IpcError(command, message);
  }
}
