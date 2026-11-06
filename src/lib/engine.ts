import init, {
  initSync,
  Layout,
  Session,
  segment as wasmSegment,
} from './wasm/khmer';

export type Layer = 'base' | 'shift' | 'altgr' | 'shift_altgr';
export type Policy = 'stop_on_error' | 'flow';
export interface Stroke {
  code: string;
  layer: Layer;
  produces: string;
  cluster_index: number;
  finger: string;
}
export interface Cluster {
  text: string;
  start: number;
  end: number;
  char_start: number;
  char_end: number;
}
export interface KeyResult {
  ignored: boolean;
  correct: boolean;
  produced: string;
  expected_next: Stroke | null;
  cluster_done: boolean;
  finished: boolean;
  cluster_index: number;
  error_position: number | null;
}
export interface KeyStats {
  code: string;
  layer: Layer;
  hits: number;
  errors: number;
  avg_hes_ms: number;
}
export interface Summary {
  duration_ms: number;
  active_ms: number;
  keystrokes: number;
  correct: number;
  errors: number;
  backspaces: number;
  kpm: number;
  wpm_equivalent: number;
  accuracy: number;
  per_key: KeyStats[];
  confusions: { expected: string; typed: string; count: number }[];
  clusters_done: number;
  clusters_per_minute: number;
  stars: number;
}
export interface LayoutData {
  id: string;
  name: { km: string; en: string };
  verified: boolean;
  keys: Record<
    string,
    {
      base: string | null;
      shift: string | null;
      altgr: string | null;
      shift_altgr: string | null;
      finger: string;
      row: number;
      scan_code: number;
    }
  >;
}

let ready: Promise<void> | undefined;
export function loadEngine(module?: WebAssembly.Module): Promise<void> {
  return (ready ??= (async () => {
    if (module) {
      initSync({ module });
    } else {
      await init({ module_or_path: fetch('/engine/khmer_bg.wasm') });
    }
  })());
}
export function segment(text: string): Cluster[] {
  return JSON.parse(wasmSegment(text)) as Cluster[];
}
export class TypingSession {
  private layout: Layout;
  private session: Session;
  constructor(
    target: string,
    layout: LayoutData,
    policy: Policy,
    strict = false,
  ) {
    this.layout = Layout.from_json(JSON.stringify(layout));
    try {
      this.session = new Session(
        target,
        this.layout,
        policy === 'flow',
        strict,
      );
    } catch (error) {
      this.layout.free();
      throw error;
    }
  }
  expected(): Stroke | null {
    return JSON.parse(this.session.expected()) as Stroke | null;
  }
  onKey(
    event: Pick<
      KeyboardEvent,
      | 'code'
      | 'shiftKey'
      | 'ctrlKey'
      | 'metaKey'
      | 'repeat'
      | 'getModifierState'
    >,
    timestamp = performance.now(),
  ): KeyResult {
    return JSON.parse(
      this.session.on_key(
        event.code,
        event.shiftKey,
        event.getModifierState('AltGraph'),
        event.ctrlKey,
        event.metaKey,
        event.repeat,
        timestamp,
      ),
    ) as KeyResult;
  }
  summary(): Summary {
    return JSON.parse(this.session.summary()) as Summary;
  }
  typed(): string {
    return this.session.typed();
  }
  free() {
    this.session.free();
    this.layout.free();
  }
}
