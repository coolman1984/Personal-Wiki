<script lang="ts">
  import IconTile from '$components/ui/IconTile.svelte';

  let {
    icon,
    accent = 'blue',
    title,
    subtitle,
    onClick
  }: { icon: string; accent?: string; title: string; subtitle: string; onClick: () => void } = $props();
</script>

<button type="button" class="quick" onclick={onClick} data-testid="quick-action">
  <IconTile {icon} accent={accent as never} size={40} />
  <strong>{title}</strong>
  <span>{subtitle}</span>
</button>

<style>
  .quick {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    text-align: center;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: var(--space-4) var(--space-2);
    transition: border-color var(--dur-fast) var(--ease), box-shadow var(--dur-fast) var(--ease);
  }
  .quick:hover { border-color: var(--accent-blue); box-shadow: var(--shadow-sm); }
  strong { font-size: 12.5px; font-weight: 600; }
  span { font-size: 11px; color: var(--text-tertiary); }
</style>
