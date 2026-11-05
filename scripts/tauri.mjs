import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { homedir } from 'node:os';
import { delimiter, join } from 'node:path';

// GUI-launched shells may not inherit rustup's PATH entry.
const cargoBin = join(homedir(), '.cargo', 'bin');
if (existsSync(cargoBin))
  process.env.PATH = cargoBin + delimiter + process.env.PATH;
const cli = join('node_modules', '@tauri-apps', 'cli', 'tauri.js');
const args = process.argv.slice(2);
if (args[0] === 'dev') args.splice(1, 0, '--no-dev-server');
const result = spawnSync(process.execPath, [cli, ...args], {
  stdio: 'inherit',
});
if (result.error) throw result.error;
process.exit(result.status ?? 1);
