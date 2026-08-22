<script lang="ts">
  import TaskCardView from '$components/cards/TaskCard.svelte';
  import PageHeader from '$components/ui/PageHeader.svelte';
  import EmptyState from '$components/ui/EmptyState.svelte';
  import ErrorState from '$components/ui/ErrorState.svelte';
  import Skeleton from '$components/ui/Skeleton.svelte';
  import { listTasks } from '$lib/api';
  import { t } from '$lib/i18n';
  import type { TaskCard } from '$lib/types';

  let tasks = $state<TaskCard[] | null>(null);
  let error = $state<string | null>(null);

  async function load() {
    error = null;
    try {
      tasks = await listTasks();
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
  }
  load();
</script>

<div class="page">
  <PageHeader
    icon="terminal"
    accent="purple"
    title={t.home.tasksTitle}
    subtitle="ابدأ من ما تريد إنجازه، وستجد الأوامر من كل الأدوات في مكان واحد."
  />

  {#if error}
    <ErrorState detail={error} onRetry={load} />
  {:else if !tasks}
    <div class="grid"><Skeleton variant="row" count={8} /></div>
  {:else if tasks.length === 0}
    <EmptyState icon="terminal" title={t.empty.items.title} body={t.empty.items.body} />
  {:else}
    <div class="grid" data-testid="task-grid">
      {#each tasks as task (task.id)}
        <TaskCardView {task} />
      {/each}
    </div>
  {/if}
</div>

<style>
  .page { display: flex; flex-direction: column; gap: var(--space-5); max-inline-size: var(--content-max); }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: var(--space-3); }
</style>
