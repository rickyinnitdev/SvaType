import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { connectWebView } from './webview-test.mjs';
const web = await connectWebView();
const { evaluate, wait, call } = web;
const invoke = (name, args = {}) =>
  evaluate(
    `window.__TAURI_INTERNALS__.invoke(${JSON.stringify(name)},${JSON.stringify(args)})`,
  );
try {
  const profiles = await invoke('profiles_list');
  assert.ok(
    profiles.length > 0 && profiles.every((p) => p.name === 'M3 Smoke'),
    'Use the isolated M3 test database only',
  );
  const profile = profiles[0];
  await evaluate(
    "[...document.querySelectorAll('nav button')].find(b=>b.textContent==='Settings').click()",
  );
  await wait("document.querySelector('.settings-grid')");
  await evaluate(
    "const select=[...document.querySelectorAll('.settings-grid select')].at(-1);select.value='dark';select.dispatchEvent(new Event('change',{bubbles:true}));",
  );
  await wait("document.documentElement.dataset.theme==='dark'");
  assert.equal(
    await invoke('settings_get', { key: `p:${profile.id}:theme` }),
    'dark',
  );
  await call('Page.reload');
  await wait("document.querySelector('.people')");
  await evaluate("document.querySelector('.person').click()");
  await wait("document.querySelector('.welcome')");
  assert.equal(
    await evaluate('document.documentElement.dataset.theme'),
    'dark',
  );
  const path = resolve(`target/m3-backup-${Date.now()}.svabak`);
  await invoke('backup_create', { path });
  const probe = await invoke('profiles_create', {
    input: {
      name: 'Backup Probe',
      avatar: '1',
      grade: null,
      path_id: 'beginner',
      layout_id: 'nida',
      preferred_mode: 'typing',
    },
  });
  assert.ok(probe.id > profile.id);
  await invoke('backup_restore', { path });
  assert.equal((await invoke('profiles_list')).length, 1);
  const csv = resolve(`target/m3-export-${Date.now()}.csv`);
  await invoke('export_csv', { path: csv, profile: profile.id });
  assert.ok(
    readFileSync(csv)
      .subarray(0, 3)
      .equals(Buffer.from([0xef, 0xbb, 0xbf])),
  );
  await invoke('settings_set', {
    key: `p:${profile.id}:theme`,
    value: 'light',
  });
  console.log({ settingsPersisted: true, backupRestored: true, csvBom: true });
} finally {
  web.close();
}
