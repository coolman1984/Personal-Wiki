<script lang="ts">
  import Breadcrumbs from '$components/ui/Breadcrumbs.svelte';
  import PageHeader from '$components/ui/PageHeader.svelte';
  import StatTile from '$components/ui/StatTile.svelte';
  import ItemCard from '$components/cards/ItemCard.svelte';
  import LinkCard from '$components/cards/LinkCard.svelte';
  import KnowledgeDetail from '$components/detail/KnowledgeDetail.svelte';
  import EmptyState from '$components/ui/EmptyState.svelte';
  import ErrorState from '$components/ui/ErrorState.svelte';
  import Skeleton from '$components/ui/Skeleton.svelte';
  import Icon from '$components/ui/Icon.svelte';
  import {
    getCategoryDetail, getCategoryPath, getItem, getTaskDetail, itemsForCategory,
    itemsForTask, listLinks, recordOpen
  } from '$lib/api';
  import { navigate, replace, type Route } from '$lib/router.svelte';
  import { count } from '$lib/format';
  import { t } from '$lib/i18n';
  import type { Category, ItemDetail, ItemSummary, SavedLink, Task } from '$lib/types';

  /**
   * The tool-independent view (reference 06). A task page and a category page are
   * the same screen with a different source query — same cards, same detail
   * component, no forked layout.
   */
  let { kind, slug, route }: { kind: 'task' | 'category'; slug: string; route: Route } = $props();

  let heading = $state<Task | Category | null>(null);
  let path = $state<Category[]>([]);
  let items = $state<ItemSummary[] | null>(null);
  let links = $state<SavedLink[]>([]);
  let selected = $state<ItemDetail | null>(null);
  let error = $state<string | null>(null);

  const selectedSlug = $derived(route.query.get('item'));

  async function load() {
    error = null;
    heading = null;
    items = null;
    try {
      if (kind === 'task') {
        heading = await getTaskDetail(slug);
        path = [];
        items = await itemsForTask(slug, 'recent');
      } else {
        heading = await getCategoryDetail(slug);
        path = await getCategoryPath(slug);
        items = await itemsForCategory(slug, 'recent');
      }
      links = (await listLinks()).slice(0, 4);
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
  }

  async function select(item: ItemSummary) {
    replace(kind === 'task' ? `/tasks/${slug}` : `/c/${slug}`, { item: item.slug });
  }

  $effect(() => {
    void kind;
    void slug;
    load();
  });

  $effect(() => {
    const wanted = selectedSlug;
    if (!wanted) {
      selected = null;
      return;
    }
    getItem(wanted)
      .then(async (detail) => {
        selected = detail;
        await recordOpen(detail.id);
      })
      .catch((err) => {
        error = err instanceof Error ? err.message : String(err);
      });
  });

  const exampleCount = $derived((items ?? []).filter((i) => i.hasExamples).length);
  const crumbs = $derived([
    { label: t.nav.home, path: '/', icon: 'home' },
    ...(kind === 'task'
      ? [{ label: t.home.tasksTitle, path: '/tasks' }]
      : path.slice(0, -1).map((c) => ({ label: c.name, path: `/c/${c.slug}` }))),
    ...(heading ? [{ label: heading.name }] : [])
  ]);
</script>

{#if error && !heading}
  <ErrorState title={t.errors.database.title} detail={error} onRetry={load} />
{:else if !heading}
  <div class="loading"><Skeleton variant="line" count={2} /><Skeleton variant="card" count={3} /></div>
{:else}
  <div class="page" class:with-detail={Boolean(selected)}>
    <div class="main">
      <Breadcrumbs {crumbs} />

      <PageHeader
        centered
        icon={'iconKey' in heading ? heading.iconKey : 'folder'}
        accent={'accentKey' in heading ? (heading as Category).accentKey : 'blue'}
        title={heading.name}
        subtitle={heading.description}
      />

      <div class="stats">
        <StatTile icon="terminal" accent="blue" value={count(items?.length ?? 0)} label={t.common.commandsCount('')} />
        <StatTile icon="code" accent="purple" value={count(exampleCount)} label={t.tool.examples} />
        <StatTile icon="link" accent="green" value={count(links.length)} label={t.nav.links} />
        <StatTile
          icon="note"
          accent="orange"
          value={count((items ?? []).filter((i) => i.isFavorite).length)}
          label={t.nav.favorites}
        />
      </div>

      <section>
        <div class="section-head">
          <h2 class="section-title">الأوامر الشائعة</h2>
        </div>
        {#if !items}
          <div class="grid"><Skeleton variant="card" count={6} /></div>
        {:else if items.length === 0}
          <EmptyState icon="terminal" title={t.empty.items.title} body={t.empty.items.body} />
        {:else}
          <div class="grid" data-testid="item-grid">
            {#each items as item (item.id)}
              <div
                class="cell"
                class:selected={selected?.slug === item.slug}
                role="button"
                tabindex="0"
                onclick={() => select(item)}
                onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && select(item)}
              >
                <ItemCard {item} iconAtEnd onChanged={load} />
              </div>
            {/each}
          </div>
        {/if}
      </section>

      {#if links.length > 0}
        <section>
          <div class="section-head">
            <h2 class="section-title">روابط مهمة</h2>
            <a href="#/links">{t.home.viewAll}</a>
          </div>
          <div class="links">
            {#each links as link (link.id)}<LinkCard {link} />{/each}
          </div>
        </section>
      {/if}
    </div>

    {#if selected}
      <aside class="detail-panel" data-testid="detail-panel">
        <KnowledgeDetail
          item={selected}
          layout="stacked"
          onBack={() => navigate(kind === 'task' ? `/tasks/${slug}` : `/c/${slug}`)}
          onChanged={load}
        />
      </aside>
    {:else if items && items.length > 0}
      <aside class="detail-panel placeholder">
        <EmptyState icon="eye" title="اختر عنصرًا" body="اضغط أي بطاقة لعرض تفاصيلها هنا." compact />
        <span class="hint"><Icon name="arrow" size={14} directional /></span>
      </aside>
    {/if}
  </div>
{/if}

<style>
  .page { display: grid; grid-template-columns: minmax(0, 1fr); gap: var(--space-5); align-items: start; }
  .with-detail { grid-template-columns: minmax(0, 1fr) minmax(360px, 420px); }
  .main { display: flex; flex-direction: column; gap: var(--space-5); min-inline-size: 0; }
  .loading { display: flex; flex-direction: column; gap: var(--space-4); }
  .stats { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: var(--space-3); }
  .section-head { display: flex; align-items: baseline; justify-content: space-between; margin-block-end: var(--space-3); }
  .section-head a { font-size: 13px; color: var(--accent-blue); text-decoration: none; }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: var(--space-3); }
  .cell { border-radius: var(--radius-lg); }
  .cell.selected { outline: 2px solid var(--accent-blue); outline-offset: 2px; }
  .links { display: grid; grid-template-columns: repeat(auto-fill, minmax(230px, 1fr)); gap: var(--space-3); }
  .detail-panel {
    position: sticky;
    top: 0;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
    max-block-size: calc(100vh - var(--titlebar-height) - var(--space-8));
    overflow-y: auto;
  }
  .placeholder { display: flex; flex-direction: column; align-items: center; }
  .hint { color: var(--text-tertiary); }

  @media (max-width: 1280px) {
    .with-detail { grid-template-columns: minmax(0, 1fr); }
    .detail-panel { position: static; max-block-size: none; }
    .placeholder { display: none; }
  }
  @media (max-width: 900px) { .stats { grid-template-columns: repeat(2, minmax(0, 1fr)); } }
</style>
