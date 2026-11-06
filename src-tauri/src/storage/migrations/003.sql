CREATE TABLE lesson_drafts (
  profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  lesson_id TEXT NOT NULL, uid TEXT NOT NULL UNIQUE, step INTEGER NOT NULL,
  summaries_json TEXT NOT NULL, texts_json TEXT NOT NULL, started_at INTEGER NOT NULL,
  layout_id TEXT NOT NULL DEFAULT 'nida',
  PRIMARY KEY(profile_id,lesson_id)) WITHOUT ROWID;
UPDATE schema_version SET version=3;
