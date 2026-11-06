<script lang="ts">
  import { api, type Progress, type Analysis, type Trend } from '../lib/api';
  import { user } from '../lib/stores/profile.svelte';
  import { messages, digits } from '../lib/i18n/index.svelte';
  import SvgChart from '../lib/components/SvgChart.svelte';
  let t = $derived(messages());
  let progress = $state<Progress | null>(null);
  let analysis = $state<Analysis | null>(null);
  let trend = $state<Trend[]>([]);
  let failed = $state(false);
  $effect(() => {
    const id = user.profile?.id;
    if (!id) return;
    let alive = true;
    void Promise.all([api.progress(id), api.analysis(id), api.trend(id, 0)])
      .then(([p, a, r]) => {
        if (alive) {
          progress = p;
          analysis = a;
          trend = r;
        }
      })
      .catch(() => {
        if (alive) failed = true;
      });
    return () => {
      alive = false;
    };
  });
</script>

<section class="page stack">
  <h1>{t.myProgress}</h1>
  {#if failed}<p role="alert" class="error">{t.loadFailed}</p>{/if}
  <div class="row">
    {#each [[t.stars, progress?.lessons.reduce((sum, l) => sum + l.best_stars, 0) ?? 0], [t.lessonsDone, progress?.lessons.filter((l) => l.best_stars > 0).length ?? 0], [t.wordsLearned, progress?.words_learned ?? 0], [t.totalTime, Math.round((progress?.duration_ms ?? 0) / 60000)]] as [label, value] (label)}<div
        class="panel"
      >
        <h2>{label}</h2>
        <strong>{digits(value)}</strong>
      </div>{/each}
  </div>
  <p>{t.starsHelp}</p>
  <section class="panel">
    <h2>{t.accuracy}</h2>
    <SvgChart values={trend.map((row) => row.accuracy)} label={t.accuracy} />
  </section>
  <section class="panel">
    <h2>{t.weakKeys}</h2>
    {#each analysis?.weak.slice(0, 5) ?? [] as key (key.code)}<p>
        {key.code}
      </p>{/each}
  </section>
  <div class="panel">
    <h2>{t.bestSpeed}</h2>
    <p>{digits((progress?.best_kpm ?? 0).toFixed(1))} {t.kpm}</p>
    <h2>{t.bestAccuracy}</h2>
    <strong
      >{digits(
        Math.max(
          0,
          ...(progress?.lessons.map((l) => l.best_accuracy) ?? []),
        ).toFixed(1),
      )}%</strong
    >
  </div>
</section>
