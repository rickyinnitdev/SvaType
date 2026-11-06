import { it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
const css = readFileSync('src/styles/tokens.css', 'utf8');
function luminance(hex: string) {
  const value =
    hex.length === 4
      ? hex
          .slice(1)
          .split('')
          .map((c) => c + c)
          .join('')
      : hex.slice(1);
  const channels = [0, 2, 4]
    .map((i) => parseInt(value.slice(i, i + 2), 16) / 255)
    .map((v) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4));
  return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
}
function contrast(a: string, b: string) {
  const values = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (values[0] + 0.05) / (values[1] + 0.05);
}
// Independent WCAG text-contrast criterion, checked against the actual theme tokens.
for (const theme of ['light', 'dark', 'contrast'])
  it(`${theme} text pairs meet 4.5:1 without rounding`, () => {
    const selector =
      theme === 'light' ? ':root' : `\\[data-theme='${theme}'\\]`;
    const block = css.match(new RegExp(`${selector}\\s*\\{([^}]+)\\}`))?.[1];
    expect(block).toBeTruthy();
    const tokens = Object.fromEntries(
      [...block!.matchAll(/--([a-z-]+):\s*(#[0-9a-f]+);/gi)].map((match) => [
        match[1],
        match[2],
      ]),
    );
    for (const [fg, bg] of [
      ['foreground', 'background'],
      ['foreground', 'surface'],
      ['muted', 'surface'],
      ['on-primary', 'primary'],
      ['error', 'error-soft'],
    ])
      expect(
        contrast(tokens[fg], tokens[bg]),
        `${fg}/${bg}`,
      ).toBeGreaterThanOrEqual(4.5);
  });
