<script lang="ts">
  import { onMount } from 'svelte';
  import {
    TypingSession,
    loadEngine,
    type LayoutData,
    type Stroke,
  } from '../lib/engine';
  import { user } from '../lib/stores/profile.svelte';
  import { messages, locale } from '../lib/i18n/index.svelte';
  import { navigate } from '../lib/router.svelte';
  import Keyboard from '../lib/components/Keyboard.svelte';
  import nida from '../../content/layouts/nida.json';
  import legacy from '../../content/layouts/khmer_legacy.json';
  let t = $derived(messages());
  let layoutId = $state(user.profile?.layout_id ?? 'nida');
  let layout = $derived((layoutId === 'nida' ? nida : legacy) as LayoutData);
  let ready = $state(false);
  let failed = $state(false);
  let capture = $state<HTMLTextAreaElement>();
  let composing = false;
  let highlight = $state<Stroke | null>(null);
  let pending: { code: string; layer: string; expected: string } | null = null;
  let rows = $state<
    { code: string; layer: string; expected: string; actual: string }[]
  >([]);
  let session: TypingSession | undefined;
  onMount(() => {
    let alive = true;
    void loadEngine()
      .then(() => {
        if (alive) ready = true;
      })
      .catch(() => {
        if (alive) failed = true;
      });
    return () => {
      alive = false;
    };
  });
  $effect(() => {
    if (!ready) return;
    const current = new TypingSession(
      '',
      layout,
      'stop_on_error',
      false,
      undefined,
      true,
    );
    session = current;
    rows = [];
    pending = null;
    highlight = null;
    return () => {
      current.free();
      session = undefined;
    };
  });
  function key(event: KeyboardEvent) {
    if (!session || event.repeat || !Object.hasOwn(layout.keys, event.code))
      return;
    if (event.ctrlKey && !event.altKey && !event.getModifierState('AltGraph'))
      return;
    const result = session.onKey(event);
    const alt =
      event.getModifierState('AltGraph') || (event.ctrlKey && event.altKey);
    const layer = alt
      ? event.shiftKey
        ? 'shift_altgr'
        : 'altgr'
      : event.shiftKey
        ? 'shift'
        : 'base';
    pending = { code: event.code, layer, expected: result.produced };
    highlight = {
      code: event.code,
      layer,
      produces: result.produced,
      cluster_index: 0,
      finger: layout.keys[event.code].finger,
    };
    if (capture && !composing) capture.value = '';
  }
  function flush(value: string) {
    if (!pending) return;
    rows = [{ ...pending, actual: value }, ...rows].slice(0, 256);
    pending = null;
    if (capture) capture.value = '';
  }
  function points(value: string) {
    return Array.from(
      value,
      (c) => `U+${c.codePointAt(0)!.toString(16).toUpperCase()}`,
    ).join(' ');
  }
</script>

<section class="page stack">
  <header class="row heading">
    <h1>{t.layoutCheck}</h1>
    <button onclick={() => navigate('settings')}>{t.settings}</button>
  </header>
  <p>{t.layoutCheckHint}</p>
  <div class="row">
    <label
      >{t.layout}<select bind:value={layoutId}
        ><option value="nida">{nida.name[locale.language]}</option><option
          value="khmer_legacy">{legacy.name[locale.language]}</option
        ></select
      ></label
    ><button class="primary" disabled={!ready} onclick={() => capture?.focus()}
      >{t.startCheck}</button
    ><button onclick={() => (rows = [])}>{t.clearRecords}</button>
  </div>
  {#if failed}<p role="alert" class="error">{t.engineError}</p>{/if}<textarea
    class="os-capture"
    bind:this={capture}
    aria-label={t.checkCapture}
    inputmode="none"
    tabindex="0"
    onkeydown={key}
    onkeyup={() => {
      if (!composing && capture) flush(capture.value);
    }}
    oninput={(event) => {
      if (!composing) flush(event.currentTarget.value);
    }}
    oncompositionstart={() => (composing = true)}
    oncompositionend={() => {
      composing = false;
      if (capture) flush(capture.value);
    }}
    onpaste={(event) => event.preventDefault()}
  ></textarea><Keyboard {layout} {highlight} />
  <div class="table-wrap">
    <table>
      <thead
        ><tr
          ><th>{t.keyCode}</th><th>{t.engineOutput}</th><th
            >{t.windowsOutput}</th
          ><th>{t.results}</th></tr
        ></thead
      ><tbody
        >{#each rows as row, i (i)}<tr
            class:mismatch={row.expected !== row.actual}
            ><td>{row.code} · {row.layer}</td><td
              ><strong>{row.expected || '—'}</strong><small
                >{points(row.expected)}</small
              ></td
            ><td
              ><strong>{row.actual || '—'}</strong><small
                >{points(row.actual)}</small
              ></td
            ><td>{row.expected === row.actual ? t.keyMatch : t.keyMismatch}</td
            ></tr
          >{/each}</tbody
      >
    </table>
  </div>
</section>

<style>
  .heading {
    justify-content: space-between;
  }
  label {
    display: flex;
    gap: 12px;
    align-items: center;
  }
  .os-capture {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    clip-path: inset(50%);
    overflow: hidden;
  }
  .table-wrap {
    overflow: auto;
  }
  table {
    width: 100%;
    border-collapse: collapse;
  }
  th,
  td {
    padding: 12px;
    text-align: left;
    border-bottom: 1px solid var(--line);
  }
  strong {
    font-size: 32px;
    line-height: 1.9;
  }
  small {
    display: block;
  }
  .mismatch {
    background: var(--error-soft);
  }
  .mismatch td:last-child {
    color: var(--error);
  }
</style>
