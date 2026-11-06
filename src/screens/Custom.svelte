<script lang="ts">
  import { api, type CustomText } from '../lib/api';
  import { user } from '../lib/stores/profile.svelte';
  import { configureActivity } from '../lib/stores/activity.svelte';
  import { navigate } from '../lib/router.svelte';
  import { messages, digits } from '../lib/i18n/index.svelte';
  import Confirm from '../lib/components/Confirm.svelte';
  let t = $derived(messages());
  let title = $state('');
  let body = $state('');
  let texts = $state<CustomText[]>([]);
  let busy = $state(false);
  let failed = $state('');
  let deleteId = $state<number | null>(null);
  $effect(() => {
    const id = user.profile?.id;
    if (!id) return;
    let alive = true;
    void api
      .customTexts(id)
      .then((value) => {
        if (alive) texts = value;
      })
      .catch(() => {
        if (alive) failed = t.loadFailed;
      });
    return () => {
      alive = false;
    };
  });
  async function save(start: boolean) {
    if (!user.profile || busy) return;
    busy = true;
    failed = '';
    try {
      const id = await api.saveCustomText(
        user.profile.id,
        title,
        body,
        user.profile.layout_id,
      );
      texts = await api.customTexts(user.profile.id);
      if (start) {
        const text = texts.find((value) => value.id === id)!;
        practice(text);
      } else {
        title = '';
        body = '';
      }
    } catch (error) {
      failed =
        (error as { code?: string }).code === 'unmapped_content'
          ? t.unmappedText
          : t.saveFailed;
    } finally {
      busy = false;
    }
  }
  function practice(text: CustomText) {
    configureActivity(text.body, 'custom', 0, String(text.id));
    void navigate('practice');
  }
  async function remove() {
    if (!user.profile || deleteId === null) return;
    busy = true;
    try {
      await api.deleteCustomText(user.profile.id, deleteId);
      deleteId = null;
      texts = await api.customTexts(user.profile.id);
    } catch {
      failed = t.operationFailed;
    } finally {
      busy = false;
    }
  }
</script>

<section class="page stack">
  <h1>{t.custom}</h1>
  <p class="muted">{t.customDescription}</p>
  <div class="panel stack">
    <label>{t.textTitle}<input bind:value={title} maxlength="100" /></label
    ><label
      >{t.pasteText}<textarea
        rows="6"
        bind:value={body}
        maxlength="20000"
        aria-describedby="text-limit"
      ></textarea></label
    >
    <p id="text-limit" class="muted">
      {t.textLimit} · {digits(Array.from(body).length)} / {digits(20_000)}
    </p>
    <div class="row">
      <button
        onclick={() => save(false)}
        disabled={busy || !title.trim() || !body.trim()}>{t.saveText}</button
      ><button
        class="primary"
        onclick={() => save(true)}
        disabled={busy || !title.trim() || !body.trim()}>{t.start}</button
      >
    </div>
    {#if failed}<p class="error" role="alert">{failed}</p>{/if}
  </div>
  <h2>{t.myTexts}</h2>
  {#each texts as text (text.id)}<article class="panel row saved-text">
      <div>
        <h2>{text.title}</h2>
        <p class="muted">{text.body.slice(0, 100)}</p>
      </div>
      <div class="row">
        <button class="primary" onclick={() => practice(text)}>{t.start}</button
        ><button onclick={() => (deleteId = text.id)}>{t.delete}</button>
      </div>
    </article>{/each}
</section>
<Confirm
  open={deleteId !== null}
  title={t.confirmTextDelete}
  description={t.deleteHint}
  confirmLabel={t.delete}
  onconfirm={() => void remove()}
  oncancel={() => (deleteId = null)}
  {busy}
/>

<style>
  label {
    display: grid;
    gap: 8px;
  }
  textarea {
    font-size: 24px;
    line-height: 1.9;
    resize: vertical;
  }
  .saved-text {
    justify-content: space-between;
  }
  .saved-text > div:first-child {
    max-width: 65%;
    overflow-wrap: anywhere;
  }
</style>
