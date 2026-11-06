// Build-time downloads are allowed; the resulting installers need no network.
import { spawnSync } from 'node:child_process';
import {
  mkdirSync,
  copyFileSync,
  readdirSync,
  cpSync,
  writeFileSync,
  readFileSync,
} from 'node:fs';
import { createHash } from 'node:crypto';
import { join, resolve } from 'node:path';
function run(args) {
  const result = spawnSync(process.execPath, args, { stdio: 'inherit' });
  if (result.status !== 0) process.exit(result.status ?? 1);
}
run(['scripts/installer-strings.mjs']);
const release = 'target/x86_64-pc-windows-msvc/release';
const output = 'target/packages';
mkdirSync(output, { recursive: true });
const bundle = () =>
  readdirSync(join(release, 'bundle/nsis')).find((name) =>
    name.endsWith('-setup.exe'),
  );
if (!process.argv.includes('--portable-only')) {
  run(['scripts/tauri.mjs', 'build']);
  copyFileSync(
    join(release, 'bundle/nsis', bundle()),
    join(output, 'SvaType-Setup-x64.exe'),
  );
  run([
    'scripts/tauri.mjs',
    'bundle',
    '--bundles',
    'nsis',
    '--config',
    'src-tauri/tauri.full.conf.json',
  ]);
  copyFileSync(
    join(release, 'bundle/nsis', bundle()),
    join(output, 'SvaType-Setup-Full-x64.exe'),
  );
  copyFileSync(
    join(output, 'SvaType-Setup-x64.exe'),
    join(release, 'bundle/nsis', bundle()),
  );
}
// A fresh staging folder avoids accidentally putting learner data into a ZIP.
const stage = join(release, `package-${Date.now()}`, 'SvaType-Portable-x64');
mkdirSync(stage, { recursive: true });
copyFileSync(join(release, 'svatype.exe'), join(stage, 'SvaType.exe'));
cpSync('content', join(stage, 'content'), { recursive: true });
writeFileSync(join(stage, 'portable.txt'), 'SvaType portable mode\r\n');
copyFileSync('docs/PORTABLE.txt', join(stage, 'README.txt'));
const archive = resolve(output, 'SvaType-Portable-x64.zip');
const zip = spawnSync(
  'powershell.exe',
  [
    '-NoProfile',
    '-Command',
    `Compress-Archive -LiteralPath '${resolve(stage).replaceAll("'", "''")}' -DestinationPath '${archive.replaceAll("'", "''")}' -Force`,
  ],
  { stdio: 'inherit', windowsHide: true },
);
if (zip.status !== 0) process.exit(zip.status ?? 1);
const artifacts = [
  'SvaType-Setup-x64.exe',
  'SvaType-Setup-Full-x64.exe',
  'SvaType-Portable-x64.zip',
];
writeFileSync(
  join(output, 'SHA256SUMS.txt'),
  artifacts
    .map(
      (name) =>
        `${createHash('sha256')
          .update(readFileSync(join(output, name)))
          .digest('hex')}  ${name}`,
    )
    .join('\n') + '\n',
);
console.log(`Packages: ${resolve(output)}`);
