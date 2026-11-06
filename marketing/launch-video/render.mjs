// Renders scene.html frame-by-frame with headless Chrome (DevTools protocol, no npm deps)
// and encodes it with ffmpeg together with the synthesized soundtrack.
//
//   node marketing/launch-video/render.mjs              -> out/SvaType-launch.mp4
//   node marketing/launch-video/render.mjs --stills 1,6,13   -> out/still-<t>.png
//
// Everything runs on this machine; the HTTP server listens on 127.0.0.1 only.
import { createServer } from 'node:http';
import { readFile, mkdir, writeFile, mkdtemp, rm } from 'node:fs/promises';
import { spawn, spawnSync } from 'node:child_process';
import { join, extname, resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { tmpdir } from 'node:os';
import { synthesize } from './audio.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, '../..');
const outDir = join(here, 'out');
const CHROME =
  process.env.CHROME || 'C:/Program Files/Google/Chrome/Application/chrome.exe';
const FFMPEG = process.env.FFMPEG || 'ffmpeg';
const W = 1920,
  H = 1080;
const args = process.argv.slice(2);
const stills = args.includes('--stills')
  ? args[args.indexOf('--stills') + 1].split(',').map(Number)
  : null;

const TYPES = {
  '.html': 'text/html',
  '.js': 'text/javascript',
  '.mjs': 'text/javascript',
  '.json': 'application/json',
  '.woff2': 'font/woff2',
  '.svg': 'image/svg+xml',
};
const server = createServer(async (req, res) => {
  const path = resolve(
    root,
    '.' + decodeURIComponent(new URL(req.url, 'http://x').pathname),
  );
  if (!path.startsWith(root)) return res.writeHead(403).end();
  try {
    const body = await readFile(path);
    res
      .writeHead(200, {
        'content-type': TYPES[extname(path)] || 'application/octet-stream',
      })
      .end(body);
  } catch {
    res.writeHead(404).end();
  }
});
await new Promise((r) => server.listen(0, '127.0.0.1', r));
const base = `http://127.0.0.1:${server.address().port}`;

const port = 9300 + Math.floor(Math.random() * 500);
const profile = await mkdtemp(join(tmpdir(), 'svatype-video-chrome-'));
const chrome = spawn(
  CHROME,
  [
    '--headless=new',
    `--remote-debugging-port=${port}`,
    `--user-data-dir=${profile}`,
    `--window-size=${W},${H}`,
    '--force-device-scale-factor=1',
    '--hide-scrollbars',
    '--mute-audio',
    '--no-first-run',
    '--no-default-browser-check',
    '--disable-background-networking',
    '--disable-extensions',
    '--font-render-hinting=none',
    'about:blank',
  ],
  { stdio: 'ignore' },
);

async function waitJson(path, method = 'GET') {
  for (let i = 0; i < 100; i++) {
    try {
      return await (
        await fetch(`http://127.0.0.1:${port}${path}`, { method })
      ).json();
    } catch {
      await new Promise((r) => setTimeout(r, 150));
    }
  }
  throw new Error('Chrome did not start');
}
await waitJson('/json/version');
const target = await waitJson('/json/new?about:blank', 'PUT');
const ws = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((r, j) => {
  ws.onopen = r;
  ws.onerror = j;
});
let id = 0;
const pending = new Map();
ws.onmessage = (m) => {
  const msg = JSON.parse(m.data);
  if (msg.id && pending.has(msg.id)) {
    const { r, j } = pending.get(msg.id);
    pending.delete(msg.id);
    if (msg.error) j(new Error(msg.error.message));
    else r(msg.result);
  }
};
const send = (method, params = {}) =>
  new Promise((r, j) => {
    const n = ++id;
    pending.set(n, { r, j });
    ws.send(JSON.stringify({ id: n, method, params }));
  });
const evaluate = async (expression) => {
  const res = await send('Runtime.evaluate', {
    expression,
    awaitPromise: true,
    returnByValue: true,
  });
  if (res.exceptionDetails)
    throw new Error(
      res.exceptionDetails.exception?.description || res.exceptionDetails.text,
    );
  return res.result.value;
};

await send('Emulation.setDeviceMetricsOverride', {
  width: W,
  height: H,
  deviceScaleFactor: 1,
  mobile: false,
});
await send('Page.enable');
await send('Page.navigate', {
  url: `${base}/marketing/launch-video/scene.html`,
});
for (let i = 0; i < 100; i++) {
  try {
    if (await evaluate('window.ready && window.ready.then(() => true)')) break;
  } catch {
    /* page still loading */
  }
  await new Promise((r) => setTimeout(r, 100));
}
const [FPS, DURATION, CUES] = await evaluate(
  '[window.FPS, window.DURATION, window.CUES]',
);
await mkdir(outDir, { recursive: true });

async function frame(t, format = 'png') {
  await evaluate(`renderAt(${t}); true`);
  const { data } = await send('Page.captureScreenshot', {
    format,
    ...(format === 'jpeg' ? { quality: 95 } : {}),
    clip: { x: 0, y: 0, width: W, height: H, scale: 1 },
    captureBeyondViewport: false,
  });
  return Buffer.from(data, 'base64');
}

try {
  if (stills) {
    for (const t of stills)
      await writeFile(join(outDir, `still-${t}.png`), await frame(t));
    console.log(`Wrote ${stills.length} stills to ${outDir}`);
  } else {
    const wav = join(outDir, 'soundtrack.wav');
    await writeFile(join(outDir, 'cues.json'), JSON.stringify(CUES, null, 1));
    await writeFile(wav, synthesize(CUES, DURATION));
    console.log(`Soundtrack: ${CUES.length} cues -> ${wav}`);
    const mp4 = join(outDir, 'SvaType-launch.mp4');
    const ff = spawn(
      FFMPEG,
      [
        '-y',
        '-loglevel',
        'error',
        '-f',
        'image2pipe',
        '-framerate',
        String(FPS),
        '-c:v',
        'png',
        '-i',
        '-',
        '-i',
        wav,
        '-map',
        '0:v',
        '-map',
        '1:a',
        '-c:v',
        'libx264',
        '-preset',
        'slow',
        '-crf',
        '17',
        '-pix_fmt',
        'yuv420p',
        '-profile:v',
        'high',
        '-c:a',
        'aac',
        '-b:a',
        '192k',
        '-movflags',
        '+faststart',
        '-shortest',
        mp4,
      ],
      { stdio: ['pipe', 'inherit', 'inherit'] },
    );
    const total = Math.round(FPS * DURATION);
    const started = Date.now();
    for (let f = 0; f < total; f++) {
      const png = await frame(f / FPS);
      if (!ff.stdin.write(png))
        await new Promise((r) => ff.stdin.once('drain', r));
      if (f % 60 === 0)
        process.stdout.write(
          `\rframe ${f}/${total}  ${((Date.now() - started) / 1000).toFixed(0)}s`,
        );
    }
    ff.stdin.end();
    const code = await new Promise((r) => ff.on('close', r));
    if (code !== 0) throw new Error('ffmpeg failed: ' + code);
    console.log(`\nVideo: ${mp4}`);
  }
} finally {
  ws.close();
  // Chrome spawns child processes; end the whole tree so nothing keeps the profile locked.
  if (process.platform === 'win32')
    spawnSync('taskkill', ['/PID', String(chrome.pid), '/T', '/F'], {
      stdio: 'ignore',
    });
  else chrome.kill();
  await rm(profile, { recursive: true, force: true }).catch(() => {});
  server.close();
}
