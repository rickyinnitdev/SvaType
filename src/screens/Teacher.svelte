<script lang="ts">
  import { onMount } from 'svelte';
  import {
    api,
    type Profile,
    type ProfileInput,
    type ClassOverview,
    type ContentIndex,
    type Assignment,
  } from '../lib/api';
  import { user } from '../lib/stores/profile.svelte';
  import { messages, locale, digits } from '../lib/i18n/index.svelte';
  import { formatDate } from '../lib/i18n/format';
  import { navigate, registerLeaveGuard } from '../lib/router.svelte';
  import Confirm from '../lib/components/Confirm.svelte';
  import TeacherContent from './TeacherContent.svelte';
  let t = $derived(messages());
  let configured = $state(false);
  let authorized = $state(false);
  let loading = $state(true);
  let busy = $state(false);
  let feedback = $state('');
  let pin = $state('');
  let confirmation = $state('');
  let tab = $state<'class' | 'students' | 'assign' | 'content' | 'pin'>(
    'class',
  );
  let overview = $state<ClassOverview | null>(null);
  let profiles = $state<Profile[]>([]);
  let index = $state<ContentIndex | null>(null);
  let assignments = $state<Assignment[]>([]);
  let search = $state('');
  let editId = $state<number | null>(null);
  let name = $state('');
  let grade = $state<number | null>(null);
  let path = $state('beginner');
  let scope = $state('all');
  let assignmentType = $state('lesson');
  let assignmentValue = $state('sitting');
  let unlock = $state(false);
  $effect(() => {
    const selected = scope;
    if (!authorized) return;
    let alive = true;
    void api
      .teacherUnlockStatus(selected === 'all' ? null : Number(selected))
      .then((value) => {
        if (alive) unlock = value.enabled;
      })
      .catch(error);
    return () => {
      alive = false;
    };
  });
  let pending = $state<{ id: number; action: 'delete' | 'reset' } | null>(null);
  let filtered = $derived(
    profiles.filter((p) => p.name.toLowerCase().includes(search.toLowerCase())),
  );
  function error(error: unknown) {
    const code = (error as { code?: string })?.code;
    feedback =
      code === 'wrong_pin'
        ? t.wrongPin
        : code === 'pin_wait'
          ? t.pinWait
          : code === 'teacher_locked'
            ? t.teacherLocked
            : code === 'invalid_pin'
              ? t.pinHint
              : t.operationFailed;
    if (code === 'teacher_locked') {
      authorized = false;
      pending = null;
    }
  }
  async function refresh() {
    const [p, o, i, a] = await Promise.all([
      api.profiles(),
      api.classOverview(),
      api.contentIndex(),
      api.assignments(null),
    ]);
    profiles = p;
    overview = o;
    index = i;
    assignments = a;
    user.profiles = p;
    if (user.profile)
      user.profile = p.find((row) => row.id === user.profile!.id) ?? null;
  }
  onMount(() => {
    let alive = true;
    void api
      .teacherStatus()
      .then(async (status) => {
        if (!alive) return;
        configured = status.configured;
        authorized = status.authorized;
        if (authorized) await refresh();
        loading = false;
      })
      .catch((e) => {
        if (alive) {
          error(e);
          loading = false;
        }
      });
    const leave = registerLeaveGuard(async () => {
      await api.teacherLock();
      const people = await api.profiles();
      user.profiles = people;
      if (user.profile)
        user.profile = people.find((p) => p.id === user.profile!.id) ?? null;
    });
    return () => {
      alive = false;
      leave();
    };
  });
  async function run(action: () => Promise<void>, reload = true) {
    if (busy) return;
    busy = true;
    feedback = '';
    try {
      await action();
      if (reload) await refresh();
      feedback = t.saved;
    } catch (e) {
      error(e);
    } finally {
      busy = false;
    }
  }
  async function enter() {
    await run(async () => {
      const value = pin;
      pin = '';
      await api.teacherVerify(value);
      authorized = true;
    }, true);
  }
  async function setPin() {
    if (pin !== confirmation) {
      feedback = t.pinMismatch;
      return;
    }
    await run(async () => {
      const value = pin;
      pin = '';
      confirmation = '';
      await api.teacherSetPin(value);
      configured = true;
      authorized = true;
    });
  }
  function edit(profile: Profile | null) {
    editId = profile?.id ?? null;
    name = profile?.name ?? '';
    grade = profile?.grade ?? null;
    path = profile?.path_id ?? 'beginner';
  }
  async function saveStudent() {
    await run(async () => {
      const old = profiles.find((p) => p.id === editId);
      const input: ProfileInput = {
        name,
        grade,
        path_id: path,
        avatar: old?.avatar ?? '0',
        layout_id: old?.layout_id ?? 'nida',
        preferred_mode: old?.preferred_mode ?? 'learning',
      };
      if (editId) await api.updateProfile(editId, input);
      else await api.createProfile(input);
      edit(null);
    });
  }
  async function confirm() {
    if (!pending) return;
    const request = pending;
    await run(async () => {
      if (request.action === 'reset') await api.resetProgress(request.id);
      else {
        await api.deleteProfile(request.id);
        if (user.profile?.id === request.id) {
          user.profile = null;
          pending = null;
          await navigate('profiles');
          return;
        }
      }
      pending = null;
      edit(null);
    });
  }
  async function exportClass() {
    await run(async () => {
      const path = await api.saveDialog('SvaType-class.csv', 'csv');
      if (path) await api.exportCsv(path, null);
    }, false);
  }
