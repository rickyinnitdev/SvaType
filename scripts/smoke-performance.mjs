// Release-only local integration probe. Start an isolated portable release with CDP 9223.
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
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
  const profiles = await invoke('profiles_list');
  assert.ok(
    profiles.every((p) => p.name === 'M7 Performance'),
    'Use isolated portable performance data',
  );
  const profile =
    profiles[0] ??
    (await invoke('profiles_create', {
      input: {
        name: 'M7 Performance',
        avatar: '0',
        grade: null,
        path_id: 'beginner',
        layout_id: 'nida',
        preferred_mode: 'typing',
      },
    }));
  await invoke('settings_set', { key: 'language', value: 'en' });
  await invoke('settings_set', {
    key: `p:${profile.id}:low_end`,
    value: 'true',
  });
  await call('Page.enable');
  await call('Page.addScriptToEvaluateOnNewDocument', {
    source: `(()=>{const intervals=new Set();const frames=new Set();const si=window.setInterval,ci=window.clearInterval,raf=window.requestAnimationFrame,caf=window.cancelAnimationFrame;window.setInterval=(fn,ms,...args)=>{const id=si(fn,ms,...args);intervals.add(id);return id;};window.clearInterval=id=>{intervals.delete(id);ci(id);};window.requestAnimationFrame=fn=>{const id=raf(t=>{frames.delete(id);fn(t);});frames.add(id);return id;};window.cancelAnimationFrame=id=>{frames.delete(id);caf(id);};window.__idleProbe=()=>({intervals:intervals.size,frames:frames.size});})()`,
  });
  await call('Page.navigate', { url: 'http://tauri.localhost/' });
  await wait("document.querySelector('.person')");
  await click("document.querySelector('.person')");
  await wait("document.querySelector('.typing-cards')");
  const results = {};
  for (const rate of [1, 6]) {
    await call('Emulation.setCPUThrottlingRate', { rate });
    await click(
      "[...document.querySelectorAll('.typing-card')].find(b=>b.querySelector('h2').textContent==='Custom text')",
    );
    await wait("document.querySelector('textarea')");
    const text = 'កខគឃង'.repeat(60);
    await evaluate(
      `(()=>{const input=document.querySelector('main input');input.value='Performance';input.dispatchEvent(new Event('input',{bubbles:true}));const area=document.querySelector('textarea');area.value=${JSON.stringify(text)};area.dispatchEvent(new Event('input',{bubbles:true}));})()`,
    );
    await click("document.querySelector('main .primary')");
    await wait("document.querySelector('.capture')");
    const strokes = JSON.parse(layout.plan(text)).slice(0, 200);
    results[rate] = await evaluate(
      `(async()=>{const capture=document.querySelector('.capture');capture.focus();let count=0;const original=window.__TAURI_INTERNALS__.invoke;window.__TAURI_INTERNALS__.invoke=(...args)=>{count++;return original(...args);};const times=[];for(const stroke of ${JSON.stringify(strokes)}){const start=performance.now();capture.dispatchEvent(new KeyboardEvent('keydown',{code:stroke.code,bubbles:true,shiftKey:stroke.layer==='shift',modifierAltGraph:stroke.layer==='altgr'}));await Promise.resolve();await Promise.resolve();await Promise.resolve();times.push(performance.now()-start);}window.__TAURI_INTERNALS__.invoke=original;times.sort((a,b)=>a-b);return {p50:times[99],p95:times[189],max:times[199],ipcCalls:count,visibleClusters:document.querySelectorAll('.track .cluster').length};})()`,
    );
    assert.equal(results[rate].ipcCalls, 0);
    assert.ok(results[rate].visibleClusters <= 120);
    assert.ok(results[rate].p95 < 16, JSON.stringify(results[rate]));
    await click("document.querySelector('.brand')");
    await wait("document.querySelector('.typing-cards')");
  }
  await call('Emulation.setCPUThrottlingRate', { rate: 1 });
  await evaluate('new Promise(resolve=>setTimeout(resolve,500))');
  results.idle = await evaluate('window.__idleProbe()');
  assert.deepEqual(results.idle, { intervals: 0, frames: 0 });
  results.fonts = await evaluate(
    'document.fonts.check(\'18px "Noto Sans Khmer"\')',
  );
  assert.equal(results.fonts, true);
  writeFileSync(
    'target/performance-webview.json',
    JSON.stringify(results, null, 2),
  );
  console.log(JSON.stringify(results, null, 2));
} finally {
  layout.free();
  web.close();
}
