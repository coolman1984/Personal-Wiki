<script lang="ts">
  import Icon from './Icon.svelte';
  import { copyText } from '$lib/api';
  import { t } from '$lib/i18n';

  let {
    code,
    copyable = true,
    label = null,
    tone = 'dark'
  }: { code: string; copyable?: boolean; label?: string | null; tone?: 'dark' | 'light' } = $props();

  let copied = $state(false);
  let failed = $state(false);

  async function copy() {
    failed = false;
    try {
      // Copy only. Personal Wiki never executes a command it displays.
      await copyText(code);
      copied = true;
      setTimeout(() => (copied = false), 2000);
    } catch {
      failed = true;
      setTimeout(() => (failed = false), 2500);
    }
  }
</script>

<figure class="block {tone}">
  {#if copyable}
    <button type="button" class="copy" onclick={copy} data-testid="copy-command">
      <Icon name={copied ? 'check' : 'copy'} size={15} />
      <span>{copied ? t.item.copied : (label ?? t.item.copy)}</span>
    </button>
  {/if}
  <pre class="ltr" dir="ltr"><code>{code}</code></pre>
  {#if failed}<figcaption class="failed">تعذّر النسخ إلى الحافظة.</figcaption>{/if}
</figure>

<style>
  .block {
    position: relative;
    margin: 0;
    border-radius: var(--radius-md);
    overflow: hidden;
  }
  .dark { background: #0f1729; color: #e6edf7; }
  .light { background: var(--surface-muted); color: var(--text-primary); border: 1px solid var(--border); }
  pre {
    margin: 0;
    padding: var(--space-4);
    padding-block-start: var(--space-7);
    overflow-x: auto;
    font-size: 13px;
    line-height: 1.75;
    text-align: left;
  }
  code { font-family: var(--font-mono); white-space: pre; }
  .copy {
    position: absolute;
    inset-block-start: var(--space-2);
    inset-inline-end: var(--space-2);
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: rgba(255, 255, 255, 0.1);
    color: inherit;
    border: 1px solid rgba(255, 255, 255, 0.18);
    border-radius: var(--radius-sm);
    padding: 4px 10px;
    font-size: 12px;
    z-index: 1;
  }
  .light .copy { background: var(--surface); border-color: var(--border); color: var(--text-secondary); }
  .copy:hover { background: rgba(255, 255, 255, 0.18); }
  .light .copy:hover { background: var(--surface-sunken); }
  .failed { padding: var(--space-2) var(--space-4); font-size: 12px; color: var(--danger); background: var(--accent-red-soft); }
</style>
