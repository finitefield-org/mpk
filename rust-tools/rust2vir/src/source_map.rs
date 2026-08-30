use crate::json::JsonValue;
use std::cmp::Ordering;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceOrigin {
    pub normalized_path: String,
    pub start: u64,
    pub end: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VirReference {
    Function {
        unit_id: String,
        function_id: String,
    },
    Instruction {
        unit_id: String,
        function_id: String,
        block_index: usize,
        instruction_index: usize,
    },
    Terminator {
        unit_id: String,
        function_id: String,
        block_index: usize,
    },
}

impl VirReference {
    fn key(&self) -> (&str, &str, u8, Option<usize>, Option<usize>) {
        match self {
            Self::Function {
                unit_id,
                function_id,
            } => (unit_id, function_id, 0, None, None),
            Self::Instruction {
                unit_id,
                function_id,
                block_index,
                instruction_index,
            } => (
                unit_id,
                function_id,
                1,
                Some(*block_index),
                Some(*instruction_index),
            ),
            Self::Terminator {
                unit_id,
                function_id,
                block_index,
            } => (unit_id, function_id, 2, Some(*block_index), None),
        }
    }

    fn json(&self) -> JsonValue {
        let mut value = BTreeMap::new();
        match self {
            Self::Function {
                unit_id,
                function_id,
            } => {
                value.insert("kind".to_owned(), string("function"));
                value.insert("unit_id".to_owned(), string(unit_id));
                value.insert("function_id".to_owned(), string(function_id));
            }
            Self::Instruction {
                unit_id,
                function_id,
                block_index,
                instruction_index,
            } => {
                value.insert("kind".to_owned(), string("instruction"));
                value.insert("unit_id".to_owned(), string(unit_id));
                value.insert("function_id".to_owned(), string(function_id));
                value.insert("block".to_owned(), string(&format!("bb{block_index}")));
                value.insert(
                    "instruction".to_owned(),
                    string(&format!("t{instruction_index}")),
                );
            }
            Self::Terminator {
                unit_id,
                function_id,
                block_index,
            } => {
                value.insert("kind".to_owned(), string("terminator"));
                value.insert("unit_id".to_owned(), string(unit_id));
                value.insert("function_id".to_owned(), string(function_id));
                value.insert("block".to_owned(), string(&format!("bb{block_index}")));
            }
        }
        JsonValue::Object(value)
    }
}

impl Ord for VirReference {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key().cmp(&other.key())
    }
}

impl PartialOrd for VirReference {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceMapEntry {
    pub reference: VirReference,
    pub origin: SourceOrigin,
}

pub fn raw_source_map(
    source_ir_hash: &str,
    mut entries: Vec<SourceMapEntry>,
    semantic_context: JsonValue,
) -> JsonValue {
    entries.sort_by(|left, right| left.reference.cmp(&right.reference));
    let entries = entries
        .into_iter()
        .map(|entry| {
            JsonValue::Object(BTreeMap::from([
                ("reference".to_owned(), entry.reference.json()),
                (
                    "origin".to_owned(),
                    JsonValue::Object(BTreeMap::from([
                        ("kind".to_owned(), string("source")),
                        ("input_kind".to_owned(), string("source")),
                        (
                            "normalized_path".to_owned(),
                            string(&entry.origin.normalized_path),
                        ),
                        (
                            "start".to_owned(),
                            JsonValue::Number(entry.origin.start.to_string()),
                        ),
                        (
                            "end".to_owned(),
                            JsonValue::Number(entry.origin.end.to_string()),
                        ),
                    ])),
                ),
            ]))
        })
        .collect();
    JsonValue::Object(BTreeMap::from([
        (
            "schema".to_owned(),
            string("mpk.rust.driver.raw_source_map.v1"),
        ),
        ("semantic_context".to_owned(), semantic_context),
        ("source_ir_schema".to_owned(), string("mpk.vir.v1")),
        ("source_ir_hash".to_owned(), string(source_ir_hash)),
        ("entries".to_owned(), JsonValue::Array(entries)),
    ]))
}

fn string(value: &str) -> JsonValue {
    JsonValue::String(value.to_owned())
}
