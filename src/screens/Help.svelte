<script lang="ts">
  import { onMount } from 'svelte';
  import { getVersion } from '@tauri-apps/api/app';
  import { messages, digits } from '../lib/i18n/index.svelte';
  import { navigate } from '../lib/router.svelte';
  let t = $derived(messages());
  let version = $state('');
  let licences = $state<Record<string, string>>({});
  let failed = $state(false);
  const fonts = ['NotoSansKhmer', 'KantumruyPro'];
  onMount(() => {
    let alive = true;
    void Promise.all([
      getVersion(),
      ...fonts.map(async (name) => {
        const response = await fetch(`/fonts/${name}-OFL.txt`);
        if (!response.ok) throw new Error('local_licence_missing');
        return response.text();
      }),
    ])
      .then(([value, ...text]) => {
        if (alive) {
          version = value;
          licences = Object.fromEntries(
            fonts.map((name, i) => [name, text[i]]),
          );
        }
      })
      .catch(() => {
        if (alive) failed = true;
      });
    return () => {
      alive = false;
    };
  });
</script>

<section class="page stack">
  <header class="row heading">
    <h1>{t.help}</h1>
    <button onclick={() => navigate('settings')}>{t.settings}</button>
  </header>
  <section class="panel stack">
    <h2>{t.metricsTitle}</h2>
    <p>{t.kpmHelp}</p>
    <p>{t.accuracyHelp}</p>
    <p>{t.activeTimeHelp}</p>
  </section>
  <section class="panel stack">
    <h2>{t.windowsKeyboard}</h2>
    <p>{t.keyboardOptional}</p>
    <p>{t.windows11Help}</p>
    <p>{t.windows10Help}</p>
    <button onclick={() => navigate('layoutCheck')}>{t.layoutCheck}</button>
  </section>
  <section class="panel stack">
    <h2>{t.dataSafety}</h2>
    <p>{t.offlineHelp}</p>
    <h3>{t.teacherPin}</h3>
    <p>{t.pinResetHelp}</p>
  </section>
  <section class="panel stack">
    <h2>{t.about} {t.appName}</h2>
    <p>{t.version} {digits(version)}</p>
    <p>{t.reviewWarning}</p>
    <h3>{t.fontLicences}</h3>
    {#each fonts as font (font)}<details>
        <summary
          >{font === 'NotoSansKhmer' ? 'Noto Sans Khmer' : 'Kantumruy Pro'} — SIL
          Open Font License 1.1 · {t.licenceDetails}</summary
        >
        <pre>{licences[font] ?? t.loading}</pre>
      </details>{/each}
  </section>
  {#if failed}<p class="error" role="alert">{t.loadFailed}</p>{/if}
</section>

<style>
  .heading {
    justify-content: space-between;
  }
  p {
    line-height: 1.9;
  }
  details {
    border-top: 1px solid var(--line);
    padding-top: 16px;
  }
  summary {
    cursor: pointer;
    min-height: 48px;
  }
  pre {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    font: inherit;
    margin-top: 16px;
  }
</style>
