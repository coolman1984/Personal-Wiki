<script lang="ts">
  import Breadcrumbs from '$components/ui/Breadcrumbs.svelte';
  import PageHeader from '$components/ui/PageHeader.svelte';
  import PanelLayout from '$components/ui/PanelLayout.svelte';
  import StatTile from '$components/ui/StatTile.svelte';
  import TagChip from '$components/ui/TagChip.svelte';
  import Icon from '$components/ui/Icon.svelte';
  import ItemCard from '$components/cards/ItemCard.svelte';
  import EmptyState from '$components/ui/EmptyState.svelte';
  import ErrorState from '$components/ui/ErrorState.svelte';
  import Skeleton from '$components/ui/Skeleton.svelte';
  import { getToolPage, itemsForTool, openExternal, setSetting, getSettings } from '$lib/api';
  import { href, navigate } from '$lib/router.svelte';
  import { absoluteDate, count } from '$lib/format';
  import { guard } from '$lib/state.svelte';
  import { t } from '$lib/i18n';
  import type { CategoryNode, ItemSummary, ToolPage } from '$lib/types';

  let { slug, section }: { slug: string; section: string | null } = $props();

  let page = $state<ToolPage | null>(null);
  let items = $state<ItemSummary[] | null>(null);
  let error = $state<string | null>(null);
  let sort = $state('recent');

  // The active section drives both the tab strip and the item query; it lives in
  // the URL so the page is deep-linkable.
  const activeSection = $derived(section);

  async function loadPage() {
    error = null;
    page = null;
    try {
      page = await getToolPage(slug);
      const settings = await getSettings();
      const stored = settings[`sort.tool.${slug}`];
      if (typeof stored === 'string') sort = stored;
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
  }

  async function loadItems() {
    items = null;
    try {
      items = await itemsForTool(slug, activeSection, sort);
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
  }

  $effect(() => {
    void slug;
    loadPage();
  });

  $effect(() => {
    void slug;
    void activeSection;
    void sort;
    loadItems();
  });

  async function changeSort(next: string) {
    sort = next;
    await guard(() => setSetting(`sort.tool.${slug}`, next));
  }

  function flatten(nodes: CategoryNode[]): CategoryNode[] {
    return nodes.flatMap((node) => [node, ...flatten(node.children)]);
  }

  const activeNode = $derived(
    page ? flatten(page.sections).find((node) => node.slug === activeSection) ?? null : null
  );
  const childSections = $derived(
    page
      ? (page.sections.find((node) => node.slug === activeSection)?.children ??
          (activeNode ? activeNode.children : []))
      : []
  );
  const crumbs = $derived([
    { label: t.nav.home, path: '/', icon: 'home' },
    { label: t.nav.tools, path: '/tools' },
    ...(page ? [{ label: page.tool.name, path: `/tools/${page.tool.slug}` }] : []),
    ...(activeNode ? [{ label: activeNode.name }] : [])
  ]);
</script>

{#if error && !page}
  <ErrorState title={t.errors.database.title} detail={error} onRetry={loadPage} />
{:else if !page}
  <div class="loading">
    <Skeleton variant="line" count={2} />
    <Skeleton variant="tile" count={1} />
    <div class="grid"><Skeleton variant="card" count={4} /></div>
  </div>
{:else}
  <div class="page">
    <Breadcrumbs {crumbs} />

    <PanelLayout>
      <section class="identity card">
        <PageHeader
          icon={page.tool.iconKey}
          accent={page.tool.accentKey}
          title={page.tool.name}
          subtitle={page.tool.description}
        >
          {#snippet badges()}
            {#each page!.tags.slice(0, 6) as tag (tag.id)}
              <TagChip label={tag.name} accent={tag.accentKey} />
            {/each}
          {/snippet}
          {#snippet actions()}
            {#if page!.tool.officialHomeUrl}
              <button
                type="button"
                class="ghost"
                onclick={() => guard(() => openExternal(page!.tool.officialHomeUrl!), 'تعذّر فتح الرابط')}
              >
                <Icon name="external" size={15} /><span>الموقع الرسمي</span>
              </button>
            {/if}
          {/snippet}
        </PageHeader>
      </section>

      <div class="stats" data-testid="tool-stats">
        <StatTile icon="terminal" accent="green" value={count(page.stats.itemCount)} label={t.tool.totalCommands} sublabel={t.tool.availableCommand} />
        <StatTile icon="code" accent="purple" value={count(page.stats.exampleCount)} label={t.tool.examples} sublabel={t.tool.practicalExample} />
        <StatTile icon="note" accent="orange" value={count(page.stats.noteCount)} label={t.tool.notes} sublabel={t.tool.note} />
        <StatTile icon="globe" accent="blue" value={count(page.stats.sourceCount)} label={t.tool.sources} sublabel={t.tool.trustedSource} />
        <StatTile
          icon="clock"
          accent="slate"
          value={page.stats.lastUpdatedAt ? absoluteDate(page.stats.lastUpdatedAt).split('،')[0] ?? '—' : '—'}
          label={t.tool.lastUpdate}
        />
      </div>

      {#if page.sections.length > 0}
        <nav class="tabs" aria-label="أقسام الأداة" data-testid="section-tabs">
          <button
            type="button"
            class:active={!activeSection}
            onclick={() => navigate(`/tools/${slug}`)}
          >
            <Icon name="list" size={15} />
            <span>الكل</span>
            <span class="tab-count ltr">{count(page.stats.itemCount)}</span>
          </button>
          {#each page.sections as node (node.id)}
            <button
              type="button"
              class:active={activeSection === node.slug}
              onclick={() => navigate(`/tools/${slug}/${node.slug}`)}
              data-slug={node.slug}
            >
              <Icon name={node.iconKey} size={15} />
              <span>{node.name}</span>
              <span class="tab-count ltr">{count(node.itemCount)}</span>
            </button>
          {/each}
        </nav>
      {/if}

      {#if childSections.length > 0}
        <nav class="subtabs" aria-label="أقسام فرعية">
          {#each childSections as child (child.id)}
            <button
              type="button"
              class:active={activeSection === child.slug}
              onclick={() => navigate(`/tools/${slug}/${child.slug}`)}
            >
              {child.name}
              <span class="ltr">{count(child.itemCount)}</span>
            </button>
          {/each}
        </nav>
      {/if}

      <section>
        <div class="section-head">
          <h2 class="section-title">{activeNode?.name ?? page.tool.name}</h2>
          <div class="head-right">
            {#if items}<span class="muted">{t.common.commandsCount(count(items.length))}</span>{/if}
            <label class="sort">
              <Icon name="filter" size={14} />
              <span class="visually-hidden">{t.tool.sortLabel}</span>
              <select value={sort} onchange={(e) => changeSort((e.currentTarget as HTMLSelectElement).value)}>
                <option value="recent">{t.tool.sortRecent}</option>
                <option value="alpha">{t.tool.sortAlpha}</option>
                <option value="popular">{t.tool.sortPopular}</option>
              </select>
            </label>
          </div>
        </div>

        {#if !items}
          <div class="grid"><Skeleton variant="card" count={4} /></div>
        {:else if items.length === 0}
          <EmptyState icon="terminal" title={t.empty.items.title} body={t.empty.items.body} />
        {:else}
          <div class="grid" data-testid="item-grid">
            {#each items as item (item.id)}
              <ItemCard {item} onChanged={loadItems} />
            {/each}
          </div>
        {/if}
      </section>

      {#snippet panel()}
        <section class="card explainer">
          <h2>{t.tool.explainerTitle}</h2>
          <p class="lead">
            تم تجهيز هذه الصفحة لمساعدتك على استكشاف جميع أوامر <bdi>{page!.tool.name}</bdi>.
          </p>
          <ol>
            {#each t.tool.explainer as step, index (step.title)}
              <li>
                <span class="num ltr">{index + 1}</span>
                <span class="step">
                  <strong>{step.title}</strong>
                  <span>{step.body}</span>
                </span>
              </li>
            {/each}
          </ol>
        </section>
        <section class="card help">
          <span class="help-icon"><Icon name="question" size={20} /></span>
          <div>
            <strong>{t.tool.helpTitle}</strong>
            <p>{t.tool.helpBody}</p>
            <a href={href('/search')}>{t.search.title}</a>
          </div>
        </section>
      {/snippet}
    </PanelLayout>
  </div>
{/if}

<style>
  .page { display: flex; flex-direction: column; gap: var(--space-4); }
  .loading { display: flex; flex-direction: column; gap: var(--space-4); }
  .identity { padding: var(--space-5); }
  .ghost {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 8px 14px;
    font-size: 13px;
    color: var(--text-secondary);
  }
  .ghost:hover { border-color: var(--accent-blue); color: var(--accent-blue); }

  .stats { display: grid; grid-template-columns: repeat(5, minmax(0, 1fr)); gap: var(--space-3); }

  .tabs {
    display: flex;
    gap: var(--space-2);
    overflow-x: auto;
    padding-block-end: var(--space-1);
    scrollbar-width: thin;
    -webkit-mask-image: linear-gradient(to left, transparent 0, #000 24px, #000 calc(100% - 24px), transparent 100%);
    mask-image: linear-gradient(to left, transparent 0, #000 24px, #000 calc(100% - 24px), transparent 100%);
  }
  .tabs button {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 8px 14px;
    font-size: 13px;
    color: var(--text-secondary);
    white-space: nowrap;
    flex: none;
  }
  .tabs button:hover { border-color: var(--border-strong); }
  .tabs button.active { background: var(--accent-blue-soft); border-color: var(--accent-blue); color: var(--accent-blue); font-weight: 600; }
  .tab-count { font-size: 11px; color: var(--text-tertiary); }
  .tabs button.active .tab-count { color: var(--accent-blue); }

  .subtabs { display: flex; flex-wrap: wrap; gap: 6px; }
  .subtabs button {
    background: var(--surface-muted);
    border: 1px solid transparent;
    border-radius: 999px;
    padding: 5px 12px;
    font-size: 12px;
    color: var(--text-secondary);
    display: inline-flex;
    gap: 6px;
  }
  .subtabs button.active { background: var(--accent-blue-soft); color: var(--accent-blue); border-color: var(--accent-blue); }

  .section-head { display: flex; align-items: center; justify-content: space-between; gap: var(--space-3); margin-block-end: var(--space-3); }
  .head-right { display: flex; align-items: center; gap: var(--space-3); font-size: 12.5px; }
  .sort { display: inline-flex; align-items: center; gap: 6px; color: var(--text-secondary); }
  .sort select { background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 5px 8px; font-size: 12.5px; }

  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: var(--space-3); }

  .explainer { padding: var(--space-4); }
  .explainer h2 { margin: 0 0 var(--space-2); font-size: 15px; font-weight: 600; }
  .lead { margin: 0 0 var(--space-3); font-size: 12.5px; color: var(--text-secondary); }
  .explainer ol { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: var(--space-3); }
  .explainer li { display: flex; gap: var(--space-3); }
  .num {
    inline-size: 24px; block-size: 24px;
    border-radius: 50%;
    background: var(--accent-blue-soft);
    color: var(--accent-blue);
    display: grid; place-items: center;
    font-size: 12px; font-weight: 600;
    flex: none;
  }
  .step { display: flex; flex-direction: column; gap: 2px; }
  .step strong { font-size: 13px; color: var(--accent-blue); }
  .step span { font-size: 12px; color: var(--text-secondary); }

  .help { display: flex; gap: var(--space-3); padding: var(--space-4); background: var(--accent-blue-soft); border-color: transparent; }
  .help-icon { color: var(--accent-blue); }
  .help strong { font-size: 13px; }
  .help p { margin: 4px 0; font-size: 12px; color: var(--text-secondary); }
  .help a { font-size: 12px; color: var(--accent-blue); }

  @media (max-width: 1400px) { .stats { grid-template-columns: repeat(3, minmax(0, 1fr)); } }
  @media (max-width: 900px) { .stats { grid-template-columns: repeat(2, minmax(0, 1fr)); } }
</style>
