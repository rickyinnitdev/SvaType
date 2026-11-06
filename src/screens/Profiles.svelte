<script lang="ts">
  import { onMount } from 'svelte';
  import Avatar from '../lib/components/Avatar.svelte';
  import { user, selectProfile } from '../lib/stores/profile.svelte';
  import { navigate } from '../lib/router.svelte';
  import { messages } from '../lib/i18n/index.svelte';
  let t = $derived(messages());
  let search = $state('');
  let busy = $state(false);
  let failed = $state(false);
  let visible = $derived(
    user.profiles.filter((p) =>
      p.name.toLowerCase().includes(search.toLowerCase()),
    ),
  );
  async function choose(id: number) {
    if (busy) return;
    const profile = user.profiles.find((p) => p.id === id);
    if (!profile) return;
    busy = true;
    failed = false;
    try {
      await selectProfile(profile);
    } catch {
      failed = true;
    } finally {
      busy = false;
    }
  }
  onMount(() => {
    const enter = (e: KeyboardEvent) => {
      if (
        e.code === 'Enter' &&
        document.activeElement === document.body &&
        visible[0]
      )
        void choose(visible[0].id);
    };
    window.addEventListener('keydown', enter);
    return () => window.removeEventListener('keydown', enter);
  });
</script>

<section class="page stack">
  <div>
    <p class="eyebrow">{t.welcome}</p>
    <h1>{t.choosePerson}</h1>
  </div>
  {#if user.profiles.length > 24}<label
      >{t.search}<input type="search" bind:value={search} /></label
    >{/if}
  {#if failed}<p role="alert" class="error">{t.loadFailed}</p>{/if}
  <div class="people">
    {#each visible as profile, i (profile.id)}<button
        class="person"
        class:last-used={i === 0}
        onclick={() => choose(profile.id)}
        disabled={busy}
        ><Avatar id={profile.avatar} size={84} /><strong>{profile.name}</strong
        ></button
      >{/each}
    <button
      class="person add"
      onclick={() => navigate('onboarding')}
      disabled={busy}
      ><span aria-hidden="true">＋</span><strong>{t.addPerson}</strong></button
    >
  </div>
</section>

<style>
  .people {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));
    gap: 20px;
  }
  .person {
    display: flex;
    flex-direction: column;
    gap: 12px;
    align-items: center;
    justify-content: center;
    min-height: 190px;
    padding: 24px;
    overflow-wrap: anywhere;
  }
  .last-used {
    border: 2px solid var(--primary);
    background: var(--surface-soft);
  }
  .add {
    border-style: dashed;
  }
  .add span {
    font-size: 46px;
    font-weight: 400;
    color: var(--primary);
    line-height: 84px;
  }
  .eyebrow {
    color: var(--primary);
  }
</style>
