<script lang="ts">
  import { api, type Analysis } from '../lib/api';
  import { user } from '../lib/stores/profile.svelte';
  import { configureActivity } from '../lib/stores/activity.svelte';
  import { navigate } from '../lib/router.svelte';
  import { messages, digits } from '../lib/i18n/index.svelte';
  import Keyboard from '../lib/components/Keyboard.svelte';
  import nida from '../../content/layouts/nida.json';
  import legacy from '../../content/layouts/khmer_legacy.json';
  import type { LayoutData } from '../lib/engine';
  let t = $derived(messages());
  let data = $state<Analysis | null>(null);
  let failed = $state(false);
  let busy = $state(false);
  let layout = $derived(
    (user.profile?.layout_id === 'khmer_legacy' ? legacy : nida) as LayoutData,
  );
  $effect(() => {
    const id = user.profile?.id;
    if (!id) return;
    let alive = true;
    void api
      .analysis(id)
      .then((value) => {
        if (alive) data = value;
      })
      .catch(() => {
        if (alive) failed = true;
      });
    return () => {
      alive = false;
    };
  });
  async function drill() {
    if (!user.profile) return;
    busy = true;
    try {
      configureActivity(await api.drill(user.profile.id), 'drill');
      await navigate('practice');
    } catch {
      failed = true;
    } finally {
      busy = false;
    }
  }
</script>

<section class="page stack">
  <header class="row heading">
    <div>
      <h1>{t.analysis}</h1>
      <p class="muted">{t.analysisDescription}</p>
    </div>
    <button
      class="primary"
      disabled={busy || !data?.weak.length}
      onclick={drill}>{t.drillThese}</button
    >
  </header>
  {#if failed}<p class="error" role="alert">{t.loadFailed}</p>{:else if !data}<p
      role="status"
    >
      {t.loading}
    </p>{:else if !data.keys.length}<p class="panel">
      {t.noAnalysis}
    </p>{:else}<div class="panel stack">
      <h2>{t.errorRate}</h2>
      <Keyboard {layout} heat={data.heat} showFingers={false} />
    </div>
    <div class="columns">
      <div class="panel stack">
        <h2>{t.weakKeys}</h2>
        {#each data.weak as key (key.code)}<p>{key.code}</p>{/each}
        <h2>{t.slowestKeys}</h2>
        {#each data.slowest as key (key.code + key.layer)}<p>
            {key.code}
            <span class="muted"
              >{digits(Math.round(key.avg_hes_ms))} {t.milliseconds}</span
            >
          </p>{/each}
      </div>
      <div class="panel stack">
        <h2>{t.mistakes}</h2>
        {#each data.confusions as item, i (i)}<div class="row pair">
            <strong>{item.typed} → {item.expected}</strong><span
              >{digits(item.count)}</span
            >
          </div>{/each}{#if !data.confusions.length}<p>{t.noMistakes}</p>{/if}
      </div>
    </div>{/if}
</section>

<style>
  .heading {
    justify-content: space-between;
  }
  .columns {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 20px;
  }
  .pair {
    justify-content: space-between;
  }
  .pair strong {
    font-size: 26px;
  }
</style>
