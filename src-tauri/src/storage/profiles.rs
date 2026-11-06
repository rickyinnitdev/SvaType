use super::db::now;
use crate::error::{AppError, AppResult};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: i64,
    pub name: String,
    pub avatar: String,
    pub grade: Option<u8>,
    pub path_id: Option<String>,
    pub layout_id: String,
    pub preferred_mode: String,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
    pub archived: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileInput {
    pub name: String,
    pub avatar: String,
    pub grade: Option<u8>,
    pub path_id: Option<String>,
    pub layout_id: String,
    pub preferred_mode: String,
}
fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Profile> {
    Ok(Profile {
        id: r.get(0)?,
        name: r.get(1)?,
        avatar: r.get(2)?,
        grade: r.get(3)?,
        path_id: r.get(4)?,
        layout_id: r.get(5)?,
        preferred_mode: r.get(6)?,
        created_at: r.get(7)?,
        last_used_at: r.get(8)?,
        archived: r.get(9)?,
    })
}
pub fn list(conn: &Connection) -> AppResult<Vec<Profile>> {
    Ok(conn.prepare("SELECT id,name,avatar,grade,path_id,layout_id,preferred_mode,created_at,last_used_at,archived FROM profiles WHERE archived=0 ORDER BY last_used_at DESC,id")?.query_map([],row)?.collect::<Result<_,_>>()?)
}
fn validate(input: &ProfileInput) -> AppResult<()> {
    if input.name.trim().is_empty()
        || input.name.chars().count() > 80
        || input.name.chars().any(char::is_control)
        || !input.avatar.parse::<u8>().is_ok_and(|n| n < 12)
        || input.grade.is_some_and(|g| !(3..=5).contains(&g))
        || !["nida", "khmer_legacy"].contains(&input.layout_id.as_str())
        || !["learning", "typing"].contains(&input.preferred_mode.as_str())
        || input.path_id.as_ref().is_some_and(|p| {
            ![
                "grade3",
                "grade4",
                "grade5",
                "beginner",
                "elementary",
                "intermediate",
                "advanced",
            ]
            .contains(&p.as_str())
        })
    {
        return Err(AppError::invalid("invalid_profile"));
    }
    Ok(())
}
pub fn create(conn: &Connection, input: ProfileInput) -> AppResult<Profile> {
    validate(&input)?;
    conn.execute("INSERT INTO profiles(name,avatar,grade,path_id,layout_id,preferred_mode,created_at,last_used_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?7)",params![input.name.trim(),input.avatar,input.grade,input.path_id,input.layout_id,input.preferred_mode,now()])?;
    let id = conn.last_insert_rowid();
    list(conn)?
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| AppError::invalid("profile_missing"))
}
pub fn update(conn: &Connection, id: i64, input: ProfileInput) -> AppResult<()> {
    validate(&input)?;
    if conn.execute("UPDATE profiles SET name=?1,avatar=?2,grade=?3,path_id=?4,layout_id=?5,preferred_mode=?6 WHERE id=?7",params![input.name.trim(),input.avatar,input.grade,input.path_id,input.layout_id,input.preferred_mode,id])?!=1{return Err(AppError::invalid("profile_missing"));}
    Ok(())
}
pub fn touch(conn: &Connection, id: i64) -> AppResult<()> {
    if conn.execute(
        "UPDATE profiles SET last_used_at=?1 WHERE id=?2 AND archived=0",
        params![now(), id],
    )? != 1
    {
        return Err(AppError::invalid("profile_missing"));
    }
    Ok(())
}
pub fn archive(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("UPDATE profiles SET archived=1 WHERE id=?1", [id])?;
    Ok(())
}
pub fn delete(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM profiles WHERE id=?1", [id])?;
    Ok(())
}
