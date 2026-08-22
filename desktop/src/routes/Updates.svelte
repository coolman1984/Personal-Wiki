<script lang="ts">
  import PageHeader from '$components/ui/PageHeader.svelte';
  import StatTile from '$components/ui/StatTile.svelte';
  import StatusPill from '$components/ui/StatusPill.svelte';
  import Icon from '$components/ui/Icon.svelte';
  import IconTile from '$components/ui/IconTile.svelte';
  import EmptyState from '$components/ui/EmptyState.svelte';
  import ErrorState from '$components/ui/ErrorState.svelte';
  import Skeleton from '$components/ui/Skeleton.svelte';
  import {
    applyBatch, importPackFile, listAudit, listBatches, listChanges, listSources,
    pickPackFile, revertBatch, reviewChanges
  } from '$lib/api';
  import { href, navigate } from '$lib/router.svelte';
  import { absoluteDate, count, relativeTime } from '$lib/format';
  import { appState, guard, toasts } from '$lib/state.svelte';
  import { t } from '$lib/i18n';
  import type { AuditEntry, Source, StagedChange, UpdateBatch } from '$lib/types';

  let sources = $state<Source[] | null>(null);
  let batches = $state<UpdateBatch[]>([]);
  let changes = $state<StagedChange[]>([]);
  let audit = $state<AuditEntry[]>([]);
  let error = $state<string | null>(null);
  let tab = $state<'all' | 'add' | 'update' | 'remove'>('all');
  let busy = $state(false);

  async function load() {
    error = null;
    try {
      [sources, batches, changes, audit] = await Promise.all([
        listSources(), listBatches(), listChanges(null, 'pending'), listAudit(12)
      ]);
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
  }
  load();

  const pendingBatch = $derived(batches.find((batch) => batch.pendingCount > 0 || batch.approvedCount > 0) ?? null);
  const lastApplied = $derived(batches.find((batch) => batch.status === 'applied' || batch.status === 'partially_applied') ?? null);
  const healthy = $derived((sources ?? []).every((source) => source.health === 'ok' || source.health === 'unchecked'));
  const counts = $derived({
    all: changes.length,
    add: changes.filter((c) => c.changeType === 'add').length,
    update: changes.filter((c) => c.changeType === 'update' || c.changeType === 'relink').length,
    remove: changes.filter((c) => c.changeType === 'remove_candidate' || c.changeType === 'deprecate').length
  });
  const visibleChanges = $derived(
    changes.filter((change) => {
      if (tab === 'all') return true;
      if (tab === 'add') return change.changeType === 'add';
      if (tab === 'update') return change.changeType === 'update' || change.changeType === 'relink';
      return change.changeType === 'remove_candidate' || change.changeType === 'deprecate';
    })
  );

  async function importPack() {
    const path = await guard(() => pickPackFile(), 'تعذّر فتح الملف');
    if (!path) return;
    busy = true;
    const result = await guard(() => importPackFile(path, false), 'تعذّر استيراد الحزمة');
    busy = false;
    if (!result) return;
    toasts.push(`تم تجهيز ${result.staged} تغييرًا للمراجعة`);
    await appState.refresh();
    navigate(`/updates/batches/${result.batchId}`);
  }

  async function review(status: 'approved' | 'rejected') {
    if (!pendingBatch) return;
    busy = true;
    await guard(() => reviewChanges(pendingBatch.id, null, status), 'تعذّر تحديث المراجعة');
    busy = false;
    await load();
    await appState.refresh();
  }

  async function apply() {
    if (!pendingBatch) return;
    busy = true;
    const outcome = await guard(() => applyBatch(pendingBatch.id), 'تعذّر تطبيق التغييرات');
    busy = false;
    if (outcome) toasts.push(`تم تطبيق ${count(outcome.applied)} تغييرًا`);
    await load();
    await appState.refresh();
  }

  async function undo() {
    if (!lastApplied) return;
    busy = true;
    const outcome = await guard(() => revertBatch(lastApplied.id), 'تعذّر التراجع');
    busy = false;
    if (outcome) toasts.push('تم التراجع عن آخر دفعة');
    await load();
    await appState.refresh();
  }
</script>

<div class="page">
  <PageHeader icon="cloud" accent="blue" title={t.updates.title} subtitle={t.updates.subtitle} />

  {#if error}
    <ErrorState detail={error} onRetry={load} />
  {:else if !sources}
    <div class="cols"><Skeleton variant="card" count={3} /></div>
  {:else}
    <div class="grid">
      <section class="card sources">
        <div class="card-head">
          <div>
            <h2>{t.updates.connectedSources}</h2>
            <p>{t.updates.sourcesSubtitle}</p>
          </div>
        </div>
        <ul data-testid="source-list">
          {#each sources as source (source.id)}
            <li>
              <IconTile icon="globe" accent={source.isOfficial ? 'green' : 'slate'} size={32} />
              <span class="s-text">
                <span class="s-name">{source.name}</span>
                <span class="s-url ltr" dir="ltr">{source.url ?? '—'}</span>
              </span>
              <span class="s-type">{source.sourceType}</span>
              <StatusPill kind={source.health} />
            </li>
          {/each}
        </ul>
        <footer class="s-foot">
          <span class="muted">
            {count(sources.filter((s) => s.isActive).length)} / {count(sources.length)} مصادر نشطة
          </span>
          <button type="button" class="ghost" onclick={importPack}>
            <Icon name="upload" size={14} />{t.updates.importFile}
          </button>
        </footer>
      </section>

      <div class="middle">
        <div class="summary">
          <StatTile
            icon="check"
            accent={appState.syncTone === 'error' ? 'red' : 'green'}
            value={appState.lastSyncAt ? relativeTime(appState.lastSyncAt) : '—'}
            label={t.updates.lastSync}
            sublabel={appState.lastSyncStatus ?? t.sync.never}
          />
          <StatTile icon="clock" accent="blue" value="—" label={t.updates.nextSync} sublabel="يدوي / عبر وكيل" />
          <StatTile icon="refresh" accent="orange" value={count(counts.all)} label={t.updates.pending} sublabel="عنصر للمراجعة" />
          <StatTile
            icon="shield"
            accent={healthy ? 'green' : 'orange'}
            value={healthy ? '✓' : '!'}
            label={t.updates.overall}
            sublabel={healthy ? t.updates.healthy : t.updates.unhealthy}
          />
        </div>

        <section class="card changes">
          <nav class="tabs" aria-label="تصنيف التغييرات">
            {#each [['all', t.updates.allChanges, counts.all], ['add', t.updates.added, counts.add], ['update', t.updates.updated, counts.update], ['remove', t.updates.removed, counts.remove]] as entry (entry[0])}
              <button
                type="button"
                class:active={tab === entry[0]}
                onclick={() => (tab = entry[0] as typeof tab)}
              >
                <span>{entry[1]}</span>
                <span class="tab-count ltr">{count(entry[2] as number)}</span>
              </button>
            {/each}
          </nav>

          {#if visibleChanges.length === 0}
            <EmptyState compact icon="check" title={t.empty.changes.title} body={t.empty.changes.body} />
          {:else}
            <ul class="change-list" data-testid="change-list">
              {#each visibleChanges.slice(0, 12) as change (change.id)}
                <li>
                  <StatusPill kind={change.changeType} />
                  <span class="c-text">
                    <span class="c-title">{change.title ?? change.entityKey}</span>
                    {#if change.detail}<span class="c-detail">{change.detail}</span>{/if}
                  </span>
                  <span class="c-time">{relativeTime(change.detectedAt)}</span>
                </li>
              {/each}
            </ul>
            {#if pendingBatch}
              <a class="see-all" href={href(`/updates/batches/${pendingBatch.id}`)}>
                <Icon name="grid" size={14} />{t.updates.review}
              </a>
            {/if}
          {/if}
        </section>
      </div>

      <section class="card timeline">
        <div class="card-head"><h2>{t.updates.timeline}</h2></div>
        {#if audit.length === 0}
          <EmptyState compact icon="clock" title={t.empty.activity.title} />
        {:else}
          <ol data-testid="audit-timeline">
            {#each audit as entry (entry.id)}
              <li>
                <span class="marker"><Icon name={entry.action.includes('fail') ? 'warning' : 'check'} size={12} /></span>
                <span class="t-text">
                  <span class="t-title">{entry.action}</span>
                  <span class="t-detail">{entry.actor}</span>
                </span>
                <span class="t-time">{relativeTime(entry.at)}</span>
              </li>
            {/each}
          </ol>
        {/if}
      </section>
    </div>

    <section class="card approval">
      <div class="approval-info">
        <h2>{t.updates.approvalTitle}</h2>
        <p>{t.updates.approvalSubtitle}</p>
        <div class="counters">
          <span><strong class="ltr">{count(counts.all)}</strong> {t.updates.totalItems}</span>
          <span><strong class="ltr">{count(counts.add)}</strong> {t.updates.newCommands}</span>
          <span><strong class="ltr">{count(counts.update)}</strong> {t.updates.updated}</span>
          <span><strong class="ltr">{count(counts.remove)}</strong> {t.updates.removed}</span>
        </div>
      </div>
      <div class="approval-actions">
        <button type="button" class="primary" disabled={!pendingBatch || busy} onclick={() => pendingBatch && navigate(`/updates/batches/${pendingBatch.id}`)}>
          <Icon name="grid" size={15} />{t.updates.review}
        </button>
        <button type="button" class="ok" disabled={!pendingBatch || busy} onclick={() => review('approved')}>
          <Icon name="check" size={15} />{t.updates.approveAll}
        </button>
        <button type="button" class="warn" disabled={!pendingBatch || busy} onclick={() => review('rejected')}>
          <Icon name="close" size={15} />{t.updates.rejectAll}
        </button>
        <button type="button" class="ghost" disabled={!pendingBatch || pendingBatch.approvedCount === 0 || busy} onclick={apply}>
          <Icon name="upload" size={15} />{t.updates.applyApproved}
        </button>
        <button type="button" class="ghost" disabled={!lastApplied || busy} onclick={undo}>
          <Icon name="refresh" size={15} />{t.updates.undoLast}
        </button>
      </div>
      <p class="reversible"><Icon name="shield" size={14} />{t.updates.reversible}</p>
    </section>

    <section class="card app-update">
      <div>
        <h2>{t.updates.appStatus}</h2>
        <p class="muted">{t.updates.appUpToDate} — <span class="ltr">v1.0.0</span></p>
      </div>
      <span class="divider" aria-hidden="true"></span>
      <div>
        <h2>{t.updates.knowledgeStatus}</h2>
        <p class="muted">
          {lastApplied ? `${t.updates.applied}: ${absoluteDate(lastApplied.appliedAt)}` : t.updates.noPending}
        </p>
      </div>
    </section>
  {/if}
</div>

<style>
  .page { display: flex; flex-direction: column; gap: var(--space-4); }
  .cols { display: grid; grid-template-columns: repeat(3, 1fr); gap: var(--space-4); }
  .grid { display: grid; grid-template-columns: 320px minmax(0, 1fr) 300px; gap: var(--space-4); align-items: start; }
  .card-head { display: flex; align-items: flex-start; justify-content: space-between; }
  .card-head h2 { margin: 0; font-size: 15px; font-weight: 600; }
  .card-head p { margin: 4px 0 0; font-size: 12px; color: var(--text-secondary); }

  .sources { padding: var(--space-4); display: flex; flex-direction: column; gap: var(--space-3); }
  .sources ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: var(--space-2); }
  .sources li { display: flex; align-items: center; gap: var(--space-2); }
  .s-text { flex: 1; min-inline-size: 0; display: flex; flex-direction: column; }
  .s-name { font-size: 13px; font-weight: 500; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .s-url { font-size: 11px; color: var(--text-tertiary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .s-type { font-size: 11px; color: var(--text-tertiary); }
  .s-foot { display: flex; align-items: center; justify-content: space-between; padding-block-start: var(--space-2); border-block-start: 1px solid var(--border); font-size: 12px; }

  .middle { display: flex; flex-direction: column; gap: var(--space-3); min-inline-size: 0; }
  .summary { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: var(--space-2); }
  .changes { padding: var(--space-4); display: flex; flex-direction: column; gap: var(--space-3); }
  .tabs { display: flex; gap: var(--space-2); flex-wrap: wrap; }
  .tabs button {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: var(--surface-muted);
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    padding: 6px 12px;
    font-size: 12.5px;
    color: var(--text-secondary);
  }
  .tabs button.active { background: var(--accent-blue-soft); border-color: var(--accent-blue); color: var(--accent-blue); font-weight: 600; }
  .tab-count { font-size: 11px; }
  .change-list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: var(--space-2); }
  .change-list li { display: flex; align-items: center; gap: var(--space-3); padding: var(--space-2); border-radius: var(--radius-sm); }
  .change-list li:hover { background: var(--surface-muted); }
  .c-text { flex: 1; min-inline-size: 0; display: flex; flex-direction: column; }
  .c-title { font-size: 13px; font-weight: 500; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .c-detail { font-size: 11.5px; color: var(--text-tertiary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .c-time { font-size: 11.5px; color: var(--text-tertiary); flex: none; }
  .see-all { display: inline-flex; align-items: center; gap: 6px; align-self: flex-start; font-size: 12.5px; color: var(--accent-blue); text-decoration: none; }

  .timeline { padding: var(--space-4); display: flex; flex-direction: column; gap: var(--space-3); }
  .timeline ol { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: var(--space-3); }
  .timeline li { display: flex; gap: var(--space-2); align-items: flex-start; }
  .marker {
    inline-size: 22px; block-size: 22px;
    border-radius: 50%;
    background: var(--accent-green-soft);
    color: var(--accent-green);
    display: grid; place-items: center;
    flex: none;
  }
  .t-text { flex: 1; min-inline-size: 0; display: flex; flex-direction: column; }
  .t-title { font-size: 12.5px; font-weight: 500; }
  .t-detail { font-size: 11px; color: var(--text-tertiary); }
  .t-time { font-size: 11px; color: var(--text-tertiary); }

  .approval { padding: var(--space-5); display: flex; flex-wrap: wrap; gap: var(--space-4); align-items: center; }
  .approval-info { flex: 1; min-inline-size: 260px; }
  .approval-info h2 { margin: 0; font-size: 15px; font-weight: 600; }
  .approval-info p { margin: 4px 0 var(--space-3); font-size: 12.5px; color: var(--text-secondary); }
  .counters { display: flex; flex-wrap: wrap; gap: var(--space-4); font-size: 12px; color: var(--text-secondary); }
  .counters strong { font-size: 18px; color: var(--text-primary); margin-inline-end: 4px; }
  .approval-actions { display: flex; flex-wrap: wrap; gap: var(--space-2); }
  .approval-actions button {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    border-radius: var(--radius-md);
    padding: 10px 16px;
    font-size: 13px;
    font-weight: 500;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-secondary);
  }
  .approval-actions button:disabled { opacity: 0.45; }
  .primary { background: var(--accent-blue) !important; border-color: var(--accent-blue) !important; color: var(--text-inverse) !important; }
  .ok { background: var(--success) !important; border-color: var(--success) !important; color: var(--text-inverse) !important; }
  .warn { color: var(--danger) !important; border-color: color-mix(in srgb, var(--danger) 40%, transparent) !important; background: var(--accent-red-soft) !important; }
  .reversible { flex-basis: 100%; margin: 0; display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--text-tertiary); }

  .app-update { padding: var(--space-4) var(--space-5); display: flex; gap: var(--space-6); align-items: center; }
  .app-update h2 { margin: 0; font-size: 14px; font-weight: 600; }
  .app-update p { margin: 2px 0 0; font-size: 12.5px; }
  .divider { inline-size: 1px; align-self: stretch; background: var(--border); }
  .ghost { background: var(--surface); }

  @media (max-width: 1500px) {
    .grid { grid-template-columns: 300px minmax(0, 1fr); }
    .timeline { grid-column: 1 / -1; }
  }
  @media (max-width: 1100px) {
    .grid { grid-template-columns: minmax(0, 1fr); }
    .summary { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  }
</style>
