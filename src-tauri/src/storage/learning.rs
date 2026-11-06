use crate::error::{AppError, AppResult};
use khmer_core::metrics::{aggregate, SessionSummary};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Draft {
    pub profile_id: i64,
    pub lesson_id: String,
    pub uid: String,
    pub step: u32,
    pub summaries: Vec<SessionSummary>,
    pub texts: Vec<String>,
    pub started_at: i64,
    pub layout_id: String,
}
pub fn get(conn: &Connection, profile: i64, lesson: &str) -> AppResult<Option<Draft>> {
    let value=conn.query_row("SELECT uid,step,summaries_json,texts_json,started_at,layout_id FROM lesson_drafts WHERE profile_id=?1 AND lesson_id=?2",params![profile,lesson],|r|Ok((r.get::<_,String>(0)?,r.get::<_,u32>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,i64>(4)?,r.get::<_,String>(5)?))).optional()?;
    value
        .map(|(uid, step, summaries, texts, started_at, layout_id)| {
            Ok(Draft {
                profile_id: profile,
                lesson_id: lesson.into(),
                uid,
                step,
                summaries: serde_json::from_str(&summaries)?,
                texts: serde_json::from_str(&texts)?,
                started_at,
                layout_id,
            })
        })
        .transpose()
}
pub fn set(conn: &mut Connection, draft: &Draft) -> AppResult<()> {
    if draft.uid.is_empty()
        || draft.uid.len() > 100
        || draft.lesson_id.len() > 100
        || draft.step > 200
        || draft.summaries.len() > 500
        || draft.texts.iter().map(|t| t.len()).sum::<usize>() > 300_000
        || !["nida", "khmer_legacy"].contains(&draft.layout_id.as_str())
    {
        return Err(AppError::invalid("invalid_lesson_draft"));
    }
    let previous = get(conn, draft.profile_id, &draft.lesson_id)?;
    let tx = conn.transaction()?;
    tx.execute("INSERT INTO lesson_drafts(profile_id,lesson_id,uid,step,summaries_json,texts_json,started_at,layout_id) VALUES(?1,?2,?3,?4,?5,?6,?7,?8) ON CONFLICT(profile_id,lesson_id) DO UPDATE SET uid=excluded.uid,step=excluded.step,summaries_json=excluded.summaries_json,texts_json=excluded.texts_json,started_at=excluded.started_at,layout_id=excluded.layout_id",params![draft.profile_id,draft.lesson_id,draft.uid,draft.step,serde_json::to_string(&draft.summaries)?,serde_json::to_string(&draft.texts)?,draft.started_at,draft.layout_id])?;
    tx.execute("INSERT INTO lesson_progress(profile_id,lesson_id,best_stars,best_accuracy,attempts,bookmark_step) VALUES(?1,?2,0,0,0,?3) ON CONFLICT(profile_id,lesson_id) DO UPDATE SET bookmark_step=excluded.bookmark_step",params![draft.profile_id,draft.lesson_id,draft.step])?;
    if previous.is_some_and(|old| old.step < draft.step) {
        tx.execute("DELETE FROM session_checkpoints WHERE uid=?1", [&draft.uid])?;
    }
    tx.commit()?;
    Ok(())
}
pub fn summary(draft: &Draft, extra: Option<SessionSummary>) -> SessionSummary {
    let mut values = draft.summaries.clone();
    if let Some(extra) = extra {
        values.push(extra);
    }
    aggregate(&values)
}
