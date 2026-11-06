import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { initSync, Layout } from '../src/lib/wasm/khmer.js';
import { connectWebView } from './webview-test.mjs';
initSync({ module: readFileSync('src/lib/wasm/khmer_bg.wasm') });
const layout = Layout.from_json(
  readFileSync('content/layouts/nida.json', 'utf8'),
);
const web = await connectWebView();
const { evaluate, call, wait } = web;
const invoke = (name, args = {}) =>
  evaluate(
    `window.__TAURI_INTERNALS__.invoke(${JSON.stringify(name)},${JSON.stringify(args)})`,
  );
const click = (expression) => evaluate(`(${expression}).click()`);
async function home() {
  await click("document.querySelector('.brand')");
  await wait("document.querySelector('.typing-cards')");
}
async function card(title) {
  await click(
    `[...document.querySelectorAll('.typing-card')].find(b=>b.querySelector('h2').textContent===${JSON.stringify(title)})`,
  );
}
async function send(code, layer = 'base') {
  await evaluate(
    `document.querySelector('.capture').dispatchEvent(new KeyboardEvent('keydown',{code:${JSON.stringify(code)},key:${JSON.stringify(code)},bubbles:true,shiftKey:${layer === 'shift' || layer === 'shift_altgr'},ctrlKey:${layer === 'altgr' || layer === 'shift_altgr'},altKey:${layer === 'altgr' || layer === 'shift_altgr'},modifierAltGraph:${layer === 'altgr' || layer === 'shift_altgr'}}))`,
  );
}
async function type(text) {
  await evaluate("document.querySelector('.capture').focus()");
  for (const stroke of JSON.parse(layout.plan(text)))
    await send(stroke.code, stroke.layer);
}
async function visibleText() {
  return evaluate(
    "[...document.querySelectorAll('.track .cluster')].map(c=>c.textContent==='␣'?' ':c.textContent==='↵'?'\\n':c.textContent).join('')",
  );
}
try {
  await call('Page.navigate', { url: 'http://tauri.localhost/' });
  await wait("document.querySelector('.people')");
  const profiles = await invoke('profiles_list');
  assert.ok(
    profiles.every((p) => p.name === 'M3 Smoke'),
    'Use isolated smoke data only',
  );
  const profile = profiles[0];
  await click("document.querySelector('.person')");
  await wait("document.querySelector('.typing-cards')");
  await card('Free typing');
  await wait("document.querySelector('.free-text')");
  await type('កខ');
  await send('Backspace');
  await send('KeyX');
  assert.equal(
    await evaluate("document.querySelector('.free-text').textContent.trim()"),
    'កខ',
  );
  await evaluate(
    "Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText:async text=>{window.__copiedText=text;}}})",
  );
  await click(
    "[...document.querySelectorAll('.practice-footer button')].find(b=>b.textContent==='Copy text')",
  );
  assert.equal(await evaluate('window.__copiedText'), 'កខ');
  await click(
    "[...document.querySelectorAll('.practice-footer button')].find(b=>b.textContent==='Finish practice')",
  );
  await wait("document.querySelector('details')");
  let history = await invoke('history_list', {
    profile: profile.id,
    filter: {},
    page: 0,
  });
  assert.equal(history.rows[0].kind, 'free');
  await home();
  await card('Custom text');
  await wait("document.querySelector('textarea')");
  await evaluate(
    "const input=document.querySelector('main input');input.value='M4 Custom';input.dispatchEvent(new Event('input',{bubbles:true}));const area=document.querySelector('textarea');area.value='កខគ';area.dispatchEvent(new Event('input',{bubbles:true}));",
  );
  await click("document.querySelector('main .primary')");
  await wait("document.querySelector('.capture')");
  await evaluate("document.querySelector('.capture').focus()");
  await send('KeyA');
  await send('Backspace');
  await type('កខគ');
  await wait("document.querySelector('details')");
  history = await invoke('history_list', {
    profile: profile.id,
    filter: {},
    page: 0,
  });
  assert.equal(history.rows[0].kind, 'custom');
  assert.ok(history.rows[0].accuracy < 100);
  await home();
  await card('Random practice');
  await wait("document.querySelector('main select')");
  await click("document.querySelector('main .primary')");
  await wait("document.querySelector('.capture')");
  await type(await visibleText());
  await wait("document.querySelector('details')");
  history = await invoke('history_list', {
    profile: profile.id,
    filter: {},
    page: 0,
  });
  assert.equal(history.rows[0].kind, 'random');
  await home();
  await card('Mistake analysis');
  await wait("document.querySelector('.columns')");
  assert.ok(
    await evaluate("[...document.querySelectorAll('.key .rate')].length>0"),
  );
  await click("document.querySelector('.heading .primary')");
  await wait("document.querySelector('.capture')");
  await type(await visibleText());
  await wait("document.querySelector('details')");
  history = await invoke('history_list', {
    profile: profile.id,
    filter: {},
    page: 0,
  });
  assert.equal(history.rows[0].kind, 'drill');
  await home();
  await card('Speed test');
  await wait("document.querySelector('fieldset')");
  assert.equal(
    await evaluate("document.querySelectorAll('fieldset button').length"),
    4,
  );
  await click("document.querySelector('main .primary')");
  await wait("document.querySelector('.capture')");
  const prefix = JSON.parse(layout.plan(await visibleText())).slice(0, 3);
  await evaluate("document.querySelector('.capture').focus()");
  for (const stroke of prefix) await send(stroke.code, stroke.layer);
  await call('Emulation.setVirtualTimePolicy', {
    policy: 'advance',
    budget: 61_000,
    maxVirtualTimeTaskStarvationCount: 1000,
  });
  for (let i = 0; i < 100; i++) {
    if (await evaluate("!!document.querySelector('details')")) break;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  assert.equal(
    await evaluate("!!document.querySelector('details')"),
    true,
    'timed test must finish at deadline',
  );
  history = await invoke('history_list', {
    profile: profile.id,
    filter: {},
    page: 0,
  });
  assert.equal(history.rows[0].kind, 'test');
  assert.equal(history.rows[0].completed, true);
  assert.ok(history.rows[0].duration_ms >= 60_000);
  console.log({
    free: true,
    custom: true,
    random: true,
    analysisDrill: true,
    timed: true,
    totalSessions: history.total,
  });
} finally {
  layout.free();
  web.close();
}
