// Development-only integration test against this app's local WebView debugger.
// Start npm run dev with --remote-debugging-port=9223 in WebView2 test args.
import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';
import { initSync, Layout } from '../src/lib/wasm/khmer.js';
initSync({ module: readFileSync('src/lib/wasm/khmer_bg.wasm') });
const target = JSON.parse(readFileSync('content/playground.json', 'utf8')).text;
const pages = await (await fetch('http://127.0.0.1:9223/json/list')).json();
const page = pages.find((p) => p.url.startsWith('http://tauri.localhost/'));
assert.ok(page, 'SvaType must be running with its local debugger');
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
    const p = pending.get(msg.id);
    pending.delete(msg.id);
    if (msg.error) p.reject(new Error(msg.error.message));
    else p.resolve(msg.result);
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
try {
  await call('Page.navigate', { url: 'http://tauri.localhost/?playground=1' });
  for (const layoutId of ['nida', 'khmer_legacy'])
    for (const flow of [false, true]) {
      await call('Page.reload');
      await evaluate(
        `new Promise((resolve,reject)=>{let attempts=0;const poll=()=>{if(document.querySelector('.capture'))resolve(true);else if(++attempts>200)reject('playground did not load');else setTimeout(poll,50);};poll();})`,
      );
      const layout = Layout.from_json(
        readFileSync(`content/layouts/${layoutId}.json`, 'utf8'),
      );
      const strokes = JSON.parse(layout.plan(target));
      layout.free();
      const result = await evaluate(`(async()=>{
   const selects=[...document.querySelectorAll('select')];
   selects[1].value=${JSON.stringify(layoutId)};selects[1].dispatchEvent(new Event('change',{bubbles:true}));
   await Promise.resolve();await Promise.resolve();
   if(${flow})document.querySelectorAll('.policy button')[1].click();
   await Promise.resolve();await Promise.resolve();
   const capture=document.querySelector('.capture');capture.focus();
   const invoke=window.__TAURI_INTERNALS__.invoke;let ipcCalls=0;
   window.__TAURI_INTERNALS__.invoke=(...args)=>{ipcCalls++;return invoke(...args);};
   const send=(code,layer='base')=>{capture.dispatchEvent(new KeyboardEvent('keydown',{code,key:code,bubbles:true,shiftKey:layer==='shift'||layer==='shift_altgr',ctrlKey:layer==='altgr'||layer==='shift_altgr',altKey:layer==='altgr'||layer==='shift_altgr',modifierAltGraph:layer==='altgr'||layer==='shift_altgr'}));};
   send('KeyA');await Promise.resolve();await Promise.resolve();
   const wrongHighlight=!!document.querySelector('.cluster.wrong');
   if(${flow}){send('Backspace');await Promise.resolve();await Promise.resolve();}
   const times=[];
   for(const stroke of ${JSON.stringify(strokes)}){const start=performance.now();send(stroke.code,stroke.layer);await Promise.resolve();await Promise.resolve();times.push(performance.now()-start);}
   times.sort((a,b)=>a-b);window.__TAURI_INTERNALS__.invoke=invoke;
   return {results:!!document.querySelector('.stats'),wrongHighlight,stats:document.querySelector('.stats')?.textContent,ipcCalls,p95:times[Math.floor(times.length*.95)],bodyHeight:document.documentElement.scrollHeight,viewportHeight:innerHeight};
  })()`);
      assert.equal(
        result.results,
        true,
        `${layoutId} ${flow ? 'flow' : 'stop'} must finish`,
      );
      assert.equal(
        result.wrongHighlight,
        true,
        'incorrect output must be marked',
      );
      assert.equal(result.ipcCalls, 0, 'keystrokes must not invoke IPC');
      assert.ok(result.p95 < 16, `input processing too slow: ${result.p95}`);
      console.log({ layoutId, flow, ...result });
    }
} finally {
  socket.close();
}
