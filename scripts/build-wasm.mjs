import { spawnSync } from 'node:child_process';
import { homedir } from 'node:os';
import { join, delimiter } from 'node:path';
import {
  mkdirSync,
  statSync,
  readFileSync,
  writeFileSync,
  copyFileSync,
} from 'node:fs';

process.env.PATH =
  join(homedir(), '.cargo', 'bin') + delimiter + process.env.PATH;
function run(command, args) {
  const result = spawnSync(command, args, { stdio: 'inherit' });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}
run('cargo', [
  'build',
  '-p',
  'khmer_wasm',
  '--target',
  'wasm32-unknown-unknown',
  '--profile',
  'wasm-release',
]);
mkdirSync('src/lib/wasm', { recursive: true });
run('wasm-bindgen', [
  'target/wasm32-unknown-unknown/wasm-release/khmer_wasm.wasm',
  '--target',
  'web',
  '--out-dir',
  'src/lib/wasm',
  '--out-name',
  'khmer',
]);
// The app supplies a Response from one fixed local path. Remove the generated
// arbitrary-URL fetch fallback so it cannot become an accidental remote loader.
const gluePath = 'src/lib/wasm/khmer.js';
const glue = readFileSync(gluePath, 'utf8')
  .replace(
    "module_or_path = new URL('khmer_bg.wasm', import.meta.url);",
    "throw new Error('Pass the bundled WASM Response');",
  )
  .replace(
    'module_or_path = fetch(module_or_path);',
    "throw new Error('Pass the bundled WASM Response');",
  );
if (/\bfetch\s*\(/.test(glue))
  throw new Error('Generated loader changed: review its network fallback');
writeFileSync(gluePath, glue);
run(process.execPath, [
  'node_modules/binaryen/bin/wasm-opt',
  'src/lib/wasm/khmer_bg.wasm',
  '-Oz',
  '--enable-bulk-memory',
  '--enable-nontrapping-float-to-int',
  '-o',
  'src/lib/wasm/khmer_bg.wasm',
]);
const size = statSync('src/lib/wasm/khmer_bg.wasm').size;
console.log(`Khmer WASM: ${size} bytes (budget: <200000)`);
if (size >= 200_000) throw new Error('WASM exceeds its size budget');
mkdirSync('public/engine', { recursive: true });
copyFileSync('src/lib/wasm/khmer_bg.wasm', 'public/engine/khmer_bg.wasm');
