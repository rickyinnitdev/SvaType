<script lang="ts">
  import type { Cluster } from '../engine';
  import { messages } from '../i18n/index.svelte';
  let {
    clusters,
    cursor,
    errors = new Set<number>(),
  }: { clusters: Cluster[]; cursor: number; errors?: Set<number> } = $props();
  let start = $derived(Math.max(0, cursor - 8));
  let visible = $derived(clusters.slice(start, start + 120));
  let t = $derived(messages());
</script>

<div class="track" aria-label={t.target}>
  {#each visible as cluster, i (start + i)}
    <span
      class="cluster"
      class:done={start + i < cursor}
      class:current={start + i === cursor}
      class:wrong={errors.has(start + i)}
      aria-current={start + i === cursor ? 'step' : undefined}
      >{cluster.text === ' '
        ? '␣'
        : cluster.text === '\n'
          ? '↵'
          : cluster.text}</span
    >
  {/each}
</div>

<style>
  .track {
    contain: layout paint;
    display: flex;
    align-content: flex-start;
    flex-wrap: wrap;
    gap: 5px;
    max-height: 186px;
    overflow: hidden;
    padding: 12px 4px;
    min-height: 120px;
  }
  .cluster {
    font-family: 'Kantumruy Pro', 'Noto Sans Khmer', sans-serif;
    font-size: var(--font-target);
    line-height: 1.9;
    min-width: 26px;
    text-align: center;
    border-bottom: 3px solid transparent;
    padding: 0 3px;
    white-space: pre;
  }
  .done {
    color: var(--success);
  }
  .current {
    border-color: var(--primary);
    background: var(--surface-soft);
    border-radius: 5px;
  }
  .wrong {
    color: var(--error);
    background: var(--error-soft);
    text-decoration: underline wavy;
  }
  @media (max-height: 800px) {
    .track {
      min-height: 90px;
      max-height: 145px;
    }
  }
  @media (max-height: 650px) {
    .track {
      min-height: 75px;
      max-height: 90px;
    }
  }
</style>
