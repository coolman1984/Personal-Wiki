<script lang="ts">
  import Icon from '$components/ui/Icon.svelte';
  import IconTile from '$components/ui/IconTile.svelte';
  import TagChip from '$components/ui/TagChip.svelte';
  import Dropdown from '$components/ui/Dropdown.svelte';
  import Modal from '$components/ui/Modal.svelte';
  import KnowledgeDetail from '$components/detail/KnowledgeDetail.svelte';
  import EmptyState from '$components/ui/EmptyState.svelte';
  import ErrorState from '$components/ui/ErrorState.svelte';
  import Skeleton from '$components/ui/Skeleton.svelte';
  import {
    clearSearchHistory, copyText, deleteSavedSearch, getItem, listItemTypes, listSavedSearches,
    listTags, listTools, saveSearch, search as runSearch, searchHistory, suggest
  } from '$lib/api';
  import { navigate, replace, type Route } from '$lib/router.svelte';
  import { count, relativeTime } from '$lib/format';
  import { guard, toasts } from '$lib/state.svelte';
  import { emptyFilters, type ItemDetail, type SavedSearch, type SearchFilters, type SearchHistoryEntry, type SearchHit, type SearchResponse, type Tag, type ToolCard } from '$lib/types';
  import { t } from '$lib/i18n';

  let { route }: { route: Route } = $props();

  const query = $derived(route.query.get('q') ?? '');

  let response = $state<SearchResponse | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let tools = $state<ToolCard[]>([]);
  let tags = $state<Tag[]>([]);
  let itemTypes = $state<string[]>([]);
  let history = $state<SearchHistoryEntry[]>([]);
  let saved = $state<SavedSearch[]>([]);
  let suggestions = $state<string[]>([]);
  let preview = $state<ItemDetail | null>(null);
  let saveOpen = $state(false);
  let saveName = $state('');
  let limit = $state(20);

  // Filters live in the URL, so a result set is a shareable, reloadable address.
  function filtersFromRoute(): SearchFilters {
    const filters = emptyFilters();
    const read = (key: string) => (route.query.get(key) ?? '').split(',').filter(Boolean);
    filters.toolSlugs = read('tool');
    filters.taskSlugs = read('task');
    filters.itemTypes = read('type');
    filters.tagSlugs = read('tag');
    filters.verificationStatuses = read('verify');
    filters.lifecycleStatuses = read('life');
    filters.favoritesOnly = route.query.get('fav') === '1';
    return filters;
  }

  const filters = $derived(filtersFromRoute());
  const activeFilterCount = $derived(
    filters.toolSlugs.length + filters.itemTypes.length + filters.tagSlugs.length +
      filters.verificationStatuses.length + filters.lifecycleStatuses.length + (filters.favoritesOnly ? 1 : 0)
  );

  function updateFilter(key: string, values: string[]) {
    const next: Record<string, string | undefined> = { q: query };
    next.tool = filters.toolSlugs.join(',') || undefined;
    next.type = filters.itemTypes.join(',') || undefined;
    next.tag = filters.tagSlugs.join(',') || undefined;
    next.verify = filters.verificationStatuses.join(',') || undefined;
    next.life = filters.lifecycleStatuses.join(',') || undefined;
    next.fav = filters.favoritesOnly ? '1' : undefined;
    next[key] = values.join(',') || undefined;
    replace('/search', next);
  }

  function toggleFavoritesOnly() {
    replace('/search', {
      q: query,
      tool: filters.toolSlugs.join(',') || undefined,
      type: filters.itemTypes.join(',') || undefined,
      tag: filters.tagSlugs.join(',') || undefined,
      verify: filters.verificationStatuses.join(',') || undefined,
      life: filters.lifecycleStatuses.join(',') || undefined,
      fav: filters.favoritesOnly ? undefined : '1'
    });
  }

  async function loadReference() {
    [tools, tags, itemTypes, history, saved] = await Promise.all([
      listTools(), listTags(), listItemTypes(), searchHistory(), listSavedSearches()
    ]);
  }
  loadReference();

  let debounce: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    const q = query;
    const f = filters;
    const cap = limit;
    if (debounce) clearTimeout(debounce);
    if (q.trim() === '') {
      response = null;
      suggestions = [];
      return;
    }
    loading = true;
    debounce = setTimeout(async () => {
      try {
        error = null;
        response = await runSearch(q, f, cap, 0);
        suggestions = await suggest(q);
        history = await searchHistory();
      } catch (err) {
        error = err instanceof Error ? err.message : String(err);
      } finally {
        loading = false;
      }
    }, 120);
  });

  const groups = $derived(() => {
    const map = new Map<string, { name: string; icon: string; accent: string; hits: SearchHit[] }>();
    for (const hit of response?.hits ?? []) {
      const key = hit.toolSlug ?? 'other';
      if (!map.has(key)) {
        map.set(key, {
          name: hit.toolName ?? 'أخرى',
          icon: tools.find((tool) => tool.slug === key)?.iconKey ?? 'book',
          accent: tools.find((tool) => tool.slug === key)?.accentKey ?? 'blue',
          hits: []
        });
      }
      map.get(key)!.hits.push(hit);
    }
    return [...map.entries()];
  });

  async function submitSave() {
    if (saveName.trim() === '') return;
    const result = await guard(() => saveSearch(saveName, query, filters), 'تعذّر حفظ البحث');
    if (result) {
      saved = await listSavedSearches();
      saveOpen = false;
      saveName = '';
      toasts.push('تم حفظ البحث');
    }
  }
