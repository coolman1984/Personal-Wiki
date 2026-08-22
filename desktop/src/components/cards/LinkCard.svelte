<script lang="ts">
  import Icon from '$components/ui/Icon.svelte';
  import IconTile from '$components/ui/IconTile.svelte';
  import { openExternal } from '$lib/api';
  import { guard } from '$lib/state.svelte';
  import type { SavedLink } from '$lib/types';

  let {
    link,
    onEdit = null
  }: { link: SavedLink; onEdit?: ((link: SavedLink) => void) | null } = $props();
</script>

<article class="link card" data-testid="link-card">
  <IconTile icon="link" accent="green" size={38} />
  <div class="text">
    <h3>{link.title}</h3>
    {#if link.description}<p>{link.description}</p>{/if}
    <span class="url ltr" dir="ltr">{link.url}</span>
  </div>
  <div class="actions">
    <button
      type="button"
      aria-label="فتح الرابط"
      onclick={() => guard(() => openExternal(link.url), 'تعذّر فتح الرابط')}
    >
      <Icon name="external" size={15} />
    </button>
    {#if onEdit}
      <button type="button" aria-label="تعديل" onclick={() => onEdit(link)}><Icon name="settings" size={15} /></button>
    {/if}
  </div>
</article>

<style>
  .link { display: flex; gap: var(--space-3); padding: var(--space-4); align-items: flex-start; }
  .link:hover { box-shadow: var(--shadow-md); }
  .text { flex: 1; min-inline-size: 0; }
  h3 { margin: 0; font-size: 14px; font-weight: 600; color: var(--accent-blue); }
  p { margin: 3px 0; font-size: 12.5px; color: var(--text-secondary); }
  .url { display: block; font-size: 11.5px; color: var(--text-tertiary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .actions { display: flex; gap: 2px; flex: none; }
  .actions button { background: transparent; border: none; color: var(--text-tertiary); padding: 5px; border-radius: var(--radius-sm); }
  .actions button:hover { color: var(--accent-blue); background: var(--accent-blue-soft); }
</style>
