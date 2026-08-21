<script lang="ts">
  /** Verification / lifecycle / source-health state, rendered from data. */
  let { kind }: { kind: string } = $props();

  const MAP: Record<string, { label: string; accent: string }> = {
    official: { label: 'رسمي', accent: 'green' },
    verified: { label: 'مُتحقَّق', accent: 'blue' },
    community: { label: 'مجتمعي', accent: 'purple' },
    unverified: { label: 'غير مُتحقَّق', accent: 'slate' },
    demo: { label: 'عرض تجريبي', accent: 'orange' },
    active: { label: 'نشط', accent: 'green' },
    experimental: { label: 'تجريبي', accent: 'purple' },
    deprecated: { label: 'مهجور', accent: 'orange' },
    removed: { label: 'محذوف', accent: 'red' },
    ok: { label: 'متصل', accent: 'green' },
    stale: { label: 'قديم', accent: 'orange' },
    failing: { label: 'تعذّر', accent: 'red' },
    disabled: { label: 'معطّل', accent: 'slate' },
    unchecked: { label: 'لم يُفحص', accent: 'slate' },
    add: { label: 'جديد', accent: 'blue' },
    update: { label: 'محدث', accent: 'orange' },
    deprecate: { label: 'مهجور', accent: 'orange' },
    remove_candidate: { label: 'محذوف', accent: 'red' },
    relink: { label: 'ربط', accent: 'purple' },
    pending: { label: 'بانتظار المراجعة', accent: 'slate' },
    approved: { label: 'معتمد', accent: 'green' },
    rejected: { label: 'مرفوض', accent: 'red' },
    applied: { label: 'مطبّق', accent: 'green' },
    failed: { label: 'فشل', accent: 'red' },
    partially_applied: { label: 'مطبّق جزئيًا', accent: 'orange' },
    reverted: { label: 'تم التراجع', accent: 'slate' }
  };

  const info = $derived(MAP[kind] ?? { label: kind, accent: 'slate' });
</script>

<span class="pill" style="--pill-fg: var(--accent-{info.accent}); --pill-bg: var(--accent-{info.accent}-soft);">
  <span class="dot"></span>{info.label}
</span>

<style>
  .pill {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: 3px 10px;
    border-radius: 999px;
    background: var(--pill-bg);
    color: var(--pill-fg);
    font-size: 12px;
    font-weight: 500;
    white-space: nowrap;
  }
  .dot { inline-size: 6px; block-size: 6px; border-radius: 50%; background: currentColor; }
</style>
