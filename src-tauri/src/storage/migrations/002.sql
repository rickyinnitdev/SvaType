ALTER TABLE sessions ADD COLUMN uid TEXT;
ALTER TABLE sessions ADD COLUMN target_text TEXT;
ALTER TABLE sessions ADD COLUMN clusters_done INTEGER NOT NULL DEFAULT 0;
CREATE UNIQUE INDEX idx_sessions_uid ON sessions(uid);
CREATE TABLE session_checkpoints (
  uid TEXT PRIMARY KEY, profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  payload TEXT NOT NULL, updated_at INTEGER NOT NULL) WITHOUT ROWID;
UPDATE schema_version SET version=2;
