import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const config = JSON.parse(readFileSync('src-tauri/tauri.conf.json', 'utf8'));
const capability = JSON.parse(
  readFileSync('src-tauri/capabilities/default.json', 'utf8'),
);

test('bundled development uses the same restrictive policy as release', () => {
  assert.equal(config.build.devUrl, undefined);
  assert.equal(config.build.beforeDevCommand.wait, true);
  assert.equal(config.app.security.devCsp, undefined);
  assert.equal(config.app.withGlobalTauri, false);
  assert.equal(
    config.app.security.csp,
    "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; media-src 'self'; connect-src 'self' ipc: http://ipc.localhost",
  );
  assert.equal(config.app.windows[0].visible, false);
  assert.equal(config.bundle.windows.webviewInstallMode.type, 'skip');
});

test('only the planned main-window permissions are granted', () => {
  assert.deepEqual(capability.windows, ['main']);
  assert.deepEqual(capability.permissions, [
    'core:default',
    'core:window:allow-show',
    'core:window:allow-set-fullscreen',
    'dialog:allow-save',
    'dialog:allow-open',
  ]);
  assert.equal(capability.remote, undefined);
});
