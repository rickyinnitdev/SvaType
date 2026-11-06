<script lang="ts">
  import { gameSelection } from '../lib/stores/game.svelte';
  import { onMount, tick } from 'svelte';
  import {
    api,
    type GameContent,
    type GameDraft,
    type Checkpoint,
  } from '../lib/api';
  import { user } from '../lib/stores/profile.svelte';
  import { preferences } from '../lib/stores/settings.svelte';
  import { messages, locale, digits } from '../lib/i18n/index.svelte';
  import { navigate, registerLeaveGuard } from '../lib/router.svelte';
  import { recordLatency } from '../lib/diagnostics.svelte';
  import { sound, pauseSounds } from '../lib/sound';
  import { GameLoop } from '../lib/game-loop';
  import {
    TypingSession,
    loadEngine,
    type Summary,
    type Stroke,
    type LayoutData,
  } from '../lib/engine';
  import Keyboard from '../lib/components/Keyboard.svelte';
  import nida from '../../content/layouts/nida.json';
  import legacy from '../../content/layouts/khmer_legacy.json';
  const profile = user.profile!;
  let t = $derived(messages());
  let catalog = $state<GameContent[]>([]);
  let game = $state<GameContent | null>(null);
  let draft = $state<GameDraft | null>(null);
  let ready = $state(false);
  let failed = $state(false);
  let saving = $state(false);
  let running = $state(false);
  let hinted = $state(false);
  let expected = $state<Stroke | null>(null);
  let cursor = $state(0);
  let y = $state(-35);
  let remaining = $state(0);
  let capture = $state<HTMLDivElement>();
  let session: TypingSession | undefined;
  let loop: GameLoop | undefined;
  let elapsed = 0;
  let lastCheckpoint = 0;
  let saved = false;
  let round = $derived(game && draft ? game.rounds[draft.step] : null);
  let layout = $derived(
    (draft?.layout_id === 'khmer_legacy' ? legacy : nida) as LayoutData,
  );
  function summary(): Summary {
    session!.observeTime(elapsed);
    const stats = session!.summary();
    stats.duration_ms = Math.max(stats.duration_ms, Math.round(elapsed));
    return stats;
  }
  function setup(checkpoint: Checkpoint | null = null) {
    session?.free();
    if (!round) return;
    session = new TypingSession(
      round.target,
      layout,
      'stop_on_error',
      false,
      checkpoint?.engine,
    );
    elapsed = checkpoint?.session.summary.duration_ms ?? 0;
    lastCheckpoint = elapsed;
    expected = session.expected();
    cursor = session.cursor();
    hinted = false;
    y = -35;
  }
  async function choose(item: GameContent) {
    if (!ready || saving) return;
    failed = false;
    saving = true;
    try {
      const value = (await api.gameDraft(profile.id, item.id)) ?? {
        profile_id: profile.id,
        game_id: item.id,
        uid: crypto.randomUUID(),
        step: 0,
        summaries: [],
        texts: [],
        started_at: Date.now(),
        layout_id: profile.layout_id,
        lives: 3,
      };
      await api.saveGameDraft(value);
      game = item;
      draft = value;
      saved = false;
      await tick();
      if (value.step >= item.rounds.length || value.lives === 0) {
        await finish();
        return;
      }
      const checkpoints = await api.checkpoints(profile.id);
      setup(
        checkpoints.find(
          (c) =>
            c.session.uid === value.uid &&
            c.session.bookmark_step === value.step,
        ) ?? null,
      );
    } catch {
      failed = true;
    } finally {
      saving = false;
    }
  }
  async function start() {
    if (!session || saving) return;
    await tick();
    capture?.focus();
    running = loop?.start() ?? false;
  }
  async function checkpoint() {
    if (!draft || !round || !session || saved) return;
    try {
      await api.checkpoint({
        session: {
          uid: draft.uid,
          profile_id: profile.id,
          mode: 'learning',
          kind: 'game',
          ref_id: draft.game_id,
          layout_id: draft.layout_id,
          started_at: draft.started_at,
          completed: false,
          target_text: round.target,
          summary: summary(),
          bookmark_step: draft.step,
        },
        engine: session.checkpoint(),
        remaining_ms: null,
      });
    } catch {
      failed = true;
    }
  }
  async function finish() {
    if (!draft || saved) return;
    const id = await api.saveGameSession(
      profile.id,
      draft.game_id,
      null,
      null,
      true,
    );
    saved = true;
    if (id !== null) await navigate('result', id);
  }
  async function complete(missed = false) {
    if (!draft || !round || !session || saving || saved) return;
    saving = true;
    const resume = running;
    loop?.pause();
    try {
      const updated = {
        ...draft,
        step: draft.step + 1,
        lives: draft.lives - (missed ? 1 : 0),
        summaries: [...draft.summaries, summary()],
        texts: [...draft.texts, round.target],
      };
      await api.saveGameDraft(updated);
      await api.discardCheckpoint(updated.uid);
      draft = updated;
      await tick();
      if (updated.step >= game!.rounds.length || updated.lives === 0) {
        await finish();
        return;
      }
      setup();
      if (resume) {
        saving = false;
        await start();
      }
    } catch {
      failed = true;
    } finally {
      saving = false;
    }
  }
  async function stop() {
    if (!draft || !session || saving) return;
    saving = true;
    loop?.pause();
    try {
      const updated = {
        ...draft,
        step: game!.rounds.length,
        summaries: [...draft.summaries, summary()],
        texts: [...draft.texts, round?.target ?? ''],
      };
      await api.saveGameDraft(updated);
      draft = updated;
      await finish();
    } catch {
      failed = true;
    } finally {
      saving = false;
    }
  }
  function key(event: KeyboardEvent) {
    const received = performance.now();
    if (
      !running ||
      saving ||
      !session ||
      event.target !== capture ||
      event.isComposing ||
      event.code === 'Tab' ||
      event.code === 'Escape'
    )
      return;
    if (
      !event.metaKey &&
      (!event.ctrlKey || event.getModifierState('AltGraph'))
    )
      event.preventDefault();
    const result = session.onKey(event, elapsed);
    if (result.ignored) return;
    sound(result.finished ? 'success' : result.correct ? 'key' : 'error');
    void tick().then(() => recordLatency(performance.now() - received));
    expected = result.expected_next;
    cursor = result.cluster_index;
    if (result.finished) void complete();
  }
  onMount(() => {
    let alive = true;
    void Promise.all([loadEngine(), api.games()])
      .then(([, data]) => {
        if (alive) {
          catalog = data.games;
          ready = true;
          const requested = gameSelection.id;
          gameSelection.id = null;
          if (requested) {
            const selected = data.games.find((item) => item.id === requested);
            if (selected) void choose(selected);
          }
        }
      })
      .catch(() => {
        if (alive) failed = true;
      });
    loop = new GameLoop(
      (dt) => {
        elapsed += dt;
        if (game?.id === 'falling') {
          const prior = draft?.summaries.at(-1)?.accuracy ?? 85;
          const duration = 12000 - prior * 40;
          y = -35 + Math.min(1, elapsed / duration) * 335;
          remaining = Math.max(0, Math.ceil((duration - elapsed) / 1000));
          if (elapsed >= duration) void complete(true);
        }
        if (elapsed - lastCheckpoint >= 30000) {
          lastCheckpoint = elapsed;
          void checkpoint();
        }
      },
      preferences.low_end || preferences.reduce_motion,
      () => {
        running = false;
        pauseSounds();
        if (!saving) void checkpoint();
      },
    );
    window.addEventListener('keydown', key);
    const leave = registerLeaveGuard(async () => {
      loop?.pause();
      if (!draft || !session || saved) return;
      await api.saveGameSession(
        profile.id,
        draft.game_id,
        summary(),
        round?.target ?? null,
        false,
      );
      await checkpoint();
    });
    return () => {
      alive = false;
      loop?.dispose();
      session?.free();
      window.removeEventListener('keydown', key);
      leave();
    };
  });
