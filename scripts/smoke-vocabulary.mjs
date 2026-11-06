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
  await call('Page.navigate', { url: 'http://tauri.localhost/' });
  await wait("document.querySelector('.person')");
  const profiles = await invoke('profiles_list');
  assert.ok(
    profiles.every((p) => p.name === 'M5 Learner'),
    'Use isolated smoke database',
  );
  await click("document.querySelector('.person')");
  await wait("document.querySelector('.continue')");
  await click(
    "[...document.querySelectorAll('main button')].find(b=>b.textContent==='Vocabulary')",
  );
  await wait("document.querySelectorAll('.category').length===11");
  await click("document.querySelector('.category')");
  await wait("document.querySelector('.word')");
  assert.equal(
    await evaluate("document.querySelector('.word').textContent"),
    'ឆ្មា',
  );
  assert.equal(
    await evaluate(
      "[...document.querySelectorAll('main button')].some(b=>b.textContent==='Play recording')",
    ),
    false,
  );
  await click("document.querySelector('.card .primary')");
  await wait("document.querySelector('.capture')");
  await evaluate("document.querySelector('.capture').focus()");
  for (const stroke of JSON.parse(layout.plan('ឆ្មា')))
    await evaluate(
      `document.querySelector('.capture').dispatchEvent(new KeyboardEvent('keydown',{code:${JSON.stringify(stroke.code)},bubbles:true,shiftKey:${stroke.layer === 'shift'},modifierAltGraph:${stroke.layer === 'altgr'}}))`,
    );
  await wait("document.querySelector('details')");
  const progress = await invoke('progress_get', { profile: profiles[0].id });
  assert.equal(progress.words_learned, 1);
  const history = await invoke('history_list', {
    profile: profiles[0].id,
    filter: { kind: 'vocab' },
    page: 0,
  });
  assert.ok(history.rows.some((r) => r.ref_id === 'animals_1' && r.completed));
  console.log(
    'Vocabulary: 11 categories, word card, missing-audio hiding, typing, words learned and History passed.',
  );
} finally {
  layout.free();
  web.close();
}
