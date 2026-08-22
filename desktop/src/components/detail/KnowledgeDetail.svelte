<script lang="ts">
  import Icon from '$components/ui/Icon.svelte';
  import IconTile from '$components/ui/IconTile.svelte';
  import TagChip from '$components/ui/TagChip.svelte';
  import StatusPill from '$components/ui/StatusPill.svelte';
  import CodeBlock from '$components/ui/CodeBlock.svelte';
  import ParamTable from './ParamTable.svelte';
  import ResultDiagramView from './ResultDiagram.svelte';
  import { iconForItemType } from '$lib/icons';
  import { copyText, openExternal, setFavorite, setPin } from '$lib/api';
  import { absoluteDate } from '$lib/format';
  import { guard, toasts } from '$lib/state.svelte';
  import { navigate } from '$lib/router.svelte';
  import { t } from '$lib/i18n';
  import type { ItemDetail } from '$lib/types';

  /**
   * ONE detail renderer for every screen that shows a knowledge item.
   * Each section renders only when its data exists — a link shows purpose and
   * source, a concept shows purpose and when-to-use, a command shows everything.
   * No placeholders, no "N/A".
   */
  let {
    item,
    layout = 'two-column',
    onBack = null,
    onChanged = null,
    onAddNote = null
  }: {
    item: ItemDetail;
    layout?: 'two-column' | 'stacked';
    onBack?: (() => void) | null;
    onChanged?: (() => void) | null;
    onAddNote?: (() => void) | null;
  } = $props();

  let favorite = $state(false);
  let pinned = $state(false);
  let exampleIndex = $state(0);

  $effect(() => {
    favorite = item.isFavorite;
    pinned = item.isPinned;
    exampleIndex = 0;
  });

  const isCommand = $derived(['command', 'cli_command', 'slash_command', 'snippet'].includes(item.itemType));
  const officialSource = $derived(item.sources.find((s) => s.isOfficial) ?? item.sources[0] ?? null);
  const sourceHref = $derived(officialSource?.sourceUrl ?? officialSource?.url ?? null);
  const activeExample = $derived(item.examples[exampleIndex] ?? item.examples[0] ?? null);

  // "when to use" is stored as text; render list markers as a real list.
  const whenToUseLines = $derived(
    (item.whenToUse ?? '')
      .split('\n')
      .map((line) => line.replace(/^[-•*]\s*/, '').trim())
      .filter(Boolean)
  );
  const whenToUseIsList = $derived((item.whenToUse ?? '').split('\n').length > 1);

  async function toggleFavorite() {
    const next = !favorite;
    favorite = next;
    const ok = await guard(() => setFavorite(item.id, next), 'تعذّر تحديث المفضلة');
    if (ok === null) favorite = !next;
    else onChanged?.();
  }

  async function togglePin() {
    const next = !pinned;
    pinned = next;
    const ok = await guard(() => setPin(item.id, next), 'تعذّر التثبيت');
    if (ok === null) pinned = !next;
    else onChanged?.();
  }

  async function copySyntax() {
    if (!item.syntax) return;
    const ok = await guard(() => copyText(item.syntax!), 'تعذّر النسخ');
    if (ok !== null) toasts.push(t.item.copied);
  }
</script>

