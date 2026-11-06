use super::db::now;
use crate::error::{AppError, AppResult};
use khmer_core::metrics::{Confusion, KeyStats, SessionSummary};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInput {
    pub uid: String,
    pub profile_id: i64,
    pub mode: String,
    pub kind: String,
    pub ref_id: Option<String>,
    pub layout_id: String,
    pub started_at: i64,
    pub completed: bool,
    pub target_text: String,
    pub summary: SessionSummary,
    pub bookmark_step: Option<u32>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HistoryFilter {
    pub mode: Option<String>,
    pub kind: Option<String>,
    pub from: Option<i64>,
    pub to: Option<i64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRow {
    pub id: i64,
    pub profile_id: i64,
    pub mode: String,
    pub kind: String,
    pub ref_id: Option<String>,
    pub layout_id: String,
    pub started_at: i64,
    pub duration_ms: u64,
    pub active_ms: u64,
    pub kpm: f64,
    pub accuracy: f64,
    pub stars: Option<u8>,
    pub completed: bool,
    pub text_sample: String,
}
#[derive(Debug, Serialize)]
pub struct HistoryPage {
    pub rows: Vec<HistoryRow>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}
#[derive(Debug, Serialize)]
pub struct HistoryDetail {
    pub session: HistoryRow,
    pub target_text: String,
    pub summary: SessionSummary,
}
#[derive(Debug, Serialize)]
pub struct Trend {
    pub day: i64,
    pub sessions: u32,
    pub kpm: f64,
    pub accuracy: f64,
    pub duration_ms: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub session: SessionInput,
    pub engine: String,
    pub remaining_ms: Option<u64>,
}
const COLUMNS:&str="id,profile_id,mode,kind,ref_id,layout_id,started_at,duration_ms,active_ms,kpm,accuracy,stars,completed,COALESCE(text_sample,'')";
fn unsigned(r: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<u64> {
    let value: i64 = r.get(index)?;
    value
        .try_into()
        .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, value))
}
fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<HistoryRow> {
    Ok(HistoryRow {
        id: r.get(0)?,
        profile_id: r.get(1)?,
        mode: r.get(2)?,
        kind: r.get(3)?,
        ref_id: r.get(4)?,
        layout_id: r.get(5)?,
        started_at: r.get(6)?,
        duration_ms: unsigned(r, 7)?,
        active_ms: unsigned(r, 8)?,
        kpm: r.get(9)?,
        accuracy: r.get(10)?,
        stars: r.get(11)?,
        completed: r.get(12)?,
        text_sample: r.get(13)?,
    })
}
pub fn validate(input: &SessionInput) -> AppResult<()> {
    let s = &input.summary;
    if input.uid.is_empty()
        || input.uid.len() > 100
        || input.target_text.chars().count() > 20_000
        || input.started_at < 0
        || !["learning", "typing"].contains(&input.mode.as_str())
        || ![
            "lesson", "vocab", "game", "free", "test", "custom", "random", "drill",
        ]
        .contains(&input.kind.as_str())
        || !["nida", "khmer_legacy"].contains(&input.layout_id.as_str())
        || s.active_ms > s.duration_ms
        || s.duration_ms > i64::MAX as u64
        || s.correct > s.keystrokes
        || s.errors > s.keystrokes
        || !s.kpm.is_finite()
        || s.kpm < 0.0
        || !s.accuracy.is_finite()
        || !(0.0..=100.0).contains(&s.accuracy)
        || s.stars > 3
        || s.per_key.len() > 1024
        || s.confusions.len() > 20_000
        || s.per_key
            .iter()
            .any(|k| !k.avg_hes_ms.is_finite() || k.avg_hes_ms < 0.0 || k.code.len() > 40)
        || input.ref_id.as_ref().is_some_and(|id| id.len() > 100)
    {
        return Err(AppError::invalid("invalid_session"));
    }
    Ok(())
}
pub fn save(conn: &mut Connection, input: &SessionInput) -> AppResult<i64> {
    validate(input)?;
    if !input.completed && input.summary.duration_ms <= 20_000 {
        return Err(AppError::invalid("short_abandoned_session"));
    }
    let tx = conn.transaction()?;
    let existing = tx
        .query_row(
            "SELECT id,profile_id,completed,keystrokes FROM sessions WHERE uid=?1",
            [&input.uid],
            |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, bool>(2)?,
                    r.get::<_, u32>(3)?,
                ))
            },
        )
        .optional()?;
    if let Some((id, profile_id, completed, hits)) = existing {
        if profile_id != input.profile_id {
            return Err(AppError::invalid("duplicate_session_uid"));
        }
        if completed || (!input.completed && input.summary.keystrokes < hits) {
            return Ok(id);
        }
    }
    let s = &input.summary;
    let sample: String = input.target_text.chars().take(500).collect();
    let stars = if input.mode == "learning" {
        Some(s.stars)
    } else {
        None
    };
    tx.execute("INSERT INTO sessions(profile_id,mode,kind,ref_id,layout_id,started_at,duration_ms,active_ms,keystrokes,correct,errors,backspaces,kpm,accuracy,stars,completed,text_sample,uid,target_text,clusters_done) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20) ON CONFLICT(uid) DO UPDATE SET duration_ms=excluded.duration_ms,active_ms=excluded.active_ms,keystrokes=excluded.keystrokes,correct=excluded.correct,errors=excluded.errors,backspaces=excluded.backspaces,kpm=excluded.kpm,accuracy=excluded.accuracy,stars=excluded.stars,completed=excluded.completed,text_sample=excluded.text_sample,target_text=excluded.target_text,clusters_done=excluded.clusters_done",params![input.profile_id,input.mode,input.kind,input.ref_id,input.layout_id,input.started_at,s.duration_ms as i64,s.active_ms as i64,s.keystrokes,s.correct,s.errors,s.backspaces,s.kpm,s.accuracy,stars,input.completed,sample,input.uid,input.target_text,s.clusters_done])?;
    let id = tx.query_row("SELECT id FROM sessions WHERE uid=?1", [&input.uid], |r| {
        r.get::<_, i64>(0)
    })?;
    tx.execute("DELETE FROM session_keys WHERE session_id=?1", [id])?;
    tx.execute("DELETE FROM session_confusions WHERE session_id=?1", [id])?;
    for k in &s.per_key {
        tx.execute("INSERT INTO session_keys(session_id,key_code,layer,hits,errors,avg_hes_ms) VALUES(?1,?2,?3,?4,?5,?6)",params![id,k.code,serde_json::to_string(&k.layer)?.trim_matches('"'),k.hits,k.errors,k.avg_hes_ms])?;
    }
    for c in &s.confusions {
        tx.execute(
            "INSERT INTO session_confusions(session_id,expected,typed,count) VALUES(?1,?2,?3,?4)",
            params![id, c.expected, c.typed, c.count],
        )?;
    }
    if input.kind == "lesson" {
        if let Some(lesson) = &input.ref_id {
            tx.execute("INSERT INTO lesson_progress(profile_id,lesson_id,best_stars,best_accuracy,attempts,completed_at,bookmark_step) VALUES(?1,?2,?3,?4,1,?5,?6) ON CONFLICT(profile_id,lesson_id) DO UPDATE SET best_stars=MAX(best_stars,excluded.best_stars),best_accuracy=MAX(best_accuracy,excluded.best_accuracy),attempts=attempts+?7,completed_at=COALESCE(excluded.completed_at,completed_at),bookmark_step=excluded.bookmark_step",params![input.profile_id,lesson,if input.completed{s.stars}else{0},if input.completed{s.accuracy}else{0.0},if input.completed&&s.stars>0{Some(now())}else{None},input.bookmark_step,if existing.is_none(){1}else{0}])?;
        }
    }
    if input.kind == "vocab" && input.completed {
        if let Some(word) = &input.ref_id {
            tx.execute("INSERT INTO words_learned(profile_id,word_id,times_typed,best_accuracy,first_at) VALUES(?1,?2,1,?3,?4) ON CONFLICT(profile_id,word_id) DO UPDATE SET times_typed=times_typed+1,best_accuracy=MAX(best_accuracy,excluded.best_accuracy)",params![input.profile_id,word,s.accuracy,now()])?;
        }
    }
    tx.execute("DELETE FROM session_checkpoints WHERE uid=?1", [&input.uid])?;
    if input.kind == "lesson" && input.completed {
        tx.execute("DELETE FROM lesson_drafts WHERE uid=?1", [&input.uid])?;
    }
    tx.commit()?;
    Ok(id)
}
pub fn history(
    conn: &Connection,
    profile: i64,
    filter: &HistoryFilter,
    page: u32,
) -> AppResult<HistoryPage> {
    let predicate="profile_id=?1 AND (?2 IS NULL OR mode=?2) AND (?3 IS NULL OR kind=?3) AND (?4 IS NULL OR started_at>=?4) AND (?5 IS NULL OR started_at<=?5)";
    let total: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM sessions WHERE {predicate}"),
        params![profile, filter.mode, filter.kind, filter.from, filter.to],
        |r| r.get(0),
    )?;
    let rows=conn.prepare(&format!("SELECT {COLUMNS} FROM sessions WHERE {predicate} ORDER BY started_at DESC,id DESC LIMIT 50 OFFSET ?6"))?.query_map(params![profile,filter.mode,filter.kind,filter.from,filter.to,i64::from(page)*50],row)?.collect::<Result<_,_>>()?;
    Ok(HistoryPage {
        rows,
        total: total as u64,
        page,
        page_size: 50,
    })
}
pub fn detail(conn: &Connection, id: i64) -> AppResult<HistoryDetail> {
    let session = conn.query_row(
        &format!("SELECT {COLUMNS} FROM sessions WHERE id=?1"),
        [id],
        row,
    )?;
    let (target_text,mut summary)=conn.query_row("SELECT COALESCE(target_text,text_sample,''),duration_ms,active_ms,keystrokes,correct,errors,backspaces,kpm,accuracy,clusters_done FROM sessions WHERE id=?1",[id],|r|Ok((r.get::<_,String>(0)?,SessionSummary{duration_ms:unsigned(r,1)?,active_ms:unsigned(r,2)?,keystrokes:r.get(3)?,correct:r.get(4)?,errors:r.get(5)?,backspaces:r.get(6)?,kpm:r.get(7)?,accuracy:r.get(8)?,clusters_done:r.get(9)?,..Default::default()})))?;
    summary.wpm_equivalent = summary.kpm / 5.0;
    summary.stars = session.stars.unwrap_or(0);
    summary.clusters_per_minute =
        khmer_core::metrics::rate(summary.clusters_done, summary.active_ms);
    summary.per_key=conn.prepare("SELECT key_code,layer,hits,errors,avg_hes_ms FROM session_keys WHERE session_id=?1 ORDER BY key_code,layer")?.query_map([id],|r|{let layer:String=r.get(1)?;Ok(KeyStats{code:r.get(0)?,layer:serde_json::from_str(&format!("\"{layer}\"")).map_err(|e|rusqlite::Error::FromSqlConversionFailure(1,rusqlite::types::Type::Text,Box::new(e)))?,hits:r.get(2)?,errors:r.get(3)?,avg_hes_ms:r.get(4)?})})?.collect::<Result<_,_>>()?;
    summary.confusions=conn.prepare("SELECT expected,typed,count FROM session_confusions WHERE session_id=?1 ORDER BY count DESC")?.query_map([id],|r|Ok(Confusion{expected:r.get(0)?,typed:r.get(1)?,count:r.get(2)?}))?.collect::<Result<_,_>>()?;
    Ok(HistoryDetail {
        session,
        target_text,
        summary,
    })
}
pub fn delete(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM sessions WHERE id=?1", [id])?;
    Ok(())
}
pub fn trend(conn: &Connection, profile: i64, from: i64) -> AppResult<Vec<Trend>> {
    Ok(conn.prepare("SELECT started_at/86400000,COUNT(*),AVG(kpm),AVG(accuracy),SUM(duration_ms) FROM sessions WHERE profile_id=?1 AND started_at>=?2 GROUP BY started_at/86400000 ORDER BY started_at/86400000")?.query_map(params![profile,from],|r|Ok(Trend{day:r.get(0)?,sessions:r.get(1)?,kpm:r.get(2)?,accuracy:r.get(3)?,duration_ms:unsigned(r,4)?}))?.collect::<Result<_,_>>()?)
}
pub fn checkpoint(conn: &Connection, input: &Checkpoint) -> AppResult<()> {
    validate(&input.session)?;
    if input.engine.len() > 1_000_000 {
        return Err(AppError::invalid("invalid_checkpoint"));
    }
    let saved: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sessions WHERE uid=?1 AND completed=1)",
        [&input.session.uid],
        |r| r.get(0),
    )?;
    if saved {
        return Ok(());
    }
    conn.execute("INSERT INTO session_checkpoints(uid,profile_id,payload,updated_at) VALUES(?1,?2,?3,?4) ON CONFLICT(uid) DO UPDATE SET payload=excluded.payload,updated_at=excluded.updated_at",params![input.session.uid,input.session.profile_id,serde_json::to_string(input)?,now()])?;
    Ok(())
}
pub fn checkpoints(conn: &Connection, profile: i64) -> AppResult<Vec<Checkpoint>> {
    let payloads: Vec<String> = conn
        .prepare(
            "SELECT payload FROM session_checkpoints WHERE profile_id=?1 ORDER BY updated_at DESC",
        )?
        .query_map([profile], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    payloads
        .into_iter()
        .map(|s| Ok(serde_json::from_str(&s)?))
        .collect()
}
