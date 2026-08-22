<script lang="ts">
  import PageHeader from '$components/ui/PageHeader.svelte';
  import Icon from '$components/ui/Icon.svelte';
  import EmptyState from '$components/ui/EmptyState.svelte';
  import ErrorState from '$components/ui/ErrorState.svelte';
  import Skeleton from '$components/ui/Skeleton.svelte';
  import { deleteNote, listNotes, saveNote } from '$lib/api';
  import { href } from '$lib/router.svelte';
  import { relativeTime } from '$lib/format';
  import { guard, toasts } from '$lib/state.svelte';
  import { t } from '$lib/i18n';
  import type { Note } from '$lib/types';

  let notes = $state<Note[] | null>(null);
  let error = $state<string | null>(null);
  let selectedId = $state<string | null>(null);
  let draftTitle = $state('');
  let draftBody = $state('');
  let dirty = $state(false);

  const selected = $derived((notes ?? []).find((note) => note.id === selectedId) ?? null);

  async function load() {
    error = null;
    try {
      notes = await listNotes();
      if (!selectedId && notes.length > 0) choose(notes[0]!);
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
  }
  load();

  function choose(note: Note) {
    selectedId = note.id;
    draftTitle = note.title;
    draftBody = note.body;
    dirty = false;
  }

  function startNew() {
    selectedId = null;
    draftTitle = '';
    draftBody = '';
    dirty = true;
  }

  async function persist() {
    if (draftTitle.trim() === '') return;
    const saved = await guard(
      () => saveNote({ id: selectedId, itemId: selected?.itemId ?? null, title: draftTitle, body: draftBody }),
      'تعذّر حفظ الملاحظة'
    );
    if (saved) {
      selectedId = saved.id;
      dirty = false;
      toasts.push(t.notes.saved);
      notes = await listNotes();
    }
  }

  async function remove(note: Note) {
    await guard(() => deleteNote(note.id), 'تعذّر الحذف');
    if (selectedId === note.id) selectedId = null;
    notes = await listNotes();
  }
</script>

<div class="page">
  <PageHeader icon="note" accent="orange" title={t.nav.notes} subtitle="ملاحظاتك الشخصية — لا تتأثر أبدًا بتحديثات المصادر.">
    {#snippet actions()}
      <button type="button" class="primary" onclick={startNew}><Icon name="plus" size={15} />{t.notes.newNote}</button>
    {/snippet}
  </PageHeader>

  {#if error}
    <ErrorState detail={error} onRetry={load} />
  {:else if !notes}
    <Skeleton variant="row" count={4} />
  {:else}
    <div class="split">
      <ul class="list">
        {#each notes as note (note.id)}
          <li>
            <button type="button" class:active={note.id === selectedId} onclick={() => choose(note)}>
              <span class="t">{note.title}</span>
              {#if note.itemTitle}<span class="linked">{note.itemTitle}</span>{/if}
              <span class="time">{relativeTime(note.updatedAt)}</span>
            </button>
            <button type="button" class="del" aria-label={t.common.delete} onclick={() => remove(note)}>
              <Icon name="trash" size={13} />
            </button>
          </li>
        {:else}
          <EmptyState compact icon="note" title={t.empty.notes.title} body={t.empty.notes.body} />
        {/each}
      </ul>

      <section class="editor card">
        {#if selectedId === null && !dirty}
          <EmptyState icon="note" title={t.empty.notes.title} body={t.empty.notes.body} actionLabel={t.notes.newNote} onAction={startNew} />
        {:else}
          <input
            class="title"
            type="text"
            bind:value={draftTitle}
            oninput={() => (dirty = true)}
            placeholder={t.notes.titleField}
            aria-label={t.notes.titleField}
          />
          {#if selected?.itemTitle && selected.itemId}
            <a class="linked-item" href={href(`/items/${selected.itemId}`)}>
              <Icon name="link" size={14} />{t.notes.linkedItem} {selected.itemTitle}
            </a>
          {/if}
          <textarea
            bind:value={draftBody}
            oninput={() => (dirty = true)}
            rows="16"
            placeholder={t.notes.bodyField}
            aria-label={t.notes.bodyField}
          ></textarea>
          <div class="editor-foot">
            <span class="state">{dirty ? 'تغييرات غير محفوظة' : t.notes.saved}</span>
            <button type="button" class="primary" onclick={persist} disabled={!dirty}>{t.common.save}</button>
          </div>
        {/if}
      </section>
    </div>
  {/if}
</div>

<style>
  .page { display: flex; flex-direction: column; gap: var(--space-5); }
  .split { display: grid; grid-template-columns: 300px minmax(0, 1fr); gap: var(--space-4); align-items: start; }
  .list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: var(--space-1); }
  .list li { display: flex; align-items: center; gap: 2px; }
  .list li > button:first-child {
    flex: 1;
    min-inline-size: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 10px 12px;
    text-align: start;
  }
  .list li > button.active { border-color: var(--accent-blue); background: var(--accent-blue-soft); }
  .t { font-size: 13.5px; font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .linked { font-size: 11.5px; color: var(--accent-blue); }
  .time { font-size: 11px; color: var(--text-tertiary); }
  .del { background: transparent; border: none; color: var(--text-tertiary); padding: 6px; border-radius: var(--radius-sm); }
  .del:hover { color: var(--danger); background: var(--accent-red-soft); }

  .editor { padding: var(--space-4); display: flex; flex-direction: column; gap: var(--space-3); }
  .title { border: none; border-block-end: 1px solid var(--border); padding: 6px 0; font-size: 17px; font-weight: 600; outline: none; background: transparent; }
  .linked-item { display: inline-flex; align-items: center; gap: 6px; font-size: 12px; color: var(--accent-blue); text-decoration: none; }
  textarea { border: 1px solid var(--border); border-radius: var(--radius-md); padding: var(--space-3); font-size: 13.5px; line-height: 1.8; resize: vertical; }
  .editor-foot { display: flex; align-items: center; justify-content: space-between; }
  .state { font-size: 12px; color: var(--text-tertiary); }
  .primary { display: inline-flex; align-items: center; gap: 6px; background: var(--accent-blue); color: var(--text-inverse); border: none; border-radius: var(--radius-md); padding: 8px 16px; font-size: 13px; font-weight: 600; }
  .primary:disabled { opacity: 0.5; }

  @media (max-width: 1000px) { .split { grid-template-columns: minmax(0, 1fr); } }
</style>
