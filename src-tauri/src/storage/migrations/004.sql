CREATE TABLE game_drafts (
 profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
 game_id TEXT NOT NULL, uid TEXT NOT NULL UNIQUE, step INTEGER NOT NULL,
 summaries_json TEXT NOT NULL,texts_json TEXT NOT NULL,started_at INTEGER NOT NULL,
 layout_id TEXT NOT NULL,lives INTEGER NOT NULL,
 PRIMARY KEY(profile_id,game_id)
) WITHOUT ROWID;
UPDATE schema_version SET version=4;
