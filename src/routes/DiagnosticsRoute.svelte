<script lang="ts">
  import PageHeader from '$components/ui/PageHeader.svelte';
  import Icon from '$components/ui/Icon.svelte';
  import ErrorState from '$components/ui/ErrorState.svelte';
  import Skeleton from '$components/ui/Skeleton.svelte';
  import { copyText, getDiagnostics } from '$lib/api';
  import { absoluteDate, bytes, count } from '$lib/format';
  import { guard, toasts } from '$lib/state.svelte';
  import { t } from '$lib/i18n';
  import type { Diagnostics } from '$lib/types';

  let data = $state<Diagnostics | null>(null);
  let error = $state<string | null>(null);

  async function load() {
    error = null;
    try {
      data = await getDiagnostics();
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
  }
  load();

  async function copyReport() {
    if (!data) return;
    const ok = await guard(() => copyText(JSON.stringify(data, null, 2)), 'تعذّر النسخ');
    if (ok !== null) toasts.push(t.item.copied);
  }
</script>

<div class="page">
  <PageHeader icon="database" accent="purple" title={t.diagnostics.title} subtitle={t.diagnostics.subtitle}>
    {#snippet actions()}
      <button type="button" class="ghost" onclick={copyReport}><Icon name="copy" size={15} />{t.diagnostics.copyReport}</button>
    {/snippet}
  </PageHeader>

  {#if error}
    <ErrorState title={t.errors.database.title} detail={error} onRetry={load} />
  {:else if !data}
    <Skeleton variant="row" count={5} />
  {:else}
    <section class="card group" data-testid="diagnostics">
      <div class="row"><span>{t.diagnostics.appVersion}</span><span class="ltr">v{data.appVersion}</span></div>
      <div class="row">
        <span>{t.diagnostics.schema}</span>
        <span class="ltr">v{data.schemaVersion} / v{data.expectedSchemaVersion}</span>
      </div>
      <div class="row"><span>{t.diagnostics.dbPath}</span><code class="ltr" dir="ltr">{data.dbPath}</code></div>
      <div class="row"><span>{t.diagnostics.dbSize}</span><span class="ltr">{bytes(data.dbSizeBytes)}</span></div>
      <div class="row">
        <span>{t.diagnostics.integrity}</span>
        <span class:good={data.integrity === 'ok'} class:bad={data.integrity !== 'ok'}>{data.integrity}</span>
      </div>
      <div class="row">
        <span>{t.diagnostics.foreignKeys}</span>
        <span class:good={data.foreignKeyViolations === 0} class:bad={data.foreignKeyViolations > 0} class="ltr">
          {count(data.foreignKeyViolations)}
        </span>
      </div>
      <div class="row">
        <span>{t.diagnostics.fts}</span>
        <span class:good={data.ftsStatus === 'ok'} class:bad={data.ftsStatus !== 'ok'}>
          {data.ftsStatus} · <span class="ltr">{count(data.ftsRowCount)}</span>
        </span>
      </div>
      {#if data.lastBatch}
        <div class="row">
          <span>{t.diagnostics.lastBatch}</span>
          <span>{data.lastBatch.status} · {absoluteDate(data.lastBatch.createdAt)}</span>
        </div>
      {/if}
      {#if data.lastBackup}
        <div class="row">
          <span>{t.diagnostics.lastBackup}</span>
          <span>{absoluteDate(data.lastBackup.createdAt)} · <span class="ltr">{bytes(data.lastBackup.sizeBytes)}</span></span>
        </div>
      {/if}
    </section>

    <section class="card group">
      <h2>{t.diagnostics.counts}</h2>
      <div class="counts">
        {#each Object.entries(data.counts) as [table, n] (table)}
          <div class="count-cell">
            <strong class="ltr">{count(n)}</strong>
            <span class="ltr">{table}</span>
          </div>
        {/each}
      </div>
    </section>

    <section class="card group">
      <h2>{t.diagnostics.problems}</h2>
      {#if data.problems.length === 0}
        <p class="good"><Icon name="check" size={15} />{t.diagnostics.noProblems}</p>
      {:else}
        <ul class="problems">
          {#each data.problems as problem (problem)}
            <li><Icon name="warning" size={15} />{problem}</li>
          {/each}
        </ul>
      {/if}
    </section>
  {/if}
</div>

<style>
  .page { display: flex; flex-direction: column; gap: var(--space-4); max-inline-size: 900px; }
  .group { padding: var(--space-5); display: flex; flex-direction: column; gap: var(--space-2); }
  .group h2 { margin: 0 0 var(--space-2); font-size: 15px; font-weight: 600; }
  .row { display: flex; align-items: center; justify-content: space-between; gap: var(--space-4); font-size: 13px; padding-block: 7px; border-block-end: 1px solid var(--border); }
  .row:last-child { border-block-end: none; }
  .row > span:first-child { color: var(--text-secondary); }
  code { font-size: 12px; background: var(--surface-muted); padding: 3px 8px; border-radius: var(--radius-sm); max-inline-size: 460px; overflow-x: auto; }
  .good { color: var(--success); display: inline-flex; align-items: center; gap: 6px; }
  .bad { color: var(--danger); }
  .counts { display: grid; grid-template-columns: repeat(auto-fill, minmax(120px, 1fr)); gap: var(--space-2); }
  .count-cell { background: var(--surface-muted); border-radius: var(--radius-md); padding: var(--space-3); display: flex; flex-direction: column; }
  .count-cell strong { font-size: 18px; }
  .count-cell span { font-size: 11px; color: var(--text-tertiary); }
  .problems { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: var(--space-2); font-size: 13px; color: var(--warning); }
  .problems li { display: flex; align-items: center; gap: var(--space-2); }
  .ghost { display: inline-flex; align-items: center; gap: 6px; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-md); padding: 8px 14px; font-size: 13px; }
</style>
