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
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
async function home() {
  await click("document.querySelector('.brand')");
  await wait("document.querySelector('.continue')");
}
async function catalog() {
  await click(
    "[...document.querySelectorAll('main button')].find(b=>b.textContent==='Games')",
  );
  await wait("document.querySelectorAll('.game-cards button').length===4");
}
async function choose(game) {
  await click(
    `[...document.querySelectorAll('.game-cards button')].find(b=>b.querySelector('h2').textContent===${JSON.stringify(game.title.en)})`,
  );
  await wait("document.querySelector('.play-area')");
}
async function start() {
  await click("document.querySelector('main .primary')");
  await wait("document.querySelector('main .primary').textContent==='Pause'");
}
async function type(text) {
  for (const stroke of JSON.parse(layout.plan(text)))
    await evaluate(
      `document.querySelector('.capture').dispatchEvent(new KeyboardEvent('keydown',{code:${JSON.stringify(stroke.code)},bubbles:true,shiftKey:${stroke.layer === 'shift' || stroke.layer === 'shift_altgr'},ctrlKey:${stroke.layer === 'altgr' || stroke.layer === 'shift_altgr'},altKey:${stroke.layer === 'altgr' || stroke.layer === 'shift_altgr'},modifierAltGraph:${stroke.layer === 'altgr' || stroke.layer === 'shift_altgr'}}))`,
    );
}
try {
  await call('Page.enable');
  await call('Emulation.setFocusEmulationEnabled', { enabled: true });
  let profiles = await invoke('profiles_list');
  assert.ok(
    profiles.every((p) => p.name === 'M9 Gamer'),
    'Use isolated M9 smoke data',
  );
  const profile = await invoke('profiles_create', {
    input: {
      name: 'M9 Gamer',
      avatar: '0',
      grade: 3,
      path_id: 'grade3',
      layout_id: 'nida',
      preferred_mode: 'learning',
    },
  });
  await invoke('settings_set', { key: 'language', value: 'en' });
  await invoke('settings_set', {
    key: `p:${profile.id}:low_end`,
    value: 'true',
  });
  await call('Page.navigate', { url: 'http://tauri.localhost/' });
  await wait("document.querySelector('.person')");
  await click("document.querySelector('.person')");
  await wait("document.querySelector('.continue')");
  const { games } = await invoke('content_games');
  for (const game of games) {
    await catalog();
    await choose(game);
    let first = 0;
    if (game.id === 'journey') {
      const old = await invoke('game_draft_get', {
        profile: profile.id,
        game: game.id,
      });
      first = old.step;
    }
    await start();
    if (game.id === 'falling') {
      await evaluate("window.dispatchEvent(new Event('blur'))");
      await wait(
        "document.querySelector('main .primary').textContent==='Start typing'",
      );
      await start();
      const updates = await evaluate(
        "new Promise(resolve=>{let n=0;const observer=new MutationObserver(()=>n++);observer.observe(document.querySelector('.sprite'),{attributes:true,attributeFilter:['style']});setTimeout(()=>{observer.disconnect();resolve(n);},1000);})",
      );
      assert.ok(updates <= 33, `low-end rendered ${updates} frames/second`);
      await evaluate(
        "Object.defineProperty(document,'visibilityState',{configurable:true,get:()=> 'hidden'});Object.defineProperty(document,'hidden',{configurable:true,get:()=>true});document.dispatchEvent(new Event('visibilitychange'))",
      );
      await wait(
        "document.querySelector('main .primary').textContent==='Start typing'",
      );
      const transform = await evaluate(
        "document.querySelector('.sprite').style.transform",
      );
      await sleep(100);
      assert.equal(
        await evaluate("document.querySelector('.sprite').style.transform"),
        transform,
      );
      await evaluate(
        "delete document.visibilityState;delete document.hidden;document.dispatchEvent(new Event('visibilitychange'))",
      );
      await start();
      console.log('Falling: blur/hide pause and 30fps cap passed.');
    }
    for (let i = first; i < game.rounds.length; i++) {
      if (game.id === 'journey' && i === 4) {
        await home();
        const draft = await invoke('game_draft_get', {
          profile: profile.id,
          game: game.id,
        });
        assert.equal(draft.step, 4);
        await catalog();
        await choose(game);
        await start();
      }
      await type(game.rounds[i].target);
      if (i < game.rounds.length - 1)
        await wait(
          `document.querySelector('main .heading strong')?.textContent.trim().replace(/\\s+/g,' ')==='Round ${i + 2} / 10'`,
        );
    }
    await wait("document.querySelector('details')");
    console.log('Completed', game.id);
    await home();
  }
  const history = await invoke('history_list', {
    profile: profile.id,
    filter: { kind: 'game' },
    page: 0,
  });
  assert.equal(history.total, 4);
  assert.ok(history.rows.every((row) => row.completed));
  const awards = await invoke('achievements_get', { profile: profile.id });
  assert.equal(awards.filter((a) => a.id === 'first_game').length, 1);
  console.log(
    'All four games saved to History; Journey resumed at stop 5; first-game achievement awarded once.',
  );
} finally {
  layout.free();
  web.close();
}
