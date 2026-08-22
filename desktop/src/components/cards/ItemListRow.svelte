<script lang="ts">
  import Icon from '$components/ui/Icon.svelte';
  import IconTile from '$components/ui/IconTile.svelte';
  import TagChip from '$components/ui/TagChip.svelte';
  import { iconForItemType } from '$lib/icons';
  import type { ItemSummary } from '$lib/types';

  let {
    item,
    selected = false,
    onSelect
  }: { item: ItemSummary; selected?: boolean; onSelect: (item: ItemSummary) => void } = $props();

  const isCommand = $derived(['command', 'cli_command', 'slash_command'].includes(item.itemType));
</script>

<button
  type="button"
  class="row"
  class:selected
  onclick={() => onSelect(item)}
  aria-current={selected ? 'true' : undefined}
  data-testid="item-row"
  data-slug={item.slug}
>
  <IconTile icon={item.iconKey ?? iconForItemType(item.itemType)} accent={item.accentKey ?? 'blue'} size={40} />
  <span class="text">
    <span class="title" class:mono={isCommand} class:ltr={isCommand}>{item.title}</span>
    {#if item.shortDescription}<span class="desc">{item.shortDescription}</span>{/if}
    {#if item.tags.length > 0}
      <span class="tags">
        {#each item.tags.slice(0, 2) as tag (tag.id)}
          <TagChip label={tag.name} accent={tag.accentKey} size="sm" />
        {/each}
      </span>
    {/if}
  </span>
  {#if item.isFavorite}<span class="fav"><Icon name="star" size={15} /></span>{/if}
  <span class="chev"><Icon name="chevron" size={16} directional /></span>
</button>

<style>
  .row {
    inline-size: 100%;
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
    padding: var(--space-3);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    text-align: start;
    transition: border-color var(--dur-fast) var(--ease), background var(--dur-fast) var(--ease);
  }
  .row:hover { border-color: var(--border-strong); }
  .selected { border-color: var(--accent-blue); border-width: 1.5px; background: var(--accent-blue-soft); }
  .text { flex: 1; min-inline-size: 0; display: flex; flex-direction: column; gap: 3px; }
  .title { font-weight: 600; font-size: 14px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .title.mono { font-family: var(--font-mono); font-size: 13px; }
  .desc {
    font-size: 12.5px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .tags { display: flex; gap: 4px; margin-block-start: 2px; }
  .fav { color: var(--accent-orange); }
  .fav :global(svg) { fill: currentColor; }
  .chev { color: var(--text-tertiary); align-self: center; }
</style>
