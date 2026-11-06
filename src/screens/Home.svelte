<script lang="ts">
  import { api, type Progress, type Checkpoint } from '../lib/api';
  import { user } from '../lib/stores/profile.svelte';
  import { messages, digits } from '../lib/i18n/index.svelte';
  import { navigate } from '../lib/router.svelte';
  import { gameSelection } from '../lib/stores/game.svelte';
  import LearningHome from './LearningHome.svelte';
  import { learning } from '../lib/stores/learning.svelte';
  import Avatar from '../lib/components/Avatar.svelte';
  let t = $derived(messages());
  let progress = $state<Progress | null>(null);
  let checkpoints = $state<Checkpoint[]>([]);
  let failed = $state(false);
  const typingCards = [
    ['free', 'free', 'freeDescription'],
    ['test', 'test', 'testDescription'],
    ['custom', 'custom', 'customDescription'],
    ['random', 'random', 'randomDescription'],
    ['analysis', 'analysis', 'analysisDescription'],
  ] as const;
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
    if (checkpoint?.session.kind === 'lesson' && checkpoint.session.ref_id) {
      learning.lessonId = checkpoint.session.ref_id;
      void navigate('lesson');
    } else if (
      checkpoint?.session.kind === 'game' &&
      checkpoint.session.ref_id
    ) {
      gameSelection.id = checkpoint.session.ref_id;
      void navigate('games');
    } else void navigate('practice');
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
  {#if user.profile?.preferred_mode === 'typing'}<p class="muted">
      {t.typingDescription}
    </p>
    <div class="typing-cards">
      {#each typingCards as [screen, title, description], i (screen)}<button
          class="typing-card"
          onclick={() => navigate(screen)}
          ><span class="badge" aria-hidden="true">{digits(i + 1)}</span>
          <h2>{t[title]}</h2>
          <p class="muted">{t[description]}</p></button
        >{/each}
    </div>{:else}<LearningHome />{/if}
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
  .typing-cards {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 20px;
  }
  .typing-card {
    text-align: left;
    display: grid;
    gap: 12px;
    align-content: start;
    padding: 24px;
    min-height: 220px;
  }
  .badge {
    width: 34px;
    height: 34px;
    background: var(--surface-soft);
    color: var(--primary);
    border-radius: 50%;
    display: grid;
    place-items: center;
    font-size: var(--font-ui);
  }
  .typing-card h2 {
    font-size: 21px;
  }
  .typing-card p {
    font-size: var(--font-ui);
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
