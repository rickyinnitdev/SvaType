<script lang="ts">
  import { onMount } from 'svelte';
  import { api, type ProfileInput } from '../lib/api';
  import { user, selectProfile } from '../lib/stores/profile.svelte';
  import { messages, locale, digits } from '../lib/i18n/index.svelte';
  import { loadEngine, TypingSession, type LayoutData } from '../lib/engine';
  import Avatar from '../lib/components/Avatar.svelte';
  import nida from '../../content/layouts/nida.json';
  import legacy from '../../content/layouts/khmer_legacy.json';
  let t = $derived(messages());
  let step = $state(user.profiles.length ? 1 : 0);
  let name = $state('');
  let avatar = $state('0');
  let grade = $state('');
  let mode = $state<'learning' | 'typing'>('learning');
  let layout = $state('nida');
  let wrong = $state(0);
  let matched = $state(false);
  let error = $state('');
  let busy = $state(false);
  let engineReady = $state(false);
  let check: TypingSession | undefined;
  $effect(() => {
    if (step !== 3 || !engineReady) return;
    const current = new TypingSession(
      'ក',
      (layout === 'nida' ? nida : legacy) as LayoutData,
      'stop_on_error',
      true,
    );
    check = current;
    matched = false;
    return () => {
      current.free();
      check = undefined;
    };
  });
  function next() {
    if (step === 1 && (!name.trim() || Array.from(name).length > 80)) {
      error = t.nameRequired;
      return;
    }
    error = '';
    step++;
  }
  async function finish() {
    if (busy) return;
    busy = true;
    error = '';
    try {
      const value: ProfileInput = {
        name: name.trim(),
        avatar,
        grade: grade ? Number(grade) : null,
        path_id: grade ? `grade${grade}` : 'beginner',
        layout_id: layout as ProfileInput['layout_id'],
        preferred_mode: mode,
      };
      const profile = await api.createProfile(value);
      await api.setSetting('language', locale.language);
      user.profiles = await api.profiles();
      await selectProfile(profile);
    } catch {
      error = t.saveFailed;
    } finally {
      busy = false;
    }
  }
  onMount(() => {
    let alive = true;
    void loadEngine()
      .then(() => {
        if (alive) engineReady = true;
      })
      .catch(() => {});
    const key = (event: KeyboardEvent) => {
      if (
        event.code === 'Tab' ||
        event.code === 'Escape' ||
        event.metaKey ||
        (event.ctrlKey && !event.getModifierState('AltGraph'))
      )
        return;
      if (
        step !== 3 ||
        !check ||
        matched ||
        event.repeat ||
        event.target instanceof HTMLSelectElement ||
        event.target instanceof HTMLButtonElement
      )
        return;
      event.preventDefault();
      const result = check.onKey(event);
      if (result.ignored) return;
      if (result.correct) matched = true;
      else wrong++;
    };
    window.addEventListener('keydown', key);
    return () => {
      alive = false;
      window.removeEventListener('keydown', key);
    };
  });
</script>

<section class="onboarding page stack">
  <p class="eyebrow">
    {t.welcome} <span>{digits(step + 1)} / {digits(4)}</span>
  </p>
  {#if step === 0}<h1>{t.chooseLanguage}</h1>
    <p class="muted">{t.welcomeHint}</p>
    <div class="row">
      <button
        class:selected={locale.language === 'km'}
        onclick={() => {
          locale.language = 'km';
          document.documentElement.lang = 'km';
        }}>{t.khmer}</button
      ><button
        class:selected={locale.language === 'en'}
        onclick={() => {
          locale.language = 'en';
          document.documentElement.lang = 'en';
        }}>{t.english}</button
      >
    </div>
  {:else if step === 1}<h1>{t.who}</h1>
    <div class="panel stack">
      <label
        >{t.name}<input
          bind:value={name}
          maxlength="80"
          autocomplete="off"
          onkeydown={(e) => {
            if (e.key === 'Enter') next();
          }}
        /></label
      ><label
        >{t.grade}<select bind:value={grade}
          ><option value="">{t.none}</option
          >{#each ['3', '4', '5'] as n (n)}<option value={n}>{digits(n)}</option
            >{/each}</select
        ></label
      >
      <fieldset>
        <legend>{t.avatar}</legend>
        <div class="avatars">
          {#each Array.from( { length: 12 }, (_, i) => String(i) ) as id (id)}<button
              class:selected={avatar === id}
              aria-label={`${t.avatar} ${digits(Number(id) + 1)}`}
              aria-pressed={avatar === id}
              onclick={() => (avatar = id)}><Avatar {id} size={56} /></button
            >{/each}
        </div>
      </fieldset>
    </div>
  {:else if step === 2}<h1>{t.chooseMode}</h1>
    <p class="muted">{t.switchLater}</p>
    <div class="mode-cards">
      {#each ['learning', 'typing'] as value (value)}<button
          class:selected={mode === value}
          onclick={() => {
            mode = value as typeof mode;
          }}
          aria-pressed={mode === value}
          ><strong>{value === 'learning' ? t.learning : t.typing}</strong><span
            class="muted"
            >{value === 'learning'
              ? t.learnDescription
              : t.typeDescription}</span
          ></button
        >{/each}
    </div>
  {:else}<h1>{t.keyboardCheck}</h1>
    <p>{t.keyboardCheckHint}</p>
    <div class="key-check" class:matched>{matched ? '✓' : 'ក'}</div>
    {#if matched}<p role="status">{t.keyboardMatch}</p>{/if}{#if wrong >= 2}<p
        class="muted"
      >
        {t.chooseLayout}
      </p>
      <label
        >{t.layout}<select bind:value={layout}
          ><option value="nida">{nida.name[locale.language]}</option><option
            value="khmer_legacy">{legacy.name[locale.language]}</option
          ></select
        ></label
      >{/if}
  {/if}
  {#if error}<p role="alert" class="error">{error}</p>{/if}
  <div class="row actions">
    {#if step > 0}<button onclick={() => step--} disabled={busy}
        >{t.back}</button
      >{/if}{#if step < 3}<button class="primary" onclick={next}
        >{t.next}</button
      >{:else}<button class="primary" onclick={finish} disabled={busy}
        >{busy ? t.saving : matched ? t.continue : t.skip}</button
      >{/if}
  </div>
</section>

<style>
  .onboarding {
    max-width: 780px;
  }
  .eyebrow {
    display: flex;
    justify-content: space-between;
    color: var(--primary);
  }
  .selected {
    outline: 2px solid var(--primary);
    background: var(--surface-soft);
  }
  label {
    display: grid;
    gap: 6px;
  }
  fieldset {
    border: 0;
    padding: 0;
    margin: 0;
  }
  .avatars {
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    gap: 10px;
  }
  .avatars button {
    padding: 8px;
    min-width: 0;
    display: grid;
    place-items: center;
  }
  .mode-cards {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 20px;
  }
  .mode-cards button {
    display: flex;
    flex-direction: column;
    text-align: left;
    gap: 20px;
    padding: 28px;
    min-height: 200px;
  }
  .mode-cards strong {
    font-size: 24px;
  }
  .key-check {
    font-size: 64px;
    line-height: 1.8;
    border: 2px solid var(--line);
    border-radius: 20px;
    width: 140px;
    text-align: center;
    background: var(--surface);
  }
  .matched {
    border-color: var(--primary);
    color: var(--primary);
  }
  .actions {
    justify-content: flex-end;
  }
</style>
