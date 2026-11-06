<script lang="ts">
  import { onMount } from 'svelte';
  import { diagnostics, latency } from '../diagnostics.svelte';
  import { messages, digits } from '../i18n/index.svelte';
  let t = $derived(messages());
  let heap = $state<number | null>(null);
  let stats = $derived.by(() => {
    void diagnostics.revision;
    return latency();
  });
  onMount(() => {
    const toggle = (e: KeyboardEvent) => {
      if (e.code === 'F9' && e.ctrlKey && e.shiftKey) {
        e.preventDefault();
        diagnostics.visible = !diagnostics.visible;
        diagnostics.revision++;
        const memory = (
          performance as Performance & { memory?: { usedJSHeapSize: number } }
        ).memory;
        heap = memory ? memory.usedJSHeapSize / 1048576 : null;
      }
    };
    window.addEventListener('keydown', toggle);
    return () => window.removeEventListener('keydown', toggle);
  });
</script>

{#if diagnostics.visible}<aside aria-label={t.diagnostics}>
    <strong>{t.diagnostics}</strong>
    <p>
      {t.keyLatency}: p50 {digits(stats.p50.toFixed(2))} / p95 {digits(
        stats.p95.toFixed(2),
      )}
      {t.milliseconds} · {digits(stats.count)}/200
    </p>
    <p>{t.jsHeap}: {heap === null ? '—' : `${digits(heap.toFixed(1))} MB`}</p>
  </aside>{/if}

<style>
  aside {
    position: fixed;
    bottom: 8px;
    right: 8px;
    z-index: 50;
    padding: 12px 18px;
    background: var(--surface);
    border: 2px solid var(--primary);
    font-size: 14px;
    pointer-events: none;
  }
</style>
