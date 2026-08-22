use crate::json::{self, JsonValue};
use crate::sha256::{digest, hex};
use std::collections::BTreeMap;
use std::sync::Arc;

pub const CONTRACT_SCHEMA: &str = "mpk.rust.contract.v0";
pub const RUST_SEMANTIC_PROFILE: &str = "mpk.rust.checked.v0";
pub const CONTRACT_CLAUSES_MAX: usize = 64;
pub const CONTRACT_NODES_FUNCTION_MAX: usize = 1_024;
pub const CONTRACT_NODES_CLOSURE_MAX: usize = 8_192;
pub const CONTRACT_EXPRESSION_DEPTH_MAX: usize = 32;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ContractCode {
    Json,
    Schema,
    Shape,
    Identity,
    Duplicate,
    Unused,
    Missing,
    Resolution,
    Profile,
    Type,
    Operator,
    Limit,
    Hash,
}

impl ContractCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Json => "RUST_CONTRACT_JSON",
            Self::Schema => "RUST_CONTRACT_SCHEMA",
            Self::Shape => "RUST_CONTRACT_SHAPE",
            Self::Identity => "RUST_CONTRACT_IDENTITY",
            Self::Duplicate => "RUST_CONTRACT_DUPLICATE",
            Self::Unused => "RUST_CONTRACT_UNUSED",
            Self::Missing => "RUST_CONTRACT_MISSING",
            Self::Resolution => "RUST_CONTRACT_RESOLUTION",
            Self::Profile => "RUST_CONTRACT_PROFILE",
            Self::Type => "RUST_CONTRACT_TYPE",
            Self::Operator => "RUST_CONTRACT_OPERATOR",
            Self::Limit => "RUST_CONTRACT_LIMIT",
            Self::Hash => "RUST_CONTRACT_HASH",
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            Self::Json => "contract sidecar is not strict JSON",
            Self::Schema => "contract schema is not mpk.rust.contract.v0",
            Self::Shape => "contract object has an invalid closed shape",
            Self::Identity => "contract function or identifier is not canonical",
            Self::Duplicate => "multiple contract sidecars target one function",
            Self::Unused => "contract sidecar does not target the selected call closure",
            Self::Missing => "a call-closure function has no contract sidecar",
            Self::Resolution => {
                "contract name or result does not resolve in the function signature"
            }
            Self::Profile => "contract semantic profile or target does not match the request",
            Self::Type => "contract expression has an invalid type",
            Self::Operator => "contract operator is outside the closed Rust contract profile",
            Self::Limit => "contract expression limit exceeded",
            Self::Hash => "normalized contract hash does not match recomputation",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractError {
    pub code: ContractCode,
    pub normalized_path: Option<String>,
    pub function_id: Option<String>,
}

impl ContractError {
    pub(crate) fn new(code: ContractCode, input: Option<&ContractInput>) -> Self {
        Self {
            code,
            normalized_path: input.map(|input| input.normalized_path.clone()),
            function_id: None,
        }
    }

