<script lang="ts">
  import type { Snippet } from 'svelte';

  /**
   * Content column + optional contextual panel. The panel collapses below
   * 1280px so the main column always keeps its reading width.
   */
  let { children, panel }: { children: Snippet; panel?: Snippet } = $props();
</script>

<div class="layout" class:has-panel={Boolean(panel)}>
  <div class="main">{@render children()}</div>
  {#if panel}
    <aside class="panel" aria-label="لوحة سياقية">{@render panel()}</aside>
  {/if}
</div>

<style>
  .layout { display: grid; grid-template-columns: minmax(0, 1fr); gap: var(--space-5); align-items: start; }
  .has-panel { grid-template-columns: minmax(0, 1fr) var(--panel-width); }
  .main { min-inline-size: 0; display: flex; flex-direction: column; gap: var(--space-5); }
  .panel { display: flex; flex-direction: column; gap: var(--space-4); min-inline-size: 0; position: sticky; top: 0; }

  @media (max-width: 1280px) {
    .has-panel { grid-template-columns: minmax(0, 1fr); }
    .panel { display: none; }
  }
</style>
