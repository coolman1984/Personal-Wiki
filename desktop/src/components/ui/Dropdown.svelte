<script lang="ts">
  import Icon from './Icon.svelte';

  let {
    label,
    options,
    selected = [],
    onChange,
    multi = true,
    icon = 'filter'
  }: {
    label: string;
    options: Array<{ value: string; label: string; count?: number }>;
    selected?: string[];
    onChange: (next: string[]) => void;
    multi?: boolean;
    icon?: string;
  } = $props();

  let open = $state(false);
  let root: HTMLDivElement | null = $state(null);

  function toggle(value: string) {
    if (!multi) {
      onChange(selected.includes(value) ? [] : [value]);
      open = false;
      return;
    }
    onChange(selected.includes(value) ? selected.filter((v) => v !== value) : [...selected, value]);
  }

  function onWindowClick(event: MouseEvent) {
    if (open && root && !root.contains(event.target as Node)) open = false;
  }
</script>

<svelte:window onclick={onWindowClick} onkeydown={(e) => e.key === 'Escape' && (open = false)} />

<div class="dropdown" bind:this={root}>
  <button
    type="button"
    class="trigger"
    class:active={selected.length > 0}
    aria-expanded={open}
    onclick={() => (open = !open)}
  >
    <Icon name={icon} size={15} />
    <span>{label}</span>
    {#if selected.length > 0}<span class="badge ltr">{selected.length}</span>{/if}
    <Icon name="chevron" size={14} />
  </button>
  {#if open}
    <div class="menu" role="listbox" aria-multiselectable={multi}>
      {#each options as option (option.value)}
        <button
          type="button"
          role="option"
          aria-selected={selected.includes(option.value)}
          class="option"
          onclick={() => toggle(option.value)}
        >
          <span class="check" class:on={selected.includes(option.value)}>
            {#if selected.includes(option.value)}<Icon name="check" size={12} />{/if}
          </span>
          <span class="label">{option.label}</span>
          {#if option.count !== undefined}<span class="count ltr">{option.count}</span>{/if}
        </button>
      {:else}
        <p class="none">لا توجد خيارات.</p>
      {/each}
    </div>
  {/if}
</div>

<style>
  .dropdown { position: relative; }
  .trigger {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 8px 12px;
    font-size: 13px;
    color: var(--text-secondary);
    white-space: nowrap;
  }
  .trigger:hover { border-color: var(--border-strong); }
  .active { border-color: var(--accent-blue); color: var(--accent-blue); background: var(--accent-blue-soft); }
  .badge {
    background: var(--accent-blue);
    color: var(--text-inverse);
    border-radius: 999px;
    font-size: 11px;
    min-inline-size: 18px;
    text-align: center;
    padding: 0 5px;
  }
  .menu {
    position: absolute;
    inset-block-start: calc(100% + 6px);
    inset-inline-start: 0;
    min-inline-size: 230px;
    max-block-size: 320px;
    overflow-y: auto;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
    padding: var(--space-1);
    z-index: 30;
  }
  .option {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    inline-size: 100%;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    padding: 7px 8px;
    font-size: 13px;
    text-align: start;
  }
  .option:hover { background: var(--surface-muted); }
  .check {
    inline-size: 16px; block-size: 16px;
    border: 1px solid var(--border-strong);
    border-radius: 5px;
    display: grid; place-items: center;
    flex: none;
  }
  .check.on { background: var(--accent-blue); border-color: var(--accent-blue); color: var(--text-inverse); }
  .label { flex: 1; min-inline-size: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .count { color: var(--text-tertiary); font-size: 12px; }
  .none { margin: 0; padding: var(--space-3); font-size: 13px; color: var(--text-tertiary); }
</style>
