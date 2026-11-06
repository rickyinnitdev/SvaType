<script lang="ts">
  import { api } from '../lib/api';
  import { user } from '../lib/stores/profile.svelte';
  import { configureActivity } from '../lib/stores/activity.svelte';
  import { navigate } from '../lib/router.svelte';
  import { messages } from '../lib/i18n/index.svelte';
  let t = $derived(messages());
  let level = $state(2);
  let kind = $state('words');
  let weak = $state(true);
  let busy = $state(false);
  let failed = $state(false);
  async function start() {
    if (!user.profile) return;
    busy = true;
    failed = false;
    try {
      const text = await api.practiceText(user.profile.id, level, kind, weak);
      configureActivity(text, 'random');
      await navigate('practice');
    } catch {
      failed = true;
    } finally {
      busy = false;
    }
  }
</script>

<section class="page stack">
  <h1>{t.random}</h1>
  <p class="muted">{t.randomDescription}</p>
  <div class="panel stack">
    <label
      >{t.difficulty}<select bind:value={level}
        >{#each [1, 2, 3, 4, 5] as value (value)}<option {value}
            >{t[`level${value}` as 'level1']}</option
          >{/each}</select
      ></label
    ><label
      >{t.activity}<select bind:value={kind}
        ><option value="words">{t.words}</option><option value="sentences"
          >{t.sentences}</option
        ><option value="paragraphs">{t.paragraphs}</option></select
      ></label
    ><label class="row"
      ><input type="checkbox" bind:checked={weak} />{t.includeWeak}</label
    ><button class="primary" onclick={start} disabled={busy}
      >{busy ? t.loading : t.start}</button
    >
  </div>
  {#if failed}<p role="alert" class="error">{t.loadFailed}</p>{/if}
</section>

<style>
  .panel {
    max-width: 720px;
  }
  label:not(.row) {
    display: grid;
    gap: 8px;
  }
  input[type='checkbox'] {
    width: 24px;
    height: 24px;
    accent-color: var(--primary);
  }
</style>
