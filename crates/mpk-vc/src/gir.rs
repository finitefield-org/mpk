//! GIR v0 JSON importer and data model.

use std::collections::BTreeSet;
use std::fmt;

use serde::{Deserialize, Serialize};

pub const GIR_SCHEMA_VERSION: &str = "mpk.gir.v0";

pub fn import_gir_json(input: &str) -> Result<GirModule, GirImportError> {
    let module = serde_json::from_str::<GirModule>(input)
        .map_err(|error| GirImportError::InvalidJson(error.to_string()))?;
    module.validate()?;
    Ok(module)
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GirModule {
    pub schema_version: String,
    #[serde(default)]
    pub packages: Vec<GirPackage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gir_hash: Option<String>,
}

impl GirModule {
    pub fn validate(&self) -> Result<(), GirImportError> {
        if self.schema_version != GIR_SCHEMA_VERSION {
            return Err(GirImportError::UnsupportedSchema {
                expected: GIR_SCHEMA_VERSION,
                found: self.schema_version.clone(),
            });
        }

        let mut function_ids = BTreeSet::new();
        for (package_index, package) in self.packages.iter().enumerate() {
            package.validate(package_index, &mut function_ids)?;
        }
        Ok(())
    }

    pub fn function(&self, id: &str) -> Option<&GirFunction> {
        self.packages
            .iter()
            .flat_map(|package| package.functions.iter())
            .find(|function| function.id == id)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GirPackage {
    pub package_path: String,
    pub name: String,
    #[serde(default)]
    pub functions: Vec<GirFunction>,
}

impl GirPackage {
    fn validate(
        &self,
        package_index: usize,
        function_ids: &mut BTreeSet<String>,
    ) -> Result<(), GirImportError> {
        if self.package_path.is_empty() {
            return Err(GirImportError::EmptyPackagePath { package_index });
        }
        if self.name.is_empty() {
            return Err(GirImportError::EmptyPackageName { package_index });
        }

        for (function_index, function) in self.functions.iter().enumerate() {
            function.validate(package_index, function_index, function_ids)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GirFunction {
    pub id: String,
    pub package: String,
    pub name: String,
    #[serde(default)]
    pub params: Vec<GirBinding>,
    #[serde(default)]
    pub results: Vec<GirBinding>,
    #[serde(default)]
    pub locals: Vec<GirBinding>,
    #[serde(default)]
    pub blocks: Vec<GirBlock>,
    pub contracts: GirContracts,
    #[serde(default)]
    pub supported_features: Vec<String>,
    #[serde(default)]
    pub rejected_features: Vec<GirRejectedFeature>,
}

impl GirFunction {
    fn validate(
        &self,
        package_index: usize,
        function_index: usize,
        function_ids: &mut BTreeSet<String>,
    ) -> Result<(), GirImportError> {
        if self.id.is_empty() {
            return Err(GirImportError::EmptyFunctionId {
                package_index,
                function_index,
            });
        }
        if self.package.is_empty() {
            return Err(GirImportError::EmptyFunctionPackage {
                function_id: self.id.clone(),
            });
        }
        if self.name.is_empty() {
            return Err(GirImportError::EmptyFunctionName {
                function_id: self.id.clone(),
            });
        }
        if !function_ids.insert(self.id.clone()) {
            return Err(GirImportError::DuplicateFunctionId {
                function_id: self.id.clone(),
            });
        }

        let mut block_labels = BTreeSet::new();
        for (block_index, block) in self.blocks.iter().enumerate() {
            block.validate(&self.id, block_index, &mut block_labels)?;
        }
        Ok(())
    }

    pub fn block(&self, label: &str) -> Option<&GirBlock> {
        self.blocks.iter().find(|block| block.label == label)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GirContracts {
    #[serde(default)]
    pub requires: Vec<GirContractExpr>,
    #[serde(default)]
    pub ensures: Vec<GirContractExpr>,
    #[serde(default)]
    pub modifies: Vec<String>,
    #[serde(default)]
    pub loops: Vec<GirLoopContract>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GirContractExpr {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub op: Option<String>,
    #[serde(default)]
    pub args: Vec<GirContractExpr>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lhs: Option<Box<GirContractExpr>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rhs: Option<Box<GirContractExpr>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<Box<GirContractExpr>>,
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<GirType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub var: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bool: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub int: Option<GirIntLiteral>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GirLoopContract {
    pub block_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(default)]
    pub invariants: Vec<GirContractExpr>,
    #[serde(default)]
    pub decreases: Vec<GirContractExpr>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GirBinding {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: GirType,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GirType {
    pub kind: GirTypeKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub length: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element: Option<Box<GirType>>,
    #[serde(default)]
    pub fields: Vec<GirFieldType>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub enum GirTypeKind {
    #[serde(rename = "bool")]
    Bool,
    #[serde(rename = "bv")]
    BitVector,
    #[serde(rename = "array")]
    Array,
    #[serde(rename = "struct")]
    Struct,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GirFieldType {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: GirType,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GirBlock {
    pub label: String,
    #[serde(default)]
    pub parameters: Vec<GirBinding>,
    #[serde(default)]
    pub instructions: Vec<GirInstruction>,
    pub terminator: GirTerminator,
}

impl GirBlock {
    fn validate(
        &self,
        function_id: &str,
        block_index: usize,
        block_labels: &mut BTreeSet<String>,
    ) -> Result<(), GirImportError> {
        if self.label.is_empty() {
            return Err(GirImportError::EmptyBlockLabel {
                function_id: function_id.to_owned(),
                block_index,
            });
        }
        if !block_labels.insert(self.label.clone()) {
            return Err(GirImportError::DuplicateBlockLabel {
                function_id: function_id.to_owned(),
                block_label: self.label.clone(),
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GirInstruction {
    pub id: String,
    pub kind: GirInstructionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub op: Option<String>,
    #[serde(rename = "type")]
    pub r#type: GirType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<GirValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base: Option<GirValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<GirValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(default)]
    pub fields: Vec<GirField>,
    #[serde(default)]
    pub elements: Vec<GirValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lhs: Option<GirValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rhs: Option<GirValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function: Option<String>,
    #[serde(default)]
    pub args: Vec<GirValue>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub enum GirInstructionKind {
    Const,
    Copy,
    BinOp,
    UnaryOp,
    Convert,
    Phi,
    Field,
    Index,
    MakeStruct,
    MakeArray,
    CallStatic,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GirField {
    pub name: String,
    pub value: GirValue,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GirTerminator {
    pub kind: GirTerminatorKind,
    #[serde(default)]
    pub values: Vec<GirValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cond: Option<GirValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub then_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub else_label: Option<String>,
    #[serde(default)]
    pub args: Vec<GirValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub enum GirTerminatorKind {
    Return,
    Jump,
    Branch,
    PanicUnsupported,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GirValue {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub var: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub int: Option<GirIntLiteral>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bool: Option<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GirIntLiteral {
    pub value: String,
    pub width: u32,
    pub signed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GirRejectedFeature {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    pub feature: String,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GirImportError {
    InvalidJson(String),
    UnsupportedSchema {
        expected: &'static str,
        found: String,
    },
    EmptyPackagePath {
        package_index: usize,
    },
    EmptyPackageName {
        package_index: usize,
    },
    EmptyFunctionId {
        package_index: usize,
        function_index: usize,
    },
    EmptyFunctionPackage {
        function_id: String,
    },
    EmptyFunctionName {
        function_id: String,
    },
    DuplicateFunctionId {
        function_id: String,
    },
    EmptyBlockLabel {
        function_id: String,
        block_index: usize,
    },
    DuplicateBlockLabel {
        function_id: String,
        block_label: String,
    },
}

impl fmt::Display for GirImportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson(message) => write!(formatter, "invalid GIR JSON: {message}"),
            Self::UnsupportedSchema { expected, found } => {
                write!(
                    formatter,
                    "unsupported GIR schema {found:?}; expected {expected:?}"
                )
            }
            Self::EmptyPackagePath { package_index } => {
                write!(formatter, "package {package_index} has empty package_path")
            }
            Self::EmptyPackageName { package_index } => {
                write!(formatter, "package {package_index} has empty name")
            }
            Self::EmptyFunctionId {
                package_index,
                function_index,
            } => write!(
                formatter,
                "function {function_index} in package {package_index} has empty id"
            ),
            Self::EmptyFunctionPackage { function_id } => {
                write!(formatter, "function {function_id:?} has empty package")
            }
            Self::EmptyFunctionName { function_id } => {
                write!(formatter, "function {function_id:?} has empty name")
            }
            Self::DuplicateFunctionId { function_id } => {
                write!(formatter, "duplicate GIR function id {function_id:?}")
            }
            Self::EmptyBlockLabel {
                function_id,
                block_index,
            } => write!(
                formatter,
                "block {block_index} in function {function_id:?} has empty label"
            ),
            Self::DuplicateBlockLabel {
                function_id,
                block_label,
            } => write!(
                formatter,
                "function {function_id:?} has duplicate block label {block_label:?}"
            ),
        }
    }
}

impl std::error::Error for GirImportError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_minimal_go2gir_module() {
        let module = import_gir_json(sample_gir_json()).expect("sample GIR imports");

        assert_eq!(module.schema_version, GIR_SCHEMA_VERSION);
        assert_eq!(module.gir_hash.as_deref(), Some("abc123"));
        let function = module
            .function("example/pkg.Identity")
            .expect("function is indexed");
        assert_eq!(function.name, "Identity");
        assert_eq!(
            function.blocks[0].terminator.kind,
            GirTerminatorKind::Return
        );
        assert_eq!(function.contracts.ensures[0].op.as_deref(), Some("eq"));
    }

    #[test]
    fn rejects_unknown_json_fields() {
        let input = sample_gir_json().replace(
            "\"schema_version\":\"mpk.gir.v0\"",
            "\"schema_version\":\"mpk.gir.v0\",\"unexpected\":true",
        );

        let error = import_gir_json(&input).expect_err("unknown field rejects");
        assert!(matches!(error, GirImportError::InvalidJson(_)));
    }

    #[test]
    fn rejects_wrong_schema() {
        let input = sample_gir_json().replace("mpk.gir.v0", "mpk.gir.v1");

        let error = import_gir_json(&input).expect_err("wrong schema rejects");
        assert_eq!(
            error,
            GirImportError::UnsupportedSchema {
                expected: GIR_SCHEMA_VERSION,
                found: "mpk.gir.v1".to_owned()
            }
        );
    }

    #[test]
    fn rejects_duplicate_function_ids() {
        let input = sample_gir_json().replace(
            "\"functions\":[{",
            "\"functions\":[{\"id\":\"example/pkg.Identity\",\"package\":\"example/pkg\",\"name\":\"Duplicate\",\"params\":[],\"results\":[],\"locals\":[],\"blocks\":[],\"contracts\":{\"requires\":[],\"ensures\":[],\"modifies\":[],\"loops\":[]},\"supported_features\":[],\"rejected_features\":[]},{",
        );

        let error = import_gir_json(&input).expect_err("duplicate function rejects");
        assert_eq!(
            error,
            GirImportError::DuplicateFunctionId {
                function_id: "example/pkg.Identity".to_owned()
            }
        );
    }

    fn sample_gir_json() -> &'static str {
        r#"{"schema_version":"mpk.gir.v0","packages":[{"package_path":"example/pkg","name":"example","functions":[{"id":"example/pkg.Identity","package":"example/pkg","name":"Identity","params":[{"name":"value","type":{"kind":"bv","width":64,"signed":true}}],"results":[{"name":"result0","type":{"kind":"bv","width":64,"signed":true}}],"locals":[],"blocks":[{"label":"entry","parameters":[],"instructions":[],"terminator":{"kind":"Return","values":[{"var":"value"}]}}],"contracts":{"requires":[],"ensures":[{"op":"eq","lhs":{"result":0},"rhs":{"var":"value"}}],"modifies":[],"loops":[]},"supported_features":["params","return"],"rejected_features":[]}]}],"gir_hash":"abc123"}"#
    }
}
