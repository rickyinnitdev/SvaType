import { it, expect } from 'vitest';
import { LeaveGuards } from './leave-guards';
it('awaits child writes before parent cleanup', async () => {
  const guards = new LeaveGuards();
  const order: string[] = [];
  guards.register(async () => {
    order.push('parent');
  });
  guards.register(async () => {
    await Promise.resolve();
    order.push('child');
  });
  await guards.flush();
  expect(order).toEqual(['child', 'parent']);
});
it('blocks cleanup after a failed save and permits retry', async () => {
  const guards = new LeaveGuards();
  let cleaned = false;
  guards.register(async () => {
    cleaned = true;
  });
  const remove = guards.register(async () => {
    throw new Error('disk full');
  });
  await expect(guards.flush()).rejects.toThrow('disk full');
  expect(cleaned).toBe(false);
  remove();
  await guards.flush();
  expect(cleaned).toBe(true);
});
