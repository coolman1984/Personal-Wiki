<script lang="ts">
  import IconTile from './IconTile.svelte';

  let {
    icon,
    accent = 'blue',
    value,
    label,
    sublabel = null
  }: {
    icon: string;
    accent?: string;
    value: string;
    label: string;
    sublabel?: string | null;
  } = $props();
</script>

<div class="stat card">
  <IconTile {icon} accent={accent as never} size={40} />
  <div class="body">
    <span class="label">{label}</span>
    <strong class="value ltr">{value}</strong>
    {#if sublabel}<span class="sub">{sublabel}</span>{/if}
  </div>
</div>

<style>
  .stat {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-4);
    min-inline-size: 0;
  }
  .body { display: flex; flex-direction: column; min-inline-size: 0; }
  .label { font-size: 12px; color: var(--text-secondary); }
  .value { font-size: 22px; font-weight: 700; line-height: 1.2; }
  .sub { font-size: 12px; color: var(--text-tertiary); }
</style>
