<script lang="ts">
  import Icon from './Icon.svelte';
  import { navigate } from '$lib/router.svelte';
  import { t } from '$lib/i18n';

  let {
    title = t.errors.generic.title,
    body = t.errors.generic.body,
    detail = null,
    onRetry = null
  }: {
    title?: string;
    body?: string;
    detail?: string | null;
    onRetry?: (() => void) | null;
  } = $props();
</script>

<div class="error" role="alert" data-testid="error-state">
  <span class="glyph"><Icon name="warning" size={24} /></span>
  <div class="text">
    <strong>{title}</strong>
    <p>{body}</p>
    {#if detail}<code class="detail">{detail}</code>{/if}
  </div>
  <div class="actions">
    {#if onRetry}<button type="button" class="primary" onclick={onRetry}>{t.common.retry}</button>{/if}
    <button type="button" class="ghost" onclick={() => navigate('/diagnostics')}>{t.common.diagnostics}</button>
  </div>
</div>

<style>
  .error {
    display: flex;
    gap: var(--space-4);
    align-items: flex-start;
    padding: var(--space-5);
    background: var(--accent-red-soft);
    border: 1px solid color-mix(in srgb, var(--danger) 25%, transparent);
    border-radius: var(--radius-lg);
  }
  .glyph { color: var(--danger); padding-block-start: 2px; }
  .text { flex: 1; min-inline-size: 0; }
  strong { display: block; color: var(--text-primary); }
  p { margin: var(--space-1) 0 0; color: var(--text-secondary); font-size: 13px; }
  .detail {
    display: block;
    margin-block-start: var(--space-2);
    font-size: 12px;
    color: var(--text-secondary);
    word-break: break-word;
  }
  .actions { display: flex; gap: var(--space-2); flex: none; }
  button { border-radius: var(--radius-md); padding: 7px 14px; font-size: 13px; font-weight: 500; }
  .primary { background: var(--danger); color: var(--text-inverse); border: none; }
  .ghost { background: var(--surface); color: var(--text-secondary); border: 1px solid var(--border); }
</style>
