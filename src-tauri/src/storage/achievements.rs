use crate::error::AppResult;
use rusqlite::{params, Connection};
use serde_json::Value;
pub fn evaluate(conn: &Connection, profile: i64) -> AppResult<()> {
    let reset = super::db::setting(conn, &format!("p:{profile}:progress_reset_at"))?
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    let data: Value = serde_json::from_str(include_str!("../../../content/achievements.json"))?;
    for item in data["achievements"].as_array().into_iter().flatten() {
        let target = item["rule"]["target"].as_i64().unwrap_or(i64::MAX);
        let kind = item["rule"]["kind"].as_str().unwrap_or("");
        let actual:i64=match kind{
 "lessons"=>conn.query_row("SELECT COUNT(*) FROM lesson_progress WHERE profile_id=?1 AND best_stars>0",[profile],|r|r.get(0))?,
 "words"=>conn.query_row("SELECT COUNT(*) FROM words_learned WHERE profile_id=?1",[profile],|r|r.get(0))?,
 "games"=>conn.query_row("SELECT COUNT(*) FROM sessions WHERE profile_id=?1 AND kind='game' AND completed=1 AND correct>0 AND started_at>=?2",params![profile,reset],|r|r.get(0))?,
 "accuracy_test"=>conn.query_row("SELECT CAST(COALESCE(MAX(accuracy),0) AS INTEGER) FROM sessions WHERE profile_id=?1 AND kind='test' AND completed=1 AND keystrokes>=10 AND started_at>=?2",params![profile,reset],|r|r.get(0))?,
 "streak"=>{let days:Vec<i64>=conn.prepare("SELECT DISTINCT CAST(julianday(date(started_at/1000,'unixepoch','localtime')) AS INTEGER) AS day FROM sessions WHERE profile_id=?1 AND completed=1 AND started_at>=?2 ORDER BY day DESC LIMIT 7")?.query_map(params![profile,reset],|r|r.get(0))?.collect::<Result<_,_>>()?;let today:i64=conn.query_row("SELECT CAST(julianday(date('now','localtime')) AS INTEGER)",[],|r|r.get(0))?;if days.first()==Some(&today){days.iter().enumerate().take_while(|(i,day)|**day==today-*i as i64).count() as i64}else{0}},_=>0};
        if actual >= target {
            if let Some(id) = item["id"].as_str() {
                conn.execute("INSERT OR IGNORE INTO achievements(profile_id,achievement_id,earned_at) VALUES(?1,?2,?3)",params![profile,id,super::db::now()])?;
            }
        }
    }
    Ok(())
}