<article class="detail" class:stacked={layout === 'stacked'} data-testid="knowledge-detail" data-slug={item.slug}>
  <header class="head">
    {#if onBack}
      <button type="button" class="back" onclick={onBack}>
        <Icon name="arrow" size={16} directional />
        <span>{t.item.backToList}</span>
      </button>
    {/if}
    <div class="identity">
      <IconTile icon={item.iconKey ?? iconForItemType(item.itemType)} accent={item.accentKey ?? 'blue'} size={52} />
      <div class="titles">
        <h1 class:mono={isCommand} class:ltr={isCommand}>{item.title}</h1>
        {#if item.subtitle}<p class="sub">{item.subtitle}</p>{/if}
        <div class="badges">
          {#if item.toolName}<TagChip label={item.toolName} accent="blue" />{/if}
          <StatusPill kind={item.verificationStatus} />
          {#if item.lifecycleStatus !== 'active'}<StatusPill kind={item.lifecycleStatus} />{/if}
          {#each item.tags as tag (tag.id)}
            <TagChip label={tag.name} accent={tag.accentKey} />
          {/each}
        </div>
      </div>
    </div>

    <div class="actions">
      <button type="button" class:on={pinned} onclick={togglePin}>
        <Icon name="pin" size={15} /><span>{pinned ? t.item.unpin : t.item.pin}</span>
      </button>
      <button type="button" class:on={favorite} onclick={toggleFavorite} data-testid="detail-favorite">
        <Icon name="star" size={15} /><span>{favorite ? t.item.unfavorite : t.item.favorite}</span>
      </button>
      {#if sourceHref}
        <button type="button" onclick={() => guard(() => openExternal(sourceHref), 'تعذّر فتح المصدر')}>
          <Icon name="external" size={15} /><span>{t.item.openSource}</span>
        </button>
      {/if}
      {#if item.syntax}
        <button type="button" class="primary" onclick={copySyntax} data-testid="detail-copy">
          <Icon name="copy" size={15} /><span>{t.item.copy}</span>
        </button>
      {/if}
    </div>
  </header>

  <div class="columns">
    <div class="col">
      {#if item.shortDescription || item.fullDescription}
        <section class="block" data-section="purpose">
          <h2><span class="glyph blue"><Icon name="target" size={16} /></span>{t.item.purpose}</h2>
          <p>{item.fullDescription ?? item.shortDescription}</p>
        </section>
      {/if}

      {#if item.whenToUse}
        <section class="block" data-section="when">
          <h2><span class="glyph purple"><Icon name="clock" size={16} /></span>{t.item.whenToUse}</h2>
          {#if whenToUseIsList}
            <ul>
              {#each whenToUseLines as line (line)}<li>{line}</li>{/each}
            </ul>
          {:else}
            <p>{item.whenToUse}</p>
          {/if}
        </section>
      {/if}

      {#if item.syntax}
        <section class="block" data-section="syntax">
          <h2><span class="glyph teal"><Icon name="terminal" size={16} /></span>{t.item.syntax}</h2>
          <CodeBlock code={item.syntax} tone="light" />
        </section>
      {/if}

      {#if item.parameters.length > 0}
        <section class="block" data-section="parameters">
          <h2><span class="glyph blue"><Icon name="sliders" size={16} /></span>{t.item.parameters}</h2>
          <ParamTable parameters={item.parameters} />
        </section>
      {/if}

      {#if item.warningText}
        <section class="block warn" data-section="warnings">
          <h2><span class="glyph orange"><Icon name="warning" size={16} /></span>{t.item.warnings}</h2>
          <ul>
            {#each item.warningText.split('\n').map((l) => l.replace(/^[-•*]\s*/, '').trim()).filter(Boolean) as line (line)}
              <li>{line}</li>
            {/each}
          </ul>
        </section>
      {/if}

      {#if item.aliases.length > 0}
        <section class="block" data-section="aliases">
          <h2><span class="glyph slate"><Icon name="search" size={16} /></span>{t.item.aliases}</h2>
          <div class="chips">
            {#each item.aliases as alias (alias)}<TagChip label={alias} accent="slate" />{/each}
          </div>
        </section>
      {/if}
    </div>

    <div class="col">
      {#if item.examples.length > 0 && activeExample}
        <section class="block" data-section="example">
          <h2><span class="glyph green"><Icon name="code" size={16} /></span>{t.item.example}</h2>
          {#if item.examples.length > 1}
            <div class="tabs" role="tablist">
              {#each item.examples as example, index (example.id)}
                <button
                  type="button"
                  role="tab"
                  aria-selected={index === exampleIndex}
                  class:active={index === exampleIndex}
                  onclick={() => (exampleIndex = index)}
                >
                  {example.title ?? `مثال ${index + 1}`}
                </button>
              {/each}
            </div>
          {/if}
          {#if activeExample.description}<p class="ex-desc">{activeExample.description}</p>{/if}
          <CodeBlock code={activeExample.code} />
          {#if activeExample.expectedOutput}
            <div class="output">
              <span class="output-label">الناتج</span>
              <pre class="ltr" dir="ltr">{activeExample.expectedOutput}</pre>
            </div>
          {/if}
        </section>
      {/if}

      {#if item.expectedResult || item.resultDiagram}
        <section class="block" data-section="expected">
          <h2><span class="glyph green"><Icon name="check" size={16} /></span>{t.item.expectedResult}</h2>
          {#if item.expectedResult}<p>{item.expectedResult}</p>{/if}
          {#if item.resultDiagram}<ResultDiagramView diagram={item.resultDiagram} />{/if}
        </section>
      {/if}

      {#if item.sources.length > 0}
        <section class="block" data-section="sources">
          <h2><span class="glyph blue"><Icon name="globe" size={16} /></span>{t.item.source}</h2>
          <ul class="sources">
            {#each item.sources as source (source.sourceId)}
              {@const url = source.sourceUrl ?? source.url}
              <li>
                {#if url}
                  <button type="button" class="source-link" onclick={() => guard(() => openExternal(url), 'تعذّر فتح المصدر')}>
                    <span>{source.name}</span>
                    <Icon name="external" size={14} />
                  </button>
                {:else}
                  <span>{source.name}</span>
                {/if}
                {#if source.isOfficial}<StatusPill kind="official" />{/if}
                {#if source.verifiedAt}<span class="verified">{absoluteDate(source.verifiedAt)}</span>{/if}
              </li>
            {/each}
          </ul>
        </section>
      {/if}

      {#if item.related.length > 0}
        <section class="block" data-section="related">
          <h2><span class="glyph purple"><Icon name="link" size={16} /></span>{t.item.related}</h2>
          <div class="chips">
            {#each item.related as related (related.id)}
              <button type="button" class="related" onclick={() => navigate(`/items/${related.slug}`)}>
                {related.title}
              </button>
            {/each}
          </div>
        </section>
      {/if}

      {#if item.notes.length > 0}
        <section class="block notes" data-section="notes">
          <h2><span class="glyph orange"><Icon name="note" size={16} /></span>{t.item.myNotes}</h2>
          {#each item.notes as note (note.id)}
            <div class="note">
              <strong>{note.title}</strong>
              <p>{note.body}</p>
            </div>
          {/each}
        </section>
      {/if}

      {#if onAddNote}
        <button type="button" class="add-note" onclick={onAddNote}>
          <Icon name="plus" size={15} /><span>{t.item.addNote}</span>
        </button>
      {/if}
    </div>
  </div>

  <footer class="foot">
    <Icon name="clock" size={14} />
    <span>{t.item.lastUpdated}: {absoluteDate(item.lastVerifiedAt ?? item.updatedAt)}</span>
  </footer>
</article>

<style>
  .detail { display: flex; flex-direction: column; gap: var(--space-4); min-inline-size: 0; }
  .head { display: flex; flex-direction: column; gap: var(--space-4); }
  .back {
    align-self: flex-start;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: transparent;
    border: none;
    color: var(--accent-blue);
    font-size: 13px;
    padding: 4px 6px;
    border-radius: var(--radius-sm);
  }
  .back:hover { background: var(--accent-blue-soft); }
  .identity { display: flex; gap: var(--space-4); align-items: flex-start; }
  .titles { min-inline-size: 0; flex: 1; }
  h1 { margin: 0; font-size: 22px; font-weight: 700; line-height: 1.35; word-break: break-word; }
  h1.mono { font-family: var(--font-mono); font-size: 19px; }
  .sub { margin: var(--space-1) 0 0; color: var(--text-secondary); font-size: 13.5px; }
  .badges { display: flex; flex-wrap: wrap; gap: 6px; margin-block-start: var(--space-3); }

  .actions { display: flex; flex-wrap: wrap; gap: var(--space-2); }
  .actions button {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 8px 14px;
    font-size: 13px;
    color: var(--text-secondary);
  }
  .actions button:hover { border-color: var(--border-strong); color: var(--text-primary); }
  .actions button.on { color: var(--accent-orange); border-color: var(--accent-orange); background: var(--accent-orange-soft); }
  .actions button.primary { background: var(--accent-blue); border-color: var(--accent-blue); color: var(--text-inverse); font-weight: 600; }

  .columns { display: grid; grid-template-columns: 1fr 1fr; gap: var(--space-4); align-items: start; }
  .stacked .columns, .columns:has(.col:only-child) { grid-template-columns: 1fr; }
  .col { display: flex; flex-direction: column; gap: var(--space-4); min-inline-size: 0; }

  .block {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    min-inline-size: 0;
  }
  .block.warn { background: var(--accent-orange-soft); border-color: color-mix(in srgb, var(--accent-orange) 30%, transparent); }
  h2 { margin: 0; font-size: 15px; font-weight: 600; display: flex; align-items: center; gap: var(--space-2); }
  .glyph { display: inline-grid; place-items: center; inline-size: 26px; block-size: 26px; border-radius: 8px; }
  .glyph.blue { background: var(--accent-blue-soft); color: var(--accent-blue); }
  .glyph.green { background: var(--accent-green-soft); color: var(--accent-green); }
  .glyph.purple { background: var(--accent-purple-soft); color: var(--accent-purple); }
  .glyph.orange { background: var(--accent-orange-soft); color: var(--accent-orange); }
  .glyph.teal { background: var(--accent-teal-soft); color: var(--accent-teal); }
  .glyph.slate { background: var(--accent-slate-soft); color: var(--accent-slate); }
  .block p { margin: 0; font-size: 13.5px; color: var(--text-secondary); line-height: 1.75; }
  .block ul { margin: 0; padding-inline-start: var(--space-5); font-size: 13.5px; color: var(--text-secondary); line-height: 1.9; }
  .chips { display: flex; flex-wrap: wrap; gap: 6px; }
  .related {
    background: var(--accent-blue-soft);
    color: var(--accent-blue);
    border: none;
    border-radius: var(--radius-sm);
    padding: 6px 12px;
    font-size: 12.5px;
  }
  .related:hover { background: var(--accent-blue); color: var(--text-inverse); }

  .tabs { display: flex; gap: 4px; flex-wrap: wrap; }
  .tabs button {
    background: var(--surface-muted);
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    padding: 4px 10px;
    font-size: 12px;
    color: var(--text-secondary);
  }
  .tabs button.active { background: var(--accent-blue-soft); border-color: var(--accent-blue); color: var(--accent-blue); }
  .ex-desc { font-size: 12.5px; }
  .output { background: var(--surface-muted); border-radius: var(--radius-sm); padding: var(--space-3); }
  .output-label { font-size: 11px; color: var(--text-tertiary); }
  .output pre { margin: 4px 0 0; font-size: 12.5px; overflow-x: auto; text-align: left; }

  .sources { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: var(--space-2); }
  .sources li { display: flex; align-items: center; gap: var(--space-2); flex-wrap: wrap; font-size: 13px; }
  .source-link {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: transparent;
    border: none;
    color: var(--accent-blue);
    padding: 2px 4px;
    border-radius: var(--radius-sm);
    font-size: 13px;
  }
  .source-link:hover { text-decoration: underline; }
  .verified { font-size: 11.5px; color: var(--text-tertiary); }

  .notes .note { background: var(--surface-muted); border-radius: var(--radius-sm); padding: var(--space-3); }
  .notes strong { font-size: 13px; }
  .notes p { margin: 4px 0 0; font-size: 12.5px; white-space: pre-wrap; }
  .add-note {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    background: transparent;
    border: 1px dashed var(--border-strong);
    border-radius: var(--radius-md);
    padding: 10px;
    color: var(--text-secondary);
    font-size: 13px;
  }
  .add-note:hover { border-color: var(--accent-blue); color: var(--accent-blue); }

  .foot {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    color: var(--text-tertiary);
    font-size: 12.5px;
    padding-block-start: var(--space-2);
    border-block-start: 1px solid var(--border);
  }

  @media (max-width: 1100px) {
    .columns { grid-template-columns: 1fr; }
  }
</style>
