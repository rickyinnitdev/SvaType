<script lang="ts">
  import { registerLeaveGuard } from '../lib/router.svelte';
  import { onMount } from 'svelte';
  import {
    api,
    type ContentOverride,
    type VocabWord,
    type VocabCategory,
  } from '../lib/api';
  import { messages, locale, digits } from '../lib/i18n/index.svelte';
  import Confirm from '../lib/components/Confirm.svelte';
  let { onerror }: { onerror: (error: unknown) => void } = $props();
  let t = $derived(messages());
  let categories = $state<Omit<VocabCategory, 'words'>[]>([]);
  let words = $state<VocabWord[]>([]);
  let overrides = $state<ContentOverride[]>([]);
  let kind = $state('word');
  let category = $state('animals');
  let level = $state(1);
  let existing = $state('');
  let rowId = $state<number | null>(null);
  let valueId = $state('');
  let word = $state('');
  let meaning = $state('');
  let english = $state('');
  let example = $state('');
  let emoji = $state('📖');
  let reviewed = $state(false);
  let busy = $state(false);
  let pendingOperation: Promise<void> | null = null;
  let failed = $state(false);
  let saved = $state(false);
  let pendingDelete = $state<number | null>(null);
  async function reload() {
    try {
      await refresh();
    } catch (error) {
      failed = true;
      onerror(error);
    }
  }
  async function refresh() {
    overrides = await api.overrides();
    if (kind === 'word') words = (await api.vocab(category)).words;
  }
  onMount(() => {
    const leave = registerLeaveGuard(async () => {
      if (pendingOperation) {
        await pendingOperation;
        if (failed) throw new Error('save_failed');
      }
    });
    void (async () => {
      try {
        categories = (await api.vocabIndex()).categories;
        await refresh();
      } catch (error) {
        failed = true;
        onerror(error);
      }
    })();
    return leave;
  });
  function clear() {
    rowId = null;
    valueId = '';
    existing = '';
    word = '';
    meaning = '';
    english = '';
    example = '';
    emoji = '📖';
    reviewed = false;
    saved = false;
  }
  function loadWord() {
    const value = words.find((w) => w.id === existing);
    if (!value) {
      clear();
      return;
    }
    rowId =
      overrides.find(
        (row) =>
          row.kind === 'word' &&
          row.category === category &&
          row.value.id === value.id,
      )?.id ?? null;
    valueId = value.id;
    word = value.word;
    meaning = value.meaning_km;
    english = value.meaning_en;
    example = value.example_km;
    emoji = value.emoji;
    level = value.level;
    reviewed = value.reviewed;
    saved = false;
  }
  async function edit(row: ContentOverride) {
    kind = row.kind;
    rowId = row.id;
    valueId = String(row.value.id ?? '');
    if (kind === 'word') {
      category = row.category;
      words = (await api.vocab(category)).words;
      existing = valueId;
      word = String(row.value.word ?? '');
      meaning = String(row.value.meaning_km ?? '');
      english = String(row.value.meaning_en ?? '');
      example = String(row.value.example_km ?? '');
      emoji = String(row.value.emoji ?? '📖');
      level = Number(row.value.level ?? 1);
    } else {
      level = Number(row.category);
      word = String(row.value.text ?? '');
    }
    reviewed = row.value.reviewed === true;
    saved = false;
  }
  function run(operation: () => Promise<void>): Promise<void> {
    if (pendingOperation) return pendingOperation;
    busy = true;
    failed = false;
    saved = false;
    pendingOperation = (async () => {
      try {
        await operation();
        await refresh();
        saved = true;
      } catch (error) {
        failed = true;
        onerror(error);
      } finally {
        busy = false;
      }
    })().finally(() => {
      pendingOperation = null;
    });
    return pendingOperation;
  }
  async function save() {
    await run(async () => {
      const value: Record<string, unknown> =
        kind === 'word'
          ? {
              id: valueId,
              word,
              meaning_km: meaning,
              meaning_en: english,
              example_km: example,
              emoji,
              level,
              reviewed,
            }
          : { id: valueId, text: word, level, reviewed };
      rowId = await api.saveOverride(
        rowId,
        kind,
        kind === 'word' ? category : String(level),
        value,
      );
      const list = await api.overrides();
      valueId = String(list.find((row) => row.id === rowId)?.value.id ?? '');
    });
  }
