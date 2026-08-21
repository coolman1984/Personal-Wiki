<script lang="ts">
  import Icon from './Icon.svelte';
  import { href } from '$lib/router.svelte';

  let { crumbs }: { crumbs: Array<{ label: string; path?: string; icon?: string }> } = $props();
</script>

<nav class="crumbs" aria-label="مسار التنقل">
  {#each crumbs as crumb, index (crumb.label + index)}
    {#if index > 0}
      <span class="sep" aria-hidden="true"><Icon name="chevron" size={14} directional /></span>
    {/if}
    {#if crumb.path && index < crumbs.length - 1}
      <a href={href(crumb.path)}>
        {#if crumb.icon}<Icon name={crumb.icon} size={15} />{/if}
        <span>{crumb.label}</span>
      </a>
    {:else}
      <span class="current" aria-current="page">
        {#if crumb.icon}<Icon name={crumb.icon} size={15} />{/if}
        <span>{crumb.label}</span>
      </span>
    {/if}
  {/each}
</nav>

<style>
  .crumbs {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: 13px;
    color: var(--text-secondary);
    flex-wrap: wrap;
  }
  a, .current { display: inline-flex; align-items: center; gap: 6px; }
  a { color: var(--text-secondary); text-decoration: none; border-radius: var(--radius-sm); }
  a:hover { color: var(--accent-blue); }
  .current { color: var(--text-primary); font-weight: 600; }
  .sep { color: var(--text-tertiary); display: inline-flex; }
</style>
