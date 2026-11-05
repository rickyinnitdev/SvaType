<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { km } from './lib/i18n/km';

  onMount(() => {
    document.title = km.appName;
    // Two one-shot frames let the initial background paint before revealing it.
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        void getCurrentWindow().show();
      });
    });
  });
</script>

<main aria-label={km.appName}></main>
