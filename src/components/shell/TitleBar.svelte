<script lang="ts">
  import Icon from '$components/ui/Icon.svelte';
  import { t } from '$lib/i18n';
  import { navigate } from '$lib/router.svelte';
  import { isDesktop } from '$lib/ipc';
  import { appState } from '$lib/state.svelte';

  let { query = '', onSearchSubmit }: { query?: string; onSearchSubmit: (value: string) => void } = $props();

  // Seeded from the route on mount and re-synced by the effect below.
  let value = $state('');
  let input: HTMLInputElement | null = $state(null);

  $effect(() => {
    value = query;
  });

  export function focusSearch() {
    input?.focus();
    input?.select();
  }

  async function windowAction(action: 'minimize' | 'toggleMaximize' | 'close') {
    if (!isDesktop()) return;
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    const win = getCurrentWindow();
    if (action === 'minimize') await win.minimize();
    else if (action === 'toggleMaximize') await win.toggleMaximize();
    else await win.close();
  }
</script>

<header class="titlebar" data-tauri-drag-region>
  <a class="brand" href="#/" onclick={() => navigate('/')}>
    <span class="logo"><Icon name="book" size={20} /></span>
    <strong>{t.app.name}</strong>
  </a>

  <form
    class="search"
    role="search"
    onsubmit={(event) => {
      event.preventDefault();
      onSearchSubmit(value);
    }}
  >
    <Icon name="search" size={18} />
    <input
      bind:this={input}
      bind:value
      type="search"
      placeholder={t.topbar.searchPlaceholder}
      aria-label={t.topbar.searchPlaceholder}
      data-testid="global-search"
      autocomplete="off"
    />
    <kbd class="ltr">{t.topbar.shortcut}</kbd>
  </form>

  <div class="actions">
    <button type="button" class="action" onclick={() => navigate('/updates')} data-testid="nav-updates-action">
      <span class="glyph">
        <Icon name="refresh" size={20} />
        {#if appState.pendingChanges > 0}<span class="dot info" aria-hidden="true"></span>{/if}
      </span>
      <span class="caption">{t.topbar.updates}</span>
    </button>
    <button type="button" class="action" onclick={() => navigate('/updates')}>
      <span class="glyph">
        <Icon name="cloud" size={20} />
        <span class="dot {appState.syncTone}" aria-hidden="true"></span>
      </span>
      <span class="caption">{t.topbar.sync}</span>
    </button>
    <button type="button" class="action" onclick={() => navigate('/settings')}>
      <span class="glyph"><Icon name="settings" size={20} /></span>
      <span class="caption">{t.topbar.settings}</span>
    </button>

    {#if isDesktop()}
      <div class="window-controls">
        <button type="button" onclick={() => windowAction('minimize')} aria-label={t.topbar.minimize}>
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true"><path d="M0 5h10" stroke="currentColor" /></svg>
        </button>
        <button type="button" onclick={() => windowAction('toggleMaximize')} aria-label={t.topbar.maximize}>
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true"><rect x="0.5" y="0.5" width="9" height="9" fill="none" stroke="currentColor" /></svg>
        </button>
        <button type="button" class="close" onclick={() => windowAction('close')} aria-label={t.topbar.close}>
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true"><path d="M0 0l10 10M10 0L0 10" stroke="currentColor" /></svg>
        </button>
      </div>
    {/if}
  </div>
</header>

<style>
  .titlebar {
    block-size: var(--titlebar-height);
    display: flex;
    align-items: center;
    gap: var(--space-5);
    padding-inline: var(--space-5);
    background: var(--surface);
    border-block-end: 1px solid var(--border);
    position: relative;
    z-index: 20;
  }
  .brand { display: inline-flex; align-items: center; gap: var(--space-3); text-decoration: none; color: inherit; flex: none; }
  .logo {
    inline-size: 34px; block-size: 34px;
    border-radius: 10px;
    background: var(--accent-blue-soft);
    color: var(--accent-blue);
    display: grid; place-items: center;
  }
  .brand strong { font-size: 16px; font-weight: 600; }

  .search {
    flex: 1 1 auto;
    max-inline-size: 640px;
    margin-inline: auto;
    display: flex;
    align-items: center;
    gap: var(--space-3);
    block-size: 46px;
    padding-inline: var(--space-4);
    background: var(--surface-muted);
    border: 1px solid transparent;
    border-radius: var(--radius-lg);
    color: var(--text-tertiary);
    transition: border-color var(--dur-fast) var(--ease), background var(--dur-fast) var(--ease);
  }
  .search:focus-within { background: var(--surface); border-color: var(--accent-blue); box-shadow: 0 0 0 3px var(--accent-blue-soft); }
  .search input {
    flex: 1;
    min-inline-size: 0;
    background: transparent;
    border: none;
    outline: none;
    font-size: 14px;
    color: var(--text-primary);
  }
  .search input::placeholder { color: var(--text-tertiary); }
  kbd {
    font-size: 11px;
    color: var(--text-tertiary);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 2px 7px;
  }

  .actions { display: flex; align-items: center; gap: var(--space-2); flex: none; }
  .action {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    border-radius: var(--radius-md);
    padding: 4px 10px;
  }
  .action:hover { background: var(--surface-muted); color: var(--text-primary); }
  .glyph { position: relative; display: grid; place-items: center; }
  .caption { font-size: 11px; }
  .dot {
    position: absolute;
    inset-block-start: -2px;
    inset-inline-end: -3px;
    inline-size: 8px; block-size: 8px;
    border-radius: 50%;
    border: 2px solid var(--surface);
  }
  .dot.info { background: var(--info); }
  .dot.ok { background: var(--success); }
  .dot.warn { background: var(--warning); }
  .dot.error { background: var(--danger); }

  .window-controls { display: flex; margin-inline-start: var(--space-3); }
  .window-controls button {
    inline-size: 44px; block-size: 32px;
    display: grid; place-items: center;
    background: transparent; border: none;
    color: var(--text-secondary);
  }
  .window-controls button:hover { background: var(--surface-muted); }
  .window-controls .close:hover { background: var(--danger); color: var(--text-inverse); }

  @media (max-width: 1100px) {
    .caption { display: none; }
  }
</style>
