use super::db;
use crate::error::{AppError, AppResult};
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::{
    path::Path,
    sync::Mutex,
    time::{Duration, Instant},
};
const PIN_KEY: &str = "teacher_pin_hash";
#[derive(Default)]
struct Access {
    until: Option<Instant>,
    failures: u8,
    last_failure: Option<Instant>,
}
#[derive(Default)]
pub struct Gate(Mutex<Access>);
pub fn configured(conn: &Connection) -> AppResult<bool> {
    Ok(db::setting(conn, PIN_KEY)?.is_some())
}
impl Gate {
    pub fn authorized(&self) -> bool {
        self.0
            .lock()
            .is_ok_and(|state| state.until.is_some_and(|until| until > Instant::now()))
    }
    pub fn require(&self, conn: &Connection) -> AppResult<()> {
        if configured(conn)? && !self.authorized() {
            Err(AppError::invalid("teacher_locked"))
        } else {
            Ok(())
        }
    }
    pub fn lock(&self) {
        if let Ok(mut state) = self.0.lock() {
            state.until = None;
        }
    }
    pub fn verify(&self, conn: &Connection, pin: &[u8]) -> AppResult<()> {
        let mut state = self
            .0
            .lock()
            .map_err(|_| AppError::invalid("teacher_locked"))?;
        if state.failures >= 5
            && state
                .last_failure
                .is_some_and(|time| time.elapsed() < Duration::from_secs(60))
        {
            return Err(AppError::invalid("pin_wait"));
        }
        let stored = db::setting(conn, PIN_KEY)?;
        let matches = match stored {
            None => true,
            Some(hash) => PasswordHash::new(&hash)
                .is_ok_and(|parsed| Argon2::default().verify_password(pin, &parsed).is_ok()),
        };
        if !matches {
            state.failures = state.failures.saturating_add(1);
            state.last_failure = Some(Instant::now());
            return Err(AppError::invalid("wrong_pin"));
        }
        state.failures = 0;
        state.until = Some(Instant::now() + Duration::from_secs(30 * 60));
        Ok(())
    }
}
pub fn set_pin(conn: &mut Connection, dir: &Path, gate: &Gate, pin: &[u8]) -> AppResult<()> {
    gate.require(conn)?;
    if !(4..=6).contains(&pin.len()) || !pin.iter().all(u8::is_ascii_digit) {
        return Err(AppError::invalid("invalid_pin"));
    }
    let mut random = [0u8; 16];
    getrandom::fill(&mut random).map_err(|_| AppError::invalid("pin_hash_failed"))?;
    let salt = SaltString::encode_b64(&random).map_err(|_| AppError::invalid("pin_hash_failed"))?;
    let hash = Argon2::default()
        .hash_password(pin, &salt)
        .map_err(|_| AppError::invalid("pin_hash_failed"))?
        .to_string();
    write_marker(dir)?;
    let tx = conn.transaction()?;
    db::set_setting(&tx, PIN_KEY, &hash)?;
    db::set_setting(&tx, "teacher_pin_reset_armed", "true")?;
    tx.commit()?;
    gate.verify(conn, pin)
}
pub fn clear_pin(conn: &Connection, gate: &Gate) -> AppResult<()> {
    gate.require(conn)?;
    conn.execute(
        "DELETE FROM settings WHERE key IN ('teacher_pin_hash','teacher_pin_reset_armed')",
        [],
    )?;
    gate.lock();
    Ok(())
}
pub fn handle_reset(conn: &Connection, dir: &Path) -> AppResult<()> {
    if configured(conn)?
        && db::setting(conn, "teacher_pin_reset_armed")?.as_deref() == Some("true")
        && !dir.join("reset-pin.txt").exists()
    {
        conn.execute(
            "DELETE FROM settings WHERE key IN ('teacher_pin_hash','teacher_pin_reset_armed')",
            [],
        )?;
    }
    Ok(())
}
pub fn overview(conn: &Connection) -> AppResult<Value> {
    let rows:Vec<Value>=conn.prepare("SELECT p.id,p.name,p.grade,p.path_id,(SELECT COUNT(*) FROM lesson_progress l WHERE l.profile_id=p.id AND l.best_stars>0),(SELECT COALESCE(AVG(accuracy),0) FROM sessions s WHERE s.profile_id=p.id AND s.completed=1),(SELECT COALESCE(SUM(duration_ms),0) FROM sessions s WHERE s.profile_id=p.id AND s.started_at>=CAST(strftime('%s',date('now','localtime','-'||((CAST(strftime('%w','now','localtime') AS INTEGER)+6)%7)||' days'),'utc') AS INTEGER)*1000),(SELECT MAX(started_at) FROM sessions s WHERE s.profile_id=p.id),(SELECT AVG(accuracy) FROM (SELECT accuracy FROM sessions s WHERE s.profile_id=p.id AND s.completed=1 ORDER BY started_at DESC,id DESC LIMIT 5)) FROM profiles p WHERE p.archived=0 ORDER BY p.name,p.id")?.query_map([],|r|{let last:Option<i64>=r.get(7)?;let recent:Option<f64>=r.get(8)?;Ok(json!({"id":r.get::<_,i64>(0)?,"name":r.get::<_,String>(1)?,"grade":r.get::<_,Option<u8>>(2)?,"path_id":r.get::<_,Option<String>>(3)?,"lessons":r.get::<_,u32>(4)?,"accuracy":r.get::<_,f64>(5)?,"week_ms":r.get::<_,i64>(6)?,"last_at":last,"needs_help":recent.is_none_or(|accuracy|accuracy<80.0)||last.is_none_or(|time|db::now()-time>7*86_400_000)}))})?.collect::<Result<_,_>>()?;
    let keys:Vec<Value>=conn.prepare("SELECT key_code,layer,SUM(k.hits),SUM(k.errors) FROM session_keys k JOIN sessions s ON s.id=k.session_id JOIN profiles p ON p.id=s.profile_id WHERE p.archived=0 GROUP BY key_code,layer HAVING SUM(k.errors)>0 ORDER BY SUM(k.errors) DESC LIMIT 20")?.query_map([],|r|Ok(json!({"code":r.get::<_,String>(0)?,"layer":r.get::<_,String>(1)?,"hits":r.get::<_,i64>(2)?,"errors":r.get::<_,i64>(3)?})))?.collect::<Result<_,_>>()?;
    let confusions:Vec<Value>=conn.prepare("SELECT expected,typed,SUM(c.count) FROM session_confusions c JOIN sessions s ON s.id=c.session_id JOIN profiles p ON p.id=s.profile_id WHERE p.archived=0 GROUP BY expected,typed ORDER BY SUM(c.count) DESC LIMIT 10")?.query_map([],|r|Ok(json!({"expected":r.get::<_,String>(0)?,"typed":r.get::<_,String>(1)?,"count":r.get::<_,i64>(2)?})))?.collect::<Result<_,_>>()?;
    Ok(json!({"students":rows,"keys":keys,"confusions":confusions}))
}
pub fn reset_progress(conn: &mut Connection, profile: i64) -> AppResult<()> {
    let tx = conn.transaction()?;
    for table in [
        "lesson_progress",
        "words_learned",
        "achievements",
        "lesson_drafts",
        "game_drafts",
        "session_checkpoints",
    ] {
        tx.execute(
            &format!("DELETE FROM {table} WHERE profile_id=?1"),
            [profile],
        )?;
    }
    db::set_setting(
        &tx,
        &format!("p:{profile}:progress_reset_at"),
        &db::now().to_string(),
    )?;
    tx.commit()?;
    Ok(())
}
pub fn assignments(conn: &Connection, profile: Option<i64>) -> AppResult<Vec<Value>> {
    Ok(conn.prepare("SELECT id,profile_id,lesson_id,path_id,created_at FROM assignments WHERE ?1 IS NULL OR profile_id IS NULL OR profile_id=?1 ORDER BY profile_id IS NOT NULL DESC,created_at DESC,id DESC")?.query_map([profile],|r|Ok(json!({"id":r.get::<_,i64>(0)?,"profile_id":r.get::<_,Option<i64>>(1)?,"lesson_id":r.get::<_,Option<String>>(2)?,"path_id":r.get::<_,Option<String>>(3)?,"created_at":r.get::<_,i64>(4)?})))?.collect::<Result<_,_>>()?)
}
pub fn assign(
    conn: &mut Connection,
    content: &crate::content::Content,
    profile: Option<i64>,
    lesson: Option<String>,
    path: Option<String>,
) -> AppResult<()> {
    if lesson.is_some() == path.is_some() {
        return Err(AppError::invalid("invalid_assignment"));
    }
    if let Some(id) = &lesson {
        content.json("lessons", id)?;
    }
    if let Some(id) = &path {
        content.json("paths", id)?;
    }
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO assignments(profile_id,lesson_id,path_id,created_at) VALUES(?1,?2,?3,?4)",
        params![profile, lesson, path, db::now()],
    )?;
    if let Some(path) = path {
        tx.execute(
            "UPDATE profiles SET path_id=?1 WHERE ?2 IS NULL OR id=?2",
            params![path, profile],
        )?;
    }
    tx.commit()?;
    Ok(())
}

fn write_marker(dir: &Path) -> AppResult<()> {
    // The marker contains recovery instructions, never a PIN. Deleting it while
    // the app is closed resets the local PIN at the next start, as specified.
    let messages: Value = serde_json::from_str(include_str!("../../../src/lib/i18n/native.json"))?;
    let text = format!(
        "{}\r\n\r\n{}\r\n",
        messages["reset_marker"]["km"].as_str().unwrap_or(""),
        messages["reset_marker"]["en"].as_str().unwrap_or("")
    );
    std::fs::write(dir.join("reset-pin.txt"), text)?;
    Ok(())
}
pub fn restore_marker(conn: &Connection, dir: &Path) -> AppResult<()> {
    if configured(conn)? {
        write_marker(dir)?;
    }
    Ok(())
}
