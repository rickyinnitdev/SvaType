import { it, expect } from 'vitest';
import { GameLoop } from './game-loop';
it('caps low-end updates at 30fps and cancels all work on pause', () => {
  let next = 0;
  const queue = new Map<number, FrameRequestCallback>();
  let updates = 0;
  let total = 0;
  const loop = new GameLoop(
    (dt) => {
      updates++;
      total += dt;
    },
    true,
    () => {},
    (callback) => {
      queue.set(++next, callback);
      return next;
    },
    (id) => {
      queue.delete(id);
    },
  );
  expect(queue.size).toBe(0);
  loop.start();
  for (let t = 0; t <= 1000; t += 1000 / 60) {
    const entries = [...queue.values()];
    queue.clear();
    entries.forEach((callback) => callback(t));
  }
  expect(updates).toBeLessThanOrEqual(31);
  expect(total).toBeGreaterThan(950);
  expect(queue.size).toBe(1);
  loop.pause();
  expect(queue.size).toBe(0);
  loop.dispose();
});
it('resuming does not advance through paused wall time', () => {
  let callback: FrameRequestCallback = () => {};
  const deltas: number[] = [];
  const loop = new GameLoop(
    (dt) => deltas.push(dt),
    false,
    () => {},
    (cb) => {
      callback = cb;
      return 1;
    },
    () => {},
  );
  loop.start();
  callback(100);
  callback(117);
  loop.pause();
  loop.start();
  callback(100000);
  expect(deltas).toEqual([0, 17, 0]);
  loop.dispose();
});
