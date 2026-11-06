import { km, type Messages } from './km';
import { en } from './en';
export const locale = $state<{ language: 'km' | 'en' }>({ language: 'km' });
export function messages(): Messages {
  return locale.language === 'km' ? km : en;
}
export function digits(value: number | string): string {
  return locale.language === 'km'
    ? String(value).replace(/[0-9]/g, (c) => '០១២៣៤៥៦៧៨៩'[Number(c)])
    : String(value);
}
