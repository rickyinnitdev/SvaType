<script lang="ts">
  import { onMount } from 'svelte';
  import { api, type Lesson, type LessonDraft } from '../lib/api';
  import { user } from '../lib/stores/profile.svelte';
  import { learning } from '../lib/stores/learning.svelte';
  import { preferences } from '../lib/stores/settings.svelte';
  import { messages, locale, digits } from '../lib/i18n/index.svelte';
  import { navigate, registerLeaveGuard } from '../lib/router.svelte';
  import Practice from './Practice.svelte';
  import Keyboard from '../lib/components/Keyboard.svelte';
  import Hands from '../lib/components/Hands.svelte';
  import nida from '../../content/layouts/nida.json';
  import legacy from '../../content/layouts/khmer_legacy.json';
  import posture from '../../content/images/posture.svg';
  import type { LayoutData, Stroke, Summary } from '../lib/engine';
  const profile = user.profile!;
  let t = $derived(messages());
  let lesson = $state<Lesson | null>(null);
  let draft = $state<LessonDraft | null>(null);
  let failed = $state(false);
  let busy = $state(false);
  let finished = false;
  let attempt = $state(0);
  let retry = $state(false);
  let demoIndex = $state(0);
  let playing = $state(false);
  let readingStarted: number | null = null;
  let readingMs = 0;
  function pauseReading() {
    if (readingStarted !== null)
      readingMs += Math.max(0, performance.now() - readingStarted);
    readingStarted = null;
  }
  function resumeReading() {
    if (
      step &&
      ['explain', 'demo'].includes(step.type) &&
      !document.hidden &&
      document.hasFocus() &&
      readingStarted === null
    )
      readingStarted = performance.now();
  }
  function resetReading() {
    readingMs = 0;
    readingStarted = null;
    resumeReading();
  }
  async function readingSummary(): Promise<Summary | undefined> {
    pauseReading();
    if (readingMs < 1) return;
    return api.readingSummary(Math.round(readingMs));
  }
  $effect(() => {
    if (
      !playing ||
      step?.type !== 'demo' ||
      preferences.low_end ||
      preferences.reduce_motion
    )
      return;
    const timer = setTimeout(() => {
      if (demoIndex + 1 < (step?.keys?.length ?? 0)) demoIndex++;
      else playing = false;
    }, 600);
    return () => clearTimeout(timer);
  });
  let step = $derived(lesson && draft ? lesson.steps[draft.step] : null);
  let layout = $derived(
    (draft?.layout_id === 'khmer_legacy' ? legacy : nida) as LayoutData,
  );
  let code = $derived(
    step?.keys?.[demoIndex] ?? lesson?.focus_keys[0] ?? 'KeyK',
  );
  let highlight = $derived<Stroke | null>(
    layout.keys[code]
      ? {
          code,
          layer: 'base',
          produces: layout.keys[code].base ?? '',
          finger: layout.keys[code].finger,
          cluster_index: 0,
        }
      : null,
  );
  let target = $derived(
    typeof step?.text === 'string'
      ? step.text
      : (step?.items ?? []).join(step?.type === 'find_key' ? '' : ' '),
  );
  onMount(() => {
    let alive = true;
    const stop = () => {
      playing = false;
      pauseReading();
    };
    const visibility = () => {
      if (document.hidden) stop();
      else resumeReading();
    };
    window.addEventListener('blur', stop);
    window.addEventListener('focus', resumeReading);
    document.addEventListener('visibilitychange', visibility);
    void (async () => {
      try {
        const l = await api.lesson(learning.lessonId);
        const d = (await api.lessonDraft(profile.id, l.id)) ?? {
          profile_id: profile.id,
          lesson_id: l.id,
          uid: crypto.randomUUID(),
          step: 0,
          summaries: [],
          texts: [],
          started_at: Date.now(),
          layout_id: profile.layout_id,
        };
        await api.saveLessonDraft(d);
        if (d.step >= l.steps.length) {
          const id = await api.saveLessonSession(
            profile.id,
            l.id,
            null,
            null,
            true,
          );
          finished = true;
          if (id !== null) await navigate('result', id);
          return;
        }
        const checkpoints = await api.checkpoints(profile.id);
        if (alive) {
          user.resume =
            checkpoints.find(
              (c) =>
                c.session.uid === d.uid && c.session.bookmark_step === d.step,
            ) ?? null;
          lesson = l;
          draft = d;
          resetReading();
        }
      } catch {
        if (alive) failed = true;
      }
    })();
    const leave = registerLeaveGuard(async () => {
      if (!draft || finished) return;
      const read = await readingSummary();
      const updated = read
        ? { ...draft, summaries: [...draft.summaries, read] }
        : draft;
      await api.saveLessonDraft(updated);
      draft = updated;
      resetReading();
      await api.saveLessonSession(profile.id, draft.lesson_id);
    });
    return () => {
      alive = false;
      window.removeEventListener('blur', stop);
      window.removeEventListener('focus', resumeReading);
      document.removeEventListener('visibilitychange', visibility);
      leave();
    };
  });
  async function advance(summary?: Summary, text?: string) {
    if (!draft || !lesson || busy) return;
    busy = true;
    failed = false;
    try {
      summary ??= await readingSummary();
      const repeat =
        summary &&
        step?.type === 'challenge' &&
        summary.accuracy < (step.min_accuracy ?? 70);
      const updated = {
        ...draft,
        step: draft.step + (repeat ? 0 : 1),
        summaries: summary ? [...draft.summaries, summary] : draft.summaries,
        texts: text ? [...draft.texts, text] : draft.texts,
      };
      await api.saveLessonDraft(updated);
      user.resume = null;
      draft = updated;
      resetReading();
      demoIndex = 0;
      retry = !!repeat;
      attempt++;
      if (updated.step >= lesson.steps.length) {
        const id = await api.saveLessonSession(
          profile.id,
          lesson.id,
          null,
          null,
          true,
        );
        finished = true;
        busy = false;
        if (id !== null) await navigate('result', id);
      }
    } catch {
      failed = true;
      throw new Error('save_failed');
    } finally {
      busy = false;
      resumeReading();
    }
  }
  async function pause(summary: Summary, text: string) {
    if (draft)
      await api.saveLessonSession(profile.id, draft.lesson_id, summary, text);
  }