    pub(crate) fn for_function(
        code: ContractCode,
        input: Option<&ContractInput>,
        function_id: &str,
    ) -> Self {
        let mut error = Self::new(code, input);
        error.function_id = Some(function_id.to_owned());
        error
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractInput {
    pub normalized_path: String,
    pub bytes: Arc<[u8]>,
    pub raw_input_sha256: String,
}

impl ContractInput {
    pub fn new(normalized_path: impl Into<String>, bytes: impl Into<Arc<[u8]>>) -> Self {
        let bytes = bytes.into();
        Self {
            normalized_path: normalized_path.into(),
            raw_input_sha256: hex(&digest(&bytes)),
            bytes,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ContractType {
    Bool,
    BitVector {
        width: u8,
        signed: bool,
    },
    Array {
        element: Box<ContractType>,
        length: u64,
    },
    Struct {
        id: String,
    },
}

impl ContractType {
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool)
    }

    pub fn as_bit_vector(&self) -> Option<(u8, bool)> {
        match self {
            Self::BitVector { width, signed } => Some((*width, *signed)),
            _ => None,
        }
    }

    pub fn is_aggregate(&self) -> bool {
        matches!(self, Self::Array { .. } | Self::Struct { .. })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractFunction {
    pub function_id: String,
    pub parameter_names: Vec<String>,
    pub parameter_types: Vec<ContractType>,
    pub result_type: ContractType,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NormalizedContract {
    pub normalized_path: String,
    pub raw_input_sha256: String,
    pub function_id: String,
    pub contract_hash: String,
    pub value: JsonValue,
}

impl NormalizedContract {
    pub fn canonical_json(&self) -> Result<Vec<u8>, ContractError> {
        json::canonical(&self.value)
            .map_err(|_| ContractError::for_function(ContractCode::Hash, None, &self.function_id))
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ContractSet {
    contracts: Vec<NormalizedContract>,
}

impl ContractSet {
    pub(crate) fn new(contracts: Vec<NormalizedContract>) -> Self {
        Self { contracts }
    }

    pub fn contracts(&self) -> &[NormalizedContract] {
        &self.contracts
    }

    pub fn get(&self, function_id: &str) -> Option<&NormalizedContract> {
        self.contracts
            .binary_search_by_key(&function_id, |contract| contract.function_id.as_str())
            .ok()
            .map(|index| &self.contracts[index])
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ParsedContract {
    pub input: ContractInput,
    pub semantic_profile: String,
    pub target_pointer_width: i64,
    pub function: String,
    pub requires: Vec<JsonValue>,
    pub ensures: Vec<JsonValue>,
}

pub(crate) fn parse_contract(input: ContractInput) -> Result<ParsedContract, ContractError> {
    let value = json::parse_with_depth(&input.bytes, input.bytes.len(), 128)
        .map_err(|_| ContractError::new(ContractCode::Json, Some(&input)))?;
    let root = value
        .as_object()
        .ok_or_else(|| ContractError::new(ContractCode::Shape, Some(&input)))?;

    if root.get("schema").and_then(JsonValue::as_str) != Some(CONTRACT_SCHEMA) {
        return Err(ContractError::new(ContractCode::Schema, Some(&input)));
    }
    if !exact_fields(
        root,
        &[
            "schema",
            "semantic_profile",
            "target_pointer_width",
            "function",
            "requires",
            "ensures",
            "modifies",
            "panic",
            "termination",
            "loops",
        ],
    ) {
        return Err(ContractError::new(ContractCode::Shape, Some(&input)));
    }

    let semantic_profile = root
        .get("semantic_profile")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| ContractError::new(ContractCode::Shape, Some(&input)))?
        .to_owned();
    let width = root
        .get("target_pointer_width")
        .and_then(JsonValue::integer)
        .ok_or_else(|| ContractError::new(ContractCode::Shape, Some(&input)))?;
    let function = root
        .get("function")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| ContractError::new(ContractCode::Shape, Some(&input)))?
        .to_owned();
    let requires = root
        .get("requires")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| ContractError::new(ContractCode::Shape, Some(&input)))?
        .to_vec();
    let ensures = root
        .get("ensures")
        .and_then(JsonValue::as_array)
        .filter(|clauses| !clauses.is_empty())
        .ok_or_else(|| ContractError::new(ContractCode::Shape, Some(&input)))?
        .to_vec();
    if root.get("modifies").and_then(JsonValue::as_array) != Some(&[])
        || root.get("loops").and_then(JsonValue::as_array) != Some(&[])
        || root.get("panic").and_then(JsonValue::as_str) != Some("forbidden")
        || root.get("termination").and_then(JsonValue::as_str) != Some("total")
    {
        return Err(ContractError::new(ContractCode::Shape, Some(&input)));
    }

    Ok(ParsedContract {
        input,
        semantic_profile,
        target_pointer_width: width,
        function,
        requires,
        ensures,
    })
}

pub(crate) fn exact_fields(root: &BTreeMap<String, JsonValue>, fields: &[&str]) -> bool {
    root.len() == fields.len() && fields.iter().all(|field| root.contains_key(*field))
}

pub(crate) fn valid_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    value != "_"
        && (first.is_ascii_alphabetic() || first == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

pub(crate) fn valid_function_id(value: &str) -> bool {
    value.len() <= 1_024
        && value.split("::").count() >= 2
        && value.split("::").all(valid_identifier)
}
