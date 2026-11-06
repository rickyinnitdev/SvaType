<script lang="ts">
  import geometry from '../../../content/hands.json';
  import fingerData from '../../../content/fingers.json';
  import { preferences } from '../stores/settings.svelte';
  import { messages } from '../i18n/index.svelte';
  let {
    active = null,
    compact = false,
  }: { active?: string | null; compact?: boolean } = $props();
  const colors = Object.fromEntries(
    fingerData.fingers.map((f) => [f.id, f.color]),
  );
  let t = $derived(messages());
</script>

<svg
  class="hands"
  viewBox={geometry.view_box}
  role="img"
  aria-label={t.keyboardIntro}
  class:compact
  class:static={preferences.low_end || preferences.reduce_motion}
  ><title>{t.keyboardIntro}</title>{#each geometry.palms as palm, i (i)}<path
      d={palm}
      fill="var(--surface-soft)"
      stroke="var(--line)"
      stroke-width="2"
    />{/each}{#each geometry.fingers as finger (finger.id)}<path
      d={finger.path}
      fill={colors[finger.id]}
      class:active={active === finger.id}
      stroke="var(--foreground)"
      stroke-width="1"
    /><rect
      x={finger.x - 10}
      y={finger.y - 13}
      width="20"
      height="18"
      rx="2"
      fill="var(--surface)"
    /><text x={finger.x} y={finger.y} text-anchor="middle">{finger.id}</text
    >{#if active === finger.id}<text
        x={finger.x}
        y={finger.arrow_y}
        class="arrow"
        text-anchor="middle">↓</text
      >{/if}{/each}</svg
>

<style>
  svg {
    display: block;
    width: 100%;
    max-height: 230px;
    overflow: visible;
  }
  .compact {
    width: 240px;
    max-height: 136px;
    flex-shrink: 0;
  }
  path {
    opacity: 0.5;
    transition:
      transform 0.3s,
      opacity 0.15s;
    transform-origin: center;
  }
  path.active {
    opacity: 1;
    transform: translateY(-3px);
  }
  text {
    font: 14px sans-serif;
    fill: var(--foreground);
  }
  .arrow {
    font-size: 20px;
    fill: var(--primary);
  }
  .static path {
    transition: none;
    transform: none;
  }
</style>
