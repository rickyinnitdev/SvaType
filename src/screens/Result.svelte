<script lang="ts">
  import { api, type HistoryDetail } from '../lib/api';
  import { router, navigate } from '../lib/router.svelte';
  import { messages, digits, locale } from '../lib/i18n/index.svelte';
  import { formatDate } from '../lib/i18n/format';
  import { learning } from '../lib/stores/learning.svelte';
  import { user } from '../lib/stores/profile.svelte';
  import { retryActivity } from '../lib/stores/activity.svelte';
  let t = $derived(messages());
  let detail = $state<HistoryDetail | null>(null);
  let failed = $state(false);
  let nextId = $state<string | null>(null);
  $effect(() => {
    const current = detail;
    if (
      !current ||
      current.session.kind !== 'lesson' ||
      !current.session.completed ||
      !current.summary.stars
    )
      return;
    let alive = true;
    void api
      .path(user.profile?.path_id ?? 'beginner')
      .then((path) => {
        const ids = path.units.flatMap((unit) => unit.lessons);
        const at = ids.indexOf(current.session.ref_id ?? '');
        if (alive) nextId = at >= 0 ? (ids[at + 1] ?? null) : null;
      })
      .catch(() => {
        if (alive) failed = true;
      });
    return () => {
      alive = false;
    };
  });
  $effect(() => {
    const id = router.resultId;
    if (id === null) return;
    let alive = true;
    void api
      .detail(id)
      .then((value) => {
        if (alive) detail = value;
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
  <header class="row heading">
    <div>
      <p class="muted">
        {detail
          ? formatDate(detail.session.started_at, locale.language)
          : t.sessionDetail}
      </p>
      <h1>{detail?.session.completed ? t.finished : t.sessionDetail}</h1>
    </div>
    <div class="row">
      {#if nextId}<button
          class="primary"
          onclick={() => {
            learning.lessonId = nextId!;
            user.resume = null;
            void navigate('lesson');
          }}>{t.nextLesson}</button
        >{/if}
      <button onclick={() => navigate('history')}>{t.history}</button><button
        class="primary"
        onclick={() => {
          if (detail?.session.kind === 'lesson' && detail.session.ref_id) {
            learning.lessonId = detail.session.ref_id;
            user.resume = null;
            void navigate('lesson');
            return;
          }
          if (detail) retryActivity(detail);
          void navigate('practice');
        }}>{t.retry}</button
      >
    </div>
  </header>
  {#if failed}<p class="error" role="alert">
      {t.loadFailed}
    </p>{:else if !detail}<p role="status">{t.loading}</p>{:else}<div
      class="stats"
    >
      {#each [[t.kpm, detail.summary.kpm.toFixed(1)], [t.wpm, detail.summary.wpm_equivalent.toFixed(1)], [t.accuracy, `${detail.summary.accuracy.toFixed(1)}%`], [t.duration, `${digits(Math.round(detail.summary.duration_ms / 1000))} ${t.seconds}`]] as [label, value] (label)}<div
          class="panel"
        >
          <strong>{digits(value)}</strong>
          <p class="muted">{label}</p>
        </div>{/each}
    </div>
    <p class="muted">{t.metricsHelp}</p>
    <p>
      {t.errors}: {digits(detail.summary.errors)} · {t.backspaces}: {digits(
        detail.summary.backspaces,
      )}{#if detail.session.stars !== null}
        · {t.stars}: {'★'.repeat(detail.session.stars)}{/if}
    </p>
    <div class="panel stack">
      <h2>{t.mistakes}</h2>
      {#if !detail.summary.confusions.length}<p>{t.noMistakes}</p>{:else}<table>
          <thead
            ><tr><th>{t.expected}</th><th>{t.typed}</th><th>{t.count}</th></tr
            ></thead
          ><tbody
            >{#each detail.summary.confusions as item, i (i)}<tr
                ><td>{item.expected}</td><td>{item.typed}</td><td
                  >{digits(item.count)}</td
                ></tr
              >{/each}</tbody
          >
        </table>{/if}
    </div>
    <div class="panel stack">
      <h2>{t.targetText}</h2>
      <p class="target">{detail.target_text}</p>
    </div>
    <details class="panel">
      <summary>{t.keyCode}</summary>
      <table>
        <thead
          ><tr
            ><th>{t.keyCode}</th><th>{t.count}</th><th>{t.errors}</th><th
              >{t.hesitation}</th
            ></tr
          ></thead
        ><tbody
          >{#each detail.summary.per_key as key (key.code + key.layer)}<tr
              ><td>{key.code} · {key.layer}</td><td>{digits(key.hits)}</td><td
                >{digits(key.errors)}</td
              ><td>{digits(key.avg_hes_ms.toFixed(0))} {t.milliseconds}</td></tr
            >{/each}</tbody
        >
      </table>
    </details>{/if}
</section>

<style>
  .heading {
    justify-content: space-between;
  }
  .stats {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 20px;
  }
  .stats strong {
    font-size: 34px;
  }
  .target {
    font-size: var(--font-target);
    line-height: 1.9;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  table {
    border-collapse: collapse;
    width: 100%;
    text-align: left;
  }
  td,
  th {
    padding: 10px;
    border-bottom: 1px solid var(--line);
  }
  summary {
    cursor: pointer;
  }
</style>
