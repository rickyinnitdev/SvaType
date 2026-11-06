import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { initSync, Layout } from '../src/lib/wasm/khmer.js';
import { connectWebView } from './webview-test.mjs';
initSync({ module: readFileSync('src/lib/wasm/khmer_bg.wasm') });
const layout = Layout.from_json(
  readFileSync('content/layouts/nida.json', 'utf8'),
);
const web = await connectWebView();
const { evaluate, wait, call } = web;
const invoke = (name, args = {}) =>
  evaluate(
    `window.__TAURI_INTERNALS__.invoke(${JSON.stringify(name)},${JSON.stringify(args)})`,
  );
const click = (expr) => evaluate(`(${expr}).click()`);
try {
  let profiles = await invoke('profiles_list');
  assert.ok(
    profiles.every((p) => p.name === 'M5 Learner'),
    'Use isolated M5 smoke data',
  );
  let profile =
    profiles[0] ??
    (await invoke('profiles_create', {
      input: {
        name: 'M5 Learner',
        avatar: '0',
        grade: 3,
        path_id: 'grade3',
        layout_id: 'nida',
        preferred_mode: 'learning',
      },
    }));
  await invoke('settings_set', { key: 'language', value: 'en' });
  await call('Page.navigate', { url: 'http://tauri.localhost/' });
  await wait("document.querySelector('.person')");
  await click("document.querySelector('.person')");
  await wait("document.querySelector('.continue .primary')");
  const path = JSON.parse(readFileSync('content/paths/grade3.json', 'utf8'));
  for (const id of path.units.flatMap((u) => u.lessons)) {
    const existing = await invoke('progress_get', { profile: profile.id });
    if (existing.lessons.some((l) => l.lesson_id === id && l.best_stars > 0))
      continue;
    const draft = await invoke('lesson_draft_get', {
      profile: profile.id,
      lesson: id,
    });
    await click("document.querySelector('.continue .primary')");
    await wait("document.querySelector('.lesson h1')");
    const lesson = JSON.parse(
      readFileSync(`content/lessons/${id}.json`, 'utf8'),
    );
    for (let i = draft?.step ?? 0; i < lesson.steps.length; i++) {
      const step = lesson.steps[i];
      if (['explain', 'demo'].includes(step.type)) {
        await click("document.querySelector('.lesson .primary')");
      } else {
        await wait("document.querySelector('.capture')");
        await evaluate("document.querySelector('.capture').focus()");
        const text =
          typeof step.text === 'string'
            ? step.text
            : step.items.join(step.type === 'find_key' ? '' : ' ');
        for (const stroke of JSON.parse(layout.plan(text)))
          await evaluate(
            `document.querySelector('.capture').dispatchEvent(new KeyboardEvent('keydown',{code:${JSON.stringify(stroke.code)},bubbles:true,shiftKey:${stroke.layer === 'shift' || stroke.layer === 'shift_altgr'},ctrlKey:${stroke.layer === 'altgr' || stroke.layer === 'shift_altgr'},altKey:${stroke.layer === 'altgr' || stroke.layer === 'shift_altgr'},modifierAltGraph:${stroke.layer === 'altgr' || stroke.layer === 'shift_altgr'}}))`,
          );
      }
      if (i < lesson.steps.length - 1)
        await wait(
          `document.querySelector('.lesson header .muted')?.textContent.trim().replace(/\\s+/g,' ')==='Step ${i + 2} / ${lesson.steps.length}'`,
        );
    }
    await wait("document.querySelector('details')");
    const progress = await invoke('progress_get', { profile: profile.id });
    assert.equal(
      progress.lessons.find((l) => l.lesson_id === id).best_stars,
      3,
    );
    await click("document.querySelector('.brand')");
    await wait("document.querySelector('.continue .primary')");
    console.log('Completed', id);
  }
  const history = await invoke('history_list', {
    profile: profile.id,
    filter: {},
    page: 0,
  });
  assert.equal(history.total, 20);
  assert.ok(history.rows.every((r) => r.kind === 'lesson' && r.completed));
  console.log('Grade 3: all 20 lessons completed; stars and History verified.');
} finally {
  layout.free();
  web.close();
}
