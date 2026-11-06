use crate::error::{AppError, AppResult};
use rusqlite::{Connection, OptionalExtension};
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
pub const VERSION: i64 = 4;
pub const MIGRATIONS: [&str; 4] = [
    include_str!("migrations/001.sql"),
    include_str!("migrations/002.sql"),
    include_str!("migrations/003.sql"),
    include_str!("migrations/004.sql"),
];
pub struct Database {
    pub connection: Mutex<Connection>,
    pub dir: PathBuf,
}
pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(i64::MAX as u128) as i64
}
pub fn version(conn: &Connection) -> AppResult<i64> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='schema_version')",
        [],
        |r| r.get(0),
    )?;
    if !exists {
        return Ok(0);
    }
    let versions: Vec<i64> = conn
        .prepare("SELECT version FROM schema_version")?
        .query_map([], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    if versions.len() != 1 {
        return Err(AppError::invalid("invalid_schema_version"));
    }
    Ok(versions[0])
}
pub fn migrate(conn: &mut Connection) -> AppResult<()> {
    let current = version(conn)?;
    if !(0..=VERSION).contains(&current) {
        return Err(AppError::invalid("newer_database"));
    }
    for sql in MIGRATIONS.iter().skip(current as usize) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.commit()?;
    }
    Ok(())
}
pub fn configure(conn: &Connection) -> AppResult<()> {
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA foreign_keys=ON; PRAGMA temp_store=MEMORY; PRAGMA cache_size=-4000;")?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    Ok(())
}
impl Database {
    pub fn open(dir: PathBuf) -> AppResult<Self> {
        std::fs::create_dir_all(&dir)?;
        std::fs::create_dir_all(dir.join("backups"))?;
        let mut conn = Connection::open(dir.join("svatype.db"))?;
        configure(&conn)?;
        let current = version(&conn)?;
        if current > 0 && current < VERSION {
            super::backup::create(
                &conn,
                &dir.join(format!("backups/pre-migration-{}.svabak", now())),
            )?;
        }
        migrate(&mut conn)?;
        super::teacher::handle_reset(&conn, &dir)?;
        let last = setting(&conn, "last_auto_backup")?
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);
        if now().saturating_sub(last) >= 7 * 24 * 60 * 60 * 1000 {
            super::backup::create(&conn, &dir.join(format!("backups/auto-{}.svabak", now())))?;
            set_setting(&conn, "last_auto_backup", &now().to_string())?;
            let mut files: Vec<_> = std::fs::read_dir(dir.join("backups"))?
                .filter_map(Result::ok)
                .filter(|f| {
                    f.file_name().to_string_lossy().starts_with("auto-")
                        && f.path().extension().is_some_and(|s| s == "svabak")
                })
                .collect();
            files.sort_by_key(|f| f.file_name());
            let remove_count = files.len().saturating_sub(4);
            for file in files.into_iter().take(remove_count) {
                std::fs::remove_file(file.path())?;
            }
        }
        Ok(Self {
            connection: Mutex::new(conn),
            dir,
        })
    }
}
pub fn setting(conn: &Connection, key: &str) -> AppResult<Option<String>> {
    Ok(conn
        .query_row("SELECT value FROM settings WHERE key=?1", [key], |r| {
            r.get(0)
        })
        .optional()?)
}
pub fn set_setting(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    if key.is_empty() || key.len() > 100 || value.len() > 100_000 {
        return Err(AppError::invalid("invalid_setting"));
    }
    conn.execute("INSERT INTO settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",[key,value])?;
    Ok(())
}
pub fn validate_file(path: &Path) -> AppResult<()> {
    let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let valid: String = conn.query_row("PRAGMA quick_check", [], |r| r.get(0))?;
    if valid != "ok" || !(1..=VERSION).contains(&version(&conn)?) {
        return Err(AppError::invalid("invalid_backup"));
    }
    for table in [
        "profiles",
        "sessions",
        "session_keys",
        "session_confusions",
        "lesson_progress",
        "words_learned",
        "custom_texts",
        "settings",
    ] {
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
            [table],
            |r| r.get(0),
        )?;
        if !exists {
            return Err(AppError::invalid("invalid_backup"));
        }
    }
    for (table,columns) in [
        ("profiles","id,name,avatar,grade,path_id,layout_id,preferred_mode,created_at,last_used_at,archived"),
        ("sessions","id,profile_id,mode,kind,ref_id,layout_id,started_at,duration_ms,active_ms,keystrokes,correct,errors,backspaces,kpm,accuracy,stars,completed,text_sample"),
        ("session_keys","session_id,key_code,layer,hits,errors,avg_hes_ms"),
        ("session_confusions","session_id,expected,typed,count"),
        ("lesson_progress","profile_id,lesson_id,best_stars,best_accuracy,attempts,completed_at,bookmark_step"),
        ("words_learned","profile_id,word_id,times_typed,best_accuracy,first_at"),
        ("custom_texts","id,profile_id,title,body,created_at"),
        ("settings","key,value"),
    ] {conn.prepare(&format!("SELECT {columns} FROM {table} LIMIT 0"))?;}
    if version(&conn)? >= 2 {
        conn.prepare("SELECT uid,target_text,clusters_done FROM sessions LIMIT 0")?;
        conn.prepare("SELECT uid,profile_id,payload,updated_at FROM session_checkpoints LIMIT 0")?;
    }
    if version(&conn)? >= 3 {
        conn.prepare("SELECT profile_id,lesson_id,uid,step,summaries_json,texts_json,started_at,layout_id FROM lesson_drafts LIMIT 0")?;
    }
    if version(&conn)? >= 4 {
        conn.prepare("SELECT profile_id,game_id,uid,step,summaries_json,texts_json,started_at,layout_id,lives FROM game_drafts LIMIT 0")?;
    }
    if conn.prepare("PRAGMA foreign_key_check")?.exists([])? {
        return Err(AppError::invalid("invalid_backup"));
    }
    Ok(())
}
