<script lang="ts">
  import PageHeader from '$components/ui/PageHeader.svelte';
  import ItemCard from '$components/cards/ItemCard.svelte';
  import Dropdown from '$components/ui/Dropdown.svelte';
  import EmptyState from '$components/ui/EmptyState.svelte';
  import ErrorState from '$components/ui/ErrorState.svelte';
  import Skeleton from '$components/ui/Skeleton.svelte';
  import { listFavorites, listPinned, listTools } from '$lib/api';
  import { t } from '$lib/i18n';
  import type { ItemSummary, ToolCard } from '$lib/types';

  let favorites = $state<ItemSummary[] | null>(null);
  let pinned = $state<ItemSummary[]>([]);
  let tools = $state<ToolCard[]>([]);
  let toolFilter = $state<string[]>([]);
  let error = $state<string | null>(null);

  async function load() {
    error = null;
    try {
      [favorites, pinned, tools] = await Promise.all([listFavorites(), listPinned(), listTools()]);
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
  }
  load();

  const visible = $derived(
    (favorites ?? []).filter((item) => toolFilter.length === 0 || (item.toolSlug && toolFilter.includes(item.toolSlug)))
  );
</script>

<div class="page">
  <PageHeader icon="star" accent="orange" title={t.nav.favorites} subtitle="الأوامر والعناصر التي حفظتها للرجوع السريع." />

  {#if error}
    <ErrorState detail={error} onRetry={load} />
  {:else if !favorites}
    <div class="grid"><Skeleton variant="card" count={4} /></div>
  {:else}
    {#if pinned.length > 0}
      <section>
        <h2 class="section-title">المثبّتة</h2>
        <div class="grid">
          {#each pinned as item (item.id)}<ItemCard {item} onChanged={load} />{/each}
        </div>
      </section>
    {/if}

    <div class="filters">
      <Dropdown
        label={t.search.allTools}
        icon="grid"
        options={tools.map((tool) => ({ value: tool.slug, label: tool.name }))}
        selected={toolFilter}
        onChange={(next) => (toolFilter = next)}
      />
    </div>

    {#if visible.length === 0}
      <EmptyState icon="star" title={t.empty.favorites.title} body={t.empty.favorites.body} />
    {:else}
      <div class="grid" data-testid="item-grid">
        {#each visible as item (item.id)}<ItemCard {item} onChanged={load} />{/each}
      </div>
    {/if}
  {/if}
</div>

<style>
  .page { display: flex; flex-direction: column; gap: var(--space-5); max-inline-size: var(--content-max); }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(290px, 1fr)); gap: var(--space-3); }
  .section-title { margin: 0 0 var(--space-3); }
  .filters { display: flex; gap: var(--space-2); }
</style>
