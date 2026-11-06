<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { SvelteSet } from 'svelte/reactivity';
  import {
    TypingSession,
    loadEngine,
    segment,
    type Policy,
    type Stroke,
    type Summary,
    type Cluster,
    type LayoutData,
  } from '../lib/engine';
  import Keyboard from '../lib/components/Keyboard.svelte';
  import TextTrack from '../lib/components/TextTrack.svelte';
  import { messages, digits } from '../lib/i18n/index.svelte';
  import nida from '../../content/layouts/nida.json';
  import legacy from '../../content/layouts/khmer_legacy.json';
  import content from '../../content/playground.json';

  let t = $derived(messages());
  let ready = $state(false);
  let failed = $state(false);
  let policy = $state<Policy>('stop_on_error');
  let layoutId = $state('nida');
  let revision = $state(0);
  let layout = $derived((layoutId === 'nida' ? nida : legacy) as LayoutData);
  let running = $state(false);
  let cursor = $state(0);
  let expected = $state<Stroke | null>(null);
  let summary = $state<Summary | null>(null);
  const errors = new SvelteSet<number>();
  let clusters = $state<Cluster[]>([]);
  let capture = $state<HTMLDivElement>();
  let session: TypingSession | undefined;
  $effect(() => {
    void revision;
    if (!ready) return;
    const current = new TypingSession(content.text, layout, policy);
    session = current;
    clusters = segment(content.text);
    cursor = 0;
    expected = current.expected();
    summary = null;
    errors.clear();
    running = false;
    return () => {
      current.free();
      session = undefined;
    };
  });
  async function start() {
    running = true;
    await tick();
    capture?.focus();
  }
  function key(event: KeyboardEvent) {
    if (!running || !session || event.target !== capture || event.isComposing)
      return;
    if (event.code === 'Tab' || event.code === 'Escape') return;
    if (
      !event.metaKey &&
      (!event.ctrlKey || event.getModifierState('AltGraph'))
    )
      event.preventDefault();
    const result = session.onKey(event);
    if (result.ignored) return;
    const next = errors;
    if (event.code === 'Backspace') {
      for (const position of next)
        if (position >= result.cluster_index) next.delete(position);
    } else if (!result.correct && result.error_position !== null)
      next.add(result.error_position);
    else if (policy === 'stop_on_error') next.delete(cursor);
    cursor = result.cluster_index;
    expected = result.expected_next;
    if (result.finished) {
      running = false;
      summary = session.summary();
    }
  }
  onMount(() => {
    let alive = true;
    void loadEngine()
      .then(() => {
        if (alive) ready = true;
      })
      .catch(() => {
        if (alive) failed = true;
      });
    const pause = () => {
      running = false;
    };
    window.addEventListener('keydown', key);
    window.addEventListener('blur', pause);
    return () => {
      alive = false;
      window.removeEventListener('keydown', key);
      window.removeEventListener('blur', pause);
    };
  });
</script>

