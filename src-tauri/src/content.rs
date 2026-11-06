use crate::error::{AppError, AppResult};
use khmer_core::{
    analysis::drill_generate,
    layout::{Layout, PreparedLayout},
    plan::{plan, prepare_target},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Mutex};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bank {
    pub id: String,
    pub level: u8,
    pub reviewed: bool,
    pub words: Vec<String>,
    pub sentences: Vec<String>,
    pub paragraphs: Vec<String>,
}
pub struct Content {
    pub root: PathBuf,
    banks: Mutex<HashMap<u8, Bank>>,
    layouts: Mutex<HashMap<String, PreparedLayout>>,
    json: Mutex<HashMap<String, serde_json::Value>>,
}
impl Content {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            banks: Mutex::new(HashMap::new()),
            layouts: Mutex::new(HashMap::new()),
            json: Mutex::new(HashMap::new()),
        }
    }
    pub fn json(&self, folder: &str, id: &str) -> AppResult<serde_json::Value> {
        if id.is_empty()
            || id.len() > 100
            || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(AppError::invalid("invalid_content_id"));
        }
        let key = if folder.is_empty() {
            format!("{id}.json")
        } else {
            format!("{folder}/{id}.json")
        };
        let mut cache = self
            .json
            .lock()
            .map_err(|_| AppError::invalid("content_locked"))?;
        if let Some(value) = cache.get(&key) {
            return Ok(value.clone());
        }
        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(self.root.join(&key))?)?;
        cache.insert(key, value.clone());
        Ok(value)
    }
    pub fn bank(&self, level: u8) -> AppResult<Bank> {
        if !(1..=5).contains(&level) {
            return Err(AppError::invalid("invalid_level"));
        }
        let mut cache = self
            .banks
            .lock()
            .map_err(|_| AppError::invalid("content_locked"))?;
        if let Some(bank) = cache.get(&level) {
            return Ok(bank.clone());
        }
        let bank: Bank = serde_json::from_str(&std::fs::read_to_string(
            self.root.join(format!("sentences/{level}.json")),
        )?)?;
        cache.insert(level, bank.clone());
        Ok(bank)
    }
    pub fn layout(&self, id: &str) -> AppResult<PreparedLayout> {
        if !["nida", "khmer_legacy"].contains(&id) {
            return Err(AppError::invalid("invalid_layout"));
        }
        let mut cache = self
            .layouts
            .lock()
            .map_err(|_| AppError::invalid("content_locked"))?;
        if let Some(layout) = cache.get(id) {
            return Ok(layout.clone());
        }
        let layout: Layout = serde_json::from_str(&std::fs::read_to_string(
            self.root.join(format!("layouts/{id}.json")),
        )?)?;
        let layout = layout
            .prepare()
            .map_err(|_| AppError::invalid("invalid_layout"))?;
        cache.insert(id.into(), layout.clone());
        Ok(layout)
    }
    pub fn practice(
        &self,
        level: u8,
        kind: &str,
        layout_id: &str,
        seed: u64,
        weak: &[String],
        long: bool,
        extra: &[String],
    ) -> AppResult<String> {
        let bank = self.bank(level)?;
        let layout = self.layout(layout_id)?;
        let mut source = match kind {
            "words" => bank.words,
            "sentences" => bank.sentences,
            "paragraphs" => bank.paragraphs,
            _ => return Err(AppError::invalid("invalid_bank_kind")),
        };
        source.extend_from_slice(extra);
        let mut entries = Vec::new();
        for text in source {
            let text = prepare_target(&text);
            let strokes =
                plan(&text, &layout).map_err(|_| AppError::invalid("unmapped_content"))?;
            entries.push((text, strokes.into_iter().map(|s| s.code).collect()));
        }
        if kind == "words" {
            for code in weak {
                if let Some(output) = layout
                    .layout
                    .keys
                    .get(code)
                    .and_then(|key| key.base.as_deref())
                {
                    let text = prepare_target(output);
                    if !text.is_empty() {
                        entries.push((text, vec![code.clone()]));
                    }
                }
            }
        }
        let count = if long {
            1500
        } else if kind == "words" {
            24
        } else if kind == "sentences" {
            4
        } else {
            1
        };
        let text = drill_generate(&entries, weak, seed, count);
        let end = khmer_core::cluster::segment(&text)
            .iter()
            .take_while(|c| c.char_end <= 20_000)
            .last()
            .map(|c| c.end)
            .unwrap_or(0);
        Ok(text[..end].to_string())
    }
}