</script>

<div class="stack">
  <h2>{t.contentEditor}</h2>
  <div class="row">
    <label
      >{t.teacherContentKind}<select
        bind:value={kind}
        onchange={(event) => {
          kind = event.currentTarget.value;
          clear();
          void reload();
        }}
        ><option value="word">{t.newWord}</option><option value="sentence"
          >{t.newSentence}</option
        ></select
      ></label
    >{#if kind === 'word'}<label
        >{t.teacherCategory}<select
          bind:value={category}
          onchange={(event) => {
            category = event.currentTarget.value;
            clear();
            void reload();
          }}
          >{#each categories as item (item.id)}<option value={item.id}
              >{item.title[locale.language]}</option
            >{/each}</select
        ></label
      >{/if}<label
      >{t.teacherLevel}<select bind:value={level}
        >{#each [1, 2, 3, 4, 5] as value (value)}<option {value}
            >{digits(value)}</option
          >{/each}</select
      ></label
    >
  </div>
  {#if kind === 'word'}<label
      >{t.teacherExistingWord}<select
        bind:value={existing}
        onchange={(event) => {
          existing = event.currentTarget.value;
          loadWord();
        }}
        ><option value="">{t.teacherNew}</option
        >{#each words as item (item.id)}<option value={item.id}
            >{item.word}</option
          >{/each}</select
      ></label
    >{/if}
  <form
    class="panel stack"
    onsubmit={(event) => {
      event.preventDefault();
      void save();
    }}
  >
    <label
      >{kind === 'word' ? t.teacherWord : t.newSentence}<textarea
        bind:value={word}
        required
        maxlength="2000"
        rows="2"
      ></textarea></label
    >{#if kind === 'word'}<div class="grid">
        <label
          >{t.meaning}<textarea bind:value={meaning} required rows="2"
          ></textarea></label
        ><label>{t.meaningEnglish}<input bind:value={english} /></label><label
          >{t.example}<textarea bind:value={example} required rows="2"
          ></textarea></label
        ><label
          >{t.teacherEmoji}<input bind:value={emoji} maxlength="20" /></label
        >
      </div>{/if}<label class="row"
      ><input type="checkbox" bind:checked={reviewed} />{t.reviewed}</label
    >
    <div class="row">
      <button type="submit" class="primary" disabled={busy}
        >{t.teacherSave}</button
      ><button type="button" onclick={clear}>{t.teacherNew}</button>
    </div>
  </form>
  {#if failed}<p class="error" role="alert">
      {t.teacherContentError}
    </p>{:else if saved}<p role="status">{t.saved}</p>{/if}
  <div class="row">
    <h3>{t.overrides}</h3>
    <button
      disabled={busy}
      onclick={() =>
        run(async () => {
          const path = await api.saveDialog('SvaType-content.json', 'json');
          if (path) await api.exportOverrides(path);
        })}>{t.exportContent}</button
    >
  </div>
  {#each overrides as row (row.id)}<div class="panel row override">
      <p>{String(row.value.word ?? row.value.text ?? '')}</p>
      <div class="row">
        <button onclick={() => void run(() => edit(row))}
          >{t.teacherEdit}</button
        ><button disabled={busy} onclick={() => (pendingDelete = row.id)}
          >{t.delete}</button
        >
      </div>
    </div>{/each}
</div>
<Confirm
  open={pendingDelete !== null}
  title={t.delete}
  description={t.deleteHint}
  confirmLabel={t.delete}
  {busy}
  oncancel={() => (pendingDelete = null)}
  onconfirm={() =>
    run(async () => {
      await api.deleteOverride(pendingDelete!);
      pendingDelete = null;
      clear();
    })}
/>

<style>
  label {
    display: grid;
    gap: 8px;
  }
  label.row {
    display: flex;
  }
  input,
  select,
  textarea {
    min-height: 48px;
    padding: 8px;
  }
  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }
  .override {
    justify-content: space-between;
  }
  .override p {
    flex: 1;
    overflow-wrap: anywhere;
  }
</style>