</script>

<section class="page stack">
  <header class="row heading">
    <div>
      <h1>{t.teacher}</h1>
      <p class="muted">{t.teacherSummary}</p>
    </div>
    {#if authorized}<button
        onclick={() =>
          run(async () => {
            await api.teacherLock();
            authorized = false;
          }, false)}>{t.lockTeacher}</button
      >{/if}
  </header>
  {#if feedback}<p role="status">{feedback}</p>{/if}
  {#if loading}<p>{t.loading}</p>{:else if !authorized}<form
      class="panel gate stack"
      onsubmit={(e) => {
        e.preventDefault();
        void enter();
      }}
    >
      <h2>{t.teacherPin}</h2>
      <p>{configured ? t.pinHint : t.noPinHint}</p>
      {#if configured}<label
          >{t.teacherPin}<input
            type="password"
            inputmode="numeric"
            autocomplete="off"
            bind:value={pin}
            pattern={'[0-9]{4,6}'}
            minlength="4"
            maxlength="6"
            required
          /></label
        >{/if}<button class="primary" type="submit" disabled={busy}
        >{t.unlockTeacher}</button
      >
      <p class="muted">{t.pinResetHelp}</p>
    </form>
  {:else}<nav class="row" aria-label={t.teacher}>
      {#each [['class', 'classOverview'], ['students', 'students'], ['assign', 'assignments'], ['content', 'contentEditor'], ['pin', 'teacherSecurity']] as [value, label] (value)}<button
          class:selected={tab === value}
          aria-pressed={tab === value}
          onclick={() => (tab = value as typeof tab)}
          >{t[label as keyof typeof t]}</button
        >{/each}
    </nav>
    {#if tab === 'class'}<div class="row heading">
        <h2>{t.classOverview} · {digits(overview?.students.length ?? 0)}</h2>
        <button disabled={busy} onclick={exportClass}>{t.classExport}</button>
      </div>
      <div class="table-wrap">
        <table>
          <thead
            ><tr
              ><th>{t.name}</th><th>{t.lessonsDone}</th><th>{t.accuracy}</th><th
                >{t.weekPractice}</th
              ><th>{t.needsHelp}</th></tr
            ></thead
          ><tbody
            >{#each overview?.students ?? [] as student (student.id)}<tr
                ><td>{student.name}</td><td>{digits(student.lessons)}</td><td
                  >{digits(student.accuracy.toFixed(1))}%</td
                ><td
                  >{digits(Math.round(student.week_ms / 60000))} {t.minutes}</td
                ><td
                  >{student.needs_help ? t.needsHelp : '✓'}<small class="muted"
                    >{student.last_at
                      ? formatDate(student.last_at, locale.language)
                      : t.noActivity}</small
                  ></td
                ></tr
              >{/each}</tbody
          >
        </table>
      </div>
      <section class="panel stack">
        <h2>{t.weakKeys}</h2>
        {#each overview?.keys ?? [] as key (key.code + key.layer)}<p>
            {key.code} · {key.layer}: {digits(key.errors)} / {digits(key.hits)}
          </p>{/each}{#each overview?.confusions ?? [] as pair (pair.expected + pair.typed)}<p
          >
            {pair.expected} → {pair.typed}: {digits(pair.count)}
          </p>{/each}
      </section>
    {:else if tab === 'students'}<div class="row heading">
        <h2>{t.students}</h2>
        <label>{t.search}<input type="search" bind:value={search} /></label>
      </div>
      <form
        class="panel row student-form"
        onsubmit={(e) => {
          e.preventDefault();
          void saveStudent();
        }}
      >
        <label
          >{t.name}<input required maxlength="80" bind:value={name} /></label
        ><label
          >{t.grade}<select bind:value={grade}
            ><option value={null}>{t.none}</option
            >{#each [3, 4, 5] as value (value)}<option {value}
                >{digits(value)}</option
              >{/each}</select
          ></label
        ><label
          >{t.learningPath}<select bind:value={path}
            >{#each index?.paths ?? [] as item (item.id)}<option value={item.id}
                >{item.title[locale.language]}</option
              >{/each}</select
          ></label
        ><button class="primary" disabled={busy} type="submit"
          >{editId ? t.teacherSave : t.addPerson}</button
        ><button type="button" onclick={() => edit(null)}>{t.teacherNew}</button
        >
      </form>
      <div class="stack">
        {#each filtered as profile (profile.id)}<div class="panel row student">
            <strong>{profile.name}</strong>
            <div class="row">
              <button onclick={() => edit(profile)}>{t.teacherEdit}</button
              ><button
                disabled={busy}
                onclick={() => (pending = { id: profile.id, action: 'reset' })}
                >{t.resetProgress}</button
              ><button
                disabled={busy}
                onclick={() => (pending = { id: profile.id, action: 'delete' })}
                >{t.deletePerson}</button
              >
            </div>
          </div>{/each}
      </div>
    {:else if tab === 'assign'}<h2>{t.assignments}</h2>
      <div class="panel stack">
        <label
          >{t.assignTo}<select bind:value={scope}
            ><option value="all">{t.everyone}</option
            >{#each profiles as profile (profile.id)}<option
                value={String(profile.id)}>{profile.name}</option
              >{/each}</select
          ></label
        >
        <div class="row">
          <label
            >{t.teacherContentKind}<select
              bind:value={assignmentType}
              onchange={(event) =>
                (assignmentValue =
                  event.currentTarget.value === 'lesson'
                    ? (index?.lessons[0]?.id ?? '')
                    : (index?.paths[0]?.id ?? ''))}
              ><option value="lesson">{t.assignLesson}</option><option
                value="path">{t.assignPath}</option
              ></select
            ></label
          ><label
            >{t.target}<select bind:value={assignmentValue}
              >{#each (assignmentType === 'lesson' ? index?.lessons : index?.paths) ?? [] as item (item.id)}<option
                  value={item.id}>{item.title[locale.language]}</option
                >{/each}</select
            ></label
          ><button
            class="primary"
            disabled={busy || !assignmentValue}
            onclick={() =>
              run(() =>
                api.assign(
                  scope === 'all' ? null : Number(scope),
                  assignmentType === 'lesson' ? assignmentValue : null,
                  assignmentType === 'path' ? assignmentValue : null,
                ),
              )}>{t.teacherSave}</button
          >
        </div>
        <label class="row"
          ><input
            type="checkbox"
            bind:checked={unlock}
            disabled={busy}
            onchange={(event) => {
              const enabled = event.currentTarget.checked;
              unlock = enabled;
              void run(() =>
                api.unlockAll(scope === 'all' ? null : Number(scope), enabled),
              );
            }}
          />{t.unlockAll}</label
        >
      </div>
      {#each assignments as item (item.id)}<div class="panel row student">
          <p>
            {item.profile_id === null
              ? t.everyone
              : profiles.find((p) => p.id === item.profile_id)?.name} · {item.lesson_id
              ? index?.lessons.find((l) => l.id === item.lesson_id)?.title[
                  locale.language
                ]
              : index?.paths.find((p) => p.id === item.path_id)?.title[
                  locale.language
                ]}
          </p>
          <button
            disabled={busy}
            onclick={() => run(() => api.deleteAssignment(item.id))}
            >{t.delete}</button
          >
        </div>{/each}
    {:else if tab === 'content'}<TeacherContent onerror={error} />
    {:else}<div class="panel gate stack">
        <h2>{configured ? t.changePin : t.setPin}</h2>
        <p>{t.pinHint}</p>
        <form
          class="stack"
          onsubmit={(e) => {
            e.preventDefault();
            void setPin();
          }}
        >
          <label
            >{t.teacherPin}<input
              type="password"
              bind:value={pin}
              inputmode="numeric"
              autocomplete="new-password"
              required
              pattern={'[0-9]{4,6}'}
              minlength="4"
              maxlength="6"
            /></label
          ><label
            >{t.confirmPin}<input
              type="password"
              bind:value={confirmation}
              inputmode="numeric"
              autocomplete="new-password"
              required
              pattern={'[0-9]{4,6}'}
              minlength="4"
              maxlength="6"
            /></label
          ><button type="submit" class="primary" disabled={busy}
            >{configured ? t.changePin : t.setPin}</button
          >
        </form>
        {#if configured}<button
            disabled={busy}
            onclick={() =>
              run(async () => {
                await api.teacherClearPin();
                configured = false;
              }, false)}>{t.removePin}</button
          >{/if}
        <p class="muted">{t.pinResetHelp}</p>
      </div>{/if}
  {/if}
</section>
<Confirm
  open={pending !== null}
  title={pending?.action === 'reset' ? t.resetProgress : t.deletePerson}
  description={pending?.action === 'reset'
    ? t.resetProgressHint
    : t.deletePersonHint}
  confirmLabel={pending?.action === 'reset' ? t.resetProgress : t.delete}
  {busy}
  onconfirm={() => void confirm()}
  oncancel={() => (pending = null)}
/>

<style>
  .heading,
  .student {
    justify-content: space-between;
  }
  .gate {
    max-width: 650px;
    margin: auto;
    width: 100%;
  }
  label {
    display: grid;
    gap: 8px;
  }
  label.row {
    display: flex;
  }
  input,
  select {
    min-height: 48px;
    padding: 8px;
  }
  .student-form {
    align-items: end;
  }
  .student-form label:first-child {
    flex: 1;
  }
  nav .selected {
    background: var(--primary);
    color: var(--on-primary);
  }
  small {
    display: block;
  }
  .table-wrap {
    overflow: auto;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    background: var(--surface);
  }
  th,
  td {
    padding: 14px;
    text-align: left;
    border-bottom: 1px solid var(--line);
  }
  .student strong {
    flex: 1;
  }
</style>
