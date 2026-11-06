<script lang="ts">
  import { api, type Progress, type Checkpoint } from '../lib/api';
  import { user } from '../lib/stores/profile.svelte';
  import { messages, digits } from '../lib/i18n/index.svelte';
  import { navigate } from '../lib/router.svelte';
  import Avatar from '../lib/components/Avatar.svelte';
  let t = $derived(messages());
  let progress = $state<Progress | null>(null);
  let checkpoints = $state<Checkpoint[]>([]);
  let failed = $state(false);
  $effect(() => {
    const profile = user.profile;
    if (!profile) return;
    let alive = true;
    void Promise.all([api.progress(profile.id), api.checkpoints(profile.id)])
      .then(([p, c]) => {
        if (alive) {
          progress = p;
          checkpoints = c;
        }
      })
      .catch(() => {
        if (alive) failed = true;
      });
    return () => {
      alive = false;
    };
  });
  function practice(checkpoint: Checkpoint | null = null) {
    user.resume = checkpoint;
    void navigate('practice');
  }
</script>

<section class="page stack">
  <header class="row">
    <Avatar id={user.profile?.avatar ?? '0'} size={76} />
    <div>
      <p class="muted">
        {user.profile?.preferred_mode === 'learning' ? t.learning : t.typing}
      </p>
      <h1>{t.hello}, {user.profile?.name}</h1>
    </div>
  </header>
  {#if failed}<p class="error" role="alert">{t.loadFailed}</p>{/if}
  {#if checkpoints[0]}<div class="panel row resume">
      <div>
        <h2>{t.resumePractice}</h2>
        <p class="muted">{t.resumeHint}</p>
      </div>
      <button class="primary" onclick={() => practice(checkpoints[0])}
        >{t.continue}</button
      >
    </div>{/if}
  <div class="welcome panel">
    <div>
      <h2>{t.practiceDescription}</h2>
      <p class="muted">{t.switchLater}</p>
    </div>
    <button class="primary" onclick={() => practice()}
      >{t.practice} <span aria-hidden="true">→</span></button
    >
  </div>
  <div class="tiles">
    {#each [[t.totalTime, digits(Math.round((progress?.duration_ms ?? 0) / 60000)), t.minutes], [t.lessonsDone, digits(progress?.lessons.filter((l) => l.completed_at !== null).length ?? 0), ''], [t.wordsLearned, digits(progress?.words_learned ?? 0), '']] as [label, value, unit] (label)}<div
        class="panel"
      >
        <p class="muted">{label}</p>
        <strong>{value}</strong><span class="muted">{unit}</span>
      </div>{/each}
  </div>
  <button onclick={() => navigate('history')}
    >{t.history} <span aria-hidden="true">→</span></button
  >
</section>

<style>
  .welcome {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 32px;
    min-height: 200px;
    background: var(--surface-soft);
  }
  .welcome h2 {
    max-width: 540px;
    font-size: 28px;
  }
  .welcome button {
    white-space: nowrap;
  }
  .tiles {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 20px;
  }
  .tiles strong {
    font-size: 42px;
    display: inline-block;
    margin: 8px 10px 0 0;
  }
  .resume {
    justify-content: space-between;
  }
</style>
