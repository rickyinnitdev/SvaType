<script lang="ts">
  import { api } from '../lib/api';
  import { user } from '../lib/stores/profile.svelte';
  import { configureActivity } from '../lib/stores/activity.svelte';
  import { navigate } from '../lib/router.svelte';
  import { messages, digits } from '../lib/i18n/index.svelte';
  let t = $derived(messages());
  let duration = $state(1);
  let level = $state(1);
  let busy = $state(false);
  let failed = $state(false);
  async function start() {
    if (!user.profile) return;
    busy = true;
    failed = false;
    try {
      const text = await api.practiceText(
        user.profile.id,
        level,
        level <= 2 ? 'words' : level === 3 ? 'sentences' : 'paragraphs',
        false,
        true,
      );
      configureActivity(text, 'test', duration * 60_000);
      await navigate('practice');
    } catch {
      failed = true;
    } finally {
      busy = false;
    }
  }
</script>

<section class="page stack">
  <h1>{t.test}</h1>
  <p class="muted">{t.testDescription}</p>
  <div class="panel stack">
    <fieldset>
      <legend>{t.testDuration}</legend>
      <div class="row">
        {#each [1, 3, 5, 10] as minutes (minutes)}<button
            aria-pressed={duration === minutes}
            class:chosen={duration === minutes}
            onclick={() => (duration = minutes)}
            disabled={busy}>{digits(minutes)} {t.minutes}</button
          >{/each}
      </div>
    </fieldset>
    <label
      >{t.difficulty}<select bind:value={level} disabled={busy}
        >{#each [1, 2, 3, 4, 5] as value (value)}<option {value}
            >{t[`level${value}` as 'level1']}</option
          >{/each}</select
      ></label
    ><button class="primary" onclick={start} disabled={busy}
      >{busy ? t.loading : t.start}</button
    >
  </div>
  {#if failed}<p role="alert" class="error">{t.loadFailed}</p>{/if}
</section>

<style>
  fieldset {
    border: 0;
    padding: 0;
    margin: 0;
  }
  legend {
    margin-bottom: 12px;
  }
  label {
    display: grid;
    gap: 8px;
    max-width: 420px;
  }
  .chosen {
    background: var(--surface-soft);
    outline: 2px solid var(--primary);
  }
  .panel {
    max-width: 760px;
  }
</style>
