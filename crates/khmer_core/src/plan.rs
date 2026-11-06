use crate::{
    cluster::segment,
    layout::{Layer, PreparedLayout},
    normalize::canonicalize,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedStroke {
    pub code: String,
    pub layer: Layer,
    pub produces: String,
    pub cluster_index: usize,
    pub finger: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanError {
    pub code: String,
    pub offset: usize,
    pub character: String,
}

pub fn validate_target(text: &str, layout: &PreparedLayout) -> Result<(), PlanError> {
    plan(text, layout).map(|_| ())
}
pub fn plan(text: &str, layout: &PreparedLayout) -> Result<Vec<PlannedStroke>, PlanError> {
    if let Some(offset) = text.find("េា") {
        return Err(PlanError {
            code: "split_vowel".into(),
            offset,
            character: "េា".into(),
        });
    }
    let canonical = canonicalize(text);
    let clusters = segment(&canonical);
    let mut result = Vec::with_capacity(canonical.len() / 3);
    let mut offset = 0;
    let mut cluster_index = 0;
    while offset < canonical.len() {
        while cluster_index + 1 < clusters.len() && offset >= clusters[cluster_index].end {
            cluster_index += 1;
        }
        let rest = &canonical[offset..];
        if rest.starts_with('\n') {
            result.push(PlannedStroke {
                code: "Enter".into(),
                layer: Layer::Base,
                produces: "\n".into(),
                cluster_index,
                finger: "R5".into(),
            });
            offset += 1;
            continue;
        }
        let Some(output) = layout.longest(rest) else {
            return Err(PlanError {
                code: "unmapped_character".into(),
                offset,
                character: rest.chars().next().unwrap().to_string(),
            });
        };
        result.push(PlannedStroke {
            code: output.code.clone(),
            layer: output.layer,
            produces: output.produces.clone(),
            cluster_index,
            finger: output.finger.clone(),
        });
        offset += output.produces.len();
    }
    Ok(result)
}
