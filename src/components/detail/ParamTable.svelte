<script lang="ts">
  import { t } from '$lib/i18n';
  import type { Parameter } from '$lib/types';

  let { parameters }: { parameters: Parameter[] } = $props();
</script>

<div class="wrap">
  <table>
    <thead>
      <tr>
        <th class="opt">{t.item.paramOption}</th>
        <th>{t.item.paramDescription}</th>
        <th class="req">{t.item.paramRequired}</th>
        <th class="def">{t.item.paramDefault}</th>
      </tr>
    </thead>
    <tbody>
      {#each parameters as param (param.id)}
        <tr>
          <td class="opt"><code class="ltr" dir="ltr">{param.name}</code></td>
          <td>{param.description ?? '—'}</td>
          <td class="req" class:required={param.isRequired}>{param.isRequired ? t.item.yes : t.item.no}</td>
          <td class="def ltr" dir="ltr">{param.defaultValue ?? param.valueHint ?? '-'}</td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<style>
  /* The table scrolls inside its own container so the page never scrolls sideways. */
  .wrap { overflow-x: auto; border: 1px solid var(--border); border-radius: var(--radius-md); }
  table { inline-size: 100%; border-collapse: collapse; font-size: 13px; min-inline-size: 480px; }
  th, td { padding: 9px 12px; text-align: start; border-block-end: 1px solid var(--border); }
  th { background: var(--surface-muted); font-weight: 600; font-size: 12.5px; color: var(--text-secondary); }
  tbody tr:last-child td { border-block-end: none; }
  tbody tr:nth-child(even) { background: var(--surface-muted); }
  .opt { inline-size: 1%; white-space: nowrap; }
  .opt code { font-size: 12.5px; color: var(--accent-purple); }
  .req { inline-size: 1%; white-space: nowrap; color: var(--text-tertiary); }
  .req.required { color: var(--danger); font-weight: 600; }
  .def { inline-size: 1%; white-space: nowrap; color: var(--text-secondary); }
</style>
