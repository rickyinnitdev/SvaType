// Development-only release budget report; never shipped in the app.
import { spawnSync } from 'node:child_process';
import { readFileSync, readdirSync, statSync, existsSync } from 'node:fs';
import { join } from 'node:path';
import { gzipSync } from 'node:zlib';
if (!process.argv.includes('--no-build')) {
  const result = spawnSync(process.execPath, ['scripts/tauri.mjs', 'build'], {
    stdio: 'inherit',
  });
  if (result.status !== 0) process.exit(result.status ?? 1);
}
const release = 'target/x86_64-pc-windows-msvc/release';
const files = (dir) =>
  readdirSync(dir, { withFileTypes: true }).flatMap((e) =>
    e.isDirectory() ? files(join(dir, e.name)) : [join(dir, e.name)],
  );
const size = (file) => statSync(file).size;
let failed = false;
function budget(label, bytes, limit) {
  console.log(
    `${bytes < limit ? 'PASS' : 'FAIL'} ${label}: ${bytes.toLocaleString()} bytes / <${limit.toLocaleString()}`,
  );
  if (bytes >= limit) failed = true;
}
const assets = files('dist');
const html = readFileSync('dist/index.html', 'utf8');
const entry = html.match(/src="([^"]+\.js)"/)?.[1];
if (!entry) throw new Error('Missing entry script');
const visited = new Set();
function initial(file) {
  if (visited.has(file)) return 0;
  visited.add(file);
  const code = readFileSync(file, 'utf8');
  let bytes = gzipSync(code).length;
  for (const match of code.matchAll(
    /(?:import|export)[^;]*?from\s*["']\.\/([^"']+)["']/g,
  ))
    bytes += initial(join('dist/assets', match[1]));
  return bytes;
}
budget(
  'Initial JavaScript (gzip)',
  initial(join('dist', entry.replace(/^\//, ''))),
  150000,
);
budget('WASM', size('dist/engine/khmer_bg.wasm'), 200000);
console.log(
  `CSS total: ${assets
    .filter((f) => f.endsWith('.css'))
    .reduce((n, f) => n + size(f), 0)
    .toLocaleString()} bytes`,
);
const exe = join(release, 'svatype.exe');
if (!existsSync(exe)) throw new Error('Missing release executable');
console.log(`Executable: ${size(exe).toLocaleString()} bytes`);
budget(
  'Installed application (EXE includes frontend; plus content)',
  size(exe) + files('content').reduce((n, f) => n + size(f), 0),
  30000000,
);
const installers = files(join(release, 'bundle/nsis')).filter(
  (f) => f.endsWith('.exe') && !f.includes('Full'),
);
if (!installers.length) throw new Error('Missing slim installer');
for (const installer of installers)
  budget('Slim installer', size(installer), 10000000);
if (failed) process.exitCode = 1;
