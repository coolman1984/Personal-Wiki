<script lang="ts">
  let {
    variant = 'card',
    count = 1
  }: { variant?: 'card' | 'row' | 'line' | 'tile'; count?: number } = $props();
</script>

{#each Array.from({ length: count }, (_, i) => i) as index (index)}
  <div class="s {variant} skeleton" data-testid="skeleton"></div>
{/each}

<style>
  .card { block-size: 168px; border-radius: var(--radius-lg); }
  .row { block-size: 68px; border-radius: var(--radius-md); }
  .tile { block-size: 84px; border-radius: var(--radius-lg); }
  .line { block-size: 14px; border-radius: 999px; margin-block: 6px; }
</style>
