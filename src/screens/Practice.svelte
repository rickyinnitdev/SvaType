<script lang="ts">
  import { onMount, tick, untrack } from 'svelte';
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
  import { api, type SessionInput } from '../lib/api';
  import { user } from '../lib/stores/profile.svelte';
  import { preferences } from '../lib/stores/settings.svelte';
  import { activity } from '../lib/stores/activity.svelte';
  import Hands from '../lib/components/Hands.svelte';
  import { recordLatency } from '../lib/diagnostics.svelte';
  import { navigate, registerLeaveGuard } from '../lib/router.svelte';
  let {
    sandbox = false,
    target = activity.target ?? content.text,
    kind = activity.kind,
    refId = activity.refId,
    free = activity.free,
    durationMs = activity.durationMs,
    strict = activity.strict,
    findKey = activity.findKey,
    layoutOverride,
    sessionUid,
    bookmarkStep = null,
    oncomplete,
    onpause,
  }: {
    sandbox?: boolean;
    target?: string;
    kind?: string;
    refId?: string | null;
    free?: boolean;
    durationMs?: number;
    strict?: boolean;
    findKey?: boolean;
    layoutOverride?: string;
    sessionUid?: string;
    bookmarkStep?: number | null;
    oncomplete?: (summary: Summary, text: string) => Promise<void>;
    onpause?: (summary: Summary, text: string) => Promise<void>;
  } = $props();
  const restored = untrack(() => (sandbox ? null : user.resume));
  const owner = user.profile ? { ...user.profile } : null;
  let isFree = $derived(restored?.session.kind === 'free' || free);
  const totalDuration = untrack(() => restored?.remaining_ms ?? durationMs);
  const hasTimer = untrack(
    () => restored?.remaining_ms != null || durationMs > 0,
  );
  let remainingMs = $state(totalDuration);
  let clock = $derived(
    `${Math.floor(Math.ceil(remainingMs / 1000) / 60)}:${String(Math.ceil(remainingMs / 1000) % 60).padStart(2, '0')}`,
  );
  let freeText = $state('');
  let liveStats = $state(false);
  let liveSummary = $state<Summary | null>(null);
  let copied = $state(false);
  let freeDisplay = $state<HTMLDivElement>();
  $effect(() => {
    void freeText;
    freeDisplay?.scrollTo({ top: 1_000_000 });
  });
  let practiceText = $derived(restored?.session.target_text ?? target);
  let uid =
    restored?.session.uid ?? untrack(() => sessionUid) ?? crypto.randomUUID();
  let startedAt = restored?.session.started_at ?? 0;
  let wallStart: number | null = restored ? performance.now() : null;
  let saved = false;
  let saving = $state(false);
  let saveError = $state(false);
  let pendingSave: Promise<number | undefined> | undefined;

  let t = $derived(messages());
  let ready = $state(false);
  let failed = $state(false);
  let policy = $state<Policy>(
    restored
      ? JSON.parse(restored.engine).policy
      : owner?.preferred_mode === 'typing'
        ? 'flow'
        : 'stop_on_error',
  );
  let layoutId = $state(
    restored?.session.layout_id ??
      untrack(() => layoutOverride) ??
      owner?.layout_id ??
      'nida',
  );
  let revision = $state(0);
  let layout = $derived((layoutId === 'nida' ? nida : legacy) as LayoutData);
  let running = $state(false);
  let hasStarted = $state(restored !== null);
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
    let current: TypingSession;
    try {
      current = new TypingSession(
        practiceText,
        layout,
        policy,
        strict,
        restored?.engine,
        isFree,
      );
    } catch {
      failed = true;
      return;
    }
    session = current;
    clusters = segment(practiceText);
    cursor = current.cursor();
    freeText = current.tail();
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
    hasStarted = true;
    if (wallStart === null) {
      wallStart = performance.now();
      startedAt = Date.now();
    }
    running = true;
    await tick();
    capture?.focus();
  }
  function key(event: KeyboardEvent) {
    const received = performance.now();
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
    void tick().then(() => recordLatency(performance.now() - received));
    const next = errors;
    if (event.code === 'Backspace') {
      for (const position of next)
        if (position >= result.cluster_index) next.delete(position);
    } else if (!result.correct && result.error_position !== null)
      next.add(result.error_position);
    else if (policy === 'stop_on_error') next.delete(cursor);
    cursor = result.cluster_index;
    expected = result.expected_next;
    if (isFree) freeText = session.tail();
    if (liveStats) liveSummary = session.live();
    copied = false;
    if (result.finished) {
      if (hasTimer) {
        session.nextRound();
        cursor = 0;
        expected = session.expected();
        errors.clear();
        return;
      }
      running = false;
      summary = session.summary();
      if (oncomplete) void completeStep();
      else if (!sandbox) void finish();
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
    const leave = registerLeaveGuard(async () => {
      if (sandbox || saved || !session || wallStart === null) return;
      if (onpause) {
        session.observeTime();
        await onpause(session.summary(), practiceText);
        await checkpoint();
        return;
      }
      const input = snapshot(false);
      if (input.summary.duration_ms > 20_000) {
        await save(false);
      } else {
        await api.discardCheckpoint(uid);
        user.resume = null;
      }
    });
    return () => {
      alive = false;
      window.removeEventListener('keydown', key);
      window.removeEventListener('blur', pause);
      leave();
    };
  });
  function snapshot(completed: boolean): SessionInput {
    session!.observeTime();
    const stats = session!.summary();
    stats.duration_ms = Math.max(
      stats.duration_ms,
      (restored?.session.summary.duration_ms ?? 0) +
        (wallStart === null ? 0 : Math.round(performance.now() - wallStart)),
    );
    return {
      uid,
      profile_id: owner!.id,
      mode: restored?.session.mode ?? owner!.preferred_mode,
      kind: restored?.session.kind ?? kind,
      ref_id: restored?.session.ref_id ?? refId,
      layout_id: layoutId,
      started_at: startedAt,
      completed,
      target_text: isFree ? session!.typed() : practiceText,
      summary: stats,
      bookmark_step: bookmarkStep,
    };
  }
  async function checkpoint() {
    if (sandbox || saved || !session || !owner || wallStart === null) return;
    try {
      await api.checkpoint({
        session: snapshot(false),
        engine: session.checkpoint(),
        remaining_ms: hasTimer ? remainingMs : null,
      });
    } catch {
      saveError = true;
    }
  }
  async function save(completed: boolean): Promise<number | undefined> {
    if (sandbox || saved) return;
    if (pendingSave) return pendingSave;
    saving = true;
    saveError = false;
    pendingSave = api
      .saveSession(snapshot(completed))
      .then((id) => {
        saved = true;
        user.resume = null;
        return id;
      })
      .finally(() => {
        saving = false;
        pendingSave = undefined;
      });
    return pendingSave;
  }
  async function finish() {
    try {
      const id = await save(true);
      if (id !== undefined) {
        user.resume = null;
        await navigate('result', id);
      }
    } catch {
      saveError = true;
    }
  }
  async function completeStep() {
    if (!oncomplete || !session) return;
    saving = true;
    saveError = false;
    saved = true;
    session.observeTime();
    try {
      await oncomplete(session.summary(), practiceText);
    } catch {
      saved = false;
      saveError = true;
    } finally {
      saving = false;
    }
  }
  $effect(() => {
    if (!running || sandbox) return;
    const timer = setInterval(() => void checkpoint(), 30_000);
    return () => {
      clearInterval(timer);
      void checkpoint();
    };
  });
  $effect(() => {
    if (!running || !hasTimer) return;
    let previous = performance.now();
    const timer = setInterval(() => {
      const now = performance.now();
      remainingMs = Math.max(0, remainingMs - (now - previous));
      previous = now;
      if (remainingMs === 0) {
        running = false;
        summary = session?.summary() ?? null;
        void finish();
      }
    }, 250);
    return () => clearInterval(timer);
  });
  async function copy() {
    if (!session) return;
    try {
      await navigator.clipboard.writeText(session.typed());
      copied = true;
    } catch {
      saveError = true;
    }
  }
