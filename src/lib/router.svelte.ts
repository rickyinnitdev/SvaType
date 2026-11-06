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
  | 'progress';
export const router = $state<{
  screen: Screen;
  resultId: number | null;
  busy: boolean;
  error: string | null;
}>({ screen: 'profiles', resultId: null, busy: false, error: null });
// Guards are an imperative registry, never rendered or read by reactive effects.
// eslint-disable-next-line svelte/prefer-svelte-reactivity
const guards = new Map<symbol, () => Promise<void>>();
export function registerLeaveGuard(next: () => Promise<void>) {
  const key = Symbol();
  guards.set(key, next);
  return () => {
    guards.delete(key);
  };
}
export async function navigate(screen: Screen, resultId: number | null = null) {
  if (router.busy) return;
  router.busy = true;
  router.error = null;
  try {
    for (const guard of [...guards.values()]) await guard();
    router.resultId = resultId;
    router.screen = screen;
  } catch {
    router.error = 'save_failed';
  } finally {
    router.busy = false;
  }
}
