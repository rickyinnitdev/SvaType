import { mkdirSync, writeFileSync } from 'node:fs';
mkdirSync('content/avatars', { recursive: true });
const animals = [
  [
    'cat',
    '#e9b94c',
    '<path d="M13 24 10 8 26 20M38 20 54 8 51 25"/>',
    '<path d="m29 37 3 3 3-3zm-9 4-12-2m12 7L8 48m36-7 12-2m-12 7 12 2"/>',
  ],
  [
    'bear',
    '#b48662',
    '<circle cx="15" cy="18" r="10"/><circle cx="49" cy="18" r="10"/>',
    '<ellipse cx="32" cy="41" rx="13" ry="10" fill="#f6e4cd"/><path d="m28 38 4 4 4-4z"/>',
  ],
  [
    'rabbit',
    '#d7cde9',
    '<ellipse cx="21" cy="16" rx="7" ry="15"/><ellipse cx="43" cy="16" rx="7" ry="15"/>',
    '<path d="m29 38 3 3 3-3z"/>',
  ],
  [
    'fox',
    '#e29862',
    '<path d="M13 27 8 6 28 20M36 20 56 6 51 27"/>',
    '<path d="m12 36 20 19 20-19-20 8z" fill="#fff4df"/><path d="m28 42 4 4 4-4z"/>',
  ],
  [
    'panda',
    '#d7e2df',
    '<circle cx="15" cy="18" r="10" fill="#283f3b"/><circle cx="49" cy="18" r="10" fill="#283f3b"/>',
    '<ellipse cx="22" cy="32" rx="8" ry="10" fill="#283f3b"/><ellipse cx="42" cy="32" rx="8" ry="10" fill="#283f3b"/><path d="m28 41 4 4 4-4z"/>',
  ],
  [
    'frog',
    '#94b894',
    '<circle cx="19" cy="19" r="11"/><circle cx="45" cy="19" r="11"/>',
    '<path d="M20 42q12 11 24 0" fill="none" stroke="#283f3b" stroke-width="3"/>',
  ],
  [
    'owl',
    '#baa2ce',
    '<path d="m12 26-2-16 18 9m8 0 18-9-2 16"/>',
    '<circle cx="22" cy="31" r="11" fill="#fff4df"/><circle cx="42" cy="31" r="11" fill="#fff4df"/><path d="m28 39 4 7 4-7z" fill="#be7b24"/>',
  ],
  [
    'elephant',
    '#99b7cb',
    '<ellipse cx="13" cy="33" rx="12" ry="17"/><ellipse cx="51" cy="33" rx="12" ry="17"/>',
    '<path d="M27 35v18q5 10 13 0" fill="none" stroke="#5c8da7" stroke-width="9" stroke-linecap="round"/>',
  ],
  [
    'tiger',
    '#e9b94c',
    '<circle cx="16" cy="17" r="9"/><circle cx="48" cy="17" r="9"/>',
    '<path d="m14 29 8 3-8 3m36-6-8 3 8 3M27 15l5 10 5-10" fill="#5b4732"/><path d="m29 40 3 3 3-3z"/>',
  ],
  [
    'monkey',
    '#bd997a',
    '<circle cx="11" cy="33" r="10"/><circle cx="53" cy="33" r="10"/>',
    '<ellipse cx="32" cy="36" rx="21" ry="18" fill="#f6e4cd"/><path d="M25 44q7 6 14 0" fill="none" stroke="#5b4732" stroke-width="2"/>',
  ],
  [
    'bird',
    '#8eb5c9',
    '<path d="M27 20 25 5l9 10 8-9-1 15"/>',
    '<path d="m25 38 7 10 7-10z" fill="#be7b24"/>',
  ],
  [
    'mouse',
    '#c0b8ca',
    '<circle cx="12" cy="18" r="12"/><circle cx="52" cy="18" r="12"/>',
    '<path d="m28 39 4 4 4-4z" fill="#b75e76"/><path d="m21 42-13-2m13 7L8 49m35-7 13-2m-13 7 13 2" fill="none" stroke="#5b4732" stroke-width="1.5"/>',
  ],
];
for (const [i, [, color, ears, detail]] of animals.entries())
  writeFileSync(
    `content/avatars/${i}.svg`,
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><g fill="${color}">${ears}<ellipse cx="32" cy="35" rx="23" ry="23"/>${detail}</g><g fill="#172c2b"><circle cx="23" cy="32" r="2.5"/><circle cx="41" cy="32" r="2.5"/></g></svg>\n`,
  );
writeFileSync(
  'content/avatars.json',
  JSON.stringify(
    {
      reviewed: false,
      avatars: animals.map(([name], id) => ({
        id: String(id),
        name,
        file: `${id}.svg`,
      })),
    },
    null,
    2,
  ) + '\n',
);
