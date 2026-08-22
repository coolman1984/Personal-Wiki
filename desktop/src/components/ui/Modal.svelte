<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';
  import { t } from '$lib/i18n';

  let {
    title,
    onClose,
    children,
    footer,
    wide = false
  }: { title: string; onClose: () => void; children: Snippet; footer?: Snippet; wide?: boolean } = $props();

  let dialog: HTMLDivElement | null = $state(null);

  function onKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.stopPropagation();
      onClose();
      return;
    }
    if (event.key !== 'Tab' || !dialog) return;
    // Focus stays inside the dialog while it is open.
    const focusable = dialog.querySelectorAll<HTMLElement>(
      'a[href], button:not([disabled]), input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );
    if (focusable.length === 0) return;
    const first = focusable[0]!;
    const last = focusable[focusable.length - 1]!;
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  $effect(() => {
    dialog?.querySelector<HTMLElement>('input, textarea, button')?.focus();
  });
</script>

<svelte:window onkeydown={onKeydown} />

<div class="scrim" role="presentation" onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events -- Escape is handled on window; the click handler only stops the scrim from closing the dialog. -->
  <div
    class="dialog"
    class:wide
    role="dialog"
    tabindex="-1"
    aria-modal="true"
    aria-label={title}
    bind:this={dialog}
    onclick={(e) => e.stopPropagation()}
  >
    <header>
      <h2>{title}</h2>
      <button type="button" class="close" onclick={onClose} aria-label={t.common.close}>
        <Icon name="close" size={18} />
      </button>
    </header>
    <div class="body">{@render children()}</div>
    {#if footer}<footer>{@render footer()}</footer>{/if}
  </div>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    background: rgba(16, 24, 43, 0.28);
    display: grid;
    place-items: center;
    z-index: 60;
    padding: var(--space-6);
  }
  .dialog {
    background: var(--surface);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-lg);
    inline-size: min(520px, 100%);
    max-block-size: min(80vh, 720px);
    display: flex;
    flex-direction: column;
  }
  .wide { inline-size: min(880px, 100%); }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-5) var(--space-5) var(--space-3);
  }
  h2 { margin: 0; font-size: 17px; font-weight: 600; }
  .close { background: transparent; border: none; color: var(--text-tertiary); border-radius: var(--radius-sm); padding: 4px; }
  .close:hover { color: var(--text-primary); background: var(--surface-muted); }
  .body { padding: 0 var(--space-5) var(--space-5); overflow-y: auto; }
  footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
    padding: var(--space-4) var(--space-5);
    border-block-start: 1px solid var(--border);
  }
</style>
