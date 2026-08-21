<script lang="ts">
  import ToolCardView from '$components/cards/ToolCard.svelte';
  import PageHeader from '$components/ui/PageHeader.svelte';
  import EmptyState from '$components/ui/EmptyState.svelte';
  import ErrorState from '$components/ui/ErrorState.svelte';
  import Skeleton from '$components/ui/Skeleton.svelte';
  import { listTools } from '$lib/api';
  import { t } from '$lib/i18n';
  import type { ToolCard } from '$lib/types';

  let tools = $state<ToolCard[] | null>(null);
  let error = $state<string | null>(null);
  let filter = $state('');

  async function load() {
    error = null;
    try {
      tools = await listTools();
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
  }
  load();

  const visible = $derived(
    (tools ?? []).filter((tool) =>
      filter.trim() === '' ? true : `${tool.name} ${tool.description ?? ''}`.includes(filter.trim())
    )
  );
</script>

<div class="page">
  <PageHeader icon="grid" accent="blue" title={t.nav.tools} subtitle="كل الأدوات المتاحة في قاعدة معرفتك." />

  <input class="filter" type="search" bind:value={filter} placeholder="تصفية الأدوات…" aria-label="تصفية الأدوات" />

  {#if error}
    <ErrorState title={t.errors.database.title} body={t.errors.database.body} detail={error} onRetry={load} />
  {:else if !tools}
    <div class="grid"><Skeleton variant="card" count={8} /></div>
  {:else if visible.length === 0}
    <EmptyState icon="grid" title={t.empty.tools.title} body={t.empty.tools.body} />
  {:else}
    <div class="grid" data-testid="tool-grid">
      {#each visible as tool (tool.id)}
        <ToolCardView {tool} />
      {/each}
    </div>
  {/if}
</div>

<style>
  .page { display: flex; flex-direction: column; gap: var(--space-5); max-inline-size: var(--content-max); }
  .filter {
    max-inline-size: 340px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 9px 14px;
    font-size: 13px;
  }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(232px, 1fr)); gap: var(--space-4); }
</style>
