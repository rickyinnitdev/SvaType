import { readFileSync, writeFileSync } from 'node:fs';
const strings = JSON.parse(readFileSync('src/lib/i18n/installer.json', 'utf8'));
const bilingual = (value) => `${value.km} / ${value.en}`;
writeFileSync(
  'src-tauri/installer/English.nsh',
  '; Generated from src/lib/i18n/installer.json. Do not edit.\n' +
    Object.entries(strings)
      .filter(([key]) => !['welcome', 'instructions'].includes(key))
      .map(
        ([key, value]) =>
          `LangString ${key} $\{LANG_ENGLISH} "${bilingual(value)}"`,
      )
      .join('\n') +
    '\n',
);
writeFileSync(
  'src-tauri/installer/welcome.nsh',
  '; Generated from src/lib/i18n/installer.json.\n' +
    `!define MUI_WELCOMEPAGE_TITLE "${bilingual(strings.welcome)}"\n!define MUI_WELCOMEPAGE_TEXT "${bilingual(strings.instructions)}"\n`,
);
