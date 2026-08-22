<script lang="ts">
  import Icon from '$components/ui/Icon.svelte';
  import type { ResultDiagram } from '$lib/types';

  /**
   * Renders the typed diagram contract only: nodes and edges, drawn by this
   * component. Imported data can never supply markup — see the ResultDiagram
   * validation in crates/pw-core/src/pack.rs.
   */
  let { diagram }: { diagram: ResultDiagram } = $props();

  const root = $derived(
    diagram.kind === 'fan_out'
      ? (diagram.nodes.find((n) => !diagram.edges.some((e) => e.to === n.id)) ?? diagram.nodes[0])
      : null
  );
  const children = $derived(root ? diagram.nodes.filter((n) => n.id !== root.id) : diagram.nodes);
</script>

<div class="diagram" class:fan={diagram.kind === 'fan_out'} data-testid="result-diagram">
  {#if root}
    <div class="node root" style="--node-accent: var(--accent-{root.accentKey ?? 'green'});">
      <strong>{root.label}</strong>
      {#if root.sublabel}<span class="ltr">{root.sublabel}</span>{/if}
    </div>
    <div class="branch" aria-hidden="true"></div>
  {/if}
  <div class="children">
    {#each children as node (node.id)}
      <div class="node" style="--node-accent: var(--accent-{node.accentKey ?? 'purple'});">
        <strong>{node.label}</strong>
        {#if node.sublabel}<span class="ltr">{node.sublabel}</span>{/if}
        {#if node.status}
          <span class="status {node.status}">
            <Icon name={node.status === 'ok' ? 'check' : node.status === 'warning' ? 'warning' : 'clock'} size={12} />
          </span>
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .diagram {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-4);
    background: var(--surface-muted);
    border-radius: var(--radius-md);
    overflow-x: auto;
  }
  .children { display: flex; flex-direction: column; gap: var(--space-2); }
  .diagram:not(.fan) .children { flex-direction: row; align-items: center; }
  .node {
    position: relative;
    min-inline-size: 148px;
    background: var(--surface);
    border: 1px solid color-mix(in srgb, var(--node-accent) 35%, transparent);
    border-inline-start: 3px solid var(--node-accent);
    border-radius: var(--radius-sm);
    padding: 9px 12px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 12.5px;
  }
  .node strong { font-weight: 600; }
  .node span { color: var(--text-tertiary); font-size: 11.5px; }
  .branch {
    inline-size: 28px;
    block-size: 1px;
    border-block-start: 2px dashed var(--border-strong);
    flex: none;
  }
  .status {
    position: absolute;
    inset-block-start: 50%;
    inset-inline-end: -10px;
    transform: translateY(-50%);
    inline-size: 18px;
    block-size: 18px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    color: var(--text-inverse);
  }
  .status.ok { background: var(--success); }
  .status.pending { background: var(--warning); }
  .status.warning { background: var(--danger); }
</style>
