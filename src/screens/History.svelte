<script lang="ts">
  import {
    api,
    type HistoryPage,
    type HistoryFilter,
    type Trend,
  } from '../lib/api';
  import { user } from '../lib/stores/profile.svelte';
  import { messages, digits, locale } from '../lib/i18n/index.svelte';
  import { formatDate } from '../lib/i18n/format';
  import { navigate } from '../lib/router.svelte';
  import SvgChart from '../lib/components/SvgChart.svelte';
  import Confirm from '../lib/components/Confirm.svelte';
  let t = $derived(messages());
  let mode = $state('');
  let kind = $state('');
  let from = $state('');
  let to = $state('');
  let page = $state(0);
  let revision = $state(0);
  let data = $state<HistoryPage | null>(null);
  let trend = $state<Trend[]>([]);
  let recent = $state<number[]>([]);
  let byDay = $state(false);
  let busy = $state(false);
  let failed = $state(false);
  let deleteId = $state<number | null>(null);
  let deleting = $state(false);
  const kinds = [
    'free',
    'test',
    'custom',
    'random',
    'drill',
    'lesson',
    'vocab',
    'game',
  ] as const;
  function activity(value: string): string {
    return kinds.includes(value as (typeof kinds)[number])
      ? t[value as (typeof kinds)[number]]
      : t.practice;
  }
  $effect(() => {
    const profile = user.profile;
    if (!profile) return;
    void revision;
    const filter: HistoryFilter = {
      mode: mode || null,
      kind: kind || null,
      from: from ? new Date(`${from}T00:00:00`).getTime() : null,
      to: to ? new Date(`${to}T23:59:59.999`).getTime() : null,
    };
    let alive = true;
    busy = true;
    failed = false;
    void Promise.all([
      api.history(profile.id, filter, page),
      api.trend(profile.id, Date.now() - 30 * 86400000),
      api.history(
        profile.id,
        { mode: null, kind: null, from: null, to: null },
        0,
      ),
    ])
      .then(([h, p, latest]) => {
        if (alive) {
          data = h;
          trend = p;
          recent = latest.rows
            .slice(0, 30)
            .reverse()
            .map((row) => row.accuracy);
        }
      })
      .catch(() => {
        if (alive) failed = true;
      })
      .finally(() => {
        if (alive) busy = false;
      });
    return () => {
      alive = false;
    };
  });
  async function remove() {
    if (deleteId === null) return;
    deleting = true;
    try {
      await api.deleteHistory(deleteId);
      deleteId = null;
      revision++;
    } catch {
      failed = true;
    } finally {
      deleting = false;
    }
  }
</script>

<section class="page stack">
  <div class="row heading">
    <h1>{t.history}</h1>
    <button onclick={() => navigate('home')}>{t.home}</button>
  </div>
  {#if recent.length}<div class="panel">
      <div class="row">
        <button aria-pressed={!byDay} onclick={() => (byDay = false)}
          >{t.recentSessions}</button
        ><button aria-pressed={byDay} onclick={() => (byDay = true)}
          >{t.byDay}</button
        >
      </div>
      <SvgChart
        values={byDay ? trend.map((p) => p.accuracy) : recent}
        label={t.accuracy}
      />
    </div>{/if}
  <div class="row filters">
    <label
      >{t.mode}<select bind:value={mode} onchange={() => (page = 0)}
        ><option value="">{t.all}</option><option value="learning"
          >{t.learning}</option
        ><option value="typing">{t.typing}</option></select
      ></label
    ><label
      >{t.activity}<select bind:value={kind} onchange={() => (page = 0)}
        ><option value="">{t.all}</option>{#each kinds as value (value)}<option
            {value}>{activity(value)}</option
          >{/each}</select
      ></label
    ><label
      >{t.from}<input
        type="date"
        bind:value={from}
        onchange={() => (page = 0)}
      /></label
    ><label
      >{t.to}<input
        type="date"
        bind:value={to}
        onchange={() => (page = 0)}
      /></label
    >
  </div>
  {#if failed}<p class="error" role="alert">{t.loadFailed}</p>{:else if busy}<p
      role="status"
    >
      {t.loading}
    </p>{:else if !data?.rows.length}<div class="panel empty">
      <h2>{t.noHistory}</h2>
      <button class="primary" onclick={() => navigate('practice')}
        >{t.practice}</button
      >
    </div>{:else}<div class="table-wrap panel">
      <table>
        <thead
          ><tr
            ><th>{t.date}</th><th>{t.activity}</th><th>{t.duration}</th><th
              >{t.kpm}</th
            ><th>{t.accuracy}</th><th>{t.progress}</th><th>{t.view}</th></tr
          ></thead
        ><tbody
          >{#each data.rows as row (row.id)}<tr
              ><td>{formatDate(row.started_at, locale.language)}</td><td
                >{activity(row.kind)}<small class="muted"
                  >{row.mode === 'learning' ? t.learning : t.typing}</small
                ></td
              ><td>{digits(Math.round(row.duration_ms / 1000))} {t.seconds}</td
              ><td>{digits(row.kpm.toFixed(1))}</td><td
                >{digits(row.accuracy.toFixed(1))}%</td
              ><td
                >{row.completed
                  ? row.stars === null
                    ? t.completed
                    : '★'.repeat(row.stars)
                  : t.abandoned}</td
              ><td
                ><div class="row">
                  <button onclick={() => navigate('result', row.id)}
                    >{t.view}</button
                  ><button class="delete" onclick={() => (deleteId = row.id)}
                    >{t.delete}</button
                  >
                </div></td
              ></tr
            >{/each}</tbody
        >
      </table>
    </div>
    <div class="row pagination">
      <button disabled={page === 0} onclick={() => page--}>{t.previous}</button
      ><span
        >{t.page}
        {digits(page + 1)} / {digits(
          Math.max(1, Math.ceil(data.total / 50)),
        )}</span
      ><button disabled={(page + 1) * 50 >= data.total} onclick={() => page++}
        >{t.next}</button
      >
    </div>{/if}
</section>
<Confirm
  open={deleteId !== null}
  title={t.confirmDelete}
  description={t.deleteHint}
  confirmLabel={t.delete}
  onconfirm={() => void remove()}
  oncancel={() => (deleteId = null)}
  busy={deleting}
/>

<style>
  .heading,
  .pagination {
    justify-content: space-between;
  }
  .filters {
    align-items: flex-end;
  }
  .filters label {
    font-size: 14px;
    display: grid;
    gap: 4px;
  }
  .table-wrap {
    overflow-x: auto;
    padding: 8px 16px;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    text-align: left;
    font-size: 15px;
  }
  th,
  td {
    padding: 12px 10px;
    border-bottom: 1px solid var(--line);
  }
  th {
    font-size: 13px;
    color: var(--muted);
    font-weight: 600;
  }
  small {
    display: block;
  }
  td button {
    padding: 6px 12px;
    font-size: 14px;
  }
  .delete {
    color: var(--error);
  }
  .empty {
    display: grid;
    place-items: start;
    gap: 24px;
    padding: 40px;
  }
  .empty h2 {
    font-size: 21px;
    max-width: 700px;
  }
</style>
