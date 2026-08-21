<script lang="ts">
  import { iconSpec } from '$lib/icons';

  let {
    name,
    size = 20,
    stroke = 1.7,
    label = null,
    directional = false
  }: {
    name: string | null | undefined;
    size?: number;
    stroke?: number;
    label?: string | null;
    directional?: boolean;
  } = $props();

  const spec = $derived(iconSpec(name));
  const mirror = $derived(directional || spec.directional === true);
</script>

<svg
  class:icon-directional={mirror}
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={stroke}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden={label ? undefined : 'true'}
  role={label ? 'img' : undefined}
  aria-label={label ?? undefined}
>
  {#each spec.paths as d (d)}
    <path {d} />
  {/each}
  {#each spec.circles ?? [] as c (c.join('-'))}
    <circle cx={c[0]} cy={c[1]} r={c[2]} />
  {/each}
</svg>

<style>
  svg { flex: none; display: block; }
</style>
