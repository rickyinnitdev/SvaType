import { test } from 'node:test';
import assert from 'node:assert/strict';
import { violations } from './check-offline.mjs';

test('accepts local assets, IPC and comments', () => {
  for (const source of [
    'fetch("./engine.wasm")',
    'fetch("/content/index.json")',
    '"http://ipc.localhost"',
    '// https://example.com documentation',
    '/* https://example.com */',
  ])
    assert.deepEqual(violations(source), []);
});

test('rejects remote resources and network APIs', () => {
  for (const source of [
    'fetch("https://example.com")',
    'fetch(endpoint)',
    'new XMLHttpRequest()',
    'new WebSocket(address)',
    'navigator.sendBeacon(address)',
    '<link href="//example.com/font.css">',
    '@import "https://example.com/font.css";',
    '"http://ipc.localhost.evil"',
  ])
    assert.ok(violations(source).length > 0, source);
});
