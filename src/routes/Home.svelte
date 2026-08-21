<script lang="ts">
  import PanelLayout from '$components/ui/PanelLayout.svelte';
  import ToolCardView from '$components/cards/ToolCard.svelte';
  import TaskCardView from '$components/cards/TaskCard.svelte';
  import QuickActionCard from '$components/cards/QuickActionCard.svelte';
  import Icon from '$components/ui/Icon.svelte';
  import IconTile from '$components/ui/IconTile.svelte';
  import EmptyState from '$components/ui/EmptyState.svelte';
  import ErrorState from '$components/ui/ErrorState.svelte';
  import Skeleton from '$components/ui/Skeleton.svelte';
  import { getHome, importPackFile, pickPackFile } from '$lib/api';
  import { href, navigate } from '$lib/router.svelte';
  import { relativeTime } from '$lib/format';
  import { appState, guard, toasts } from '$lib/state.svelte';
  import { t } from '$lib/i18n';
  import type { HomeSummary } from '$lib/types';

  let data = $state<HomeSummary | null>(null);
  let error = $state<string | null>(null);

  async function load() {
    error = null;
    try {
      data = await getHome();
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
  }
  load();

  async function importPack() {
    const path = await guard(() => pickPackFile(), 'تعذّر فتح الملف');
    if (!path) return;
    const result = await guard(() => importPackFile(path, false), 'تعذّر استيراد الحزمة');
    if (!result) return;
    toasts.push(`تم تجهيز ${result.staged} تغييرًا للمراجعة`);
    await appState.refresh();
    navigate(`/updates/batches/${result.batchId}`);
  }
</script>

<PanelLayout>
  <section class="hero">
    <div class="hero-text">
      <span class="spark"><Icon name="sparkle" size={26} /></span>
      <h1>{t.home.heroTitle}</h1>
      <p>{t.home.heroSubtitle}</p>
    </div>
    <svg class="hero-art" viewBox="0 0 320 180" aria-hidden="true">
      <circle cx="90" cy="52" r="5" fill="var(--accent-blue)" opacity="0.55" />
      <circle cx="150" cy="28" r="4" fill="var(--accent-purple)" opacity="0.5" />
      <circle cx="52" cy="118" r="4" fill="var(--accent-teal)" opacity="0.5" />
      <circle cx="128" cy="132" r="6" fill="var(--accent-blue)" opacity="0.35" />
      <path d="M90 52 150 28M90 52 52 118M90 52 128 132" stroke="var(--accent-blue)" stroke-opacity="0.28" stroke-width="1.5" />
      <rect x="188" y="34" width="112" height="118" rx="14" fill="var(--surface)" stroke="var(--border)" />
      <rect x="204" y="56" width="80" height="8" rx="4" fill="var(--accent-blue)" opacity="0.35" />
      <rect x="204" y="76" width="62" height="8" rx="4" fill="var(--border-strong)" />
      <rect x="204" y="96" width="72" height="8" rx="4" fill="var(--border-strong)" />
      <rect x="204" y="116" width="50" height="8" rx="4" fill="var(--border-strong)" />
    </svg>
  </section>

  {#if error}
    <ErrorState title={t.errors.database.title} body={t.errors.database.body} detail={error} onRetry={load} />
  {:else if !data}
    <div class="grid tools"><Skeleton variant="card" count={8} /></div>
  {:else if data.tools.length === 0}
    <EmptyState
      icon="grid"
      title={t.empty.tools.title}
      body={t.empty.tools.body}
      actionLabel={t.home.quickImport.title}
      onAction={importPack}
    />
  {:else}
    {#if data.tasks.length > 0}
      <section>
        <div class="section-head">
          <h2 class="section-title">{t.home.tasksTitle}</h2>
          <a href={href('/tasks')}>{t.home.viewAll}</a>
        </div>
        <div class="grid tasks">
          {#each data.tasks.slice(0, 6) as task (task.id)}
            <TaskCardView {task} />
          {/each}
        </div>
      </section>
    {/if}

    <section>
      <div class="section-head">
        <h2 class="section-title">{t.home.toolsTitle}</h2>
        <a href={href('/tools')}>{t.home.viewAll}</a>
      </div>
      <div class="grid tools" data-testid="tool-grid">
        {#each data.tools as tool (tool.id)}
          <ToolCardView {tool} />
        {/each}
      </div>
    </section>

    <a class="all-sections" href={href('/tools')}>
      <Icon name="grid" size={16} />
      <span>{t.home.allSections}</span>
    </a>
  {/if}

  {#snippet panel()}
    <section class="card panel-card">
      <div class="panel-head">
        <h2>{t.home.recentUpdates}</h2>
        <a href={href('/updates')}>{t.home.viewAll}</a>
      </div>
      {#if !data}
        <Skeleton variant="line" count={4} />
      {:else if data.recentActivity.length === 0}
        <EmptyState compact icon="bell" title={t.empty.activity.title} body={t.empty.activity.body} />
      {:else}
        <ul class="activity">
          {#each data.recentActivity as entry (entry.id)}
            <li>
              <span class="dot {entry.kind}"></span>
              <span class="text">
                <span class="title">{entry.title}</span>
                <span class="time">{relativeTime(entry.at)}</span>
              </span>
              {#if entry.itemSlug}
                <a class="go" href={href(`/items/${entry.itemSlug}`)} aria-label={entry.title}>
                  <Icon name="chevron" size={14} directional />
                </a>
              {/if}
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <section class="card panel-card">
      <div class="panel-head">
        <h2>{t.home.recentItems}</h2>
      </div>
      {#if data && data.recentItems.length > 0}
        <ul class="recent">
          {#each data.recentItems as item (item.id)}
            <li>
              <a href={href(`/items/${item.slug}`)}>
                <IconTile icon={item.iconKey ?? 'terminal'} accent={item.accentKey ?? 'blue'} size={32} />
                <span class="text">
                  <span class="title">{item.title}</span>
                  <span class="time">{relativeTime(item.updatedAt)}</span>
                </span>
                <Icon name="chevron" size={14} directional />
              </a>
            </li>
          {/each}
        </ul>
      {:else}
        <p class="none">ستظهر هنا العناصر التي تفتحها.</p>
      {/if}
    </section>

    <section class="card panel-card">
      <div class="panel-head"><h2>{t.home.quickActions}</h2></div>
      <div class="quick-grid">
        <QuickActionCard
          icon="refresh"
          accent="blue"
          title={t.home.quickUpdate.title}
          subtitle={t.home.quickUpdate.subtitle}
          onClick={() => navigate('/updates')}
        />
        <QuickActionCard
          icon="upload"
          accent="purple"
          title={t.home.quickImport.title}
          subtitle={t.home.quickImport.subtitle}
          onClick={importPack}
        />
        <QuickActionCard
          icon="plus"
          accent="green"
          title={t.home.quickAddLink.title}
          subtitle={t.home.quickAddLink.subtitle}
          onClick={() => navigate('/links')}
        />
      </div>
    </section>
  {/snippet}
</PanelLayout>

<style>
  .hero {
    display: flex;
    align-items: center;
    gap: var(--space-6);
    padding: var(--space-7);
    border-radius: var(--radius-xl);
    background: linear-gradient(115deg, var(--surface) 40%, var(--accent-blue-soft) 100%);
    border: 1px solid var(--border);
    overflow: hidden;
  }
  .hero-text { flex: 1; min-inline-size: 0; }
  .spark { color: var(--accent-purple); display: block; margin-block-end: var(--space-2); }
  .hero h1 { margin: 0; font-size: 30px; font-weight: 700; line-height: 1.3; }
  .hero p { margin: var(--space-3) 0 0; color: var(--text-secondary); max-inline-size: 62ch; }
  .hero-art { inline-size: 320px; block-size: 180px; flex: none; }

  .section-head { display: flex; align-items: baseline; justify-content: space-between; margin-block-end: var(--space-3); }
  .section-head a { font-size: 13px; color: var(--accent-blue); text-decoration: none; }

  .grid { display: grid; gap: var(--space-4); }
  .tools { grid-template-columns: repeat(auto-fill, minmax(232px, 1fr)); }
  .tasks { grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); }

  .all-sections {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    padding: var(--space-4);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    color: var(--accent-blue);
    text-decoration: none;
    font-weight: 600;
    font-size: 14px;
  }
  .all-sections:hover { border-color: var(--accent-blue); background: var(--accent-blue-soft); }

  .panel-card { padding: var(--space-4); display: flex; flex-direction: column; gap: var(--space-3); }
  .panel-head { display: flex; align-items: baseline; justify-content: space-between; }
  .panel-head h2 { margin: 0; font-size: 14px; font-weight: 600; }
  .panel-head a { font-size: 12px; color: var(--accent-blue); text-decoration: none; }

  .activity, .recent { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: var(--space-2); }
  .activity li { display: flex; align-items: center; gap: var(--space-2); }
  .activity .dot { inline-size: 7px; block-size: 7px; border-radius: 50%; background: var(--accent-slate); flex: none; }
  .activity .dot.add { background: var(--success); }
  .activity .dot.update { background: var(--info); }
  .activity .dot.remove_candidate { background: var(--danger); }
  .text { flex: 1; min-inline-size: 0; display: flex; flex-direction: column; }
  .title { font-size: 12.5px; font-weight: 500; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .time { font-size: 11px; color: var(--text-tertiary); }
  .go { color: var(--text-tertiary); }
  .recent a {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    text-decoration: none;
    color: inherit;
    padding: 4px;
    border-radius: var(--radius-sm);
  }
  .recent a:hover { background: var(--surface-muted); }
  .none { margin: 0; font-size: 12.5px; color: var(--text-tertiary); }
  .quick-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: var(--space-2); }

  @media (max-width: 900px) {
    .hero-art { display: none; }
    .hero h1 { font-size: 24px; }
  }
</style>
