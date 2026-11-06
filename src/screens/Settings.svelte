<script lang="ts">
  import { api } from '../lib/api';
  import { user, initialize } from '../lib/stores/profile.svelte';
  import {
    preferences,
    savePreference,
    type Preferences,
  } from '../lib/stores/settings.svelte';
  import { messages, locale } from '../lib/i18n/index.svelte';
  import { navigate } from '../lib/router.svelte';
  import Confirm from '../lib/components/Confirm.svelte';
  import nida from '../../content/layouts/nida.json';
  import legacy from '../../content/layouts/khmer_legacy.json';
  let t = $derived(messages());
  let busy = $state(false);
  let feedback = $state('');
  let failed = $state(false);
  let restorePath = $state<string | null>(null);
  const toggles = [
    ['show_keyboard', 'showKeyboard'],
    ['show_hands', 'showHands'],
    ['sound', 'sound'],
    ['low_end', 'lowEnd'],
    ['reduce_motion', 'reduceMotion'],
  ] as const;
  async function change<K extends keyof Preferences>(
    key: K,
    value: Preferences[K],
  ) {
    if (!user.profile) return;
    failed = false;
    feedback = '';
    try {
      await savePreference(user.profile.id, key, value);
    } catch {
      failed = true;
    }
  }
  async function language(value: string) {
    if (value !== 'km' && value !== 'en') return;
    try {
      await api.setSetting('language', value);
      locale.language = value;
      document.documentElement.lang = value;
    } catch {
      failed = true;
    }
  }
  async function layout(value: string) {
    if (!user.profile || (value !== 'nida' && value !== 'khmer_legacy')) return;
    try {
      const profile = {
        ...user.profile,
        layout_id: value as 'nida' | 'khmer_legacy',
      };
      await api.updateProfile(profile.id, profile);
      user.profile = { ...profile, layout_id: value };
    } catch {
      failed = true;
    }
  }
  async function run(operation: () => Promise<void>) {
    if (busy) return;
    busy = true;
    failed = false;
    feedback = '';
    try {
      await operation();
      feedback = t.saved;
    } catch (error) {
      failed = true;
      feedback =
        (error as { code?: string }).code === 'file_already_exists'
          ? t.chooseNewFilename
          : t.operationFailed;
    } finally {
      busy = false;
    }
  }
  function backup() {
    void run(async () => {
      const path = await api.saveDialog(
        `SvaType-${Date.now()}.svabak`,
        'svabak',
      );
      if (path) await api.backup(path);
    });
  }
  function exportCsv() {
    void run(async () => {
      const path = await api.saveDialog(`SvaType-${Date.now()}.csv`, 'csv');
      if (path) await api.exportCsv(path, user.profile?.id ?? null);
    });
  }
  async function chooseRestore() {
    try {
      restorePath = await api.openDialog('svabak');
    } catch {
      failed = true;
    }
  }
  async function restore() {
    const path = restorePath;
    if (!path) return;
    await run(async () => {
      await api.restore(path);
      restorePath = null;
      user.profile = null;
      await initialize();
    });
  }
</script>

<section class="page stack">
  <header class="row heading">
    <h1>{t.settings}</h1>
    <button onclick={() => navigate('home')}>{t.home}</button>
  </header>
  <div class="panel settings-grid">
    <label
      >{t.language}<select
        value={locale.language}
        onchange={(e) => language(e.currentTarget.value)}
        ><option value="km">{t.khmer}</option><option value="en"
          >{t.english}</option
        ></select
      ></label
    ><label
      >{t.layout}<select
        value={user.profile?.layout_id}
        onchange={(e) => layout(e.currentTarget.value)}
        ><option value="nida">{nida.name[locale.language]}</option><option
          value="khmer_legacy">{legacy.name[locale.language]}</option
        ></select
      ></label
    >
    <label
      >{t.fontSize}<select
        value={preferences.font_size}
        onchange={(e) =>
          change(
            'font_size',
            e.currentTarget.value as Preferences['font_size'],
          )}
        ><option value="s">{t.small}</option><option value="m"
          >{t.medium}</option
        ><option value="l">{t.large}</option><option value="xl"
          >{t.extraLarge}</option
        ></select
      ></label
    ><label
      >{t.theme}<select
        value={preferences.theme}
        onchange={(e) =>
          change('theme', e.currentTarget.value as Preferences['theme'])}
        ><option value="light">{t.light}</option><option value="dark"
          >{t.dark}</option
        ><option value="contrast">{t.highContrast}</option></select
      ></label
    >
    {#each toggles as [key, label] (key)}<label class="toggle"
        ><span>{t[label]}</span><input
          type="checkbox"
          checked={preferences[key]}
          onchange={(e) => change(key, e.currentTarget.checked)}
        /></label
      >{/each}
  </div>
  <div class="panel stack">
    <h2>{t.dataSafety}</h2>
    <p class="muted">{t.backupHint}</p>
    <div class="row">
      <button onclick={backup} disabled={busy}>{t.backup}</button><button
        onclick={chooseRestore}
        disabled={busy}>{t.restore}</button
      ><button onclick={exportCsv} disabled={busy}>{t.exportCsv}</button><button
        onclick={() => run(() => api.openDataFolder())}
        disabled={busy}>{t.openDataFolder}</button
      >
    </div>
  </div>
  <button onclick={() => navigate('teacher')}>{t.teacher}</button>
  {#if feedback || failed}<p
      class:error={failed}
      role={failed ? 'alert' : 'status'}
    >
      {feedback || (failed ? t.saveFailed : t.saved)}
    </p>{/if}
</section>
<Confirm
  open={restorePath !== null}
  title={t.restoreConfirm}
  description={t.restoreHint}
  confirmLabel={t.restore}
  onconfirm={() => void restore()}
  oncancel={() => (restorePath = null)}
  {busy}
/>

<style>
  .heading {
    justify-content: space-between;
  }
  .settings-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 24px 40px;
  }
  label {
    display: grid;
    gap: 8px;
  }
  .toggle {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 48px;
    border-bottom: 1px solid var(--line);
  }
  .toggle input {
    width: 24px;
    height: 24px;
    accent-color: var(--primary);
  }
</style>