<div class="workspace stack">
  <header class="intro">
    <div>
      <p class="eyebrow">{t.development}</p>
      <h1>{t.playground}</h1>
      <p class="muted">{t.playgroundHint}</p>
    </div>
    <div class="status">
      <span class:running></span>{summary
        ? t.finished
        : running
          ? t.active
          : t.ready}
    </div>
  </header>
  {#if failed}<p role="alert" class="error">{t.engineError}</p>
  {:else if !ready}<p role="status">{t.loading}</p>
  {:else}
    <div class="controls row">
      <label
        >{t.layout}<select bind:value={layoutId} disabled={running}
          ><option value="nida">{nida.name.km}</option><option
            value="khmer_legacy">{legacy.name.km}</option
          ></select
        ></label
      >
      <div class="policy" role="group" aria-label={t.mode}>
        <button
          class:selected={policy === 'stop_on_error'}
          onclick={() => (policy = 'stop_on_error')}
          disabled={running}
          aria-pressed={policy === 'stop_on_error'}>{t.stopOnError}</button
        ><button
          class:selected={policy === 'flow'}
          onclick={() => (policy = 'flow')}
          disabled={running}
          aria-pressed={policy === 'flow'}>{t.flow}</button
        >
      </div>
    </div>
    {#if summary}
      <section class="panel stack" aria-label={t.results}>
        <h2>{t.finished}</h2>
        <div class="stats">
          {#each [[t.kpm, summary.kpm.toFixed(1)], ['WPM*', summary.wpm_equivalent.toFixed(1)], [t.accuracy, `${summary.accuracy.toFixed(1)}%`], [t.errors, summary.errors]] as [label, value] (label)}<div
            >
              <strong>{digits(value)}</strong><span class="muted">{label}</span>
            </div>{/each}
        </div>
        <p class="muted">{t.metricsHelp}</p>
        <button class="primary" onclick={() => revision++}>{t.retry}</button>
      </section>
    {:else}
      <section class="panel practice">
        <div class="row progress">
          <span class="muted">{t.target}</span><span
            >{digits(cursor)} / {digits(clusters.length)}</span
          >
        </div>
        <TextTrack {clusters} {cursor} {errors} />
        <div
          bind:this={capture}
          class="capture"
          class:running
          role="textbox"
          aria-label={t.capture}
          aria-readonly="true"
          aria-describedby="capture-hint"
          tabindex="0"
          inputmode="none"
          oncompositionstart={(event) => event.preventDefault()}
          onfocus={() => {
            if (!summary) running = true;
          }}
          onblur={() => (running = false)}
        >
          <p id="capture-hint" class="muted">
            {running ? t.active : t.captureHint}
          </p>
        </div>
        <div class="row practice-footer">
          <p class="muted">
            {t.nextKey}:
            <strong
              >{expected?.produces === ' '
                ? t.space
                : expected?.produces}</strong
            ><span class="key-code">{expected?.code}</span>
          </p>
          <button
            class="primary"
            onclick={() => {
              if (running) running = false;
              else void start();
            }}>{running ? t.pause : t.start}</button
          >
        </div>
      </section>
    {/if}
    {#if !summary}<Keyboard {layout} highlight={expected} />{/if}
  {/if}
</div>

<style>
  .workspace {
    max-width: 1160px;
    margin: 0 auto;
    padding: 28px 32px;
  }
  .intro {
    display: flex;
    justify-content: space-between;
    gap: 24px;
    align-items: center;
  }
  .eyebrow {
    font-size: 13px;
    color: var(--primary);
    font-weight: 700;
    letter-spacing: 0.06em;
  }
  .status {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    white-space: nowrap;
  }
  .status span {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--line);
  }
  .status span.running {
    background: var(--primary);
  }
  .controls {
    justify-content: space-between;
  }
  label {
    display: flex;
    gap: 12px;
    align-items: center;
    font-size: 16px;
  }
  .policy {
    display: flex;
    gap: 8px;
  }
  .policy .selected {
    border-color: var(--primary);
    background: var(--surface-soft);
    color: var(--primary);
  }
  .practice {
    padding: 16px 24px;
  }
  .progress {
    justify-content: space-between;
    font-size: 14px;
  }
  .capture {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }
  .practice:focus-within {
    outline: 2px solid var(--primary);
    outline-offset: 2px;
  }
  .practice-footer {
    justify-content: space-between;
    margin-top: 16px;
    font-size: 16px;
  }
  .practice-footer strong {
    font-size: 26px;
    margin-left: 8px;
  }
  .key-code {
    font: 12px monospace;
    margin-left: 12px;
  }
  .stats {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 24px;
  }
  .stats > div {
    display: flex;
    flex-direction: column;
  }
  .stats strong {
    font-size: 36px;
  }
  .stats span {
    font-size: 15px;
  }
  @media (max-width: 1050px) {
    .workspace {
      padding: 20px;
    }
    .intro {
      align-items: flex-start;
    }
    .status {
      display: none;
    }
  }
  @media (max-height: 800px) {
    .workspace {
      padding: 12px 24px;
      gap: 12px;
    }
    .intro {
      display: none;
    }
    .practice {
      padding: 12px 20px;
    }
    .practice-footer {
      margin-top: 8px;
    }
  }
</style>
