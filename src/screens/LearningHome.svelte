<script lang="ts">
  import {
    api,
    type ContentIndex,
    type LearningPath,
    type Progress,
  } from '../lib/api';
  import { user } from '../lib/stores/profile.svelte';
  import { learning } from '../lib/stores/learning.svelte';
  import { navigate } from '../lib/router.svelte';
  import { messages, locale } from '../lib/i18n/index.svelte';
  import { configureActivity } from '../lib/stores/activity.svelte';
  let t = $derived(messages());
  let path = $state<LearningPath | null>(null);
  let index = $state<ContentIndex | null>(null);
  let progress = $state<Progress | null>(null);
  let weak = $state(false);
  let failed = $state(false);
  $effect(() => {
    const profile = user.profile;
    if (!profile) return;
    let alive = true;
    void Promise.all([
      api.path(profile.path_id ?? 'beginner'),
      api.contentIndex(),
      api.progress(profile.id),
      api.analysis(profile.id),
    ])
      .then(([p, i, r, a]) => {
        if (alive) {
          path = p;
          index = i;
          progress = r;
          weak = a.weak.length > 0;
        }
      })
      .catch(() => {
        if (alive) failed = true;
      });
    return () => {
      alive = false;
    };
  });
  let next = $derived(
    path?.units
      .flatMap((unit) => unit.lessons)
      .find(
        (id) =>
          !progress?.lessons.some(
            (p) => p.lesson_id === id && p.best_stars > 0,
          ),
      ) ?? null,
  );
  let title = $derived(
    index?.lessons.find((lesson) => lesson.id === next)?.title[locale.language],
  );
  function start() {
    if (!next) return;
    learning.lessonId = next;
    user.resume = null;
    void navigate('lesson');
  }
  async function drill() {
    if (!user.profile) return;
    try {
      configureActivity(await api.drill(user.profile.id), 'drill');
      await navigate('practice');
    } catch {
      failed = true;
    }
  }
</script>

<div class="stack">
  <div class="panel continue">
    <div>
      <p class="muted">{path?.title[locale.language] ?? t.learningPath}</p>
      <h2>{next ? title : t.pathComplete}</h2>
    </div>
    <button class="primary" onclick={start} disabled={!next}
      >{t.continueLesson} <span aria-hidden="true">→</span></button
    >
  </div>
  <div class="row">
    <button onclick={() => navigate('path')}>{t.learningPath}</button><button
      onclick={() => navigate('intro')}>{t.keyboardIntro}</button
    ><button onclick={() => navigate('games')}>{t.games}</button><button
      onclick={() => navigate('vocabulary')}>{t.vocabulary}</button
    ><button onclick={() => navigate('progress')}>{t.myProgress}</button
    >{#if weak}<button onclick={drill}>{t.drillThese}</button>{/if}
  </div>
  {#if failed}<p class="error" role="alert">{t.loadFailed}</p>{/if}
</div>

<style>
  .continue {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 24px;
    min-height: 190px;
    background: var(--surface-soft);
  }
  .continue h2 {
    font-size: 28px;
  }
  .continue button {
    white-space: nowrap;
  }
</style>
