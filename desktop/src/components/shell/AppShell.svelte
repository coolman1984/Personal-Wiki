<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from '$components/ui/Icon.svelte';
  import NavTree from './NavTree.svelte';
  import TitleBar from './TitleBar.svelte';
  import { t } from '$lib/i18n';
  import { href, navigate } from '$lib/router.svelte';
  import { relativeTime } from '$lib/format';
  import { appState, toasts } from '$lib/state.svelte';
  import type { Route } from '$lib/router.svelte';

  let { route, children }: { route: Route; children: Snippet } = $props();

  let titleBar = $state<{ focusSearch: () => void } | null>(null);

  const NAV: Array<{ path: string; label: string; icon: string; match: Route['name'][] }> = [
    { path: '/', label: t.nav.home, icon: 'home', match: ['home'] },
    { path: '/tools', label: t.nav.tools, icon: 'grid', match: ['tools', 'tool'] },
    { path: '/tasks', label: t.nav.tasks, icon: 'terminal', match: ['tasks', 'task'] },
    { path: '/links', label: t.nav.links, icon: 'link', match: ['links'] },
    { path: '/favorites', label: t.nav.favorites, icon: 'star', match: ['favorites'] },
    { path: '/notes', label: t.nav.notes, icon: 'note', match: ['notes'] },
    { path: '/updates', label: t.nav.updates, icon: 'bell', match: ['updates', 'batch'] },
    { path: '/settings', label: t.nav.settings, icon: 'settings', match: ['settings'] }
  ];

  const syncLabel = $derived(
    appState.lastSyncStatus === 'failed'
      ? t.sync.failed
      : appState.lastSyncStatus === 'running'
        ? t.sync.running
        : appState.lastSyncAt
          ? t.sync.synced
          : t.sync.never
  );

  function onKeydown(event: KeyboardEvent) {
    const target = event.target as HTMLElement | null;
    const typing = target && ['INPUT', 'TEXTAREA'].includes(target.tagName);
    if ((event.ctrlKey || event.metaKey) && (event.key === 'k' || event.key === 'f')) {
      event.preventDefault();
      titleBar?.focusSearch();
      return;
    }
    if (event.altKey && (event.key === 'ArrowRight' || event.key === 'ArrowLeft')) {
      // RTL: ArrowRight is "back" visually; history order stays logical.
      event.preventDefault();
      if (event.key === 'ArrowRight') window.history.back();
      else window.history.forward();
      return;
    }
    if (event.key === '/' && !typing) {
      event.preventDefault();
      titleBar?.focusSearch();
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="shell">
  <TitleBar
    bind:this={titleBar}
    query={route.name === 'search' ? (route.query.get('q') ?? '') : ''}
    onSearchSubmit={(value) => navigate('/search', { q: value })}
  />

  <div class="body">
    <nav class="rail" aria-label="التنقل الرئيسي">
      <ul class="primary">
        {#each NAV as entry (entry.path)}
          <li>
            <a
              class="nav-item"
              class:active={entry.match.includes(route.name)}
              href={href(entry.path)}
              data-testid={`nav-${entry.path === '/' ? 'home' : entry.path.slice(1)}`}
            >
              <Icon name={entry.icon} size={19} />
              <span>{entry.label}</span>
              {#if entry.path === '/updates' && appState.pendingChanges > 0}
                <span class="pending ltr">{appState.pendingChanges}</span>
              {/if}
            </a>
          </li>
        {/each}
      </ul>

      {#if appState.categories.length > 0}
        <div class="tree-section">
          <span class="tree-title">الأقسام</span>
          <NavTree nodes={appState.categories} activeSlug={route.params.slug ?? null} />
        </div>
      {/if}

      <div class="rail-footer">
        <div class="sync-card">
          <span class="sync-state">
            <span class="dot {appState.syncTone}"></span>
            {syncLabel}
          </span>
          <span class="sync-time">
            <Icon name="cloud" size={14} />
            {t.sync.last}: {relativeTime(appState.lastSyncAt)}
          </span>
        </div>
        <a class="version" href={href('/diagnostics')}>
          <span class="ltr">v1.0.0</span>
          <Icon name="chevron" size={14} directional />
        </a>
      </div>
    </nav>

    <main class="content" data-testid="route-content">
      {@render children()}
    </main>

  </div>

  {#if toasts.items.length > 0}
    <div class="toasts" role="status" aria-live="polite">
      {#each toasts.items as toast (toast.id)}
        <div class="toast {toast.tone}">
          <Icon name={toast.tone === 'ok' ? 'check' : 'warning'} size={16} />
          <span>{toast.text}</span>
          <button type="button" onclick={() => toasts.dismiss(toast.id)} aria-label={t.common.close}>
            <Icon name="close" size={14} />
          </button>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .shell { block-size: 100%; display: flex; flex-direction: column; background: var(--background); }
  .body { flex: 1; display: flex; min-block-size: 0; }

  .rail {
    inline-size: var(--rail-width);
    flex: none;
    background: var(--surface);
    display: flex;
    flex-direction: column;
    padding: var(--space-4) var(--space-3);
    gap: var(--space-4);
    overflow-y: auto;
    border-inline-end: 1px solid var(--border);
  }
  .primary { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: var(--space-1); }
  .nav-item {
    position: relative;
    display: flex;
    align-items: center;
    gap: var(--space-3);
    block-size: 44px;
    padding-inline: var(--space-3);
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    text-decoration: none;
    font-size: 14px;
  }
  .nav-item:hover { background: var(--surface-muted); color: var(--text-primary); }
  .nav-item.active { background: var(--accent-blue-soft); color: var(--accent-blue); font-weight: 600; }
  .nav-item.active::before {
    content: '';
    position: absolute;
    inset-inline-start: 0;
    inset-block: 10px;
    inline-size: 3px;
    border-radius: 999px;
    background: var(--accent-blue);
  }
  .pending {
    margin-inline-start: auto;
    background: var(--accent-blue);
    color: var(--text-inverse);
    border-radius: 999px;
    font-size: 11px;
    padding: 1px 7px;
  }

  .tree-section { display: flex; flex-direction: column; gap: var(--space-2); }
  .tree-title { font-size: 11px; font-weight: 600; color: var(--text-tertiary); padding-inline: var(--space-3); }

  .rail-footer { margin-block-start: auto; display: flex; flex-direction: column; gap: var(--space-2); }
  .sync-card {
    background: var(--surface-muted);
    border-radius: var(--radius-md);
    padding: var(--space-3);
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 12px;
  }
  .sync-state { display: flex; align-items: center; gap: var(--space-2); font-weight: 600; }
  .sync-time { display: flex; align-items: center; gap: var(--space-2); color: var(--text-tertiary); }
  .dot { inline-size: 8px; block-size: 8px; border-radius: 50%; flex: none; }
  .dot.ok { background: var(--success); }
  .dot.warn { background: var(--warning); }
  .dot.error { background: var(--danger); }
  .version {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-2) var(--space-3);
    color: var(--text-tertiary);
    font-size: 12px;
    text-decoration: none;
    border-radius: var(--radius-md);
  }
  .version:hover { background: var(--surface-muted); color: var(--text-primary); }

  .content { flex: 1; min-inline-size: 0; overflow-y: auto; padding: var(--space-6); }
  @media (max-width: 1024px) {
    .rail { inline-size: 68px; }
    .nav-item span, .tree-section, .sync-time, .version .ltr { display: none; }
    .nav-item { justify-content: center; }
  }

  .toasts {
    position: fixed;
    inset-block-end: var(--space-5);
    inset-inline-start: 50%;
    transform: translateX(50%);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    z-index: 80;
  }
  .toast {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    background: var(--surface);
    border: 1px solid var(--border);
    box-shadow: var(--shadow-md);
    border-radius: var(--radius-md);
    padding: 10px 14px;
    font-size: 13px;
    max-inline-size: 560px;
  }
  .toast.ok { border-inline-start: 3px solid var(--success); }
  .toast.error { border-inline-start: 3px solid var(--danger); }
  .toast button { background: transparent; border: none; color: var(--text-tertiary); padding: 2px; }
</style>