</script>

<div class="workspace stack">
  <header class="intro">
    <div>
      <p class="eyebrow">{t.development}</p>
      <h1>{sandbox ? t.playground : t.practice}</h1>
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
    {#if saveError}<p class="error" role="alert">{t.saveFailed}</p>{/if}
    {#if restored}<p class="muted recovery">{t.restoredPrefix}</p>{/if}
    <div class="controls row">
      <label
        >{t.layout}<select
          bind:value={layoutId}
          disabled={!!oncomplete || running || (!sandbox && hasStarted)}
          ><option value="nida">{nida.name.km}</option><option
            value="khmer_legacy">{legacy.name.km}</option
          ></select
        ></label
      >
      {#if !isFree && !oncomplete}<div
          class="policy"
          role="group"
          aria-label={t.mode}
        >
          <button
            class:selected={policy === 'stop_on_error'}
            onclick={() => (policy = 'stop_on_error')}
            disabled={running || (!sandbox && hasStarted)}
            aria-pressed={policy === 'stop_on_error'}>{t.stopOnError}</button
          ><button
            class:selected={policy === 'flow'}
            onclick={() => (policy = 'flow')}
            disabled={running || (!sandbox && hasStarted)}
            aria-pressed={policy === 'flow'}>{t.flow}</button
          >
        </div>{:else}<label class="row"
          ><input type="checkbox" bind:checked={liveStats} />{t.showLive}</label
        >{/if}
      {#if hasTimer}<strong class="countdown"
          >{t.timeLeft}: {digits(clock)}</strong
        >{/if}
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
        <button
          class="primary"
          disabled={saving}
          onclick={() => {
            if (sandbox) revision++;
            else if (oncomplete) void completeStep();
            else void finish();
          }}>{saving ? t.saving : t.retry}</button
        >
      </section>
    {:else}
      <section class="panel practice">
        <div class="row progress">
          <span class="muted">{isFree ? t.free : t.target}</span
          >{#if !isFree}<span>{digits(cursor)} / {digits(clusters.length)}</span
            >{/if}
        </div>
        <div class="work-area">
          <div class="text-area">
            {#if isFree}<p class="muted free-hint" title={t.freeLimit}>
                {t.freeHint}
              </p>
              <div class="free-text" bind:this={freeDisplay} aria-live="off">
                {freeText || ' '}
              </div>{:else if findKey}<div
                class="find-key"
                aria-label={t.findKey}
              >
                {expected?.produces}
              </div>{:else}<TextTrack {clusters} {cursor} {errors} />{/if}
          </div>
          {#if !isFree && preferences.show_hands && owner?.preferred_mode === 'learning'}<Hands
              active={expected?.finger ?? null}
              compact={true}
            />{/if}
        </div>
        {#if liveStats && liveSummary}<p>
            {t.kpm}: {digits(liveSummary.kpm.toFixed(1))} · {t.accuracy}: {digits(
              liveSummary.accuracy.toFixed(1),
            )}%
          </p>{/if}
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
            if (!summary) {
              hasStarted = true;
              if (wallStart === null) {
                wallStart = performance.now();
                startedAt = Date.now();
              }
              running = true;
            }
          }}
          onblur={() => (running = false)}
        >
          <p id="capture-hint" class="muted">
            {running ? t.active : t.captureHint}
          </p>
        </div>
        <div class="row practice-footer">
          {#if !isFree}<p class="muted">
              {t.nextKey}:
              <strong
                >{expected?.produces === ' '
                  ? t.space
                  : expected?.produces}</strong
              ><span class="key-code">{expected?.code}</span>
            </p>{:else}<div class="row">
              <button onclick={copy}>{copied ? t.copied : t.copyText}</button
              ><button
                onclick={() => {
                  running = false;
                  summary = session?.summary() ?? null;
                  void finish();
                }}
                disabled={!hasStarted || saving}>{t.leavePractice}</button
              >
            </div>{/if}
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
    {#if !summary && (sandbox || preferences.show_keyboard)}<Keyboard
        {layout}
        highlight={expected}
        showFingers={preferences.show_hands}
      />{/if}
  {/if}
</div>

<style>
  .work-area {
    display: flex;
    gap: 16px;
    align-items: center;
  }
  .text-area {
    flex: 1;
    min-width: 0;
  }
  .find-key {
    font-size: 64px;
    line-height: 1.9;
    text-align: center;
    min-height: 130px;
    color: var(--primary);
  }
  .free-text {
    font-size: var(--font-target);
    line-height: 1.9;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    height: 200px;
    overflow: auto;
    contain: layout paint;
  }
  .free-hint {
    font-size: 14px;
  }
  .countdown {
    font-variant-numeric: tabular-nums;
  }
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
