<script lang="ts">
  import Icon from './Icon.svelte';
  let {
    icon = 'book',
    title,
    body = null,
    actionLabel = null,
    onAction = null,
    compact = false
  }: {
    icon?: string;
    title: string;
    body?: string | null;
    actionLabel?: string | null;
    onAction?: (() => void) | null;
    compact?: boolean;
  } = $props();
</script>

<div class="empty" class:compact data-testid="empty-state">
  <span class="glyph"><Icon name={icon} size={compact ? 20 : 28} /></span>
  <strong>{title}</strong>
  {#if body}<p>{body}</p>{/if}
  {#if actionLabel && onAction}
    <button type="button" class="action" onclick={onAction}>{actionLabel}</button>
  {/if}
</div>

<style>
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    padding: var(--space-8) var(--space-6);
    text-align: center;
    color: var(--text-secondary);
  }
  .compact { padding: var(--space-5); }
  .glyph {
    inline-size: 56px; block-size: 56px;
    display: grid; place-items: center;
    border-radius: 50%;
    background: var(--surface-muted);
    color: var(--text-tertiary);
    margin-block-end: var(--space-2);
  }
  .compact .glyph { inline-size: 40px; block-size: 40px; }
  strong { color: var(--text-primary); font-size: 15px; }
  p { margin: 0; max-inline-size: 42ch; font-size: 13px; }
  .action {
    margin-block-start: var(--space-3);
    background: var(--accent-blue);
    color: var(--text-inverse);
    border: none;
    border-radius: var(--radius-md);
    padding: 9px 18px;
    font-weight: 600;
  }
</style>
