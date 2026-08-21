<script lang="ts">
  import PageHeader from '$components/ui/PageHeader.svelte';
  import Icon from '$components/ui/Icon.svelte';
  import ErrorState from '$components/ui/ErrorState.svelte';
  import Skeleton from '$components/ui/Skeleton.svelte';
  import { createBackup, getDiagnostics, getSettings, listBackups, rebuildSearchIndex, restoreBackup, setSetting } from '$lib/api';
  import { href } from '$lib/router.svelte';
  import { absoluteDate, bytes, count } from '$lib/format';
  import { appState, guard, toasts } from '$lib/state.svelte';
  import { t } from '$lib/i18n';
  import type { BackupRecord, Diagnostics } from '$lib/types';

  let diagnostics = $state<Diagnostics | null>(null);
  let backups = $state<BackupRecord[]>([]);
  let settings = $state<Record<string, unknown>>({});
  let error = $state<string | null>(null);
  let busy = $state(false);

  async function load() {
    error = null;
    try {
      [diagnostics, backups, settings] = await Promise.all([getDiagnostics(), listBackups(), getSettings()]);
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
  }
  load();

  const startupScreen = $derived(typeof settings['startupScreen'] === 'string' ? (settings['startupScreen'] as string) : '/');

  async function changeStartup(value: string) {
    await guard(() => setSetting('startupScreen', value), 'تعذّر حفظ الإعداد');
    settings = await getSettings();
  }

  async function backupNow() {
    busy = true;
    const record = await guard(() => createBackup(), 'تعذّر إنشاء نسخة احتياطية');
    busy = false;
    if (record) {
      toasts.push('تم إنشاء نسخة احتياطية');
      backups = await listBackups();
    }
  }

  async function restore(record: BackupRecord) {
    busy = true;
    const result = await guard(() => restoreBackup(record.id), 'تعذّرت الاستعادة');
    busy = false;
    if (result) {
      toasts.push('تمت الاستعادة');
      await appState.refresh();
      await load();
    }
  }

  async function rebuild() {
    busy = true;
    const n = await guard(() => rebuildSearchIndex(), 'تعذّرت إعادة البناء');
    busy = false;
    if (n !== null) {
      toasts.push(`تمت إعادة بناء الفهرس (${count(n)})`);
      await load();
    }
  }
</script>

<div class="page">
  <PageHeader icon="settings" accent="slate" title={t.settings.title} subtitle="إعدادات التطبيق وقاعدة المعرفة والنسخ الاحتياطية." />

  {#if error}
    <ErrorState detail={error} onRetry={load} />
  {:else if !diagnostics}
    <Skeleton variant="row" count={4} />
  {:else}
    <section class="card group">
      <h2>{t.settings.general}</h2>
      <div class="row">
        <span class="label">{t.settings.language}</span>
        <span class="value">العربية · RTL <span class="tertiary">(الإنجليزية لاحقًا)</span></span>
      </div>
      <div class="row">
        <span class="label">{t.settings.startupScreen}</span>
        <select value={startupScreen} onchange={(e) => changeStartup((e.currentTarget as HTMLSelectElement).value)}>
          <option value="/">{t.nav.home}</option>
          <option value="/tools">{t.nav.tools}</option>
          <option value="/favorites">{t.nav.favorites}</option>
          <option value="/search">{t.search.title}</option>
        </select>
      </div>
    </section>

    <section class="card group">
      <h2>{t.settings.knowledge}</h2>
      <div class="row">
        <span class="label">{t.settings.dataFolder}</span>
        <code class="value ltr" dir="ltr">{diagnostics.dbPath}</code>
      </div>
      <div class="row">
        <span class="label">{t.diagnostics.fts}</span>
        <span class="value">
          {diagnostics.ftsStatus} · <span class="ltr">{count(diagnostics.ftsRowCount)}</span>
          <button type="button" class="inline" disabled={busy} onclick={rebuild}>{t.settings.rebuildIndex}</button>
        </span>
      </div>
      <div class="row">
        <span class="label">{t.settings.backups}</span>
        <span class="value">
          <button type="button" class="primary" disabled={busy} onclick={backupNow}>
            <Icon name="download" size={14} />{t.settings.backupNow}
          </button>
        </span>
      </div>
      {#if backups.length > 0}
        <ul class="backups">
          {#each backups.slice(0, 6) as record (record.id)}
            <li>
              <span class="b-reason">{record.reason}</span>
              <span class="b-date">{absoluteDate(record.createdAt)}</span>
              <span class="b-size ltr">{bytes(record.sizeBytes)}</span>
              <span class="b-items"><span class="ltr">{count(record.itemCount)}</span> عنصر</span>
              <button type="button" class="inline" disabled={busy} onclick={() => restore(record)}>{t.settings.restore}</button>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <section class="card group">
      <h2>{t.settings.appUpdates}</h2>
      <div class="row">
        <span class="label">{t.app.version}</span>
        <span class="value ltr">v{diagnostics.appVersion}</span>
      </div>
      <p class="hint">
        تحديث التطبيق منفصل تمامًا عن تحديث المعرفة: الأول يتطلب إصدارًا جديدًا، والثاني يحدث عبر حزم المعرفة.
      </p>
    </section>

    <section class="card group">
      <h2>{t.settings.about}</h2>
      <a class="link-row" href={href('/diagnostics')}>
        <Icon name="database" size={16} />
        <span>{t.diagnostics.title}</span>
        <Icon name="chevron" size={15} directional />
      </a>
    </section>
  {/if}
</div>

<style>
  .page { display: flex; flex-direction: column; gap: var(--space-4); max-inline-size: 900px; }
  .group { padding: var(--space-5); display: flex; flex-direction: column; gap: var(--space-3); }
  .group h2 { margin: 0; font-size: 15px; font-weight: 600; }
  .row { display: flex; align-items: center; justify-content: space-between; gap: var(--space-4); font-size: 13px; padding-block: var(--space-2); border-block-end: 1px solid var(--border); }
  .row:last-of-type { border-block-end: none; }
  .label { color: var(--text-secondary); }
  .value { display: inline-flex; align-items: center; gap: var(--space-2); }
  code.value { font-size: 12px; background: var(--surface-muted); padding: 4px 8px; border-radius: var(--radius-sm); overflow-x: auto; max-inline-size: 420px; }
  select { background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-md); padding: 7px 10px; font-size: 13px; }
  .inline { background: transparent; border: none; color: var(--accent-blue); font-size: 12.5px; padding: 2px 4px; }
  .primary { display: inline-flex; align-items: center; gap: 6px; background: var(--accent-blue); color: var(--text-inverse); border: none; border-radius: var(--radius-md); padding: 7px 14px; font-size: 12.5px; font-weight: 600; }
  .backups { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: var(--space-1); }
  .backups li { display: flex; align-items: center; gap: var(--space-3); font-size: 12px; color: var(--text-secondary); padding: 6px 8px; background: var(--surface-muted); border-radius: var(--radius-sm); }
  .b-reason { font-weight: 600; color: var(--text-primary); min-inline-size: 84px; }
  .b-date { flex: 1; }
  .hint { margin: 0; font-size: 12px; color: var(--text-tertiary); }
  .link-row { display: flex; align-items: center; gap: var(--space-3); text-decoration: none; color: inherit; font-size: 13px; padding: var(--space-2); border-radius: var(--radius-md); }
  .link-row:hover { background: var(--surface-muted); }
  .link-row span { flex: 1; }
</style>
