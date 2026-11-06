/** One scheduler for all games. No scheduled work while paused or disposed. */
export class GameLoop {
  private frame: number | null = null;
  private previous: number | null = null;
  private rendered: number | null = null;
  private running = false;
  constructor(
    private update: (deltaMs: number) => void,
    private lowEnd: boolean,
    private paused: () => void = () => {},
    private request: (callback: FrameRequestCallback) => number = (callback) =>
      requestAnimationFrame(callback),
    private cancel: (id: number) => void = (id) => cancelAnimationFrame(id),
  ) {
    if (typeof document !== 'undefined')
      document.addEventListener('visibilitychange', this.visibility);
    if (typeof window !== 'undefined')
      window.addEventListener('blur', this.pause);
  }
  private visibility = () => {
    if (document.hidden) this.pause();
  };
  start() {
    if (this.running) return true;
    if (
      typeof document !== 'undefined' &&
      (document.hidden || !document.hasFocus())
    )
      return false;
    this.running = true;
    this.previous = null;
    this.rendered = null;
    this.frame = this.request(this.tick);
    return true;
  }
  pause = () => {
    this.running = false;
    if (this.frame !== null) this.cancel(this.frame);
    this.frame = null;
    this.previous = null;
    this.rendered = null;
    this.paused();
  };
  private tick = (time: number) => {
    this.frame = null;
    if (!this.running) return;
    const interval = this.lowEnd ? 1000 / 30 : 1000 / 60;
    const dt = this.previous === null ? 0 : time - this.previous;
    this.previous = time;
    if (this.rendered === null || time - this.rendered >= interval - 0.2) {
      this.update(
        Math.min(
          100,
          dt +
            (this.rendered === null
              ? 0
              : Math.max(0, time - this.rendered - dt)),
        ),
      );
      this.rendered = time;
    }
    if (this.running) this.frame = this.request(this.tick);
  };
  dispose() {
    this.pause();
    if (typeof document !== 'undefined')
      document.removeEventListener('visibilitychange', this.visibility);
    if (typeof window !== 'undefined')
      window.removeEventListener('blur', this.pause);
  }
}
