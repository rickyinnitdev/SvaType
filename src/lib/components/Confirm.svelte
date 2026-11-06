<script lang="ts">
  import { messages } from '../i18n/index.svelte';
  let {
    open,
    title,
    description,
    confirmLabel,
    onconfirm,
    oncancel,
    busy = false,
  }: {
    open: boolean;
    title: string;
    description: string;
    confirmLabel: string;
    onconfirm: () => void;
    oncancel: () => void;
    busy?: boolean;
  } = $props();
  const titleId = $props.id();
  let dialog = $state<HTMLDialogElement>();
  let t = $derived(messages());
  $effect(() => {
    if (!dialog) return;
    if (open && !dialog.open) dialog.showModal();
    else if (!open && dialog.open) dialog.close();
  });
</script>

<dialog
  bind:this={dialog}
  oncancel={(e) => {
    e.preventDefault();
    if (!busy) oncancel();
  }}
  aria-labelledby={titleId}
>
  <div class="stack">
    <h2 id={titleId}>{title}</h2>
    <p class="muted">{description}</p>
    <div class="row actions">
      <button onclick={oncancel} disabled={busy}>{t.cancel}</button><button
        class="primary"
        onclick={onconfirm}
        disabled={busy}>{busy ? t.saving : confirmLabel}</button
      >
    </div>
  </div>
</dialog>

<style>
  dialog {
    max-width: 560px;
    width: calc(100% - 48px);
    border: 1px solid var(--line);
    border-radius: 16px;
    padding: 28px;
    background: var(--surface);
    color: var(--foreground);
  }
  dialog::backdrop {
    background: rgba(0, 20, 16, 0.45);
  }
  .actions {
    justify-content: flex-end;
  }
</style>
