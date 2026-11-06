import { beforeAll, describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { loadEngine, segment, TypingSession, type LayoutData } from './engine';
import nida from '../../content/layouts/nida.json';

beforeAll(async () => {
  await loadEngine(
    await WebAssembly.compile(readFileSync('src/lib/wasm/khmer_bg.wasm')),
  );
});
function event(code: string, shift = false, altgr = false) {
  return {
    code,
    shiftKey: shift,
    ctrlKey: altgr,
    metaKey: false,
    repeat: false,
    getModifierState: (key: string) => key === 'AltGraph' && altgr,
  };
}
describe('real WASM engine', () => {
  it('supports free text, live counters and undo', () => {
    const session = new TypingSession(
      '',
      nida as LayoutData,
      'flow',
      false,
      undefined,
      true,
    );
    session.onKey(event('KeyK'), 0);
    session.onKey(event('KeyX'), 1000);
    expect(session.tail()).toBe('កខ');
    expect(session.live().kpm).toBe(120);
    session.onKey(event('Backspace'), 1100);
    expect(session.typed()).toBe('ក');
    expect(session.summary().accuracy).toBe(100);
    session.free();
  });
  it('keeps counters across timed text rounds', () => {
    const session = new TypingSession('ក', nida as LayoutData, 'flow');
    session.onKey(event('KeyK'), 0);
    session.nextRound();
    expect(session.cursor()).toBe(0);
    session.onKey(event('KeyK'), 100);
    expect(session.summary().correct).toBe(2);
    session.free();
  });
  it('restores an aggregate checkpoint', () => {
    const session = new TypingSession('កខ', nida as LayoutData, 'flow');
    session.onKey(event('KeyK'), 0);
    const snapshot = session.checkpoint();
    session.free();
    const restored = new TypingSession(
      'កខ',
      nida as LayoutData,
      'flow',
      false,
      snapshot,
    );
    expect(restored.typed()).toBe('ក');
    expect(restored.onKey(event('KeyX'), 1000).finished).toBe(true);
    expect(restored.summary().correct).toBe(2);
    restored.free();
  });
  it('initializes only once', async () => {
    const first = loadEngine();
    expect(loadEngine()).toBe(first);
    await first;
  });
  it('returns safe cluster strings with byte offsets', () => {
    expect(segment('🙂ខ្មែរ').map((c) => c.text)).toEqual(['🙂', 'ខ្មែ', 'រ']);
    expect(segment('ខ្មែរ')[0].end).toBe(12);
  });
  it('handles a multi-output key and result metrics', () => {
    const session = new TypingSession(
      'កុំ',
      nida as LayoutData,
      'stop_on_error',
    );
    expect(session.onKey(event('KeyK'), 0).correct).toBe(true);
    const result = session.onKey(event('Comma'), 1000);
    expect(result.finished).toBe(true);
    expect(session.typed()).toBe('កុំ');
    expect(session.summary().kpm).toBe(120);
    expect(session.summary().accuracy).toBe(100);
    session.free();
  });
  it('uses Rust for flow errors and undo', () => {
    const session = new TypingSession('កខ', nida as LayoutData, 'flow');
    expect(session.onKey(event('KeyA'), 0).correct).toBe(false);
    expect(session.typed()).toBe('ា');
    session.onKey(event('Backspace'), 100);
    expect(session.typed()).toBe('');
    expect(session.expected()?.produces).toBe('ក');
    session.free();
  });
  it('uses Rust to ignore repeated keys', () => {
    const session = new TypingSession('ក', nida as LayoutData, 'flow');
    expect(session.onKey({ ...event('KeyK'), repeat: true }, 0).ignored).toBe(
      true,
    );
    expect(session.summary().keystrokes).toBe(0);
    session.free();
  });
  it('rejects unsupported custom characters', () => {
    expect(() => new TypingSession('🙂', nida as LayoutData, 'flow')).toThrow();
  });
});
