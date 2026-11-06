// Use only with SVATYPE_TEST_DATA_DIR pointing inside target/.
import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';
import { initSync, Layout } from '../src/lib/wasm/khmer.js';
import { connectWebView } from './webview-test.mjs';
initSync({ module: readFileSync('src/lib/wasm/khmer_bg.wasm') });
const l = Layout.from_json(readFileSync('content/layouts/nida.json', 'utf8'));
const target = JSON.parse(readFileSync('content/playground.json', 'utf8')).text;
const strokes = JSON.parse(l.plan(target));
l.free();
const web = await connectWebView();
const { evaluate, wait, call } = web;
const invoke = (name, args = {}) =>
  evaluate(
    `window.__TAURI_INTERNALS__.invoke(${JSON.stringify(name)},${JSON.stringify(args)})`,
  );
const click = (expression) => evaluate(`(${expression}).click()`);
const key = (stroke) =>
  evaluate(
    `document.querySelector('.capture').dispatchEvent(new KeyboardEvent('keydown',{code:${JSON.stringify(stroke.code)},key:${JSON.stringify(stroke.code)},bubbles:true,shiftKey:${stroke.layer === 'shift' || stroke.layer === 'shift_altgr'},ctrlKey:${stroke.layer === 'altgr' || stroke.layer === 'shift_altgr'},altKey:${stroke.layer === 'altgr' || stroke.layer === 'shift_altgr'},modifierAltGraph:${stroke.layer === 'altgr' || stroke.layer === 'shift_altgr'}}))`,
  );
try {
  await call('Page.navigate', { url: 'http://tauri.localhost/' });
  await wait(
    "document.querySelector('.onboarding')||document.querySelector('.people')",
  );
  let profiles = await invoke('profiles_list');
  if (!profiles.length) {
    await click(
      "[...document.querySelectorAll('.onboarding button')].find(b=>b.textContent.trim()==='English')",
    );
    await click("document.querySelector('.actions .primary')");
    await wait("document.querySelector('.onboarding input')");
    await evaluate(
      "const input=document.querySelector('.onboarding input');input.value='M3 Smoke';input.dispatchEvent(new Event('input',{bubbles:true}));",
    );
    await click("document.querySelector('.actions .primary')");
    await wait("document.querySelector('.mode-cards')");
    await click(
      "[...document.querySelectorAll('.mode-cards button')].find(b=>b.querySelector('strong').textContent==='Typing practice')",
    );
    await click("document.querySelector('.actions .primary')");
    await wait("document.querySelector('.key-check')");
    await click("document.querySelector('.actions .primary')");
    await wait("document.querySelector('.welcome')");
  } else {
    await click(
      "[...document.querySelectorAll('.person')].find(b=>b.textContent.trim()==='M3 Smoke')",
    );
    await wait("document.querySelector('.welcome')");
  }
  profiles = await invoke('profiles_list');
  const profile = profiles.find((p) => p.name === 'M3 Smoke');
  assert.ok(profile);
  const before = await invoke('history_list', {
    profile: profile.id,
    filter: {},
    page: 0,
  });
  await click("document.querySelector('.welcome .primary')");
  await wait("document.querySelector('.capture')");
  await evaluate("document.querySelector('.capture').focus()");
  for (const stroke of strokes) await key(stroke);
  await wait(
    "document.querySelector('.stats')&&document.querySelector('details')",
  );
  const after = await invoke('history_list', {
    profile: profile.id,
    filter: {},
    page: 0,
  });
  assert.equal(after.total, before.total + 1);
  const detail = await invoke('history_detail', { id: after.rows[0].id });
  assert.equal(detail.target_text, target);
  assert.equal(detail.summary.accuracy, 100);
  assert.ok(detail.summary.per_key.length > 0);
  await call('Page.reload');
  await wait("document.querySelector('.people')");
  assert.ok(
    await evaluate(
      "document.querySelector('.people').textContent.includes('M3 Smoke')",
    ),
  );
  await click(
    "[...document.querySelectorAll('.person')].find(b=>b.textContent.trim()==='M3 Smoke')",
  );
  await wait("document.querySelector('.welcome')");
  await click(
    "[...document.querySelectorAll('nav button')].find(b=>b.textContent==='History')",
  );
  await wait("document.querySelector('table tbody tr')");
  assert.ok(
    await evaluate(
      "document.querySelector('table').textContent.includes('Speed test')",
    ),
  );
  console.log({
    profile: profile.id,
    sessionId: after.rows[0].id,
    persistedSessions: after.total,
    accuracy: detail.summary.accuracy,
  });
  if (process.argv.includes('--checkpoint')) {
    await click("document.querySelector('.brand')");
    await wait("document.querySelector('.welcome')");
    await click("document.querySelector('.welcome .primary')");
    await wait("document.querySelector('.capture')");
    await evaluate("document.querySelector('.capture').focus()");
    for (const stroke of strokes.slice(0, 3)) await key(stroke);
    console.log(
      'Checkpoint timer running; leave this test process for 32 seconds.',
    );
    await new Promise((resolve) => setTimeout(resolve, 32_000));
    const checkpoints = await invoke('checkpoints_list', {
      profile: profile.id,
    });
    assert.equal(checkpoints.length, 1);
    assert.equal(checkpoints[0].session.summary.correct, 3);
    console.log({
      checkpointUid: checkpoints[0].session.uid,
      correct: checkpoints[0].session.summary.correct,
    });
  }
} finally {
  web.close();
}
