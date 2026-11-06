import { api } from '../api';
export interface Preferences {
  show_keyboard: boolean;
  show_hands: boolean;
  sound: boolean;
  font_size: 's' | 'm' | 'l' | 'xl';
  low_end: boolean;
  reduce_motion: boolean;
  theme: 'light' | 'dark' | 'contrast';
}
const defaults: Preferences = {
  show_keyboard: true,
  show_hands: true,
  sound: false,
  font_size: 'm',
  low_end: navigator.hardwareConcurrency <= 4,
  reduce_motion: matchMedia('(prefers-reduced-motion: reduce)').matches,
  theme: 'light',
};
export const preferences = $state<Preferences>({ ...defaults });
export function applyPreferences() {
  document.documentElement.dataset.theme = preferences.theme;
  document.documentElement.dataset.lowEnd = String(preferences.low_end);
  document.documentElement.dataset.reduceMotion = String(
    preferences.reduce_motion,
  );
  const sizes = { s: [18, 30], m: [18, 36], l: [20, 42], xl: [24, 48] }[
    preferences.font_size
  ];
  document.documentElement.style.setProperty('--font-ui', `${sizes[0]}px`);
  document.documentElement.style.setProperty('--font-target', `${sizes[1]}px`);
}
export async function loadPreferences(profile: number) {
  Object.assign(preferences, defaults);
  for (const key of Object.keys(preferences) as (keyof Preferences)[]) {
    const value = await api.getSetting(`p:${profile}:${key}`);
    if (value === null) continue;
    if (typeof preferences[key] === 'boolean') {
      Object.assign(preferences, { [key]: value === 'true' });
    } else if (
      key === 'theme' &&
      ['light', 'dark', 'contrast'].includes(value)
    ) {
      preferences.theme = value as Preferences['theme'];
    } else if (key === 'font_size' && ['s', 'm', 'l', 'xl'].includes(value)) {
      preferences.font_size = value as Preferences['font_size'];
    }
  }
  applyPreferences();
}
export async function savePreference<K extends keyof Preferences>(
  profile: number,
  key: K,
  value: Preferences[K],
) {
  await api.setSetting(`p:${profile}:${key}`, String(value));
  preferences[key] = value;
  applyPreferences();
}
