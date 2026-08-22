/**
 * Small shared UI state. Everything that is *knowledge* comes from the database
 * on demand; this holds only cross-screen chrome state.
 */

import type { CategoryNode, ToolCard } from './types';
import { getCategoryTree, getHome, listTools } from './api';

class AppState {
  pendingChanges = $state(0);
  lastSyncAt = $state<string | null>(null);
  lastSyncStatus = $state<string | null>(null);
  tools = $state<ToolCard[]>([]);
  categories = $state<CategoryNode[]>([]);
  loaded = $state(false);
  loadError = $state<string | null>(null);

  get syncTone(): 'ok' | 'warn' | 'error' {
    if (this.lastSyncStatus === 'failed') return 'error';
    if (this.lastSyncStatus === 'running' || this.lastSyncStatus === 'partial') return 'warn';
    return 'ok';
  }

  /** Refreshed after any action that can change counts (import, apply, undo). */
  async refresh(): Promise<void> {
    try {
      const [home, tools, categories] = await Promise.all([getHome(), listTools(), getCategoryTree()]);
      this.pendingChanges = home.pendingChanges;
      this.lastSyncAt = home.lastSyncAt;
      this.lastSyncStatus = home.lastSyncStatus;
      this.tools = tools;
      this.categories = categories;
      this.loadError = null;
    } catch (error) {
      this.loadError = error instanceof Error ? error.message : String(error);
    } finally {
      this.loaded = true;
    }
  }
}

export const appState = new AppState();

/** Toasts are used only for failures and for confirmations the user must notice. */
class Toasts {
  items = $state<Array<{ id: number; text: string; tone: 'ok' | 'error' }>>([]);
  #next = 1;

  push(text: string, tone: 'ok' | 'error' = 'ok') {
    const id = this.#next++;
    this.items = [...this.items, { id, text, tone }];
    setTimeout(() => this.dismiss(id), tone === 'error' ? 6000 : 3000);
  }

  dismiss(id: number) {
    this.items = this.items.filter((item) => item.id !== id);
  }
}

export const toasts = new Toasts();

/** Wrap an async action so failures surface instead of disappearing. */
export async function guard<T>(action: () => Promise<T>, failureMessage?: string): Promise<T | null> {
  try {
    return await action();
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    toasts.push(failureMessage ? `${failureMessage}: ${detail}` : detail, 'error');
    return null;
  }
}
