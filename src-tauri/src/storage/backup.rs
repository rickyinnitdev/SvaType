use super::{
    db::{self, Database},
    sessions::{self, HistoryFilter},
};
use crate::error::{AppError, AppResult};
use rusqlite::{backup::Backup, Connection};
use std::{io::Write, path::Path, time::Duration};
pub fn create(conn: &Connection, path: &Path) -> AppResult<()> {
    if path.exists() {
        return Err(AppError::invalid("file_already_exists"));
    }
    let mut destination = Connection::open(path)?;
    let result =
        Backup::new(conn, &mut destination)?.run_to_completion(128, Duration::from_millis(5), None);
    if let Err(error) = result {
        drop(destination);
        let _ = std::fs::remove_file(path);
        return Err(error.into());
    }
    Ok(())
}
pub fn restore(database: &Database, path: &Path) -> AppResult<()> {
    if path.canonicalize()? == database.dir.join("svatype.db").canonicalize()? {
        return Err(AppError::invalid("invalid_backup"));
    }
    db::validate_file(path)?;
    let mut conn = database
        .connection
        .lock()
        .map_err(|_| AppError::invalid("database_locked"))?;
    create(
        &conn,
        &database.dir.join(format!("pre-restore-{}.db", db::now())),
    )?;
    // Online restore replaces database pages while preserving the connection and
    // recovering WAL consistently; no copying an active database file.
    conn.restore("main", path, None::<fn(rusqlite::backup::Progress)>)?;
    db::configure(&conn)?;
    db::migrate(&mut conn)?;
    Ok(())
}
fn cell(value: &str) -> String {
    // Text columns are prefixed when Excel could interpret them as formulas.
    let safe = if value.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        format!("'{value}")
    } else {
        value.to_string()
    };
    format!("\"{}\"", safe.replace('"', "\"\""))
}
pub fn export_csv(conn: &Connection, path: &Path, profile: Option<i64>) -> AppResult<()> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(
        b"\xef\xbb\xbfprofile,started_at,mode,activity,duration_ms,kpm,accuracy,completed\r\n",
    )?;
    let profiles: Vec<(i64, String)> = conn
        .prepare("SELECT id,name FROM profiles ORDER BY id")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<_, _>>()?;
    for (id, name) in profiles {
        if profile.is_some_and(|p| p != id) {
            continue;
        }
        let mut page = 0;
        loop {
            let rows = sessions::history(conn, id, &HistoryFilter::default(), page)?.rows;
            if rows.is_empty() {
                break;
            }
            for row in rows {
                writeln!(
                    file,
                    "{},{},{},{},{},{:.2},{:.2},{}\r",
                    cell(&name),
                    row.started_at,
                    cell(&row.mode),
                    cell(row.ref_id.as_deref().unwrap_or(&row.kind)),
                    row.duration_ms,
                    row.kpm,
                    row.accuracy,
                    row.completed
                )?;
            }
            page += 1;
        }
    }
    file.sync_all()?;
    Ok(())
}
