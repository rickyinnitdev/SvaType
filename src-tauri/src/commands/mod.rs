use crate::{
    error::{AppError, AppResult},
    storage::{
        backup,
        db::{self, Database},
        profiles::{self, Profile, ProfileInput},
        sessions::{
            self, Checkpoint, HistoryDetail, HistoryFilter, HistoryPage, SessionInput, Trend,
        },
    },
};
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;
type DbState<'a> = State<'a, Arc<Database>>;
type ContentState<'a> = State<'a, Arc<crate::content::Content>>;
#[tauri::command]
pub async fn custom_text_list(
    db: DbState<'_>,
    profile: i64,
) -> AppResult<Vec<crate::storage::typing::CustomText>> {
    query(db, move |conn| {
        crate::storage::typing::custom_list(conn, profile)
    })
    .await
}
#[tauri::command]
pub async fn custom_text_save(
    db: DbState<'_>,
    content: ContentState<'_>,
    input: crate::storage::typing::CustomInput,
) -> AppResult<i64> {
    let content = Arc::clone(content.inner());
    query(db, move |conn| {
        crate::storage::typing::custom_save(conn, &content, input)
    })
    .await
}
#[tauri::command]
pub async fn custom_text_delete(db: DbState<'_>, profile: i64, id: i64) -> AppResult<()> {
    query(db, move |conn| {
        crate::storage::typing::custom_delete(conn, profile, id)
    })
    .await
}
#[tauri::command]
pub async fn analysis_get(
    db: DbState<'_>,
    profile: i64,
    n: u32,
) -> AppResult<crate::storage::typing::Analysis> {
    query(db, move |conn| {
        crate::storage::typing::analysis(conn, profile, n)
    })
    .await
}
#[tauri::command]
pub async fn weak_keys(
    db: DbState<'_>,
    profile: i64,
    n: u32,
) -> AppResult<Vec<khmer_core::analysis::WeakKey>> {
    query(db, move |conn| {
        Ok(crate::storage::typing::analysis(conn, profile, n)?.weak)
    })
    .await
}
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn practice_text(
    db: DbState<'_>,
    content: ContentState<'_>,
    profile: i64,
    level: u8,
    kind: String,
    seed: u64,
    weak: bool,
    long: bool,
) -> AppResult<String> {
    let content = Arc::clone(content.inner());
    query(db, move |conn| {
        let layout: String = conn.query_row(
            "SELECT layout_id FROM profiles WHERE id=?1",
            [profile],
            |r| r.get(0),
        )?;
        let weak = if weak {
            crate::storage::typing::analysis(conn, profile, 50)?
                .weak
                .into_iter()
                .map(|key| key.code)
                .collect()
        } else {
            vec![]
        };
        content.practice(level, &kind, &layout, seed, &weak, long)
    })
    .await
}
#[tauri::command]
pub async fn drill_generate(
    db: DbState<'_>,
    content: ContentState<'_>,
    profile: i64,
    seed: u64,
) -> AppResult<String> {
    practice_text(db, content, profile, 2, "words".into(), seed, true, false).await
}
async fn query<T: Send + 'static>(
    db: DbState<'_>,
    operation: impl FnOnce(&mut rusqlite::Connection) -> AppResult<T> + Send + 'static,
) -> AppResult<T> {
    let db = Arc::clone(db.inner());
    tauri::async_runtime::spawn_blocking(move || {
        let mut conn = db
            .connection
            .lock()
            .map_err(|_| AppError::invalid("database_locked"))?;
        operation(&mut conn)
    })
    .await
    .map_err(|_| AppError::invalid("database_worker_failed"))?
}
#[tauri::command]
pub async fn profiles_list(db: DbState<'_>) -> AppResult<Vec<Profile>> {
    query(db, |conn| profiles::list(conn)).await
}
#[tauri::command]
pub async fn profiles_create(db: DbState<'_>, input: ProfileInput) -> AppResult<Profile> {
    query(db, move |conn| profiles::create(conn, input)).await
}
#[tauri::command]
pub async fn profiles_update(db: DbState<'_>, id: i64, input: ProfileInput) -> AppResult<()> {
    query(db, move |conn| profiles::update(conn, id, input)).await
}
#[tauri::command]
pub async fn profiles_archive(db: DbState<'_>, id: i64) -> AppResult<()> {
    query(db, move |conn| profiles::archive(conn, id)).await
}
#[tauri::command]
pub async fn profiles_delete(db: DbState<'_>, id: i64) -> AppResult<()> {
    query(db, move |conn| profiles::delete(conn, id)).await
}
#[tauri::command]
pub async fn profile_touch(db: DbState<'_>, id: i64) -> AppResult<()> {
    query(db, move |conn| profiles::touch(conn, id)).await
}
#[tauri::command]
pub async fn session_save(db: DbState<'_>, input: SessionInput) -> AppResult<i64> {
    query(db, move |conn| sessions::save(conn, &input)).await
}
#[tauri::command]
pub async fn session_checkpoint(db: DbState<'_>, input: Checkpoint) -> AppResult<()> {
    query(db, move |conn| sessions::checkpoint(conn, &input)).await
}
#[tauri::command]
pub async fn checkpoints_list(db: DbState<'_>, profile: i64) -> AppResult<Vec<Checkpoint>> {
    query(db, move |conn| sessions::checkpoints(conn, profile)).await
}
#[tauri::command]
pub async fn checkpoint_discard(db: DbState<'_>, uid: String) -> AppResult<()> {
    query(db, move |conn| {
        conn.execute("DELETE FROM session_checkpoints WHERE uid=?1", [uid])?;
        Ok(())
    })
    .await
}
#[tauri::command]
pub async fn history_list(
    db: DbState<'_>,
    profile: i64,
    filter: HistoryFilter,
    page: u32,
) -> AppResult<HistoryPage> {
    query(db, move |conn| {
        sessions::history(conn, profile, &filter, page)
    })
    .await
}
#[tauri::command]
pub async fn history_detail(db: DbState<'_>, id: i64) -> AppResult<HistoryDetail> {
    query(db, move |conn| sessions::detail(conn, id)).await
}
#[tauri::command]
pub async fn history_delete(db: DbState<'_>, id: i64) -> AppResult<()> {
    query(db, move |conn| sessions::delete(conn, id)).await
}
#[tauri::command]
pub async fn history_trend(db: DbState<'_>, profile: i64, from: i64) -> AppResult<Vec<Trend>> {
    query(db, move |conn| sessions::trend(conn, profile, from)).await
}
fn public_setting(key: &str) -> bool {
    let name = if key.starts_with("p:") {
        let mut parts = key.splitn(3, ':');
        parts.next();
        if !parts
            .next()
            .is_some_and(|p| p.parse::<i64>().is_ok_and(|n| n > 0))
        {
            return false;
        }
        parts.next().unwrap_or("")
    } else {
        key
    };
    [
        "language",
        "layout",
        "show_keyboard",
        "show_hands",
        "sound",
        "font_size",
        "low_end",
        "reduce_motion",
        "theme",
        "onboarding_done",
        "last_profile",
    ]
    .contains(&name)
}
#[tauri::command]
pub async fn settings_get(db: DbState<'_>, key: String) -> AppResult<Option<String>> {
    if !public_setting(&key) {
        return Err(AppError::invalid("invalid_setting"));
    }
    query(db, move |conn| db::setting(conn, &key)).await
}
#[tauri::command]
pub async fn settings_set(db: DbState<'_>, key: String, value: String) -> AppResult<()> {
    if !public_setting(&key) {
        return Err(AppError::invalid("invalid_setting"));
    }
    query(db, move |conn| db::set_setting(conn, &key, &value)).await
}
#[tauri::command]
pub async fn progress_get(db: DbState<'_>, profile: i64) -> AppResult<Value> {
    query(db,move|conn|{
    let lessons:Vec<Value>=conn.prepare("SELECT lesson_id,best_stars,best_accuracy,attempts,completed_at,bookmark_step FROM lesson_progress WHERE profile_id=?1")?.query_map([profile],|r|Ok(json!({"lesson_id":r.get::<_,String>(0)?,"best_stars":r.get::<_,u8>(1)?,"best_accuracy":r.get::<_,f64>(2)?,"attempts":r.get::<_,u32>(3)?,"completed_at":r.get::<_,Option<i64>>(4)?,"bookmark_step":r.get::<_,Option<u32>>(5)?})))?.collect::<Result<_,_>>()?;
    let words:u32=conn.query_row("SELECT COUNT(*) FROM words_learned WHERE profile_id=?1",[profile],|r|r.get(0))?;
    let time:i64=conn.query_row("SELECT COALESCE(SUM(duration_ms),0) FROM sessions WHERE profile_id=?1",[profile],|r|r.get(0))?;
    Ok(json!({"lessons":lessons,"words_learned":words,"duration_ms":time}))
}).await
}
#[tauri::command]
pub async fn backup_create(db: DbState<'_>, path: String) -> AppResult<()> {
    query(db, move |conn| {
        backup::create(conn, std::path::Path::new(&path))
    })
    .await
}
#[tauri::command]
pub async fn backup_restore(db: DbState<'_>, path: String) -> AppResult<()> {
    let db = Arc::clone(db.inner());
    tauri::async_runtime::spawn_blocking(move || backup::restore(&db, std::path::Path::new(&path)))
        .await
        .map_err(|_| AppError::invalid("database_worker_failed"))?
}
#[tauri::command]
pub async fn export_csv(db: DbState<'_>, path: String, profile: Option<i64>) -> AppResult<()> {
    query(db, move |conn| {
        backup::export_csv(conn, std::path::Path::new(&path), profile)
    })
    .await
}
#[tauri::command]
pub async fn open_data_folder(db: DbState<'_>) -> AppResult<()> {
    let dir = db.dir.clone();
    tauri::async_runtime::spawn_blocking(move || {
        std::process::Command::new("explorer.exe")
            .arg(dir)
            .spawn()?;
        Ok(())
    })
    .await
    .map_err(|_| AppError::invalid("folder_open_failed"))?
}
