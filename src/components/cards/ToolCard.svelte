<script lang="ts">
  import IconTile from '$components/ui/IconTile.svelte';
  import Icon from '$components/ui/Icon.svelte';
  import { href } from '$lib/router.svelte';
  import { count, relativeTime } from '$lib/format';
  import { t } from '$lib/i18n';
  import type { ToolCard } from '$lib/types';

  let { tool }: { tool: ToolCard } = $props();
</script>

<a class="tool card" href={href(`/tools/${tool.slug}`)} data-testid="tool-card" data-slug={tool.slug}>
  <IconTile icon={tool.iconKey} accent={tool.accentKey} size={56} />
  <h3>{tool.name}</h3>
  {#if tool.description}<p>{tool.description}</p>{/if}
  <div class="metrics">
    <span class="metric">
      <Icon name="terminal" size={14} />
      <span class="ltr">{count(tool.itemCount)}</span>
      <span class="unit">{t.tool.availableCommand}</span>
    </span>
    <span class="metric">
      <Icon name="book" size={14} />
      <span class="ltr">{count(tool.exampleCount)}</span>
      <span class="unit">{t.tool.practicalExample}</span>
    </span>
    <span class="metric">
      <Icon name="clock" size={14} />
      <span class="unit">{relativeTime(tool.lastUpdatedAt)}</span>
    </span>
  </div>
</a>

<style>
  .tool {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-5);
    text-decoration: none;
    color: inherit;
    transition: box-shadow var(--dur-fast) var(--ease), transform var(--dur-fast) var(--ease),
      border-color var(--dur-fast) var(--ease);
  }
  .tool:hover { box-shadow: var(--shadow-md); border-color: var(--border-strong); transform: translateY(-1px); }
  h3 { margin: 0; font-size: 16px; font-weight: 600; }
  p {
    margin: 0;
    color: var(--text-secondary);
    font-size: 13px;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .metrics { display: flex; flex-wrap: wrap; gap: var(--space-2); margin-block-start: auto; padding-block-start: var(--space-2); }
  .metric {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: var(--surface-muted);
    border-radius: var(--radius-sm);
    padding: 5px 9px;
    font-size: 12px;
    color: var(--text-secondary);
  }
  .unit { color: var(--text-tertiary); }
</style>
