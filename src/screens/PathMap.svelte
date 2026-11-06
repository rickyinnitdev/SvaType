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
  import { messages, locale, digits } from '../lib/i18n/index.svelte';
  let t = $derived(messages());
  let pathId = $state(user.profile?.path_id ?? 'beginner');
  let index = $state<ContentIndex | null>(null);
  let path = $state<LearningPath | null>(null);
  let progress = $state<Progress | null>(null);
  let unlock = $state(false);
  let failed = $state(false);
  $effect(() => {
    const id = user.profile?.id;
    if (!id) return;
    const selected = pathId;
    let alive = true;
    void Promise.all([
      api.contentIndex(),
      api.path(selected),
      api.progress(id),
      api.learningOptions(id),
    ])
      .then(([i, p, r, o]) => {
        if (alive) {
          index = i;
          path = p;
          progress = r;
          unlock = o.unlock_all;
        }
      })
      .catch(() => {
        if (alive) failed = true;
      });
    return () => {
      alive = false;
    };
  });
  let ids = $derived(path?.units.flatMap((unit) => unit.lessons) ?? []);
  function stars(id: string) {
    return (
      progress?.lessons.find((row) => row.lesson_id === id)?.best_stars ?? 0
    );
  }
  function unlocked(id: string) {
    const at = ids.indexOf(id);
    return unlock || at === 0 || stars(id) > 0 || stars(ids[at - 1]) > 0;
  }
  async function choose(value: string) {
    pathId = value;
    if (!user.profile) return;
    try {
      const profile = { ...user.profile, path_id: pathId };
      await api.updateProfile(profile.id, profile);
      user.profile = profile;
    } catch {
      failed = true;
    }
  }
  function open(id: string) {
    learning.lessonId = id;
    user.resume = null;
    void navigate('lesson');
  }
</script>

<section class="page stack">
  <header class="row heading">
    <h1>{t.learningPath}</h1>
    <label
      >{t.choosePath}<select
        bind:value={pathId}
        onchange={(event) => choose(event.currentTarget.value)}
        >{#each index?.paths ?? [] as item (item.id)}<option value={item.id}
            >{item.title[locale.language]}</option
          >{/each}</select
      ></label
    >
  </header>
  {#if failed}<p role="alert" class="error">
      {t.loadFailed}
    </p>{/if}{#each path?.units ?? [] as unit (unit.id)}<section
      class="unit stack"
    >
      <h2>{unit.title[locale.language]}</h2>
      {#each unit.lessons as id (id)}<button
          class="lesson-row"
          disabled={!unlocked(id)}
          onclick={() => open(id)}
          title={!unlocked(id) ? t.locked : undefined}
          ><span class="number">{digits(ids.indexOf(id) + 1)}</span><span
            class="title"
            >{index?.lessons.find((lesson) => lesson.id === id)?.title[
              locale.language
            ]}</span
          ><span class="stars" aria-label={`${t.stars}: ${digits(stars(id))}`}
            >{stars(id)
              ? '★'.repeat(stars(id))
              : unlocked(id)
                ? '○'
                : '—'}</span
          ></button
        >{/each}
    </section>{/each}
</section>

<style>
  .heading {
    justify-content: space-between;
  }
  label {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .unit {
    gap: 12px;
    border-left: 2px solid var(--line);
    padding-left: 24px;
  }
  .lesson-row {
    display: flex;
    align-items: center;
    gap: 20px;
    text-align: left;
    min-height: 78px;
    padding: 16px 24px;
  }
  .number {
    width: 38px;
    height: 38px;
    border-radius: 50%;
    background: var(--surface-soft);
    display: grid;
    place-items: center;
    color: var(--primary);
  }
  .title {
    flex: 1;
  }
  .stars {
    color: var(--primary);
    letter-spacing: 4px;
  }
</style>
