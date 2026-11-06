// Local development test transport. Never bundled with the app.
import assert from 'node:assert/strict';
export async function connectWebView() {
  const pages = await (await fetch('http://127.0.0.1:9223/json/list')).json();
  const page = pages.find((p) => p.url.startsWith('http://tauri.localhost/'));
  assert.ok(page, 'SvaType local debugger must be running');
  const socket = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => {
    socket.onopen = resolve;
    socket.onerror = reject;
  });
  let id = 0;
  const pending = new Map();
  socket.onmessage = (event) => {
    const msg = JSON.parse(event.data);
    if (msg.id) {
      const entry = pending.get(msg.id);
      if (!entry) return;
      pending.delete(msg.id);
      if (msg.error) entry.reject(new Error(msg.error.message));
      else entry.resolve(msg.result);
    }
  };
  function call(method, params = {}) {
    return new Promise((resolve, reject) => {
      const seq = ++id;
      pending.set(seq, { resolve, reject });
      socket.send(JSON.stringify({ id: seq, method, params }));
    });
  }
  async function evaluate(expression) {
    const result = await call('Runtime.evaluate', {
      expression,
      awaitPromise: true,
      returnByValue: true,
    });
    if (result.exceptionDetails)
      throw new Error(JSON.stringify(result.exceptionDetails));
    return result.result.value;
  }
  async function wait(expression) {
    return evaluate(
      `new Promise((resolve,reject)=>{let n=0;const poll=()=>{if(${expression})resolve(true);else if(++n>200)reject('UI state timed out: '+${JSON.stringify(expression)});else setTimeout(poll,50);};poll();})`,
    );
  }
  return { call, evaluate, wait, close: () => socket.close() };
}