</script>

<div class="page">
  <div class="results">
    <div class="filterbar" data-testid="search-filters">
      <Dropdown
        label={t.search.allTools}
        icon="grid"
        options={tools.map((tool) => ({ value: tool.slug, label: tool.name, count: tool.itemCount }))}
        selected={filters.toolSlugs}
        onChange={(next) => updateFilter('tool', next)}
      />
      <Dropdown
        label={t.search.contentType}
        icon="list"
        options={itemTypes.map((type) => ({ value: type, label: type }))}
        selected={filters.itemTypes}
        onChange={(next) => updateFilter('type', next)}
      />
      <Dropdown
        label={t.search.tags}
        icon="filter"
        options={tags.map((tag) => ({ value: tag.slug, label: tag.name }))}
        selected={filters.tagSlugs}
        onChange={(next) => updateFilter('tag', next)}
      />
      <Dropdown
        label={t.search.source}
        icon="globe"
        options={[
          { value: 'official', label: 'رسمي' },
          { value: 'verified', label: 'مُتحقَّق' },
          { value: 'community', label: 'مجتمعي' },
          { value: 'unverified', label: 'غير مُتحقَّق' },
          { value: 'demo', label: 'عرض تجريبي' }
        ]}
        selected={filters.verificationStatuses}
        onChange={(next) => updateFilter('verify', next)}
      />
      <button type="button" class="toggle" class:on={filters.favoritesOnly} onclick={toggleFavoritesOnly}>
        <Icon name="star" size={15} /><span>{t.search.favoritesOnly}</span>
      </button>
      {#if activeFilterCount > 0}
        <button type="button" class="clear" onclick={() => replace('/search', { q: query })}>
          <Icon name="close" size={14} /><span>{t.search.clearFilters}</span>
        </button>
      {/if}
    </div>

    {#if error}
      <ErrorState detail={error} onRetry={() => replace('/search', { q: query })} />
    {:else if query.trim() === ''}
      <EmptyState icon="search" title={t.search.title} body="اكتب ما تريد فعله في شريط البحث بالأعلى." />
    {:else if loading && !response}
      <div class="loading"><Skeleton variant="row" count={5} /></div>
    {:else if response}
      <div class="meta">
        <span class="found">
          <span class="pulse"></span>
          {t.search.resultsFound(count(response.total), response.tookMs.toFixed(2))}
        </span>
        <span class="sort"><Icon name="list" size={14} />{t.search.sortRelevance}</span>
      </div>

      {#if response.hits.length === 0}
        <EmptyState icon="search" title={t.empty.search.title} body={t.empty.search.body} />
      {:else}
        {#each groups() as [slug, group] (slug)}
          <section class="group">
            <header>
              <IconTile icon={group.icon} accent={group.accent as never} size={28} />
              <h2>{group.name}</h2>
            </header>
            {#each group.hits as hit (hit.id)}
              <article class="hit" data-testid="search-hit" data-slug={hit.slug}>
                <span class="life {hit.lifecycleStatus}"></span>
                <div class="body">
                  <h3>{hit.title}</h3>
                  {#if hit.shortDescription}<p>{hit.shortDescription}</p>{/if}
                  <div class="chips">
                    {#if hit.toolName}<TagChip label={hit.toolName} accent="blue" size="sm" />{/if}
                    {#each hit.tags.slice(0, 2) as tag (tag.id)}
                      <TagChip label={tag.name} accent={tag.accentKey} size="sm" />
                    {/each}
                    {#if hit.matchedAlias}<TagChip label={hit.matchedAlias} accent="purple" size="sm" />{/if}
                  </div>
                </div>
                <span class="score" class:high={hit.relevance >= 90} class:mid={hit.relevance >= 70 && hit.relevance < 90}>
                  <span class="ltr">{hit.relevance}%</span> {t.search.relevance}
                </span>
                <div class="actions">
                  <button type="button" onclick={() => navigate(`/items/${hit.slug}`)}>
                    <Icon name="external" size={14} /><span>{t.search.open}</span>
                  </button>
                  {#if hit.syntax}
                    <button
                      type="button"
                      onclick={async () => {
                        const ok = await guard(() => copyText(hit.syntax!), 'تعذّر النسخ');
                        if (ok !== null) toasts.push(t.item.copied);
                      }}
                    >
                      <Icon name="copy" size={14} /><span>{t.search.copy}</span>
                    </button>
                  {/if}
                  <button
                    type="button"
                    onclick={async () => {
                      preview = await getItem(hit.slug);
                    }}
                  >
                    <Icon name="eye" size={14} /><span>{t.search.preview}</span>
                  </button>
                </div>
              </article>
            {/each}
          </section>
        {/each}

        {#if response.total > response.hits.length}
          <button type="button" class="more" onclick={() => (limit += 20)}>
            <Icon name="chevron" size={15} />
            <span>{t.search.more(count(response.total - response.hits.length))}</span>
          </button>
        {/if}
      {/if}
    {/if}
  </div>

  <aside class="side">
    <section class="card">
      <div class="side-head">
        <h2><Icon name="sparkle" size={15} />{t.search.suggestions}</h2>
      </div>
      {#if suggestions.length === 0}
        <p class="none">اكتب كلمة للحصول على اقتراحات.</p>
      {:else}
        <ul class="list">
          {#each suggestions as suggestion (suggestion)}
            <li>
              <button type="button" onclick={() => replace('/search', { q: suggestion })}>
                <Icon name="search" size={14} />
                <span>{suggestion}</span>
                <Icon name="arrow" size={13} directional />
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <section class="card">
      <div class="side-head">
        <h2><Icon name="clock" size={15} />{t.search.recent}</h2>
        {#if history.length > 0}
          <button
            type="button"
            class="link"
            onclick={async () => {
              await clearSearchHistory();
              history = [];
            }}
          >{t.search.clearAll}</button>
        {/if}
      </div>
      {#if history.length === 0}
        <p class="none">لا توجد عمليات بحث سابقة.</p>
      {:else}
        <ul class="list">
          {#each history.slice(0, 6) as entry (entry.id)}
            <li>
              <button type="button" onclick={() => replace('/search', { q: entry.query })}>
                <Icon name="clock" size={14} />
                <span>{entry.query}</span>
                <span class="time">{relativeTime(entry.executedAt)}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <section class="card">
      <div class="side-head">
        <h2><Icon name="star" size={15} />{t.search.saved}</h2>
      </div>
      {#if saved.length === 0}
        <p class="none">لم تحفظ أي بحث بعد.</p>
      {:else}
        <ul class="list">
          {#each saved as entry (entry.id)}
            <li class="saved-row">
              <button type="button" onclick={() => replace('/search', { q: entry.query })}>
                <Icon name="star" size={14} /><span>{entry.name}</span>
              </button>
              <button
                type="button"
                class="icon"
                aria-label={t.common.delete}
                onclick={async () => {
                  await guard(() => deleteSavedSearch(entry.id));
                  saved = await listSavedSearches();
                }}
              >
                <Icon name="trash" size={13} />
              </button>
            </li>
          {/each}
        </ul>
      {/if}
      {#if query.trim() !== ''}
        <button type="button" class="save-current" onclick={() => (saveOpen = true)}>
          <Icon name="plus" size={14} /><span>{t.search.saveCurrent}</span>
        </button>
      {/if}
    </section>
  </aside>
</div>

{#if preview}
  <Modal title={preview.title} wide onClose={() => (preview = null)}>
    <KnowledgeDetail item={preview} layout="stacked" />
  </Modal>
{/if}

{#if saveOpen}
  <Modal title={t.search.saveCurrent} onClose={() => (saveOpen = false)}>
    <label class="field">
      <span>{t.search.namePrompt}</span>
      <input bind:value={saveName} type="text" />
    </label>
    {#snippet footer()}
      <button type="button" class="ghost" onclick={() => (saveOpen = false)}>{t.common.cancel}</button>
      <button type="button" class="primary" onclick={submitSave}>{t.common.save}</button>
    {/snippet}
  </Modal>
{/if}

<style>
  .page { display: grid; grid-template-columns: minmax(0, 1fr) var(--panel-width); gap: var(--space-5); align-items: start; }
  .results { display: flex; flex-direction: column; gap: var(--space-4); min-inline-size: 0; }
  .filterbar { display: flex; flex-wrap: wrap; gap: var(--space-2); align-items: center; }
  .toggle, .clear {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 8px 12px;
    font-size: 13px;
    color: var(--text-secondary);
  }
  .toggle.on { background: var(--accent-orange-soft); border-color: var(--accent-orange); color: var(--accent-orange); }
  .clear { color: var(--danger); border-color: transparent; }

  .meta { display: flex; align-items: center; justify-content: space-between; font-size: 12.5px; color: var(--text-secondary); }
  .found { display: inline-flex; align-items: center; gap: var(--space-2); }
  .pulse { inline-size: 7px; block-size: 7px; border-radius: 50%; background: var(--success); }
  .sort { display: inline-flex; align-items: center; gap: 6px; }

  .group { display: flex; flex-direction: column; gap: var(--space-2); }
  .group header { display: flex; align-items: center; gap: var(--space-2); }
  .group h2 { margin: 0; font-size: 14px; font-weight: 600; }

  .hit {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: var(--space-3) var(--space-4);
  }
  .hit:hover { border-color: var(--border-strong); box-shadow: var(--shadow-sm); }
  .life { inline-size: 8px; block-size: 8px; border-radius: 50%; background: var(--success); flex: none; }
  .life.deprecated { background: var(--warning); }
  .life.removed { background: var(--danger); }
  .life.experimental { background: var(--accent-purple); }
  .body { flex: 1; min-inline-size: 0; }
  .body h3 { margin: 0; font-size: 14px; font-weight: 600; }
  .body p { margin: 2px 0 0; font-size: 12.5px; color: var(--text-secondary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .chips { display: flex; gap: 5px; margin-block-start: 6px; flex-wrap: wrap; }
  .score {
    font-size: 11.5px;
    color: var(--text-tertiary);
    background: var(--surface-muted);
    border-radius: 999px;
    padding: 3px 9px;
    flex: none;
  }
  .score.high { background: var(--accent-green-soft); color: var(--accent-green); }
  .score.mid { background: var(--accent-blue-soft); color: var(--accent-blue); }
  .actions { display: flex; gap: 4px; flex: none; }
  .actions button {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 4px 9px;
    font-size: 12px;
    color: var(--text-secondary);
  }
  .actions button:hover { border-color: var(--accent-blue); color: var(--accent-blue); }

  .more {
    align-self: center;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 9px 18px;
    font-size: 13px;
    color: var(--text-secondary);
  }

  .side { display: flex; flex-direction: column; gap: var(--space-3); position: sticky; top: 0; }
  .side .card { padding: var(--space-4); display: flex; flex-direction: column; gap: var(--space-2); }
  .side-head { display: flex; align-items: center; justify-content: space-between; }
  .side-head h2 { margin: 0; font-size: 13.5px; font-weight: 600; display: inline-flex; align-items: center; gap: 6px; }
  .link { background: none; border: none; color: var(--accent-blue); font-size: 12px; padding: 0; }
  .list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 2px; }
  .list button {
    inline-size: 100%;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    padding: 7px 8px;
    font-size: 12.5px;
    color: var(--text-secondary);
    text-align: start;
  }
  .list button:hover { background: var(--surface-muted); color: var(--text-primary); }
  .list span:not(.time) { flex: 1; min-inline-size: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .time { font-size: 11px; color: var(--text-tertiary); }
  .saved-row { display: flex; align-items: center; }
  .icon { inline-size: auto !important; padding: 6px !important; color: var(--text-tertiary); }
  .none { margin: 0; font-size: 12.5px; color: var(--text-tertiary); }
  .save-current {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    border: 1px dashed var(--border-strong);
    background: transparent;
    border-radius: var(--radius-md);
    padding: 8px;
    font-size: 12.5px;
    color: var(--text-secondary);
    margin-block-start: var(--space-2);
  }

  .field { display: flex; flex-direction: column; gap: 6px; font-size: 13px; }
  .field input { background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-md); padding: 9px 12px; }
  .ghost { background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-md); padding: 8px 16px; font-size: 13px; }
  .primary { background: var(--accent-blue); color: var(--text-inverse); border: none; border-radius: var(--radius-md); padding: 8px 16px; font-size: 13px; font-weight: 600; }

  @media (max-width: 1280px) {
    .page { grid-template-columns: minmax(0, 1fr); }
    .side { display: none; }
  }
</style>
