import { afterEach, expect, it, vi } from 'vitest';
const prefs = vi.hoisted(() => ({ sound: false }));
vi.mock('./stores/settings.svelte', () => ({ preferences: prefs }));
afterEach(() => {
  vi.unstubAllGlobals();
  vi.resetModules();
  prefs.sound = false;
});
it('creates no audio while disabled and cancels queued playback on pause', async () => {
  const suspend = vi.fn(async () => {});
  const resume = vi.fn(async () => {});
  const start = vi.fn();
  const createSource = vi.fn(() => ({ connect: vi.fn(), start }));
  const construct = vi.fn();
  vi.stubGlobal('document', { hidden: false, hasFocus: () => true });
  vi.stubGlobal(
    'AudioContext',
    class {
      constructor() {
        construct();
      }
      suspend = suspend;
      resume = resume;
      createBufferSource = createSource;
      createBuffer = (_channels: number, length: number) => ({
        getChannelData: () => new Float32Array(length),
      });
    },
  );
  const { sound, pauseSounds } = await import('./sound');
  sound('key');
  expect(construct).not.toHaveBeenCalled();
  prefs.sound = true;
  sound('key');
  pauseSounds();
  await Promise.resolve();
  expect(construct).toHaveBeenCalledOnce();
  expect(resume).toHaveBeenCalledOnce();
  expect(start).not.toHaveBeenCalled();
  expect(suspend).toHaveBeenCalled();
});
