<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { km } from './lib/i18n/km';
  import { locale, messages } from './lib/i18n/index.svelte';
  import type { Component } from 'svelte';
  let Screen = $state<Component | null>(null);
  let t = $derived(messages());

  onMount(() => {
    document.title = km.appName;
    if (import.meta.env.MODE === 'development')
      void import('./screens/Playground.svelte').then((module) => {
        Screen = module.default;
      });
    // Two one-shot frames let the initial background paint before revealing it.
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        void getCurrentWindow().show();
      });
    });
  });
</script>

{#if Screen}
  <header class="topbar">
    <div class="brand">
      <svg viewBox="0 0 32 32" aria-hidden="true"
        ><rect
          x="2"
          y="6"
          width="28"
          height="20"
          rx="5"
          fill="currentColor"
        /><path
          d="M8 12h3m3 0h3m3 0h3M8 17h3m3 0h3m3 0h3M11 22h10"
          stroke="var(--surface)"
          stroke-width="2"
        /></svg
      >{t.appName}
    </div>
    <label class="language"
      ><span>{t.language}</span><select
        bind:value={locale.language}
        onchange={() => (document.documentElement.lang = locale.language)}
        ><option value="km">ខ្មែរ</option><option value="en">English</option
        ></select
      ></label
    >
  </header>
{/if}
<main aria-label={t.appName}>
  {#if Screen}<Screen />{/if}
</main>

<style>
  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 32px;
    border-bottom: 1px solid var(--line);
    background: var(--surface);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
    color: var(--primary);
    font-size: 23px;
    font-weight: 750;
  }
  .brand svg {
    width: 32px;
    height: 32px;
  }
  .language {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 14px;
  }
</style>
