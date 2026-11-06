CREATE TABLE profiles (
  id INTEGER PRIMARY KEY, name TEXT NOT NULL, avatar TEXT NOT NULL,
  grade INTEGER, path_id TEXT, layout_id TEXT NOT NULL DEFAULT 'nida',
  preferred_mode TEXT NOT NULL DEFAULT 'learning',
  created_at INTEGER NOT NULL, last_used_at INTEGER, archived INTEGER NOT NULL DEFAULT 0);
CREATE TABLE sessions (
  id INTEGER PRIMARY KEY, profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  mode TEXT NOT NULL, kind TEXT NOT NULL, ref_id TEXT, layout_id TEXT NOT NULL,
  started_at INTEGER NOT NULL, duration_ms INTEGER NOT NULL, active_ms INTEGER NOT NULL,
  keystrokes INTEGER NOT NULL, correct INTEGER NOT NULL, errors INTEGER NOT NULL,
  backspaces INTEGER NOT NULL, kpm REAL NOT NULL, accuracy REAL NOT NULL,
  stars INTEGER, completed INTEGER NOT NULL, text_sample TEXT);
CREATE INDEX idx_sessions_profile_time ON sessions(profile_id, started_at DESC);
CREATE TABLE session_keys (
  session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  key_code TEXT NOT NULL, layer TEXT NOT NULL, hits INTEGER NOT NULL,
  errors INTEGER NOT NULL, avg_hes_ms REAL NOT NULL,
  PRIMARY KEY(session_id,key_code,layer)) WITHOUT ROWID;
CREATE TABLE session_confusions (
  session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  expected TEXT NOT NULL, typed TEXT NOT NULL, count INTEGER NOT NULL,
  PRIMARY KEY(session_id,expected,typed)) WITHOUT ROWID;
CREATE TABLE lesson_progress (
  profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  lesson_id TEXT NOT NULL, best_stars INTEGER NOT NULL, best_accuracy REAL NOT NULL,
  attempts INTEGER NOT NULL, completed_at INTEGER, bookmark_step INTEGER,
  PRIMARY KEY(profile_id,lesson_id)) WITHOUT ROWID;
CREATE TABLE words_learned (
  profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  word_id TEXT NOT NULL, times_typed INTEGER NOT NULL, best_accuracy REAL NOT NULL,
  first_at INTEGER NOT NULL, PRIMARY KEY(profile_id,word_id)) WITHOUT ROWID;
CREATE TABLE custom_texts (
  id INTEGER PRIMARY KEY, profile_id INTEGER REFERENCES profiles(id) ON DELETE CASCADE,
  title TEXT NOT NULL, body TEXT NOT NULL, created_at INTEGER NOT NULL);
CREATE TABLE achievements (
  profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  achievement_id TEXT NOT NULL, earned_at INTEGER NOT NULL,
  PRIMARY KEY(profile_id,achievement_id)) WITHOUT ROWID;
CREATE TABLE assignments (
  id INTEGER PRIMARY KEY, profile_id INTEGER REFERENCES profiles(id) ON DELETE CASCADE,
  lesson_id TEXT, path_id TEXT, created_at INTEGER NOT NULL);
CREATE TABLE content_overrides (
  id INTEGER PRIMARY KEY, kind TEXT NOT NULL, category TEXT, json TEXT NOT NULL,
  created_at INTEGER NOT NULL, deleted INTEGER NOT NULL DEFAULT 0);
CREATE TABLE settings (key TEXT PRIMARY KEY,value TEXT NOT NULL) WITHOUT ROWID;
CREATE TABLE schema_version(version INTEGER NOT NULL);
INSERT INTO schema_version VALUES(1);
