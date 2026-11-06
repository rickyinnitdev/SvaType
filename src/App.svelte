<script lang="ts">
  import { onMount, type Component } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { messages, locale } from './lib/i18n/index.svelte';
  import { user, initialize } from './lib/stores/profile.svelte';
  import {
    router,
    navigate,
    type Screen as ScreenName,
  } from './lib/router.svelte';
  import { api } from './lib/api';
  import Avatar from './lib/components/Avatar.svelte';
  import Diagnostics from './lib/components/Diagnostics.svelte';
  let t = $derived(messages());
  let Screen = $state<Component | null>(null);
  let sandbox = $state(false);
  let failed = $state(false);
  let loading = $state(true);
  const loaders: Record<ScreenName, () => Promise<{ default: Component }>> = {
    profiles: () => import('./screens/Profiles.svelte'),
    onboarding: () => import('./screens/Onboarding.svelte'),
    home: () => import('./screens/Home.svelte'),
    practice: () => import('./screens/Practice.svelte'),
    history: () => import('./screens/History.svelte'),
    result: () => import('./screens/Result.svelte'),
    settings: () => import('./screens/Settings.svelte'),
    free: () => import('./screens/Free.svelte'),
    test: () => import('./screens/Test.svelte'),
    custom: () => import('./screens/Custom.svelte'),
    random: () => import('./screens/Random.svelte'),
    analysis: () => import('./screens/Analysis.svelte'),
    path: () => import('./screens/PathMap.svelte'),
    lesson: () => import('./screens/LessonPlayer.svelte'),
    intro: () => import('./screens/KeyboardIntro.svelte'),
    progress: () => import('./screens/Progress.svelte'),
    vocabulary: () => import('./screens/Vocabulary.svelte'),
  };
  $effect(() => {
    if (!user.ready || sandbox) return;
    const load = loaders[router.screen];
    let alive = true;
    loading = true;
    void load()
      .then((module) => {
        if (alive) {
          Screen = module.default;
          loading = false;
        }
      })
      .catch(() => {
        if (alive) {
          failed = true;
          loading = false;
        }
      });
    return () => {
      alive = false;
    };
  });
  async function switchMode(mode: 'learning' | 'typing') {
    const profile = user.profile;
    if (!profile || profile.preferred_mode === mode) return;
    await navigate('home');
    try {
      await api.updateProfile(profile.id, { ...profile, preferred_mode: mode });
      user.profile = { ...profile, preferred_mode: mode };
    } catch {
      router.error = 'save_failed';
    }
  }
  onMount(() => {
    document.title = t.appName;
    const isSandbox =
      import.meta.env.MODE === 'development' &&
      new URLSearchParams(location.search).has('playground');
    if (isSandbox) {
      sandbox = true;
      void import('./screens/Playground.svelte').then((module) => {
        Screen = module.default;
        loading = false;
      });
    } else {
      void initialize().catch(() => {
        failed = true;
        loading = false;
      });
    }
    requestAnimationFrame(() =>
      requestAnimationFrame(() => void getCurrentWindow().show()),
    );
  });
</script>

<Diagnostics />

<header class="topbar">
  <button
    class="brand"
    onclick={() => navigate(user.profile ? 'home' : 'profiles')}
    aria-label={t.home}
    ><svg viewBox="0 0 32 32" aria-hidden="true"
      ><rect
        x="2"
        y="6"
        width="28"
        height="20"
        rx="5"
        fill="currentColor"
      /><path
        d="M8 12h3m3 0h3m3 0h3M8 17h3m3 0h3m3 0h3M11 22h10"
        stroke="var(--surface)"
        stroke-width="2"
      /></svg
    >{t.appName}</button
  >
  {#if user.profile && !sandbox}<button
      class="profile"
      onclick={() => navigate('profiles')}
      ><Avatar id={user.profile.avatar} size={32} /><span
        >{user.profile.name}</span
      ></button
    >
    <nav aria-label={t.mode} class="row">
      <div class="modes">
        <button
          class:selected={user.profile.preferred_mode === 'learning'}
          aria-pressed={user.profile.preferred_mode === 'learning'}
          onclick={() => switchMode('learning')}>{t.learning}</button
        ><button
          class:selected={user.profile.preferred_mode === 'typing'}
          aria-pressed={user.profile.preferred_mode === 'typing'}
          onclick={() => switchMode('typing')}>{t.typing}</button
        >
      </div>
      <button onclick={() => navigate('history')}>{t.history}</button><button
        onclick={() => navigate('settings')}>{t.settings}</button
      >
    </nav>
  {:else}<label class="language"
      ><span>{t.language}</span><select
        bind:value={locale.language}
        onchange={() => {
          document.documentElement.lang = locale.language;
          void api.setSetting('language', locale.language);
        }}
        ><option value="km">{t.khmer}</option><option value="en"
          >{t.english}</option
        ></select
      ></label
    >{/if}
</header>
<main aria-label={t.appName} aria-busy={router.busy || loading}>
  {#if router.error}<p class="page error" role="alert">{t.saveFailed}</p>{/if}
  {#if failed}<div class="page stack">
      <p class="error" role="alert">{t.loadFailed}</p>
      <button onclick={() => location.reload()}>{t.retry}</button>
    </div>{:else if loading}<p class="page" role="status">
      {t.loading}
    </p>{:else if Screen}<Screen />{/if}
</main>

<style>
  .topbar {
    display: flex;
    align-items: center;
    gap: 16px;
    justify-content: space-between;
    padding: 10px 24px;
    border-bottom: 1px solid var(--line);
    background: var(--surface);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--primary);
    font-size: 22px;
    font-weight: 750;
    border: 0;
    padding: 0;
    background: transparent;
    white-space: nowrap;
  }
  .brand svg {
    width: 30px;
    height: 30px;
  }
  .profile {
    display: flex;
    align-items: center;
    gap: 8px;
    border: 0;
    padding: 0 8px;
    max-width: 170px;
  }
  .profile span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .language {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 14px;
  }
  nav {
    gap: 8px;
  }
  nav button {
    font-size: 15px;
    padding: 6px 12px;
  }
  .modes {
    display: flex;
    gap: 4px;
    background: var(--background);
    padding: 3px;
    border: 1px solid var(--line);
    border-radius: 12px;
  }
  .modes button {
    border: 0;
    min-height: 40px;
    background: transparent;
  }
  .modes .selected {
    background: var(--primary);
    color: var(--on-primary);
  }
</style>
