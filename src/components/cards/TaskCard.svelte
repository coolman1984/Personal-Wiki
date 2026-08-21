<script lang="ts">
  import IconTile from '$components/ui/IconTile.svelte';
  import { href } from '$lib/router.svelte';
  import { count } from '$lib/format';
  import type { TaskCard } from '$lib/types';

  let { task }: { task: TaskCard } = $props();
</script>

<a class="task card" href={href(`/tasks/${task.slug}`)} data-testid="task-card" data-slug={task.slug}>
  <IconTile icon={task.iconKey} accent="blue" size={44} />
  <div class="text">
    <h3>{task.name}</h3>
    {#if task.description}<p>{task.description}</p>{/if}
  </div>
  <div class="meta">
    <span class="ltr">{count(task.itemCount)}</span>
    <span class="tertiary">عنصر</span>
    {#if task.toolCount > 1}
      <span class="tools">
        <span class="ltr">{count(task.toolCount)}</span>
        <span class="tertiary">أدوات</span>
      </span>
    {/if}
  </div>
</a>

<style>
  .task {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-4);
    text-decoration: none;
    color: inherit;
    transition: box-shadow var(--dur-fast) var(--ease), border-color var(--dur-fast) var(--ease);
  }
  .task:hover { box-shadow: var(--shadow-md); border-color: var(--border-strong); }
  .text { flex: 1; min-inline-size: 0; }
  h3 { margin: 0; font-size: 15px; font-weight: 600; }
  p {
    margin: 2px 0 0;
    color: var(--text-secondary);
    font-size: 12.5px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .meta { display: flex; align-items: center; gap: 5px; font-size: 12px; flex: none; }
  .tools { display: inline-flex; gap: 5px; margin-inline-start: var(--space-3); }
</style>
