<script lang="ts">
  import AppShell from '$components/shell/AppShell.svelte';
  import Home from './routes/Home.svelte';
  import Tools from './routes/Tools.svelte';
  import ToolFamily from './routes/ToolFamily.svelte';
  import Tasks from './routes/Tasks.svelte';
  import Collection from './routes/Collection.svelte';
  import ItemRoute from './routes/ItemRoute.svelte';
  import Search from './routes/Search.svelte';
  import Favorites from './routes/Favorites.svelte';
  import Notes from './routes/Notes.svelte';
  import Links from './routes/Links.svelte';
  import Updates from './routes/Updates.svelte';
  import BatchReview from './routes/BatchReview.svelte';
  import Settings from './routes/Settings.svelte';
  import DiagnosticsRoute from './routes/DiagnosticsRoute.svelte';
  import EmptyState from '$components/ui/EmptyState.svelte';
  import { createRouteState } from '$lib/router.svelte';
  import { appState } from '$lib/state.svelte';
  import { t } from '$lib/i18n';

  const router = createRouteState();
  const route = $derived(router.route);

  appState.refresh();
</script>

<AppShell {route}>
  {#key route.path + route.query.toString()}
    {#if route.name === 'home'}
      <Home />
    {:else if route.name === 'tools'}
      <Tools />
    {:else if route.name === 'tool'}
      <ToolFamily slug={route.params.slug ?? ''} section={route.params.section ?? null} />
    {:else if route.name === 'tasks'}
      <Tasks />
    {:else if route.name === 'task'}
      <Collection kind="task" slug={route.params.slug ?? ''} {route} />
    {:else if route.name === 'category'}
      <Collection kind="category" slug={route.params.slug ?? ''} {route} />
    {:else if route.name === 'item'}
      <ItemRoute slug={route.params.slug ?? ''} />
    {:else if route.name === 'search'}
      <Search {route} />
    {:else if route.name === 'favorites'}
      <Favorites />
    {:else if route.name === 'notes'}
      <Notes />
    {:else if route.name === 'links'}
      <Links />
    {:else if route.name === 'updates'}
      <Updates />
    {:else if route.name === 'batch'}
      <BatchReview batchId={route.params.id ?? ''} />
    {:else if route.name === 'settings'}
      <Settings />
    {:else if route.name === 'diagnostics'}
      <DiagnosticsRoute />
    {:else}
      <EmptyState icon="question" title={t.errors.notFound.title} body={t.errors.notFound.body} />
    {/if}
  {/key}
</AppShell>
