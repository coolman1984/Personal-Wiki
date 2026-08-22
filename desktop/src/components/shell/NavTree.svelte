<script lang="ts">
  // Svelte 5 recurses by importing the component itself rather than <svelte:self>.
  import Self from './NavTree.svelte';
  import Icon from '$components/ui/Icon.svelte';
  import { href } from '$lib/router.svelte';
  import type { CategoryNode } from '$lib/types';

  let {
    nodes,
    activeSlug = null,
    depth = 0
  }: { nodes: CategoryNode[]; activeSlug?: string | null; depth?: number } = $props();

  // Expansion is per-node local state; the tree itself is entirely data-driven
  // and recurses to whatever depth `parent_id` produces.
  let expanded = $state<Record<string, boolean>>({});

  function isOpen(node: CategoryNode): boolean {
    if (node.slug in expanded) return expanded[node.slug]!;
    return containsActive(node);
  }

  function containsActive(node: CategoryNode): boolean {
    if (!activeSlug) return false;
    if (node.slug === activeSlug) return true;
    return node.children.some(containsActive);
  }
</script>

<ul class="tree" style="--depth:{depth}">
  {#each nodes as node (node.id)}
    <li>
      <div class="row" class:active={node.slug === activeSlug}>
        <a class="link" href={href(`/c/${node.slug}`)}>
          <Icon name={node.iconKey} size={17} />
          <span class="label">{node.name}</span>
          {#if node.itemCount > 0}<span class="count ltr">{node.itemCount}</span>{/if}
        </a>
        {#if node.children.length > 0}
          <button
            type="button"
            class="toggle"
            class:open={isOpen(node)}
            aria-expanded={isOpen(node)}
            aria-label={node.name}
            onclick={() => (expanded = { ...expanded, [node.slug]: !isOpen(node) })}
          >
            <Icon name="chevron" size={14} />
          </button>
        {/if}
      </div>
      {#if node.children.length > 0 && isOpen(node)}
        <Self nodes={node.children} {activeSlug} depth={depth + 1} />
      {/if}
    </li>
  {/each}
</ul>

<style>
  .tree { list-style: none; margin: 0; padding: 0; }
  .tree[style*='--depth:0'] { padding-inline-start: 0; }
  .tree:not([style*='--depth:0']) { padding-inline-start: var(--space-4); }
  .row {
    display: flex;
    align-items: center;
    gap: 2px;
    border-radius: var(--radius-md);
  }
  .row:hover { background: var(--surface-muted); }
  .row.active { background: var(--accent-blue-soft); }
  .row.active .link { color: var(--accent-blue); font-weight: 600; }
  .link {
    flex: 1;
    min-inline-size: 0;
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: 8px 10px;
    color: var(--text-secondary);
    text-decoration: none;
    font-size: 13px;
    border-radius: var(--radius-md);
  }
  .label { flex: 1; min-inline-size: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .count { font-size: 11px; color: var(--text-tertiary); }
  .toggle {
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    padding: 6px;
    border-radius: var(--radius-sm);
    display: grid;
    place-items: center;
    transition: transform var(--dur-fast) var(--ease);
  }
  .toggle.open { transform: rotate(-90deg); }
  :global([dir='rtl']) .toggle { transform: scaleX(-1); }
  :global([dir='rtl']) .toggle.open { transform: scaleX(-1) rotate(-90deg); }
</style>
