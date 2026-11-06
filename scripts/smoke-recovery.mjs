import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { initSync, Layout } from '../src/lib/wasm/khmer.js';
import { connectWebView } from './webview-test.mjs';
initSync({ module: readFileSync('src/lib/wasm/khmer_bg.wasm') });
const l = Layout.from_json(readFileSync('content/layouts/nida.json', 'utf8'));
const strokes = JSON.parse(
  l.plan(JSON.parse(readFileSync('content/playground.json', 'utf8')).text),
);
l.free();
const web = await connectWebView();
const { evaluate, wait } = web;
const invoke = (name, args = {}) =>
  evaluate(
    `window.__TAURI_INTERNALS__.invoke(${JSON.stringify(name)},${JSON.stringify(args)})`,
  );
try {
  await wait("document.querySelector('.people')");
  const profiles = await invoke('profiles_list');
  const profile = profiles.find((p) => p.name === 'M3 Smoke');
  assert.ok(profile);
  const checkpoints = await invoke('checkpoints_list', { profile: profile.id });
  assert.equal(checkpoints.length, 1);
  assert.equal(checkpoints[0].session.summary.correct, 3);
  await evaluate(
    "[...document.querySelectorAll('.person')].find(b=>b.textContent.trim()==='M3 Smoke').click()",
  );
  await wait("document.querySelector('.resume')");
  await evaluate("document.querySelector('.resume .primary').click()");
  await wait("document.querySelector('.capture')");
  assert.ok(
    await evaluate("document.querySelector('.key-code').textContent==='KeyX'"),
    'recovered cursor should follow the first three consonants',
  );
  await evaluate("document.querySelector('.capture').focus()");
  for (const stroke of strokes.slice(3))
    await evaluate(
      `document.querySelector('.capture').dispatchEvent(new KeyboardEvent('keydown',{code:${JSON.stringify(stroke.code)},key:${JSON.stringify(stroke.code)},bubbles:true,shiftKey:${stroke.layer === 'shift' || stroke.layer === 'shift_altgr'},ctrlKey:${stroke.layer === 'altgr' || stroke.layer === 'shift_altgr'},altKey:${stroke.layer === 'altgr' || stroke.layer === 'shift_altgr'},modifierAltGraph:${stroke.layer === 'altgr' || stroke.layer === 'shift_altgr'}}))`,
    );
  await wait("document.querySelector('details')");
  const history = await invoke('history_list', {
    profile: profile.id,
    filter: {},
    page: 0,
  });
  const restored = await invoke('history_detail', { id: history.rows[0].id });
  assert.equal(restored.summary.correct, strokes.length);
  assert.equal(restored.summary.accuracy, 100);
  assert.equal(
    (await invoke('checkpoints_list', { profile: profile.id })).length,
    0,
  );
  console.log({
    restoredSessionId: restored.session.id,
    correct: restored.summary.correct,
    checkpointRemoved: true,
  });
} finally {
  web.close();
}
