<script lang="ts">
  import IconTile from './IconTile.svelte';
  import type { Snippet } from 'svelte';

  let {
    icon = 'book',
    accent = 'blue',
    title,
    subtitle = null,
    centered = false,
    badges,
    actions
  }: {
    icon?: string | null;
    accent?: string | null;
    title: string;
    subtitle?: string | null;
    centered?: boolean;
    badges?: Snippet;
    actions?: Snippet;
  } = $props();
</script>

<header class="header" class:centered>
  <IconTile {icon} accent={(accent ?? 'blue') as never} size={centered ? 56 : 52} />
  <div class="text">
    <h1>{title}</h1>
    {#if subtitle}<p>{subtitle}</p>{/if}
    {#if badges}<div class="badges">{@render badges()}</div>{/if}
  </div>
  {#if actions}<div class="actions">{@render actions()}</div>{/if}
</header>

<style>
  .header {
    display: flex;
    align-items: flex-start;
    gap: var(--space-4);
  }
  .centered {
    flex-direction: row-reverse;
    justify-content: center;
    align-items: center;
    text-align: center;
  }
  .centered .text { text-align: center; }
  .text { flex: 1; min-inline-size: 0; }
  .centered .text { flex: 0 1 auto; }
  h1 { margin: 0; font-size: 24px; font-weight: 700; line-height: 1.35; }
  p { margin: var(--space-1) 0 0; color: var(--text-secondary); font-size: 14px; max-inline-size: 78ch; }
  .badges { display: flex; flex-wrap: wrap; gap: var(--space-2); margin-block-start: var(--space-3); }
  .centered .badges { justify-content: center; }
  .actions { display: flex; gap: var(--space-2); flex: none; align-items: center; }
</style>