</script>

<section class="page stack">
  <header class="row heading">
    <h1>{game ? game.title[locale.language] : t.games}</h1>
    {#if game}<button onclick={() => navigate('home')}>{t.pauseLesson}</button
      >{/if}
  </header>
  {#if failed}<p class="error" role="alert">{t.saveFailed}</p>{/if}
  {#if !ready}<p>{t.loading}</p>{:else if !game}<div class="game-cards">
      {#each catalog as item, i (item.id)}<button
          disabled={saving}
          onclick={() => choose(item)}
          ><span aria-hidden="true">{['⌨', '▧', '＿', '→'][i]}</span>
          <h2>{item.title[locale.language]}</h2></button
        >{/each}
    </div>
  {:else if draft && round}<div class="row heading">
      <strong
        >{t.gameRound}
        {digits(draft.step + 1)} / {digits(game.rounds.length)}</strong
      >{#if game.id === 'falling'}<span
          >{t.gameLives}: {digits(draft.lives)}{#if preferences.reduce_motion}
            · {t.timeLeft}: {digits(remaining)} {t.seconds}{/if}</span
        >{/if}
    </div>
    {#if game.id === 'journey'}<div class="stops" aria-label={t.gameRound}>
        {#each game.rounds as item, i (item.target + i)}<span
            class:done={i < draft.step}
            class:current={i === draft.step}>{digits(i + 1)}</span
          >{/each}
      </div>{/if}
    <section
      class="panel play-area"
      class:falling={game.id === 'falling'}
      aria-label={game.title[locale.language]}
    >
      {#if game.id === 'falling'}<div
          class="sprite"
          style:transform={preferences.reduce_motion
            ? 'translate3d(0,120px,0)'
            : `translate3d(0,${y}px,0)`}
        >
          {round.target}
        </div>{:else}<div class="picture" aria-hidden="true">{round.emoji}</div>
        <p class="target">
          {game.id === 'complete'
            ? round.prompt
            : game.id === 'picture' && !hinted
              ? ''
              : round.target}
        </p>{/if}
      <div
        bind:this={capture}
        class="capture"
        tabindex="0"
        role="textbox"
        aria-label={t.capture}
        aria-readonly="true"
        inputmode="none"
        oncompositionstart={(event) => event.preventDefault()}
      ></div>
    </section>
    <div class="row heading">
      <p class="muted">{running ? t.active : t.ready} · {digits(cursor)}</p>
      <div class="row">
        <button onclick={() => (hinted = true)}>{t.gameHint}</button><button
          class="primary"
          disabled={saving}
          onclick={() => {
            if (running) loop?.pause();
            else void start();
          }}>{running ? t.pause : t.start}</button
        ><button disabled={saving} onclick={stop}>{t.gameStop}</button>
      </div>
    </div>
    {#if hinted}<p class="target hint">{round.target}</p>
      <Keyboard {layout} highlight={expected} showFingers={true} />{/if}
  {:else if draft}<p>{t.gameEncouragement}</p>
    <button class="primary" disabled={saving} onclick={finish}
      >{t.continue}</button
    >{/if}
</section>

<style>
  .heading {
    justify-content: space-between;
  }
  .game-cards {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 24px;
  }
  .game-cards button {
    min-height: 180px;
    padding: 24px;
  }
  .game-cards span {
    font-size: 44px;
    color: var(--primary);
  }
  .play-area {
    min-height: 240px;
    text-align: center;
    position: relative;
    contain: layout paint;
  }
  .falling {
    height: 360px;
    overflow: hidden;
  }
  .sprite {
    font-size: 64px;
    line-height: 1.9;
    will-change: transform;
  }
  .picture {
    font-size: 64px;
  }
  .target {
    font-family: 'Kantumruy Pro', sans-serif;
    font-size: 40px;
    line-height: 1.9;
  }
  .hint {
    text-align: center;
  }
  .capture {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
  }
  .play-area:focus-within {
    outline: 2px solid var(--primary);
    outline-offset: 3px;
  }
  .stops {
    display: flex;
    justify-content: space-between;
    gap: 8px;
  }
  .stops span {
    display: grid;
    place-items: center;
    width: 44px;
    height: 44px;
    border: 1px solid var(--line);
    border-radius: 50%;
  }
  .stops .done {
    background: var(--primary);
    color: var(--on-primary);
  }
  .stops .current {
    border: 3px solid var(--primary);
  }
</style>
