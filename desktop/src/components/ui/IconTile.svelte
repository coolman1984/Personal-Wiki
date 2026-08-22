<script lang="ts">
  import Icon from './Icon.svelte';
  import type { AccentKey } from '$lib/types';

  let {
    icon,
    accent = 'blue',
    size = 48
  }: { icon: string | null | undefined; accent?: AccentKey | null; size?: number } = $props();

  const key = $derived(accent ?? 'blue');
</script>

<span
  class="tile"
  style="--tile-size:{size}px; --tile-fg: var(--accent-{key}); --tile-bg: var(--accent-{key}-soft);"
>
  <Icon name={icon} size={Math.round(size * 0.5)} stroke={1.8} />
</span>

<style>
  .tile {
    inline-size: var(--tile-size);
    block-size: var(--tile-size);
    border-radius: calc(var(--tile-size) / 3.4);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: var(--tile-bg);
    color: var(--tile-fg);
    flex: none;
  }
</style>
