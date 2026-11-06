import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { initSync, Layout, Session } from '../src/lib/wasm/khmer.js';
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
const fill = (selector, value, event = 'input') =>
  evaluate(
    `(()=>{const field=document.querySelector(${JSON.stringify(selector)});field.value=${JSON.stringify(value)};field.dispatchEvent(new Event(${JSON.stringify(event)},{bubbles:true}));})()`,
  );
async function tab(text) {
  await click(
    `[...document.querySelectorAll('main nav button')].find(b=>b.textContent.trim()===${JSON.stringify(text)})`,
  );
}
async function unlock(pin) {
  await fill('.gate input', pin);
  await click("document.querySelector('.gate .primary')");
  await wait("document.querySelector('main nav')");
}
let student;
try {
  if (!process.argv.includes('--resume')) {
    let profiles = await invoke('profiles_list');
    assert.equal(profiles.length, 0, 'Use a fresh isolated M10 test directory');
    await invoke('profiles_create', {
      input: {
        name: 'M10 Teacher',
        avatar: '0',
        grade: null,
        path_id: 'beginner',
        layout_id: 'nida',
        preferred_mode: 'learning',
      },
    });
    await invoke('settings_set', { key: 'language', value: 'en' });
    await call('Page.navigate', { url: 'http://tauri.localhost/' });
    await wait("document.querySelector('.person')");
    await click("document.querySelector('.person')");
    await wait("document.querySelector('.continue')");
    await click(
      "[...document.querySelectorAll('.topbar nav button')].find(b=>b.textContent==='Settings')",
    );
    await wait(
      "[...document.querySelectorAll('main button')].some(b=>b.textContent==='Teacher')",
    );
    await button('Teacher');
    await wait("document.querySelector('.gate .primary')");
    await button('Open teacher area');
    await wait("document.querySelector('main nav')");
    await tab('PIN settings');
    await fill('.gate input', '1234');
    await fill('.gate input[autocomplete="new-password"]:last-of-type', '1234');
    // The two inputs are nested labels, so select them explicitly rather than :last-of-type.
    await evaluate(
      "(()=>{const fields=document.querySelectorAll('.gate input');fields[0].value='1234';fields[1].value='1234';fields.forEach(field=>field.dispatchEvent(new Event('input',{bubbles:true})));})()",
    );
    await click("document.querySelector('.gate .primary')");
    await wait(
      "[...document.querySelectorAll('.gate button')].some(b=>b.textContent==='Remove PIN')&&!document.querySelector('.gate .primary').disabled",
    );
    await button('Lock teacher area');
    await wait("document.querySelector('.gate input')");
    assert.equal(
      await evaluate(
        "window.__TAURI_INTERNALS__.invoke('class_overview').catch(e=>e.code)",
      ),
      'teacher_locked',
    );
    assert.equal(
      await evaluate(
        "window.__TAURI_INTERNALS__.invoke('settings_get',{key:'teacher_pin_hash'}).catch(e=>e.code)",
      ),
      'invalid_setting',
    );
    await fill('.gate input', '0000');
    await click("document.querySelector('.gate .primary')");
    await wait("document.body.innerText.includes('That PIN did not match')");
    await unlock('1234');
    await tab('Students');
    await fill('.student-form input', 'M10 Student 01');
    await fill('.student-form select', '3', 'change');
    await evaluate(
      "(()=>{const field=document.querySelectorAll('.student-form select')[1];field.value='grade3';field.dispatchEvent(new Event('change',{bubbles:true}));})()",
    );
    await click("document.querySelector('.student-form .primary')");
    await wait(
      "[...document.querySelectorAll('.student strong')].some(e=>e.textContent==='M10 Student 01')",
    );
    for (let i = 2; i <= 39; i++)
      await invoke('profiles_create', {
        input: {
          name: `M10 Student ${String(i).padStart(2, '0')}`,
          avatar: '1',
          grade: 3,
          path_id: 'grade3',
          layout_id: 'nida',
          preferred_mode: 'learning',
        },
      });
    await click(
      "[...document.querySelectorAll('.student')].find(row=>row.querySelector('strong').textContent==='M10 Student 01').querySelector('button')",
    );
    await fill('.student-form input', 'M10 Renamed');
    await evaluate(
      "(()=>{const fields=document.querySelectorAll('.student-form select');fields[0].value='4';fields[1].value='grade4';fields.forEach(field=>field.dispatchEvent(new Event('change',{bubbles:true})));})()",
    );
    await click("document.querySelector('.student-form .primary')");
    await wait("document.querySelectorAll('.student').length===40");
    await tab('Class overview');
    await wait("document.querySelectorAll('tbody tr').length===40");
    console.log(
      'Teacher UI: PIN gate, create/rename/grade/path and 40-student class view passed.',
    );
    profiles = await invoke('profiles_list');
    student = profiles.find((p) => p.name === 'M10 Renamed');
    initSync({ module: readFileSync('src/lib/wasm/khmer_bg.wasm') });
    const layout = Layout.from_json(
      readFileSync('content/layouts/nida.json', 'utf8'),
    );
    const session = new Session('ក', layout, false, false);
    session.on_key('KeyK', false, false, false, false, false, 100);
    await invoke('session_save', {
      input: {
        uid: 'm10-history-preserved',
        profile_id: student.id,
        mode: 'learning',
        kind: 'lesson',
        ref_id: 'sitting',
        layout_id: 'nida',
        started_at: Date.now(),
        completed: true,
        target_text: 'ក',
        summary: JSON.parse(session.summary()),
        bookmark_step: null,
      },
    });
    session.free();
    layout.free();
    await tab('Students');
    await click(
      "[...document.querySelectorAll('.student')].find(row=>row.querySelector('strong').textContent==='M10 Renamed').querySelectorAll('button')[1]",
    );
    await wait("document.querySelector('dialog[open]')");
    await click("document.querySelector('dialog[open] .primary')");
    await wait("!document.querySelector('dialog[open]')");
    assert.equal(
      (
        await invoke('history_list', {
          profile: student.id,
          filter: {},
          page: 0,
        })
      ).total,
      1,
    );
    assert.equal(
      (await invoke('progress_get', { profile: student.id })).lessons.length,
      0,
    );
  } else {
    const profiles = await invoke('profiles_list');
    assert.equal(profiles.length, 40);
    assert.ok(profiles.every((p) => p.name.startsWith('M10 ')));
    student = profiles.find((p) => p.name === 'M10 Renamed');
  }
  await tab('Assignments');
  await evaluate(
    `(()=>{const fields=document.querySelectorAll('main .panel select');fields[0].value='${student.id}';fields[0].dispatchEvent(new Event('change',{bubbles:true}));fields[2].value='g3_accuracy';fields[2].dispatchEvent(new Event('change',{bubbles:true}));})()`,
  );
  await button('Save changes');
  await wait(
    "[...document.querySelectorAll('main .student p')].some(p=>p.textContent.includes('Your first accuracy challenge'))&&!document.querySelector('main .primary').disabled",
  );
  if (!(await invoke('learning_options', { profile: student.id })).unlock_all)
    await click("document.querySelector('main .panel input[type=checkbox]')");
  await wait(
    "document.querySelector('main [role=status]')?.textContent==='Saved'&&!document.querySelector('main .primary').disabled",
  );
  assert.equal(
    (await invoke('learning_options', { profile: student.id })).unlock_all,
    true,
  );
  await tab('Content editor');
  await wait("document.querySelector('main form textarea')");
  await fill('main form textarea', 'ក');
  await fill('main form .grid textarea', 'អក្សរ ក');
  await fill('main form .grid textarea:last-of-type', 'ក ។');
  await evaluate(
    "(()=>{const areas=document.querySelectorAll('main form .grid textarea');areas[0].value='អក្សរ ក';areas[1].value='ក ។';areas.forEach(field=>field.dispatchEvent(new Event('input',{bubbles:true})));})()",
  );
  await click("document.querySelector('main form .primary')");
  await wait(
    "document.querySelectorAll('.override').length===1&&!document.querySelector('main form .primary').disabled",
  );
  const words = await invoke('content_vocab', { category: 'animals' });
  assert.ok(
    words.words.some((word) => word.word === 'ក' && word.reviewed === false),
  );
  await evaluate(
    "(()=>{const select=document.querySelector('main select');select.value='sentence';select.dispatchEvent(new Event('change',{bubbles:true}));})()",
  );
  await fill('main form textarea', 'កូន មាន បាយ ។');
  await click("document.querySelector('main form .primary')");
  await wait(
    "document.querySelectorAll('.override').length===2&&!document.querySelector('main form .primary').disabled",
  );
  let included = false;
  for (let seed = 0; seed < 20; seed++) {
    const text = await invoke('practice_text', {
      profile: student.id,
      level: 1,
      kind: 'sentences',
      seed,
      weak: false,
      long: false,
    });
    if (text.includes('កូន មាន បាយ ។')) included = true;
  }
  assert.equal(included, true);
  const stamp = Date.now();
  const csv = `D:/SvaType/target/m10-class-${stamp}.csv`;
  const json = `D:/SvaType/target/m10-content-${stamp}.json`;
  await invoke('export_csv', { path: csv, profile: null });
  await invoke('content_overrides_export', { path: json });
  assert.equal(readFileSync(csv).subarray(0, 3).toString('hex'), 'efbbbf');
  assert.equal(JSON.parse(readFileSync(json)).overrides.length, 2);
  console.log(
    'Teacher UI: progress reset kept History, assignment/unlock, word/sentence overrides and exports passed.',
  );
  await tab('PIN settings');
  await button('Remove PIN');
  await wait("document.querySelector('.gate h2').textContent==='Set PIN'");
  assert.equal((await invoke('teacher_pin_status')).configured, false);
  await evaluate(
    "(()=>{const fields=document.querySelectorAll('.gate input');fields[0].value='456789';fields[1].value='456789';fields.forEach(field=>field.dispatchEvent(new Event('input',{bubbles:true})));})()",
  );
  await click("document.querySelector('.gate .primary')");
  await wait(
    "[...document.querySelectorAll('.gate button')].some(b=>b.textContent==='Remove PIN')&&!document.querySelector('.gate .primary').disabled",
  );
  await click("document.querySelector('.profile')");
  await wait("document.querySelector('.people')");
  assert.equal((await invoke('teacher_pin_status')).authorized, false);
  await click(
    "[...document.querySelectorAll('.person')].find(button=>button.textContent.includes('M10 Renamed'))",
  );
  await wait("document.querySelector('.continue h2')");
  assert.equal(
    await evaluate("document.querySelector('.continue h2').textContent"),
    'Your first accuracy challenge',
  );
  console.log(
    'Teacher PIN removal/new PIN, leave-lock and pinned lesson on student Home passed.',
  );
} finally {
  web.close();
}
