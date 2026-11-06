import { invoke } from '@tauri-apps/api/core';
import type { Summary } from './engine';
export interface Profile {
  id: number;
  name: string;
  avatar: string;
  grade: number | null;
  path_id: string | null;
  layout_id: 'nida' | 'khmer_legacy';
  preferred_mode: 'learning' | 'typing';
  created_at: number;
  last_used_at: number | null;
  archived: boolean;
}
export type ProfileInput = Pick<
  Profile,
  'name' | 'avatar' | 'grade' | 'path_id' | 'layout_id' | 'preferred_mode'
>;
export interface SessionInput {
  uid: string;
  profile_id: number;
  mode: 'learning' | 'typing';
  kind: string;
  ref_id: string | null;
  layout_id: string;
  started_at: number;
  completed: boolean;
  target_text: string;
  summary: Summary;
  bookmark_step: number | null;
}
export interface HistoryRow {
  id: number;
  profile_id: number;
  mode: string;
  kind: string;
  ref_id: string | null;
  layout_id: string;
  started_at: number;
  duration_ms: number;
  active_ms: number;
  kpm: number;
  accuracy: number;
  stars: number | null;
  completed: boolean;
  text_sample: string;
}
export interface HistoryFilter {
  mode: string | null;
  kind: string | null;
  from: number | null;
  to: number | null;
}
export interface HistoryPage {
  rows: HistoryRow[];
  total: number;
  page: number;
  page_size: number;
}
export interface HistoryDetail {
  session: HistoryRow;
  target_text: string;
  summary: Summary;
}
export interface Trend {
  day: number;
  sessions: number;
  kpm: number;
  accuracy: number;
  duration_ms: number;
}
export interface Checkpoint {
  session: SessionInput;
  engine: string;
  remaining_ms: number | null;
}
export interface LessonProgress {
  lesson_id: string;
  best_stars: number;
  best_accuracy: number;
  attempts: number;
  completed_at: number | null;
  bookmark_step: number | null;
}
export interface Progress {
  lessons: LessonProgress[];
  words_learned: number;
  duration_ms: number;
}
export const api = {
  profiles: () => invoke<Profile[]>('profiles_list'),
  createProfile: (input: ProfileInput) =>
    invoke<Profile>('profiles_create', { input }),
  updateProfile: (id: number, input: ProfileInput) =>
    invoke<void>('profiles_update', { id, input }),
  touchProfile: (id: number) => invoke<void>('profile_touch', { id }),
  saveSession: (input: SessionInput) =>
    invoke<number>('session_save', { input }),
  checkpoint: (input: Checkpoint) =>
    invoke<void>('session_checkpoint', { input }),
  checkpoints: (profile: number) =>
    invoke<Checkpoint[]>('checkpoints_list', { profile }),
  discardCheckpoint: (uid: string) =>
    invoke<void>('checkpoint_discard', { uid }),
  history: (profile: number, filter: HistoryFilter, page: number) =>
    invoke<HistoryPage>('history_list', { profile, filter, page }),
  detail: (id: number) => invoke<HistoryDetail>('history_detail', { id }),
  deleteHistory: (id: number) => invoke<void>('history_delete', { id }),
  trend: (profile: number, from: number) =>
    invoke<Trend[]>('history_trend', { profile, from }),
  getSetting: (key: string) => invoke<string | null>('settings_get', { key }),
  setSetting: (key: string, value: string) =>
    invoke<void>('settings_set', { key, value }),
  progress: (profile: number) => invoke<Progress>('progress_get', { profile }),
  backup: (path: string) => invoke<void>('backup_create', { path }),
  restore: (path: string) => invoke<void>('backup_restore', { path }),
  exportCsv: (path: string, profile: number | null) =>
    invoke<void>('export_csv', { path, profile }),
  openDataFolder: () => invoke<void>('open_data_folder'),
  saveDialog: (defaultPath: string, extension: string) =>
    invoke<string | null>('plugin:dialog|save', {
      options: {
        defaultPath,
        filters: [{ name: extension, extensions: [extension] }],
      },
    }),
  openDialog: (extension: string) =>
    invoke<string | null>('plugin:dialog|open', {
      options: {
        multiple: false,
        directory: false,
        filters: [{ name: extension, extensions: [extension] }],
      },
    }),
};
