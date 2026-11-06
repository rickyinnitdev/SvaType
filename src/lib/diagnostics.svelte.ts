export const diagnostics = $state({ visible: false, revision: 0 });
const samples: number[] = [];
export function recordLatency(ms: number) {
  if (samples.length === 200) samples.shift();
  samples.push(ms);
  if (diagnostics.visible) diagnostics.revision++;
}
export function latency() {
  const sorted = [...samples].sort((a, b) => a - b);
  return {
    count: sorted.length,
    p50: sorted[Math.floor((sorted.length - 1) * 0.5)] ?? 0,
    p95: sorted[Math.floor((sorted.length - 1) * 0.95)] ?? 0,
  };
}
