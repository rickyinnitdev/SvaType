import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { initSync, Layout, Session } from '../src/lib/wasm/khmer.js';
import { connectWebView } from './webview-test.mjs';
const web = await connectWebView();
const { evaluate, wait } = web;
const invoke = (name, args = {}) =>
  evaluate(
    `window.__TAURI_INTERNALS__.invoke(${JSON.stringify(name)},${JSON.stringify(args)})`,
  );
try {
  await wait("document.querySelector('main button')");
  const profiles = await invoke('profiles_list');
  assert.ok(
    profiles.every((p) => p.name === 'M8 Installer'),
    'Isolated installer test profile only',
  );
  if (process.argv.includes('--seed')) {
    assert.equal(profiles.length, 0);
    const profile = await invoke('profiles_create', {
      input: {
        name: 'M8 Installer',
        avatar: '0',
        grade: 3,
        path_id: 'grade3',
        layout_id: 'nida',
        preferred_mode: 'learning',
      },
    });
    initSync({ module: readFileSync('src/lib/wasm/khmer_bg.wasm') });
    const layout = Layout.from_json(
      readFileSync('content/layouts/nida.json', 'utf8'),
    );
    const session = new Session('កខ', layout, false, false);
    for (const [i, stroke] of JSON.parse(layout.plan('កខ')).entries())
      session.on_key(
        stroke.code,
        stroke.layer === 'shift',
        false,
        false,
        false,
        false,
        i * 200,
      );
    const input = {
      uid: 'm8-installer-persistence',
      profile_id: profile.id,
      mode: 'learning',
      kind: 'drill',
      ref_id: null,
      layout_id: 'nida',
      started_at: Date.now(),
      completed: true,
      target_text: 'កខ',
      summary: JSON.parse(session.summary()),
      bookmark_step: null,
    };
    const id = await invoke('session_save', { input });
    writeFileSync(
      'target/install-smoke-expect.json',
      JSON.stringify({ profile: profile.id, session: id }),
    );
    session.free();
    layout.free();
  } else {
    const expected = JSON.parse(
      readFileSync('target/install-smoke-expect.json'),
    );
    assert.equal(profiles[0].id, expected.profile);
    const history = await invoke('history_list', {
      profile: expected.profile,
      filter: {},
      page: 0,
    });
    assert.equal(history.total, 1);
    assert.equal(history.rows[0].id, expected.session);
    console.log('Installer data retained: profile and completed History row.');
  }
} finally {
  web.close();
}
