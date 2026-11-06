import { LeaveGuards } from './leave-guards';
export type Screen =
  | 'profiles'
  | 'onboarding'
  | 'home'
  | 'practice'
  | 'history'
  | 'result'
  | 'settings'
  | 'free'
  | 'test'
  | 'custom'
  | 'random'
  | 'analysis'
  | 'path'
  | 'lesson'
  | 'intro'
  | 'progress'
  | 'vocabulary'
  | 'games'
  | 'teacher'
  | 'help'
  | 'layoutCheck';
export const router = $state<{
  screen: Screen;
  resultId: number | null;
  busy: boolean;
  error: string | null;
}>({ screen: 'profiles', resultId: null, busy: false, error: null });
const guards = new LeaveGuards();
export function registerLeaveGuard(next: () => Promise<void>) {
  return guards.register(next);
}
export async function navigate(screen: Screen, resultId: number | null = null) {
  if (router.busy) return;
  router.busy = true;
  router.error = null;
  try {
    await guards.flush();
    router.resultId = resultId;
    router.screen = screen;
  } catch {
    router.error = 'save_failed';
  } finally {
    router.busy = false;
  }
}
export async function flushForClose() {
  if (router.busy) return false;
  router.busy = true;
  router.error = null;
  try {
    await guards.flush();
    return true;
  } catch {
    router.error = 'save_failed';
    return false;
  } finally {
    router.busy = false;
  }
}
