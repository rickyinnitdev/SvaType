import type { HistoryDetail } from '../api';
export const activity = $state<{
  target: string | null;
  kind: string;
  refId: string | null;
  durationMs: number;
  free: boolean;
}>({ target: null, kind: 'test', refId: null, durationMs: 0, free: false });
export function configureActivity(
  target: string,
  kind: string,
  durationMs = 0,
  refId: string | null = null,
) {
  Object.assign(activity, {
    target,
    kind,
    refId,
    durationMs,
    free: kind === 'free',
  });
}
export function retryActivity(detail: HistoryDetail) {
  configureActivity(
    detail.target_text,
    detail.session.kind,
    detail.session.kind === 'test' &&
      detail.session.mode === 'typing' &&
      detail.session.duration_ms >= 60_000
      ? Math.max(
          60_000,
          Math.round(detail.session.duration_ms / 60_000) * 60_000,
        )
      : 0,
    detail.session.ref_id,
  );
}
