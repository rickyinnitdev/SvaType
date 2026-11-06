import { it, expect } from 'vitest';
import { toKhmerDigits, formatDate } from './format';
import { km } from './km';
import { en } from './en';
it('converts digits without changing signs and decimals', () => {
  expect(toKhmerDigits('12.5% / 0')).toBe('១២.៥% / ០');
});
it('keeps both dictionaries complete', () => {
  expect(Object.keys(km).sort()).toEqual(Object.keys(en).sort());
});
it('formats dates without Khmer ICU data', () => {
  const date = new Date(2026, 10, 6, 9, 5);
  expect(formatDate(date.getTime(), 'en')).toBe('2026-11-06 09:05');
  expect(formatDate(date.getTime(), 'km')).toBe('២០២៦-១១-០៦ ០៩:០៥');
  expect(formatDate(NaN, 'en')).toBe('—');
});
