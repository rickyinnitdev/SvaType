<script lang="ts">
  import { api, type VocabCategory } from '../lib/api';
  import { messages, locale, digits } from '../lib/i18n/index.svelte';
  import { configureActivity } from '../lib/stores/activity.svelte';
  import { user } from '../lib/stores/profile.svelte';
  import { navigate } from '../lib/router.svelte';
  let t = $derived(messages());
  let categories = $state<Omit<VocabCategory, 'words'>[]>([]);
  let category = $state<VocabCategory | null>(null);
  let selected = $state('');
  let at = $state(0);
  let failed = $state(false);
  let player: HTMLAudioElement | undefined;
  const images = import.meta.glob('../../content/images/vocab/*.svg', {
    eager: true,
    query: '?url',
    import: 'default',
  }) as Record<string, string>;
  const recordings = import.meta.glob('../../content/audio/*.ogg', {
    eager: true,
    query: '?url',
    import: 'default',
  }) as Record<string, string>;
  let word = $derived(category?.words[at]);
  let audio = $derived(
    word?.audio ? recordings[`../../content/audio/${word.audio}`] : undefined,
  );
  $effect(() => {
    let alive = true;
    void api
      .vocabIndex()
      .then((value) => {
        if (alive) categories = value.categories;
      })
      .catch(() => {
        if (alive) failed = true;
      });
    return () => {
      alive = false;
    };
  });
  $effect(() => {
    const id = selected;
    if (!id) return;
    let alive = true;
    failed = false;
    category = null;
    void api
      .vocab(id)
      .then((value) => {
        if (alive) {
          category = value;
          at = 0;
        }
      })
      .catch(() => {
        if (alive) failed = true;
      });
    return () => {
      alive = false;
    };
  });
  $effect(() => {
    void audio;
    return () => {
      player?.pause();
      player = undefined;
    };
  });
  function practice() {
    if (!word) return;
    configureActivity(word.word, 'vocab', 0, word.id);
    user.resume = null;
    void navigate('practice');
  }
  async function play() {
    if (!audio) return;
    player?.pause();
    player = new Audio(audio);
    try {
      await player.play();
    } catch {
      failed = true;
    }
  }
</script>

<section class="page stack">
  <header class="row heading">
    <h1>{t.vocabulary}</h1>
    {#if selected}<button
        onclick={() => {
          selected = '';
          category = null;
        }}>{t.chooseCategory}</button
      >{/if}
  </header>
  {#if failed}<p class="error" role="alert">{t.loadFailed}</p>{/if}
  {#if !selected}<div class="categories">
      {#each categories as item (item.id)}<button
          class="category"
          onclick={() => (selected = item.id)}
          ><img
            src={images[`../../content/images/vocab/${item.icon}`]}
            alt=""
            width="90"
            height="75"
          />
          <h2>{item.title[locale.language]}</h2></button
        >{/each}
    </div>
  {:else if word && category}<p class="muted">
      {category.title[locale.language]} · {digits(at + 1)} / {digits(
        category.words.length,
      )}
    </p>
    <article class="panel card stack">
      <div class="picture" aria-hidden="true">
        {#if word.image && images[`../../content/images/vocab/${word.image}`]}<img
            src={images[`../../content/images/vocab/${word.image}`]}
            alt=""
            width="140"
            height="140"
          />{:else}{word.emoji}{/if}
      </div>
      <h2 class="word">{word.word}</h2>
      <div>
        <h3>{t.meaning}</h3>
        <p>{locale.language === 'km' ? word.meaning_km : word.meaning_en}</p>
      </div>
      <div>
        <h3>{t.example}</h3>
        <p>{word.example_km}</p>
      </div>
      <div class="row">
        {#if audio}<button onclick={play}>{t.playAudio}</button>{/if}<button
          class="primary"
          onclick={practice}>{t.typeWord}</button
        >
      </div>
    </article>
    <div class="row heading">
      <button disabled={at === 0} onclick={() => at--}>{t.previousWord}</button
      ><button disabled={at === category.words.length - 1} onclick={() => at++}
        >{t.nextWord}</button
      >
    </div>
  {:else}<p role="status">{t.loading}</p>{/if}
</section>

<style>
  .heading {
    justify-content: space-between;
  }
  .categories {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 18px;
  }
  .category {
    display: flex;
    align-items: center;
    gap: 18px;
    text-align: left;
    padding: 18px;
    min-height: 120px;
  }
  .category h2 {
    font-size: 21px;
  }
  .card {
    max-width: 760px;
    width: 100%;
    margin: auto;
    align-items: center;
    text-align: center;
  }
  .picture {
    font-size: 70px;
    line-height: 1.5;
  }
  .word {
    font-size: 48px;
    line-height: 1.9;
  }
  h3 {
    font-size: 17px;
    color: var(--muted);
  }
  .card p {
    font-size: 22px;
    line-height: 1.9;
  }
</style>
