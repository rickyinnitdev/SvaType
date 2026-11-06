export class LeaveGuards {
  private callbacks = new Map<symbol, () => Promise<void>>();
  register(callback: () => Promise<void>) {
    const id = Symbol();
    this.callbacks.set(id, callback);
    return () => {
      this.callbacks.delete(id);
    };
  }
  async flush() {
    // Nested screens finish their writes before their parent locks/unloads.
    for (const callback of [...this.callbacks.values()].reverse())
      await callback();
  }
}
