<script lang="ts">
  let {
    label,
    accent = 'slate',
    size = 'md'
  }: { label: string; accent?: string | null; size?: 'sm' | 'md' } = $props();
  const key = $derived(accent ?? 'slate');
</script>

<span
  class="chip {size}"
  style="--chip-fg: var(--accent-{key}); --chip-bg: var(--accent-{key}-soft);"
>{label}</span>

<style>
  .chip {
    display: inline-flex;
    align-items: center;
    background: var(--chip-bg);
    color: var(--chip-fg);
    border-radius: 999px;
    font-weight: 500;
    white-space: nowrap;
  }
  .md { font-size: 12px; padding: 3px 10px; }
  .sm { font-size: 11px; padding: 2px 8px; }
</style>
