import { readdirSync, readFileSync } from 'node:fs';
import { join, extname } from 'node:path';
import { fileURLToPath } from 'node:url';

export function violations(source) {
  // Remove comments, but retain URLs inside quoted code and CSS declarations.
  const text = source
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/<!--([\s\S]*?)-->/g, '')
    .replace(/^\s*\/\/.*$/gm, '');
  const errors = [];
  const urls = text.match(/https?:\/\/[^\s'"`<>;)]+/g) ?? [];
  for (const url of urls) {
    if (
      ![
        'http://ipc.localhost',
        'http://tauri.localhost',
        'http://www.w3.org/1999/xhtml',
        'http://www.w3.org/2000/svg',
        'http://www.w3.org/1998/Math/MathML',
      ].includes(url)
    ) {
      errors.push(`external URL: ${url}`);
    }
  }
  if (
    /\b(XMLHttpRequest|WebSocket)\b|navigator\s*\.\s*sendBeacon\b/.test(text)
  ) {
    errors.push('forbidden network API');
  }
  for (const match of text.matchAll(/\bfetch\s*\(\s*([^)]*)/g)) {
    if (!/^['"`](?:\/(?!\/)|\.\.?\/)/.test(match[1])) {
      errors.push('fetch must use a literal relative path');
    }
  }
  if (/(?:src|href)\s*=\s*['"]\/\/|@import\s+(?:url\()?['"]\/\//i.test(text)) {
    errors.push('protocol-relative external resource');
  }
  return errors;
}

function scan(path) {
  const entries = readdirSync(path, { withFileTypes: true });
  return entries.flatMap((entry) => {
    const file = join(path, entry.name);
    if (entry.isDirectory()) return scan(file);
    if (
      !['.rs', '.ts', '.js', '.svelte', '.html', '.css'].includes(extname(file))
    )
      return [];
    return violations(readFileSync(file, 'utf8')).map(
      (error) => `${file}: ${error}`,
    );
  });
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const errors = ['src', 'src-tauri/src', 'dist'].flatMap(scan);
  errors.push(...violations(readFileSync('index.html', 'utf8')));
  const cargo = readFileSync('src-tauri/Cargo.toml', 'utf8');
  if (/tauri-plugin-(http|updater|shell|opener)/.test(cargo))
    errors.push('forbidden plugin');
  if (errors.length) {
    console.error(errors.join('\n'));
    process.exitCode = 1;
  } else console.log('Offline check passed (sources and built assets).');
}
