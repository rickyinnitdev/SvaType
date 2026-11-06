use crate::{
    content::Content,
    error::{AppError, AppResult},
};
use rusqlite::{params, Connection};
use serde_json::{json, Value};
pub fn list(conn: &Connection) -> AppResult<Vec<Value>> {
    Ok(conn.prepare("SELECT id,kind,category,json FROM content_overrides WHERE deleted=0 ORDER BY id DESC")?.query_map([],|r|{let text:String=r.get(3)?;let value:Value=serde_json::from_str(&text).map_err(|_|rusqlite::Error::InvalidQuery)?;Ok(json!({"id":r.get::<_,i64>(0)?,"kind":r.get::<_,String>(1)?,"category":r.get::<_,Option<String>>(2)?,"value":value}))})?.collect::<Result<_,_>>()?)
}
pub fn apply_vocab(conn: &Connection, mut category: Value) -> AppResult<Value> {
    let id = category["id"].as_str().unwrap_or("").to_owned();
    let entries:Vec<String>=conn.prepare("SELECT json FROM content_overrides WHERE kind='word' AND category=?1 AND deleted=0 ORDER BY id")?.query_map([id],|r|r.get(0))?.collect::<Result<_,_>>()?;
    if let Some(words) = category["words"].as_array_mut() {
        for raw in entries {
            let value: Value = serde_json::from_str(&raw)?;
            if let Some(at) = words.iter().position(|word| word["id"] == value["id"]) {
                words[at] = value;
            } else {
                words.push(value);
            }
        }
    }
    Ok(category)
}
pub fn sentences(conn: &Connection, level: u8) -> AppResult<Vec<String>> {
    let rows:Vec<String>=conn.prepare("SELECT json FROM content_overrides WHERE kind='sentence' AND category=?1 AND deleted=0 ORDER BY id")?.query_map([level.to_string()],|r|r.get(0))?.collect::<Result<_,_>>()?;
    rows.into_iter()
        .map(|raw| {
            let value: Value = serde_json::from_str(&raw)?;
            Ok(value["text"].as_str().unwrap_or("").to_owned())
        })
        .collect()
}
pub fn save(
    conn: &mut Connection,
    content: &Content,
    id: Option<i64>,
    kind: &str,
    category: &str,
    mut value: Value,
) -> AppResult<i64> {
    let tx = conn.transaction()?;
    let conn = &tx;
    if !["word", "sentence"].contains(&kind)
        || !value.is_object()
        || value.to_string().len() > 30000
    {
        return Err(AppError::invalid("invalid_override"));
    }
    if kind == "word" {
        content.json("vocabulary", category)?;
    } else if !category.parse::<u8>().is_ok_and(|n| (1..=5).contains(&n)) {
        return Err(AppError::invalid("invalid_level"));
    }
    let field = if kind == "word" { "word" } else { "text" };
    let text = khmer_core::plan::prepare_target(value[field].as_str().unwrap_or(""));
    if text.is_empty() || text.chars().count() > 2000 {
        return Err(AppError::invalid("invalid_content"));
    }
    for layout in ["nida", "khmer_legacy"] {
        khmer_core::plan::validate_target(&text, &content.layout(layout)?)
            .map_err(|_| AppError::invalid("unmapped_content"))?;
    }
    value[field] = json!(text);
    value["reviewed"] = json!(value["reviewed"].as_bool().unwrap_or(false));
    if kind == "word" {
        let meaning = value["meaning_km"].as_str().unwrap_or("");
        let example = khmer_core::plan::prepare_target(value["example_km"].as_str().unwrap_or(""));
        if meaning.trim().is_empty() || example.is_empty() {
            return Err(AppError::invalid("invalid_content"));
        }
        for layout in ["nida", "khmer_legacy"] {
            khmer_core::plan::validate_target(&example, &content.layout(layout)?)
                .map_err(|_| AppError::invalid("unmapped_content"))?;
        }
        value["example_km"] = json!(example);
        value["image"] = Value::Null;
        value["audio"] = Value::Null;
        value["grade"] = json!([3, 4, 5]);
        if value["emoji"]
            .as_str()
            .is_none_or(|s| s.chars().count() > 12)
        {
            value["emoji"] = json!("📖");
        }
        if !value["level"]
            .as_u64()
            .is_some_and(|n| (1..=5).contains(&n))
        {
            value["level"] = json!(1);
        }
        if !value["meaning_en"].is_string() {
            value["meaning_en"] = json!("");
        }
    }
    let row_id = if let Some(id) = id {
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM content_overrides WHERE id=?1)",
            [id],
            |r| r.get(0),
        )?;
        if !exists {
            return Err(AppError::invalid("missing_override"));
        }
        id
    } else {
        conn.execute(
            "INSERT INTO content_overrides(kind,category,json,created_at) VALUES(?1,?2,'{}',?3)",
            params![kind, category, super::db::now()],
        )?;
        conn.last_insert_rowid()
    };
    let supplied = value["id"].as_str().unwrap_or("");
    if supplied.is_empty() {
        value["id"] = json!(format!("teacher_{kind}_{row_id}"));
    } else if supplied.len() > 100
        || !supplied
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(AppError::invalid("invalid_content_id"));
    }
    conn.execute(
        "UPDATE content_overrides SET kind=?1,category=?2,json=?3,deleted=0 WHERE id=?4",
        params![kind, category, serde_json::to_string(&value)?, row_id],
    )?;
    tx.commit()?;
    Ok(row_id)
}
