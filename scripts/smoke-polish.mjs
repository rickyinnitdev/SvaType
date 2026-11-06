import assert from 'node:assert/strict';
import { connectWebView } from './webview-test.mjs';
const web = await connectWebView();
const { evaluate, wait, call } = web;
const invoke = (name, args = {}) =>
  evaluate(
    `window.__TAURI_INTERNALS__.invoke(${JSON.stringify(name)},${JSON.stringify(args)})`,
  );
const click = (expr) => evaluate(`(${expr}).click()`);
const button = (text) =>
  click(
    `[...document.querySelectorAll('main button')].find(b=>b.textContent.trim()===${JSON.stringify(text)})`,
  );
async function settings() {
  await click(
    "[...document.querySelectorAll('.topbar nav button')].find(b=>b.textContent==='Settings')",
  );
  await wait("document.querySelector('.settings-grid')");
}
try {
  let profiles = await invoke('profiles_list');
  assert.equal(profiles.length, 0, 'Use fresh isolated M11 data');
  const profile = await invoke('profiles_create', {
    input: {
      name: 'M11 Learner',
      avatar: '0',
      grade: 3,
      path_id: 'grade3',
      layout_id: 'nida',
      preferred_mode: 'learning',
    },
  });
  await invoke('settings_set', { key: 'language', value: 'en' });
  await call('Emulation.setFocusEmulationEnabled', { enabled: true });
  await call('Page.navigate', { url: 'http://tauri.localhost/' });
  await wait("document.querySelector('.person')");
  await click("document.querySelector('.person')");
  await wait("document.querySelector('.continue')");
  await settings();
  await button('Help / About');
  await wait(
    "document.querySelectorAll('details pre').length===2&&document.querySelectorAll('details pre')[0].textContent.includes('SIL OPEN FONT LICENSE')",
  );
  assert.ok(
    await evaluate(
      "document.body.innerText.includes('Understanding your results')",
    ),
  );
  await button('Check Windows layout');
  await wait("document.querySelector('textarea.os-capture')");
  await button('Start check');
  await evaluate(
    "(()=>{const area=document.querySelector('textarea');for(const actual of ['ក','k']){area.dispatchEvent(new KeyboardEvent('keydown',{code:'KeyK',bubbles:true}));area.value=actual;area.dispatchEvent(new InputEvent('input',{bubbles:true,data:actual,inputType:'insertText'}));area.dispatchEvent(new KeyboardEvent('keyup',{code:'KeyK',bubbles:true}));}})()",
  );
  assert.equal(
    await evaluate("document.querySelectorAll('tbody tr').length"),
    2,
  );
  assert.equal(
    await evaluate("document.querySelectorAll('tbody tr.mismatch').length"),
    1,
  );
  console.log(
    'Help, bundled full OFL licences and exact Windows/engine comparison passed.',
  );
  await settings();
  await evaluate(
    "(()=>{const fields=document.querySelectorAll('.settings-grid select');fields[2].value='xl';fields[2].dispatchEvent(new Event('change',{bubbles:true}));fields[3].value='contrast';fields[3].dispatchEvent(new Event('change',{bubbles:true}));})()",
  );
  await wait(
    "document.documentElement.dataset.theme==='contrast'&&getComputedStyle(document.documentElement).getPropertyValue('--font-ui')==='24px'",
  );
  await click("document.querySelector('.brand')");
  await wait("document.querySelector('.continue')");
  await click("document.querySelector('.continue .primary')");
  await wait("document.querySelector('.lesson .explanation')");
  // Controlled monotonic clock: verify >20s reading is recorded without an idle timer.
  await evaluate(
    'window.__originalNow=performance.now.bind(performance);performance.now=()=>window.__originalNow()+21000',
  );
  await button('Save and leave');
  await wait("document.querySelector('.continue')");
  await evaluate('performance.now=window.__originalNow');
  let history = await invoke('history_list', {
    profile: profile.id,
    filter: { kind: 'lesson' },
    page: 0,
  });
  assert.equal(history.total, 1);
  assert.equal(history.rows[0].completed, false);
  assert.ok(history.rows[0].duration_ms >= 21000);
  await click("document.querySelector('.continue .primary')");
  await wait("document.querySelector('.lesson .explanation')");
  await click("document.querySelector('.lesson .primary')");
  await wait("document.querySelector('.lesson .hands')");
  await click(
    "[...document.querySelectorAll('.lesson button')].find(b=>b.textContent==='Continue')",
  );
  await wait("document.querySelector('.capture')");
  assert.equal(
    await evaluate(
      "getComputedStyle(document.querySelector('.lesson')).getPropertyValue('--font-target')",
    ),
    '48px',
  );
  assert.equal(
    await evaluate(
      "document.querySelectorAll('.controls input[type=checkbox]').length",
    ),
    0,
  );
  assert.equal(
    await evaluate(
      "getComputedStyle(document.querySelector('.find-key')).fontSize",
    ),
    '64px',
  );
  await button('Save and leave');
  await wait("document.querySelector('.continue')");
  await button('Your learning path');
  await wait("document.querySelector('main select')");
  await evaluate(
    "(()=>{const field=document.querySelector('main select');field.value='grade4';field.dispatchEvent(new Event('change',{bubbles:true}));})()",
  );
  await wait("document.querySelectorAll('.lesson-row').length===22");
  assert.equal((await invoke('profiles_list'))[0].path_id, 'grade4');
  await click("document.querySelector('.brand')");
  await wait("document.querySelector('.continue')");
  console.log(
    'Contrast/XL, Grade 3 size floor, reading History, bookmarks and path persistence passed.',
  );
  await settings();
  await evaluate(
    "(()=>{const fields=document.querySelectorAll('.settings-grid select');fields[2].value='m';fields[2].dispatchEvent(new Event('change',{bubbles:true}));fields[3].value='light';fields[3].dispatchEvent(new Event('change',{bubbles:true}));})()",
  );
  await wait("document.documentElement.dataset.theme==='light'");
  await evaluate(
    "(()=>{const field=document.querySelector('.settings-grid select');field.value='km';field.dispatchEvent(new Event('change',{bubbles:true}));})()",
  );
  await wait("document.documentElement.lang==='km'");
  assert.ok(
    await evaluate("document.querySelector('h1').textContent==='ការកំណត់'"),
  );
  await call('Emulation.setDeviceMetricsOverride', {
    width: 1024,
    height: 640,
    deviceScaleFactor: 1,
    mobile: false,
  });
  assert.ok(await evaluate('document.documentElement.scrollWidth<=innerWidth'));
  await call('Emulation.clearDeviceMetricsOverride');
  console.log('Khmer UI and 1024px Settings layout passed.');
} finally {
  web.close();
}
