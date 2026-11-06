export type Screen =
  | 'profiles'
  | 'onboarding'
  | 'home'
  | 'practice'
  | 'history'
  | 'result'
  | 'settings';
export const router = $state<{
  screen: Screen;
  resultId: number | null;
  busy: boolean;
  error: string | null;
}>({ screen: 'profiles', resultId: null, busy: false, error: null });
let guard: (() => Promise<void>) | undefined;
export function registerLeaveGuard(next: () => Promise<void>) {
  guard = next;
  return () => {
    if (guard === next) guard = undefined;
  };
}
export async function navigate(screen: Screen, resultId: number | null = null) {
  if (router.busy) return;
  router.busy = true;
  router.error = null;
  try {
    await guard?.();
    router.resultId = resultId;
    router.screen = screen;
  } catch {
    router.error = 'save_failed';
  } finally {
    router.busy = false;
  }
}
