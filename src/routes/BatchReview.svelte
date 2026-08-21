<script lang="ts">
  import Breadcrumbs from '$components/ui/Breadcrumbs.svelte';
  import PageHeader from '$components/ui/PageHeader.svelte';
  import StatusPill from '$components/ui/StatusPill.svelte';
  import Icon from '$components/ui/Icon.svelte';
  import EmptyState from '$components/ui/EmptyState.svelte';
  import ErrorState from '$components/ui/ErrorState.svelte';
  import Skeleton from '$components/ui/Skeleton.svelte';
  import { applyBatch, listBatches, listChanges, reviewChanges } from '$lib/api';
  import { navigate } from '$lib/router.svelte';
  import { absoluteDate, count } from '$lib/format';
  import { appState, guard, toasts } from '$lib/state.svelte';
  import { t } from '$lib/i18n';
  import type { StagedChange, UpdateBatch } from '$lib/types';

  /** The human gate: field-level before/after, per-row approve or reject, then apply. */
  let { batchId }: { batchId: string } = $props();

  let batch = $state<UpdateBatch | null>(null);
  let changes = $state<StagedChange[] | null>(null);
  let error = $state<string | null>(null);
  let expanded = $state<Record<string, boolean>>({});
  let busy = $state(false);

  async function load() {
    error = null;
    try {
      const all = await listBatches();
      batch = all.find((entry) => entry.id === batchId) ?? null;
      changes = await listChanges(batchId, null);
      if (!batch) error = `الدفعة ${batchId} غير موجودة.`;
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
  }
  load();

  async function decide(ids: string[] | null, status: 'approved' | 'rejected') {
    busy = true;
    await guard(() => reviewChanges(batchId, ids, status), 'تعذّر تحديث المراجعة');
    busy = false;
    await load();
    await appState.refresh();
  }

  async function apply() {
    busy = true;
    const outcome = await guard(() => applyBatch(batchId), 'تعذّر تطبيق التغييرات');
    busy = false;
    if (outcome) {
      toasts.push(`تم تطبيق ${count(outcome.applied)} تغييرًا`);
      await appState.refresh();
      await load();
    }
  }
</script>

<div class="page">
  <Breadcrumbs
    crumbs={[
      { label: t.nav.home, path: '/', icon: 'home' },
      { label: t.nav.updates, path: '/updates' },
      { label: `${t.updates.batch} ${batchId.slice(0, 8)}` }
    ]}
  />

  {#if error}
    <ErrorState detail={error} onRetry={load} />
  {:else if !batch || !changes}
    <Skeleton variant="row" count={5} />
  {:else}
    <PageHeader
      icon="refresh"
      accent="blue"
      title={batch.summary ?? `${t.updates.batch} ${batch.id.slice(0, 8)}`}
      subtitle={`${batch.producer ?? batch.createdBy} · ${absoluteDate(batch.createdAt)}`}
    >
      {#snippet badges()}
        <StatusPill kind={batch!.status} />
      {/snippet}
      {#snippet actions()}
        <button type="button" class="ok" disabled={busy || batch!.pendingCount === 0} onclick={() => decide(null, 'approved')}>
          <Icon name="check" size={15} />{t.updates.approveAll}
        </button>
        <button type="button" class="warn" disabled={busy || batch!.pendingCount === 0} onclick={() => decide(null, 'rejected')}>
          <Icon name="close" size={15} />{t.updates.rejectAll}
        </button>
        <button type="button" class="primary" disabled={busy || batch!.approvedCount === 0} onclick={apply}>
          <Icon name="upload" size={15} />{t.updates.applyApproved}
        </button>
      {/snippet}
    </PageHeader>

    <div class="counters">
      <span><strong class="ltr">{count(changes.length)}</strong>{t.updates.totalItems}</span>
      <span><strong class="ltr">{count(batch.pendingCount)}</strong>{t.updates.approve}</span>
      <span><strong class="ltr">{count(batch.approvedCount)}</strong>معتمد</span>
      <span><strong class="ltr">{count(batch.appliedCount)}</strong>{t.updates.applied}</span>
    </div>

    {#if changes.length === 0}
      <EmptyState icon="check" title={t.empty.changes.title} body={t.empty.changes.body} />
    {:else}
      <ul class="changes" data-testid="batch-changes">
        {#each changes as change (change.id)}
          <li class="change card">
            <div class="row">
              <StatusPill kind={change.changeType} />
              <span class="c-text">
                <span class="c-title">{change.title ?? change.entityKey}</span>
                <span class="c-key ltr" dir="ltr">{change.entityType} · {change.entityKey}</span>
              </span>
              <StatusPill kind={change.status} />
              <button
                type="button"
                class="expand"
                aria-expanded={expanded[change.id] ?? false}
                onclick={() => (expanded = { ...expanded, [change.id]: !(expanded[change.id] ?? false) })}
              >
                <Icon name="chevron" size={15} />
              </button>
            </div>

            {#if expanded[change.id]}
              <div class="diff" data-testid="diff-viewer">
                {#if change.fieldDiff.length === 0}
                  <p class="none">لا توجد فروق حقلية.</p>
                {:else}
                  <table>
                    <thead>
                      <tr><th>الحقل</th><th>{t.updates.before}</th><th>{t.updates.after}</th></tr>
                    </thead>
                    <tbody>
                      {#each change.fieldDiff as field (field.field)}
                        <tr>
                          <td class="f ltr" dir="ltr">{field.field}</td>
                          <td class="before">{field.before ?? '—'}</td>
                          <td class="after">{field.after ?? '—'}</td>
                        </tr>
                      {/each}
                    </tbody>
                  </table>
                {/if}
                {#if change.errorText}<p class="err">{change.errorText}</p>{/if}
                {#if change.status === 'pending'}
                  <div class="row-actions">
                    <button type="button" class="ok" disabled={busy} onclick={() => decide([change.id], 'approved')}>
                      {t.updates.approve}
                    </button>
                    <button type="button" class="warn" disabled={busy} onclick={() => decide([change.id], 'rejected')}>
                      {t.updates.reject}
                    </button>
                  </div>
                {/if}
              </div>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}

    <p class="reversible"><Icon name="shield" size={14} />{t.updates.reversible}</p>
    <button type="button" class="back" onclick={() => navigate('/updates')}>
      <Icon name="arrow" size={15} directional />{t.item.backToList}
    </button>
  {/if}
</div>

<style>
  .page { display: flex; flex-direction: column; gap: var(--space-4); max-inline-size: var(--content-max); }
  .counters { display: flex; gap: var(--space-5); font-size: 12.5px; color: var(--text-secondary); }
  .counters strong { display: block; font-size: 20px; color: var(--text-primary); }
  .changes { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: var(--space-2); }
  .change { padding: var(--space-3) var(--space-4); }
  .row { display: flex; align-items: center; gap: var(--space-3); }
  .c-text { flex: 1; min-inline-size: 0; display: flex; flex-direction: column; }
  .c-title { font-size: 13.5px; font-weight: 600; }
  .c-key { font-size: 11px; color: var(--text-tertiary); }
  .expand { background: transparent; border: none; color: var(--text-tertiary); padding: 4px; }
  .expand[aria-expanded='true'] { transform: rotate(90deg); }

  .diff { margin-block-start: var(--space-3); border-block-start: 1px solid var(--border); padding-block-start: var(--space-3); overflow-x: auto; }
  table { inline-size: 100%; border-collapse: collapse; font-size: 12.5px; }
  th, td { padding: 7px 10px; text-align: start; border-block-end: 1px solid var(--border); vertical-align: top; }
  th { color: var(--text-secondary); font-weight: 600; }
  .f { color: var(--accent-purple); white-space: nowrap; }
  .before { color: var(--text-tertiary); text-decoration: line-through; }
  .after { color: var(--text-primary); }
  .none, .err { margin: 0; font-size: 12.5px; }
  .err { color: var(--danger); }
  .row-actions { display: flex; gap: var(--space-2); margin-block-start: var(--space-3); }

  button.ok, button.warn, button.primary {
    display: inline-flex; align-items: center; gap: 6px;
    border-radius: var(--radius-md); padding: 8px 16px; font-size: 13px; font-weight: 500; border: 1px solid transparent;
  }
  .ok { background: var(--success); color: var(--text-inverse); }
  .warn { background: var(--accent-red-soft); color: var(--danger); border-color: color-mix(in srgb, var(--danger) 40%, transparent); }
  .primary { background: var(--accent-blue); color: var(--text-inverse); }
  button:disabled { opacity: 0.45; }

  .reversible { margin: 0; display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--text-tertiary); }
  .back { align-self: flex-start; display: inline-flex; align-items: center; gap: 6px; background: transparent; border: none; color: var(--accent-blue); font-size: 13px; }
</style>
