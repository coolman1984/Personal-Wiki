<script lang="ts">
  import PageHeader from '$components/ui/PageHeader.svelte';
  import LinkCard from '$components/cards/LinkCard.svelte';
  import Modal from '$components/ui/Modal.svelte';
  import Icon from '$components/ui/Icon.svelte';
  import EmptyState from '$components/ui/EmptyState.svelte';
  import ErrorState from '$components/ui/ErrorState.svelte';
  import Skeleton from '$components/ui/Skeleton.svelte';
  import { deleteLink, listLinks, listTools, saveLink } from '$lib/api';
  import { guard, toasts } from '$lib/state.svelte';
  import { t } from '$lib/i18n';
  import type { SavedLink, ToolCard } from '$lib/types';

  let links = $state<SavedLink[] | null>(null);
  let tools = $state<ToolCard[]>([]);
  let error = $state<string | null>(null);
  let editing = $state<SavedLink | null>(null);
  let open = $state(false);
  let form = $state({ title: '', url: '', description: '', toolId: '' });
  let formError = $state<string | null>(null);

  async function load() {
    error = null;
    try {
      [links, tools] = await Promise.all([listLinks(), listTools()]);
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
  }
  load();

  function startNew() {
    editing = null;
    form = { title: '', url: '', description: '', toolId: '' };
    formError = null;
    open = true;
  }

  function startEdit(link: SavedLink) {
    editing = link;
    form = {
      title: link.title,
      url: link.url,
      description: link.description ?? '',
      toolId: link.toolId ?? ''
    };
    formError = null;
    open = true;
  }

  async function submit() {
    formError = null;
    try {
      // The core rejects anything that is not an absolute http(s) url.
      await saveLink({
        id: editing?.id ?? null,
        title: form.title,
        url: form.url,
        description: form.description || null,
        toolId: form.toolId || null
      });
      open = false;
      toasts.push('تم الحفظ');
      await load();
    } catch (err) {
      formError = err instanceof Error ? err.message : String(err);
    }
  }

  async function remove() {
    if (!editing) return;
    await guard(() => deleteLink(editing!.id), 'تعذّر الحذف');
    open = false;
    await load();
  }
</script>

<div class="page">
  <PageHeader icon="link" accent="green" title={t.nav.links} subtitle="روابطك المحفوظة ومراجعك المفضلة.">
    {#snippet actions()}
      <button type="button" class="primary" onclick={startNew}><Icon name="plus" size={15} />{t.links.newLink}</button>
    {/snippet}
  </PageHeader>

  {#if error}
    <ErrorState detail={error} onRetry={load} />
  {:else if !links}
    <div class="grid"><Skeleton variant="row" count={6} /></div>
  {:else if links.length === 0}
    <EmptyState icon="link" title={t.empty.links.title} body={t.empty.links.body} actionLabel={t.links.newLink} onAction={startNew} />
  {:else}
    <div class="grid" data-testid="link-grid">
      {#each links as link (link.id)}<LinkCard {link} onEdit={startEdit} />{/each}
    </div>
  {/if}
</div>

{#if open}
  <Modal title={editing ? t.common.edit : t.links.newLink} onClose={() => (open = false)}>
    <label class="field"><span>{t.links.titleField}</span><input bind:value={form.title} type="text" /></label>
    <label class="field"><span>{t.links.urlField}</span><input bind:value={form.url} type="url" dir="ltr" class="ltr" placeholder="https://" /></label>
    <label class="field"><span>{t.links.descField}</span><textarea bind:value={form.description} rows="3"></textarea></label>
    <label class="field">
      <span>{t.links.toolField}</span>
      <select bind:value={form.toolId}>
        <option value="">—</option>
        {#each tools as tool (tool.id)}<option value={tool.id}>{tool.name}</option>{/each}
      </select>
    </label>
    {#if formError}<p class="form-error">{formError}</p>{/if}
    {#snippet footer()}
      {#if editing}<button type="button" class="danger" onclick={remove}>{t.common.delete}</button>{/if}
      <button type="button" class="ghost" onclick={() => (open = false)}>{t.common.cancel}</button>
      <button type="button" class="primary" onclick={submit}>{t.common.save}</button>
    {/snippet}
  </Modal>
{/if}

<style>
  .page { display: flex; flex-direction: column; gap: var(--space-5); max-inline-size: var(--content-max); }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: var(--space-3); }
  .field { display: flex; flex-direction: column; gap: 6px; margin-block-end: var(--space-3); font-size: 13px; }
  .field input, .field textarea, .field select {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 9px 12px;
    font-size: 13px;
  }
  .form-error { margin: 0; color: var(--danger); font-size: 12.5px; }
  .primary { display: inline-flex; align-items: center; gap: 6px; background: var(--accent-blue); color: var(--text-inverse); border: none; border-radius: var(--radius-md); padding: 8px 16px; font-size: 13px; font-weight: 600; }
  .ghost { background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-md); padding: 8px 16px; font-size: 13px; }
  .danger { background: transparent; border: 1px solid var(--danger); color: var(--danger); border-radius: var(--radius-md); padding: 8px 16px; font-size: 13px; margin-inline-end: auto; }
</style>
