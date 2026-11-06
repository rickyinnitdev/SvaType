use crate::{
    content::Content,
    error::{AppError, AppResult},
};
use khmer_core::{
    analysis::{weak_keys, WeakKey},
    layout::Layer,
    metrics::{Confusion, KeyStats},
    plan::{prepare_target, validate_target},
};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize)]
pub struct CustomText {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub created_at: i64,
}
#[derive(Debug, Serialize)]
pub struct Analysis {
    pub keys: Vec<KeyStats>,
    pub weak: Vec<WeakKey>,
    pub confusions: Vec<Confusion>,
    pub slowest: Vec<KeyStats>,
    pub heat: std::collections::BTreeMap<String, f64>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct CustomInput {
    pub profile: i64,
    pub title: String,
    pub body: String,
    pub layout_id: String,
}
pub fn custom_list(conn: &Connection, profile: i64) -> AppResult<Vec<CustomText>> {
    Ok(conn.prepare("SELECT id,title,body,created_at FROM custom_texts WHERE profile_id=?1 ORDER BY created_at DESC")?.query_map([profile],|r|Ok(CustomText{id:r.get(0)?,title:r.get(1)?,body:r.get(2)?,created_at:r.get(3)?}))?.collect::<Result<_,_>>()?)
}
pub fn custom_save(conn: &Connection, content: &Content, input: CustomInput) -> AppResult<i64> {
    let body = prepare_target(&input.body);
    if body.trim().is_empty()
        || body.chars().count() > 20_000
        || input.title.trim().is_empty()
        || input.title.chars().count() > 100
    {
        return Err(AppError::invalid("invalid_custom_text"));
    }
    validate_target(&body, &content.layout(&input.layout_id)?)
        .map_err(|_| AppError::invalid("unmapped_content"))?;
    conn.execute(
        "INSERT INTO custom_texts(profile_id,title,body,created_at) VALUES(?1,?2,?3,?4)",
        params![input.profile, input.title.trim(), body, super::db::now()],
    )?;
    Ok(conn.last_insert_rowid())
}
pub fn custom_delete(conn: &Connection, profile: i64, id: i64) -> AppResult<()> {
    conn.execute(
        "DELETE FROM custom_texts WHERE profile_id=?1 AND id=?2",
        params![profile, id],
    )?;
    Ok(())
}
pub fn analysis(conn: &Connection, profile: i64, n: u32) -> AppResult<Analysis> {
    let n = n.clamp(1, 100);
    let mut statement=conn.prepare("SELECT k.key_code,k.layer,SUM(k.hits),SUM(k.errors),CASE WHEN SUM(k.hits)=0 THEN 0 ELSE SUM(k.avg_hes_ms*k.hits)/SUM(k.hits) END FROM session_keys k WHERE k.session_id IN(SELECT id FROM sessions WHERE profile_id=?1 ORDER BY started_at DESC,id DESC LIMIT ?2) GROUP BY k.key_code,k.layer")?;
    let keys = statement
        .query_map(params![profile, n], |r| {
            let layer: String = r.get(1)?;
            Ok(KeyStats {
                code: r.get(0)?,
                layer: serde_json::from_str::<Layer>(&format!("\"{layer}\"")).unwrap_or_default(),
                hits: r.get(2)?,
                errors: r.get(3)?,
                avg_hes_ms: r.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let confusions=conn.prepare("SELECT c.expected,c.typed,SUM(c.count) FROM session_confusions c WHERE c.session_id IN(SELECT id FROM sessions WHERE profile_id=?1 ORDER BY started_at DESC,id DESC LIMIT ?2) GROUP BY c.expected,c.typed ORDER BY SUM(c.count) DESC LIMIT 10")?.query_map(params![profile,n],|r|Ok(Confusion{expected:r.get(0)?,typed:r.get(1)?,count:r.get(2)?}))?.collect::<Result<_,_>>()?;
    let weak = weak_keys(&keys, 5);
    let mut slowest = keys
        .iter()
        .filter(|k| k.hits > 0)
        .cloned()
        .collect::<Vec<_>>();
    slowest.sort_by(|a, b| b.avg_hes_ms.total_cmp(&a.avg_hes_ms));
    slowest.truncate(5);
    let mut grouped = std::collections::BTreeMap::<String, (u64, u64)>::new();
    for key in &keys {
        let counts = grouped.entry(key.code.clone()).or_default();
        counts.0 += u64::from(key.hits);
        counts.1 += u64::from(key.errors);
    }
    let heat = grouped
        .into_iter()
        .map(|(code, (hits, errors))| {
            (
                code,
                if hits + errors == 0 {
                    0.0
                } else {
                    errors as f64 / (hits + errors) as f64
                },
            )
        })
        .collect();
    Ok(Analysis {
        keys,
        weak,
        confusions,
        slowest,
        heat,
    })
}