</script>

<section
  class="lesson stack"
  style:--font-target={profile.path_id === 'grade3' || profile.grade === 3
    ? `${Math.max(40, { s: 30, m: 36, l: 42, xl: 48 }[preferences.font_size])}px`
    : undefined}
>
  {#if failed}<p class="error" role="alert">{t.saveFailed}</p>{/if}
  {#if lesson && draft && step}<header class="row">
      <div>
        <p class="muted">
          {t.step}
          {digits(draft.step + 1)} / {digits(lesson.steps.length)}
        </p>
        <h1>{lesson.title[locale.language]}</h1>
      </div>
      <button onclick={() => navigate('home')}>{t.pauseLesson}</button>
    </header>
    {#if retry}<p role="status">{t.lessonRetry}</p>{/if}
    {#if step.type === 'explain'}<div class="panel stack explanation">
        {#if step.image === 'posture.svg'}<img
            src={posture}
            alt={t.posture}
          />{/if}
        <p>
          {typeof step.text === 'object'
            ? step.text[locale.language]
            : step.text}
        </p>
        <button class="primary" disabled={busy} onclick={() => advance()}
          >{t.continue}</button
        >
      </div>
    {:else if step.type === 'demo'}<div class="panel stack">
        <Hands active={highlight?.finger ?? null} /><Keyboard
          {layout}
          {highlight}
          showFingers={true}
        />
        <div class="row">
          {#if !preferences.low_end && !preferences.reduce_motion}<button
              onclick={() => {
                demoIndex = 0;
                playing = true;
              }}>{t.replayDemo}</button
            >{/if}
          <button
            onclick={() =>
              (demoIndex = (demoIndex + 1) % (step?.keys?.length || 1))}
            >{t.nextKeyDemo}</button
          ><button class="primary" disabled={busy} onclick={() => advance()}
            >{t.continue}</button
          >
        </div>
      </div>
    {:else}{#key `${draft.step}-${attempt}`}<Practice
          {target}
          kind="lesson"
          refId={lesson.id}
          free={false}
          durationMs={0}
          strict={step.type === 'find_key'}
          findKey={step.type === 'find_key'}
          layoutOverride={draft.layout_id}
          sessionUid={draft.uid}
          bookmarkStep={draft.step}
          oncomplete={advance}
          onpause={pause}
        />{/key}{/if}
  {:else if !failed}<p role="status">{t.loading}</p>{/if}
</section>

<style>
  .lesson {
    max-width: 1200px;
    margin: auto;
    padding: 20px 28px;
  }
  header {
    justify-content: space-between;
  }
  h1 {
    font-size: 25px;
  }
  .explanation {
    max-width: 850px;
    margin: auto;
    font-size: 22px;
    line-height: 1.9;
  }
  .explanation img {
    height: 230px;
    object-fit: contain;
  }
</style>
