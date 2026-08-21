<script lang="ts">
  import Icon from '$components/ui/Icon.svelte';
  import IconTile from '$components/ui/IconTile.svelte';
  import TagChip from '$components/ui/TagChip.svelte';
  import { iconForItemType } from '$lib/icons';
  import { copyText, openExternal, setFavorite } from '$lib/api';
  import { navigate } from '$lib/router.svelte';
  import { guard, toasts } from '$lib/state.svelte';
  import { t } from '$lib/i18n';
  import type { ItemSummary } from '$lib/types';

  let {
    item,
    iconAtEnd = false,
    sourceUrl = null,
    onChanged = null
  }: {
    item: ItemSummary;
    iconAtEnd?: boolean;
    sourceUrl?: string | null;
    onChanged?: (() => void) | null;
  } = $props();

  let favorite = $state(false);
  $effect(() => {
    favorite = item.isFavorite;
  });

  const isCommand = $derived(['command', 'cli_command', 'slash_command', 'snippet'].includes(item.itemType));
  const visibleTags = $derived(item.tags.slice(0, 3));
  const hiddenTags = $derived(Math.max(0, item.tags.length - 3));

  async function toggleFavorite(event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    const next = !favorite;
    favorite = next; // optimistic
    const result = await guard(() => setFavorite(item.id, next), 'تعذّر تحديث المفضلة');
    if (result === null) favorite = !next;
    else onChanged?.();
  }

  async function copy(event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    if (!item.syntax) return;
    const ok = await guard(() => copyText(item.syntax!), 'تعذّر النسخ');
    if (ok !== null) toasts.push(t.item.copied);
  }

  async function open(event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    if (sourceUrl) await guard(() => openExternal(sourceUrl), 'تعذّر فتح الرابط');
  }
</script>

<article
  class="item card"
  class:reverse={iconAtEnd}
  class:deprecated={item.lifecycleStatus === 'deprecated'}
  data-testid="item-card"
  data-slug={item.slug}
>
  <a class="main" href={`#/items/${item.slug}`} onclick={() => navigate(`/items/${item.slug}`)}>
    <div class="head">
      <IconTile icon={item.iconKey ?? iconForItemType(item.itemType)} accent={item.accentKey ?? 'blue'} size={38} />
      <div class="titles">
        <h3 class:mono={isCommand} class:ltr={isCommand}>{item.title}</h3>
        {#if item.shortDescription}<p>{item.shortDescription}</p>{/if}
      </div>
      <button
        type="button"
        class="fav"
        class:on={favorite}
        onclick={toggleFavorite}
        aria-pressed={favorite}
        aria-label={favorite ? t.item.unfavorite : t.item.favorite}
        data-testid="favorite-toggle"
      >
        <Icon name="star" size={16} />
      </button>
    </div>

    {#if visibleTags.length > 0}
      <div class="tags">
        {#each visibleTags as tag (tag.id)}
          <TagChip label={tag.name} accent={tag.accentKey} size="sm" />
        {/each}
        {#if hiddenTags > 0}<TagChip label={`+${hiddenTags}`} accent="slate" size="sm" />{/if}
      </div>
    {/if}
  </a>

  <footer class="actions">
    {#if item.syntax}
      <button type="button" onclick={copy}><Icon name="copy" size={14} /><span>نسخ</span></button>
    {/if}
    {#if item.hasExamples}
      <a href={`#/items/${item.slug}`}><Icon name="code" size={14} /><span>مثال</span></a>
    {/if}
    {#if item.hasSources && sourceUrl}
      <button type="button" onclick={open}><Icon name="external" size={14} /><span>المصدر</span></button>
    {:else if item.hasSources}
      <a href={`#/items/${item.slug}`}><Icon name="globe" size={14} /><span>المصدر</span></a>
    {/if}
  </footer>
</article>

<style>
  .item {
    display: flex;
    flex-direction: column;
    transition: box-shadow var(--dur-fast) var(--ease), border-color var(--dur-fast) var(--ease);
  }
  .item:hover { box-shadow: var(--shadow-md); border-color: var(--border-strong); }
  .deprecated { opacity: 0.72; }
  .main { display: flex; flex-direction: column; gap: var(--space-3); padding: var(--space-4); text-decoration: none; color: inherit; }
  .head { display: flex; align-items: flex-start; gap: var(--space-3); }
  .reverse .head { flex-direction: row-reverse; }
  .reverse .titles { text-align: start; }
  .titles { flex: 1; min-inline-size: 0; }
  h3 {
    margin: 0;
    font-size: 14.5px;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  h3.mono { font-family: var(--font-mono); font-size: 13.5px; }
  p {
    margin: 3px 0 0;
    font-size: 12.5px;
    color: var(--text-secondary);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .fav { background: transparent; border: none; color: var(--text-tertiary); padding: 3px; border-radius: var(--radius-sm); flex: none; }
  .fav:hover { color: var(--accent-orange); }
  .fav.on { color: var(--accent-orange); }
  .fav.on :global(svg) { fill: currentColor; }
  .tags { display: flex; flex-wrap: wrap; gap: 5px; }
  .actions {
    display: flex;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-4) var(--space-3);
    border-block-start: 1px solid var(--border);
    margin-block-start: auto;
  }
  .actions button, .actions a {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 4px 9px;
    font-size: 12px;
    color: var(--text-secondary);
    text-decoration: none;
  }
  .actions button:hover, .actions a:hover { border-color: var(--accent-blue); color: var(--accent-blue); background: var(--accent-blue-soft); }
</style>
