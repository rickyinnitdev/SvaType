use super::{query, ContentState, DbState};
use crate::{
    error::{AppError, AppResult},
    storage::{self, db, teacher::Gate},
};
use serde_json::{json, Value};
use std::{io::Write, sync::Arc};
use tauri::State;
pub type GateState<'a> = State<'a, Arc<Gate>>;
#[tauri::command]
pub async fn teacher_unlock_status(
    db: DbState<'_>,
    gate: GateState<'_>,
    profile: Option<i64>,
) -> AppResult<Value> {
    guarded(db, gate, move |conn| {
        let key = profile
            .map(|id| format!("p:{id}:unlock_all"))
            .unwrap_or_else(|| "teacher_unlock_all".into());
        Ok(json!({"enabled":db::setting(conn,&key)?.as_deref()==Some("true")}))
    })
    .await
}
pub async fn guarded<T: Send + 'static>(
    db: DbState<'_>,
    gate: GateState<'_>,
    operation: impl FnOnce(&mut rusqlite::Connection) -> AppResult<T> + Send + 'static,
) -> AppResult<T> {
    let gate = Arc::clone(gate.inner());
    query(db, move |conn| {
        gate.require(conn)?;
        operation(conn)
    })
    .await
}
#[tauri::command]
pub async fn teacher_pin_status(db: DbState<'_>, gate: GateState<'_>) -> AppResult<Value> {
    let gate = Arc::clone(gate.inner());
    query(db, move |conn| {
        Ok(json!({"configured":storage::teacher::configured(conn)?,"authorized":gate.authorized()}))
    })
    .await
}
#[tauri::command]
pub async fn teacher_pin_verify(
    db: DbState<'_>,
    gate: GateState<'_>,
    pin: String,
) -> AppResult<()> {
    let gate = Arc::clone(gate.inner());
    query(db, move |conn| {
        let mut bytes = pin.into_bytes();
        let result = gate.verify(conn, &bytes);
        bytes.fill(0);
        result
    })
    .await
}
#[tauri::command]
pub async fn teacher_pin_set(db: DbState<'_>, gate: GateState<'_>, pin: String) -> AppResult<()> {
    let dir = db.dir.clone();
    let gate = Arc::clone(gate.inner());
    query(db, move |conn| {
        let mut bytes = pin.into_bytes();
        let result = storage::teacher::set_pin(conn, &dir, &gate, &bytes);
        bytes.fill(0);
        result
    })
    .await
}
#[tauri::command]
pub async fn teacher_pin_clear(db: DbState<'_>, gate: GateState<'_>) -> AppResult<()> {
    let gate = Arc::clone(gate.inner());
    query(db, move |conn| storage::teacher::clear_pin(conn, &gate)).await
}
#[tauri::command]
pub async fn teacher_lock(gate: GateState<'_>) -> AppResult<()> {
    gate.lock();
    Ok(())
}
#[tauri::command]
pub async fn class_overview(db: DbState<'_>, gate: GateState<'_>) -> AppResult<Value> {
    guarded(db, gate, |conn| storage::teacher::overview(conn)).await
}
#[tauri::command]
pub async fn teacher_reset_progress(
    db: DbState<'_>,
    gate: GateState<'_>,
    profile: i64,
) -> AppResult<()> {
    guarded(db, gate, move |conn| {
        storage::teacher::reset_progress(conn, profile)
    })
    .await
}
#[tauri::command]
pub async fn assignments_list(
    db: DbState<'_>,
    gate: GateState<'_>,
    profile: Option<i64>,
) -> AppResult<Vec<Value>> {
    let gate = Arc::clone(gate.inner());
    query(db, move |conn| {
        if profile.is_none() {
            gate.require(conn)?;
        }
        storage::teacher::assignments(conn, profile)
    })
    .await
}
#[tauri::command]
pub async fn assignments_save(
    db: DbState<'_>,
    gate: GateState<'_>,
    content: ContentState<'_>,
    profile: Option<i64>,
    lesson: Option<String>,
    path: Option<String>,
) -> AppResult<()> {
    let content = Arc::clone(content.inner());
    guarded(db, gate, move |conn| {
        storage::teacher::assign(conn, &content, profile, lesson, path)
    })
    .await
}
#[tauri::command]
pub async fn assignments_delete(db: DbState<'_>, gate: GateState<'_>, id: i64) -> AppResult<()> {
    guarded(db, gate, move |conn| {
        conn.execute("DELETE FROM assignments WHERE id=?1", [id])?;
        Ok(())
    })
    .await
}
#[tauri::command]
pub async fn teacher_unlock_all(
    db: DbState<'_>,
    gate: GateState<'_>,
    profile: Option<i64>,
    enabled: bool,
) -> AppResult<()> {
    guarded(db, gate, move |conn| {
        db::set_setting(
            conn,
            &profile
                .map(|id| format!("p:{id}:unlock_all"))
                .unwrap_or_else(|| "teacher_unlock_all".into()),
            if enabled { "true" } else { "false" },
        )
    })
    .await
}
#[tauri::command]
pub async fn content_overrides_list(db: DbState<'_>, gate: GateState<'_>) -> AppResult<Vec<Value>> {
    guarded(db, gate, |conn| storage::overrides::list(conn)).await
}
#[tauri::command]
pub async fn content_override_save(
    db: DbState<'_>,
    gate: GateState<'_>,
    content: ContentState<'_>,
    id: Option<i64>,
    kind: String,
    category: String,
    value: Value,
) -> AppResult<i64> {
    let content = Arc::clone(content.inner());
    guarded(db, gate, move |conn| {
        storage::overrides::save(conn, &content, id, &kind, &category, value)
    })
    .await
}
#[tauri::command]
pub async fn content_override_delete(
    db: DbState<'_>,
    gate: GateState<'_>,
    id: i64,
) -> AppResult<()> {
    guarded(db, gate, move |conn| {
        conn.execute("UPDATE content_overrides SET deleted=1 WHERE id=?1", [id])?;
        Ok(())
    })
    .await
}
#[tauri::command]
pub async fn content_overrides_export(
    db: DbState<'_>,
    gate: GateState<'_>,
    path: String,
) -> AppResult<()> {
    guarded(db, gate, move |conn| {
        let data = storage::overrides::list(conn)?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::AlreadyExists {
                    AppError::invalid("file_already_exists")
                } else {
                    error.into()
                }
            })?;
        file.write_all(
            serde_json::to_string_pretty(&json!({"version":1,"overrides":data}))?.as_bytes(),
        )?;
        Ok(())
    })
    .await
}
