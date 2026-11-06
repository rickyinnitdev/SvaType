import { api, type Profile, type Checkpoint } from '../api';
import { locale } from '../i18n/index.svelte';
import { navigate, router } from '../router.svelte';
import { loadPreferences } from './settings.svelte';
export const user = $state<{
  profiles: Profile[];
  profile: Profile | null;
  resume: Checkpoint | null;
  ready: boolean;
}>({ profiles: [], profile: null, resume: null, ready: false });
export async function initialize() {
  const [profiles, language] = await Promise.all([
    api.profiles(),
    api.getSetting('language'),
  ]);
  user.profiles = profiles;
  if (language === 'en' || language === 'km') locale.language = language;
  document.documentElement.lang = locale.language;
  router.screen = profiles.length ? 'profiles' : 'onboarding';
  user.ready = true;
}
export async function selectProfile(profile: Profile) {
  await api.touchProfile(profile.id);
  await api.setSetting('last_profile', String(profile.id));
  await loadPreferences(profile.id);
  user.profile = profile;
  await navigate('home');
}
