<script lang="ts">
  import { digits } from '../i18n/index.svelte';
  let { values, label }: { values: number[]; label: string } = $props();
  let max = $derived(Math.max(100, ...values));
  let points = $derived(
    values.map((value, i) => ({
      x: 40 + (i * 540) / Math.max(1, values.length - 1),
      y: 120 - (value * 100) / max,
    })),
  );
  let line = $derived(
    points.map((point, i) => `${i ? 'L' : 'M'}${point.x},${point.y}`).join(' '),
  );
</script>

<svg viewBox="0 0 600 150" role="img" aria-label={label}
  ><title>{label}</title><path
    d="M40 20H580M40 70H580M40 120H580"
    stroke="var(--line)"
    fill="none"
    stroke-dasharray="3 5"
  /><text x="2" y="25">{digits(max.toFixed(0))}</text><text x="14" y="124"
    >{digits(0)}</text
  ><path
    d={line}
    fill="none"
    stroke="var(--primary)"
    stroke-width="3"
  />{#each points as point, i (i)}<circle
      cx={point.x}
      cy={point.y}
      r="4"
      fill="var(--primary)"
      ><title>{digits(values[i].toFixed(1))}</title></circle
    >{/each}</svg
>

<style>
  svg {
    width: 100%;
    max-height: 170px;
    display: block;
  }
  text {
    fill: var(--muted);
    font: 12px sans-serif;
  }
</style>
