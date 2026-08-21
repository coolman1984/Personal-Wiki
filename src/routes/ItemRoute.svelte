<script lang="ts">
  import Breadcrumbs from '$components/ui/Breadcrumbs.svelte';
  import KnowledgeDetail from '$components/detail/KnowledgeDetail.svelte';
  import ItemListRow from '$components/cards/ItemListRow.svelte';
  import Icon from '$components/ui/Icon.svelte';
  import Dropdown from '$components/ui/Dropdown.svelte';
  import EmptyState from '$components/ui/EmptyState.svelte';
  import ErrorState from '$components/ui/ErrorState.svelte';
  import Skeleton from '$components/ui/Skeleton.svelte';
  import Modal from '$components/ui/Modal.svelte';
  import { getItem, itemsForTool, listItemTypes, recordOpen, saveNote } from '$lib/api';
  import { navigate } from '$lib/router.svelte';
  import { count } from '$lib/format';
  import { guard, toasts } from '$lib/state.svelte';
  import { t } from '$lib/i18n';
  import type { ItemDetail, ItemSummary } from '$lib/types';

  /** Master + detail (reference 03): the tool's items on one side, the item on the other. */
  let { slug }: { slug: string } = $props();

  let item = $state<ItemDetail | null>(null);
  let siblings = $state<ItemSummary[]>([]);
  let itemTypes = $state<string[]>([]);
  let error = $state<string | null>(null);
  let scopedQuery = $state('');
  let typeFilter = $state<string[]>([]);
  let noteOpen = $state(false);
  let noteTitle = $state('');
  let noteBody = $state('');

  async function load() {
    error = null;
    item = null;
    try {
      const detail = await getItem(slug);
      item = detail;
      await recordOpen(detail.id);
      if (detail.toolSlug) {
        siblings = await itemsForTool(detail.toolSlug, null, 'alpha');
      } else {
        siblings = [];
      }
      itemTypes = await listItemTypes();
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
  }

  $effect(() => {
    void slug;
    load();
  });

  const visible = $derived(
    siblings.filter((sibling) => {
      const matchesText =
        scopedQuery.trim() === '' ||
        `${sibling.title} ${sibling.shortDescription ?? ''} ${sibling.syntax ?? ''}`
          .toLowerCase()
          .includes(scopedQuery.trim().toLowerCase());
      const matchesType = typeFilter.length === 0 || typeFilter.includes(sibling.itemType);
      return matchesText && matchesType;
    })
  );

  function onListKeydown(event: KeyboardEvent) {
    if (event.key !== 'ArrowDown' && event.key !== 'ArrowUp') return;
    event.preventDefault();
    const index = visible.findIndex((sibling) => sibling.slug === slug);
    const next = event.key === 'ArrowDown' ? index + 1 : index - 1;
    const target = visible[next];
    if (target) navigate(`/items/${target.slug}`);
  }

  async function submitNote() {
    if (!item || noteTitle.trim() === '') return;
    const saved = await guard(
      () => saveNote({ itemId: item!.id, title: noteTitle, body: noteBody }),
      'تعذّر حفظ الملاحظة'
    );
    if (saved) {
      toasts.push(t.notes.saved);
      noteOpen = false;
      noteTitle = '';
      noteBody = '';
      await load();
    }
  }

  const crumbs = $derived([
    { label: t.nav.home, path: '/', icon: 'home' },
    ...(item?.toolSlug ? [{ label: t.nav.tools, path: '/tools' }] : []),
    ...(item?.toolName && item.toolSlug ? [{ label: item.toolName, path: `/tools/${item.toolSlug}` }] : []),
    ...(item ? [{ label: item.title }] : [])
  ]);
</script>

{#if error && !item}
  <ErrorState title={t.errors.notFound.title} body={t.errors.notFound.body} detail={error} onRetry={load} />
{:else if !item}
  <div class="loading"><Skeleton variant="line" count={2} /><Skeleton variant="card" count={2} /></div>
{:else}
  <div class="page">
    <Breadcrumbs {crumbs} />

    <div class="split">
      <section class="master" aria-label="قائمة الأوامر">
        <div class="master-head">
          <label class="scoped">
            <Icon name="search" size={16} />
            <input
              type="search"
              bind:value={scopedQuery}
              placeholder={t.search.scopedPlaceholder(item.toolName ?? '')}
              aria-label={t.search.scopedPlaceholder(item.toolName ?? '')}
            />
          </label>
          <Dropdown
            label="تصفية"
            options={itemTypes.map((type) => ({ value: type, label: type }))}
            selected={typeFilter}
            onChange={(next) => (typeFilter = next)}
          />
        </div>
        <p class="count">{t.common.availableCount(count(visible.length))}</p>

        <div class="rows" role="listbox" tabindex="0" onkeydown={onListKeydown} aria-label="الأوامر">
          {#each visible as sibling (sibling.id)}
            <ItemListRow
              item={sibling}
              selected={sibling.slug === slug}
              onSelect={(next) => navigate(`/items/${next.slug}`)}
            />
          {:else}
            <EmptyState compact icon="search" title={t.empty.search.title} body={t.empty.search.body} />
          {/each}
        </div>
      </section>

      <section class="detail">
        <KnowledgeDetail {item} onChanged={load} onAddNote={() => (noteOpen = true)} />
      </section>
    </div>
  </div>

  {#if noteOpen}
    <Modal title={t.item.addNote} onClose={() => (noteOpen = false)}>
      <label class="field">
        <span>{t.notes.titleField}</span>
        <input bind:value={noteTitle} type="text" />
      </label>
      <label class="field">
        <span>{t.notes.bodyField}</span>
        <textarea bind:value={noteBody} rows="5"></textarea>
      </label>
      {#snippet footer()}
        <button type="button" class="ghost" onclick={() => (noteOpen = false)}>{t.common.cancel}</button>
        <button type="button" class="primary" onclick={submitNote}>{t.common.save}</button>
      {/snippet}
    </Modal>
  {/if}
{/if}

<style>
  .page { display: flex; flex-direction: column; gap: var(--space-4); }
  .loading { display: flex; flex-direction: column; gap: var(--space-4); }
  .split { display: grid; grid-template-columns: 380px minmax(0, 1fr); gap: var(--space-4); align-items: start; }

  .master {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    position: sticky;
    top: 0;
    max-block-size: calc(100vh - var(--titlebar-height) - var(--space-8));
  }
  .master-head { display: flex; gap: var(--space-2); }
  .scoped {
    flex: 1;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    background: var(--surface-muted);
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    padding: 0 12px;
    color: var(--text-tertiary);
    min-inline-size: 0;
  }
  .scoped:focus-within { background: var(--surface); border-color: var(--accent-blue); }
  .scoped input { flex: 1; min-inline-size: 0; background: transparent; border: none; outline: none; padding: 9px 0; font-size: 13px; }
  .count { margin: 0; font-size: 12px; color: var(--text-tertiary); }
  .rows { display: flex; flex-direction: column; gap: var(--space-2); overflow-y: auto; padding-inline-end: 2px; }
  .rows:focus-visible { outline-offset: 4px; }

  .detail {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: var(--space-5);
    min-inline-size: 0;
  }

  .field { display: flex; flex-direction: column; gap: 6px; margin-block-end: var(--space-3); font-size: 13px; }
  .field input, .field textarea {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 9px 12px;
    font-size: 13px;
    resize: vertical;
  }
  .ghost { background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-md); padding: 8px 16px; font-size: 13px; }
  .primary { background: var(--accent-blue); color: var(--text-inverse); border: none; border-radius: var(--radius-md); padding: 8px 16px; font-size: 13px; font-weight: 600; }

  @media (max-width: 1180px) {
    .split { grid-template-columns: minmax(0, 1fr); }
    .master { position: static; max-block-size: 380px; }
  }
</style>
