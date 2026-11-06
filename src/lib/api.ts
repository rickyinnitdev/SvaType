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
  best_kpm: number;
  recent_duration_ms: number;
  duration_ms: number;
}
export interface CustomText {
  id: number;
  title: string;
  body: string;
  created_at: number;
}
export interface Analysis {
  keys: Summary['per_key'];
  weak: { code: string; score: number }[];
  confusions: Summary['confusions'];
  slowest: Summary['per_key'];
  heat: Record<string, number>;
}
export interface Localized {
  km: string;
  en: string;
}
export interface LessonIndex {
  id: string;
  title: Localized;
  reviewed: boolean;
}
export interface PathIndex {
  id: string;
  title: Localized;
  audience: 'grade' | 'skill';
}
export interface ContentIndex {
  paths: PathIndex[];
  lessons: LessonIndex[];
}
export interface LearningPath extends PathIndex {
  units: { id: string; title: Localized; lessons: string[] }[];
}
export interface LessonStep {
  type:
    | 'explain'
    | 'demo'
    | 'find_key'
    | 'type_chars'
    | 'type_clusters'
    | 'type_words'
    | 'type_sentence'
    | 'challenge';
  text?: string | Localized;
  items?: string[];
  keys?: string[];
  strict?: boolean;
  policy?: 'stop_on_error' | 'flow';
  min_accuracy?: number;
  image?: string;
}
export interface Lesson extends LessonIndex {
  version: number;
  focus_keys: string[];
  steps: LessonStep[];
  bonus_kpm?: number;
}
export interface LessonDraft {
  profile_id: number;
  lesson_id: string;
  uid: string;
  step: number;
  summaries: Summary[];
  texts: string[];
  started_at: number;
  layout_id: string;
}
export interface VocabWord {
  id: string;
  word: string;
  level: number;
  image: string | null;
  emoji: string;
  audio: string | null;
  meaning_km: string;
  meaning_en: string;
  example_km: string;
  grade: number[];
  reviewed: boolean;
}
export interface VocabCategory {
  id: string;
  title: Localized;
  icon: string;
  words: VocabWord[];
}
export const api = {
  vocabIndex: () =>
    invoke<{ categories: Omit<VocabCategory, 'words'>[] }>(
      'content_vocab_index',
    ),
  vocab: (category: string) =>
    invoke<VocabCategory>('content_vocab', { category }),
  contentIndex: () => invoke<ContentIndex>('content_index'),
  path: (id: string) => invoke<LearningPath>('content_path', { id }),
  lesson: (id: string) => invoke<Lesson>('content_lesson', { id }),
  lessonDraft: (profile: number, lesson: string) =>
    invoke<LessonDraft | null>('lesson_draft_get', { profile, lesson }),
  saveLessonDraft: (draft: LessonDraft) =>
    invoke<void>('lesson_progress_set', { draft }),
  saveLessonSession: (
    profile: number,
    lesson: string,
    extra: Summary | null = null,
    text: string | null = null,
    completed = false,
  ) =>
    invoke<number | null>('lesson_session_save', {
      profile,
      lesson,
      extra,
      text,
      completed,
    }),
  learningOptions: (profile: number) =>
    invoke<{ unlock_all: boolean }>('learning_options', { profile }),
  customTexts: (profile: number) =>
    invoke<CustomText[]>('custom_text_list', { profile }),
  saveCustomText: (
    profile: number,
    title: string,
    body: string,
    layoutId: string,
  ) =>
    invoke<number>('custom_text_save', {
      input: { profile, title, body, layout_id: layoutId },
    }),
  deleteCustomText: (profile: number, id: number) =>
    invoke<void>('custom_text_delete', { profile, id }),
  analysis: (profile: number) =>
    invoke<Analysis>('analysis_get', { profile, n: 50 }),
  practiceText: (
    profile: number,
    level: number,
    kind: string,
    weak = false,
    long = false,
    seed = Date.now(),
  ) =>
    invoke<string>('practice_text', { profile, level, kind, weak, long, seed }),
  drill: (profile: number, seed = Date.now()) =>
    invoke<string>('drill_generate', { profile, seed }),
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
