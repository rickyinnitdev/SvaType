use crate::error::{AppError, AppResult};
use khmer_core::metrics::{aggregate, SessionSummary};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Draft {
    pub profile_id: i64,
    pub game_id: String,
    pub uid: String,
    pub step: u32,
    pub summaries: Vec<SessionSummary>,
    pub texts: Vec<String>,
    pub started_at: i64,
    pub layout_id: String,
    pub lives: u8,
}
pub fn get(conn: &Connection, profile: i64, game: &str) -> AppResult<Option<Draft>> {
    let row=conn.query_row("SELECT uid,step,summaries_json,texts_json,started_at,layout_id,lives FROM game_drafts WHERE profile_id=?1 AND game_id=?2",params![profile,game],|r|Ok((r.get::<_,String>(0)?,r.get::<_,u32>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,i64>(4)?,r.get::<_,String>(5)?,r.get::<_,u8>(6)?))).optional()?;
    row.map(
        |(uid, step, summaries, texts, started_at, layout_id, lives)| {
            Ok(Draft {
                profile_id: profile,
                game_id: game.into(),
                uid,
                step,
                summaries: serde_json::from_str(&summaries)?,
                texts: serde_json::from_str(&texts)?,
                started_at,
                layout_id,
                lives,
            })
        },
    )
    .transpose()
}
pub fn set(conn: &mut Connection, d: &Draft) -> AppResult<()> {
    if !["falling", "picture", "complete", "journey"].contains(&d.game_id.as_str())
        || d.step > 10
        || d.lives > 3
        || d.uid.is_empty()
        || d.uid.len() > 100
        || d.summaries.len() > 11
        || d.texts.iter().map(|t| t.len()).sum::<usize>() > 20000
        || !["nida", "khmer_legacy"].contains(&d.layout_id.as_str())
    {
        return Err(AppError::invalid("invalid_game_draft"));
    }
    conn.execute("INSERT INTO game_drafts(profile_id,game_id,uid,step,summaries_json,texts_json,started_at,layout_id,lives) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9) ON CONFLICT(profile_id,game_id) DO UPDATE SET uid=excluded.uid,step=excluded.step,summaries_json=excluded.summaries_json,texts_json=excluded.texts_json,started_at=excluded.started_at,layout_id=excluded.layout_id,lives=excluded.lives",params![d.profile_id,d.game_id,d.uid,d.step,serde_json::to_string(&d.summaries)?,serde_json::to_string(&d.texts)?,d.started_at,d.layout_id,d.lives])?;
    Ok(())
}
pub fn save(
    conn: &mut Connection,
    profile: i64,
    game: &str,
    extra: Option<SessionSummary>,
    text: Option<String>,
    completed: bool,
) -> AppResult<Option<i64>> {
    let mut d = get(conn, profile, game)?.ok_or_else(|| AppError::invalid("missing_game_draft"))?;
    if let Some(s) = extra {
        d.summaries.push(s);
    }
    if let Some(text) = text {
        d.texts.push(text);
    }
    let summary = aggregate(&d.summaries);
    if !completed && summary.duration_ms <= 20000 {
        return Ok(None);
    }
    let input = super::sessions::SessionInput {
        uid: d.uid,
        profile_id: profile,
        mode: "learning".into(),
        kind: "game".into(),
        ref_id: Some(game.into()),
        layout_id: d.layout_id,
        started_at: d.started_at,
        completed,
        target_text: d.texts.join(" "),
        summary,
        bookmark_step: if completed { None } else { Some(d.step) },
    };
    Ok(Some(super::sessions::save(conn, &input)?))
}
