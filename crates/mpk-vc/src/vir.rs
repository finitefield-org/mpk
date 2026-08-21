//! Strict VIR v0 importer and exact serialized data model.
//!
//! This module intentionally models instructions and other unions with Rust
//! enums. A successfully deserialized value therefore cannot retain fields
//! that are inapplicable to its selected variant.

use crate::canonical_json::{
    canonical_json_bytes, parse_strict_json, CanonicalJsonError, StrictJsonError, StrictJsonLimits,
};
use crate::semantic_profile::{
    validate_semantic_context, validate_semantic_parameters, SemanticParameters, SemanticProfile,
    SemanticProfileError, SourceLanguage,
};
use crate::vir_validate::{validate_vir, VirValidationError};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

pub const VIR_SCHEMA_VERSION: &str = "mpk.vir.v0";
pub const VIR_INPUT_JSON_BYTES_MAX: u64 = 268_435_456;
pub const VIR_JSON_NESTING_MAX: u64 = 256;
pub const VIR_STRING_BYTES_MAX: u64 = 1_048_576;

const VIR_STRICT_JSON_LIMITS: StrictJsonLimits = StrictJsonLimits::new(
    VIR_INPUT_JSON_BYTES_MAX,
    VIR_INPUT_JSON_BYTES_MAX,
    VIR_JSON_NESTING_MAX,
    VIR_STRING_BYTES_MAX,
);

/// Strictly parses and fully validates one VIR document.
///
/// No partially validated module crosses this boundary. The importer remains
/// test-only at call sites until the atomic Go cutover.
pub fn import_vir_json(input: &[u8]) -> Result<VirModule, VirImportError> {
    let strict = parse_strict_json(input, VIR_STRICT_JSON_LIMITS)?;
    let canonical = canonical_json_bytes(&strict)?;
    let wire: WireModule = serde_json::from_slice(&canonical).map_err(|error| {
        let message = error.to_string();
        if message.contains("SemanticParameters") {
            VirImportError::Validation(VirValidationError::new("VIR_SEMANTIC_PARAMETERS", message))
        } else {
            VirImportError::InvalidShape(message)
        }
    })?;
    let module = wire.into();
    validate_vir(&module)?;
    Ok(module)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct VirModule {
    pub schema: String,
    pub source_language: SourceLanguage,
    pub semantic_profile: SemanticProfile,
    pub semantic_parameters: SemanticParameters,
    pub units: Vec<VirUnit>,
    pub vir_hash: LowercaseSha256,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireModule {
    schema: String,
    source_language: SourceLanguage,
    semantic_profile: SemanticProfile,
    semantic_parameters: SemanticParameters,
    units: Vec<VirUnit>,
    vir_hash: LowercaseSha256,
}

impl From<WireModule> for VirModule {
    fn from(wire: WireModule) -> Self {
        Self {
            schema: wire.schema,
            source_language: wire.source_language,
            semantic_profile: wire.semantic_profile,
            semantic_parameters: wire.semantic_parameters,
            units: wire.units,
            vir_hash: wire.vir_hash,
        }
    }
}

impl<'de> Deserialize<'de> for VirModule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = WireModule::deserialize(deserializer)?;
        let module = Self::from(wire);
        module
            .validate_structure()
            .map_err(serde::de::Error::custom)?;
        Ok(module)
    }
}

impl VirModule {
    pub fn validate_structure(&self) -> Result<(), VirImportError> {
        if self.schema != VIR_SCHEMA_VERSION {
            return Err(VirImportError::UnsupportedSchema {
                found: self.schema.clone(),
            });
        }
        validate_semantic_context(
            self.source_language,
            self.semantic_profile,
            &self.semantic_parameters,
        )?;

        for unit in &self.units {
            for function in &unit.functions {
                validate_semantic_parameters(
                    function.contracts.semantic_profile,
                    &function.contracts.semantic_parameters,
                )?;
                if !function.contracts.modifies.is_empty() {
                    return Err(VirImportError::NonemptyModifies {
                        function_id: function.id.clone(),
                    });
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VirUnit {
    pub id: String,
    pub name: String,
    pub type_decls: Vec<VirStructDecl>,
    pub const_decls: Vec<VirConstDecl>,
    pub functions: Vec<VirFunction>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VirStructDecl {
    pub id: String,
    pub name: String,
    pub fields: Vec<VirStructField>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VirStructField {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: VirType,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VirConstDecl {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: VirType,
    pub value: VirLiteral,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VirBinding {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: VirType,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BitVectorWidth {
    Bits8,
    Bits16,
    Bits32,
    Bits64,
}

impl BitVectorWidth {
    pub const fn bits(self) -> u32 {
        match self {
            Self::Bits8 => 8,
            Self::Bits16 => 16,
            Self::Bits32 => 32,
            Self::Bits64 => 64,
        }
    }
}

impl TryFrom<u32> for BitVectorWidth {
    type Error = BitVectorWidthError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            8 => Ok(Self::Bits8),
            16 => Ok(Self::Bits16),
            32 => Ok(Self::Bits32),
            64 => Ok(Self::Bits64),
            _ => Err(BitVectorWidthError(value)),
        }
    }
}

impl<'de> Deserialize<'de> for BitVectorWidth {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let width = u32::deserialize(deserializer)?;
        Self::try_from(width).map_err(serde::de::Error::custom)
    }
}

impl Serialize for BitVectorWidth {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.bits())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BitVectorWidthError(u32);

impl fmt::Display for BitVectorWidthError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "bitvector width must be 8, 16, 32, or 64, found {}",
            self.0
        )
    }
}

impl Error for BitVectorWidthError {}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ArrayLength(u16);

impl ArrayLength {
    pub const MAX: u16 = 256;

    pub const fn get(self) -> u16 {
        self.0
    }
}

impl TryFrom<u64> for ArrayLength {
    type Error = ArrayLengthError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value <= u64::from(Self::MAX) {
            Ok(Self(value as u16))
        } else {
            Err(ArrayLengthError(value))
        }
    }
}

impl<'de> Deserialize<'de> for ArrayLength {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let length = u64::deserialize(deserializer)?;
        Self::try_from(length).map_err(serde::de::Error::custom)
    }
}

impl Serialize for ArrayLength {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u16(self.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArrayLengthError(u64);

impl fmt::Display for ArrayLengthError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "array length must be at most {}, found {}",
            ArrayLength::MAX,
            self.0
        )
    }
}

impl Error for ArrayLengthError {}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum VirType {
    Bool {},
    Bv {
        width: BitVectorWidth,
        signed: bool,
    },
    Array {
        length: ArrayLength,
        element: Box<VirType>,
    },
    Struct {
        id: String,
    },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DecimalInteger(String);

impl DecimalInteger {
    pub fn new(value: String) -> Result<Self, DecimalIntegerError> {
        if is_canonical_decimal(&value) {
            Ok(Self(value))
        } else {
            Err(DecimalIntegerError(value))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn is_canonical_decimal(value: &str) -> bool {
    if value == "0" {
        return true;
    }
    let digits = value.strip_prefix('-').unwrap_or(value);
    !digits.is_empty()
        && !digits.starts_with('0')
        && digits.bytes().all(|byte| byte.is_ascii_digit())
}

impl TryFrom<String> for DecimalInteger {
    type Error = DecimalIntegerError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for DecimalInteger {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

impl Serialize for DecimalInteger {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecimalIntegerError(String);

impl fmt::Display for DecimalIntegerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "noncanonical decimal integer {:?}", self.0)
    }
}

impl Error for DecimalIntegerError {}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VirIntLiteral {
    pub value: DecimalInteger,
    pub width: BitVectorWidth,
    pub signed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VirBooleanLiteral {
    #[serde(rename = "bool")]
    pub value: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VirIntegerLiteral {
    pub int: VirIntLiteral,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum VirLiteral {
    Boolean(VirBooleanLiteral),
    Integer(VirIntegerLiteral),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VirVariableRef {
    pub var: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VirConstantRef {
    #[serde(rename = "const")]
    pub constant: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum VirValue {
    Variable(VirVariableRef),
    Constant(VirConstantRef),
    Boolean(VirBooleanLiteral),
    Integer(VirIntegerLiteral),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VirUnaryOperator {
    Not,
    BvNeg,
    BvNot,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VirBinaryOperator {
    Eq,
    NotEq,
    BvAdd,
    BvSub,
    BvMul,
    BvSdiv,
    BvSrem,
    BvUdiv,
    BvUrem,
    BvAnd,
    BvOr,
    BvXor,
    BvShl,
    BvAshr,
    BvLshr,
    SignedLt,
    SignedLe,
    SignedGt,
    SignedGe,
    UnsignedLt,
    UnsignedLe,
    UnsignedGt,
    UnsignedGe,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OverflowOperation {
    Add,
    Sub,
    Mul,
    Neg,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DivRemOperation {
    Div,
    Rem,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VirSafetyCheckKind {
    IntegerNoOverflow,
    DivisorNonzero,
    SignedDivremRepresentable,
    ShiftCountNonnegative,
    ShiftCountLessThanWidth,
    IndexInBounds,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum VirSafetyCheck {
    IntegerNoOverflow {
        operation: OverflowOperation,
        signed: bool,
    },
    DivisorNonzero {},
    SignedDivremRepresentable {
        operation: DivRemOperation,
    },
    ShiftCountNonnegative {},
    ShiftCountLessThanWidth {},
    IndexInBounds {},
}

impl VirSafetyCheck {
    pub const fn kind(&self) -> VirSafetyCheckKind {
        match self {
            Self::IntegerNoOverflow { .. } => VirSafetyCheckKind::IntegerNoOverflow,
            Self::DivisorNonzero { .. } => VirSafetyCheckKind::DivisorNonzero,
            Self::SignedDivremRepresentable { .. } => VirSafetyCheckKind::SignedDivremRepresentable,
            Self::ShiftCountNonnegative { .. } => VirSafetyCheckKind::ShiftCountNonnegative,
            Self::ShiftCountLessThanWidth { .. } => VirSafetyCheckKind::ShiftCountLessThanWidth,
            Self::IndexInBounds { .. } => VirSafetyCheckKind::IndexInBounds,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LowercaseSha256(String);

impl LowercaseSha256 {
    pub fn new(value: String) -> Result<Self, LowercaseSha256Error> {
        if value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            Ok(Self(value))
        } else {
            Err(LowercaseSha256Error(value))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for LowercaseSha256 {
    type Error = LowercaseSha256Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for LowercaseSha256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

impl Serialize for LowercaseSha256 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LowercaseSha256Error(String);

impl fmt::Display for LowercaseSha256Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "expected 64 lowercase hexadecimal characters, found {:?}",
            self.0
        )
    }
}

impl Error for LowercaseSha256Error {}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VirNamedValue {
    pub name: String,
    pub value: VirValue,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VirInstructionKind {
    Const,
    Copy,
    BinOp,
    UnaryOp,
    Convert,
    Field,
    Index,
    MakeStruct,
    MakeArray,
    CallStatic,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum VirInstruction {
    Const {
        id: String,
        #[serde(rename = "type")]
        r#type: VirType,
        value: VirLiteral,
        safety_checks: Vec<VirSafetyCheck>,
    },
    Copy {
        id: String,
        #[serde(rename = "type")]
        r#type: VirType,
        target: String,
        value: VirValue,
        safety_checks: Vec<VirSafetyCheck>,
    },
    BinOp {
        id: String,
        op: VirBinaryOperator,
        #[serde(rename = "type")]
        r#type: VirType,
        lhs: VirValue,
        rhs: VirValue,
        safety_checks: Vec<VirSafetyCheck>,
    },
    UnaryOp {
        id: String,
        op: VirUnaryOperator,
        #[serde(rename = "type")]
        r#type: VirType,
        value: VirValue,
        safety_checks: Vec<VirSafetyCheck>,
    },
    Convert {
        id: String,
        #[serde(rename = "type")]
        r#type: VirType,
        value: VirValue,
        safety_checks: Vec<VirSafetyCheck>,
    },
    Field {
        id: String,
        #[serde(rename = "type")]
        r#type: VirType,
        base: VirValue,
        field: String,
        safety_checks: Vec<VirSafetyCheck>,
    },
    Index {
        id: String,
        #[serde(rename = "type")]
        r#type: VirType,
        base: VirValue,
        index: VirValue,
        safety_checks: Vec<VirSafetyCheck>,
    },
    MakeStruct {
        id: String,
        #[serde(rename = "type")]
        r#type: VirType,
        fields: Vec<VirNamedValue>,
        safety_checks: Vec<VirSafetyCheck>,
    },
    MakeArray {
        id: String,
        #[serde(rename = "type")]
        r#type: VirType,
        elements: Vec<VirValue>,
        safety_checks: Vec<VirSafetyCheck>,
    },
    CallStatic {
        id: String,
        #[serde(rename = "type")]
        r#type: VirType,
        function: String,
        contract_hash: LowercaseSha256,
        args: Vec<VirValue>,
        safety_checks: Vec<VirSafetyCheck>,
    },
}

impl VirInstruction {
    pub const fn kind(&self) -> VirInstructionKind {
        match self {
            Self::Const { .. } => VirInstructionKind::Const,
            Self::Copy { .. } => VirInstructionKind::Copy,
            Self::BinOp { .. } => VirInstructionKind::BinOp,
            Self::UnaryOp { .. } => VirInstructionKind::UnaryOp,
            Self::Convert { .. } => VirInstructionKind::Convert,
            Self::Field { .. } => VirInstructionKind::Field,
            Self::Index { .. } => VirInstructionKind::Index,
            Self::MakeStruct { .. } => VirInstructionKind::MakeStruct,
            Self::MakeArray { .. } => VirInstructionKind::MakeArray,
            Self::CallStatic { .. } => VirInstructionKind::CallStatic,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VirTerminatorKind {
    Return,
    Jump,
    Branch,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum VirTerminator {
    Return {
        values: Vec<VirValue>,
    },
    Jump {
        label: String,
        args: Vec<VirValue>,
    },
    Branch {
        cond: VirValue,
        then_label: String,
        then_args: Vec<VirValue>,
        else_label: String,
        else_args: Vec<VirValue>,
    },
}

impl VirTerminator {
    pub const fn kind(&self) -> VirTerminatorKind {
        match self {
            Self::Return { .. } => VirTerminatorKind::Return,
            Self::Jump { .. } => VirTerminatorKind::Jump,
            Self::Branch { .. } => VirTerminatorKind::Branch,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VirBlock {
    pub label: String,
    pub parameters: Vec<VirBinding>,
    pub instructions: Vec<VirInstruction>,
    pub terminator: VirTerminator,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VirFeature {
    Array,
    Branch,
    CallStatic,
    ConstantDecl,
    Conversion,
    CyclicCfg,
    MutableLocal,
    Struct,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VirFunction {
    pub id: String,
    pub unit_id: String,
    pub name: String,
    pub params: Vec<VirBinding>,
    pub results: Vec<VirBinding>,
    pub locals: Vec<VirBinding>,
    pub blocks: Vec<VirBlock>,
    pub contracts: VirContract,
    pub features_used: Vec<VirFeature>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VirResultRef {
    pub result: u32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VirContractUnaryOperator {
    Not,
    BvNeg,
    BvNot,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VirContractNaryOperator {
    And,
    Or,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VirContractConvertOperator {
    Convert,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VirContractUnaryExpr {
    pub op: VirContractUnaryOperator,
    pub value: Box<VirContractExpr>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VirContractNaryExpr {
    pub op: VirContractNaryOperator,
    pub args: Vec<VirContractExpr>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VirContractBinaryExpr {
    pub op: VirBinaryOperator,
    pub lhs: Box<VirContractExpr>,
    pub rhs: Box<VirContractExpr>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VirContractConvertExpr {
    pub op: VirContractConvertOperator,
    pub value: Box<VirContractExpr>,
    #[serde(rename = "type")]
    pub r#type: VirType,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum VirContractExpr {
    Variable(VirVariableRef),
    Result(VirResultRef),
    Boolean(VirBooleanLiteral),
    Integer(VirIntegerLiteral),
    Unary(VirContractUnaryExpr),
    Nary(VirContractNaryExpr),
    Binary(VirContractBinaryExpr),
    Convert(VirContractConvertExpr),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VirLoopContract {
    pub header: String,
    pub invariants: Vec<VirContractExpr>,
    pub decreases: Vec<VirContractExpr>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VirPanicPolicy {
    Forbidden,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VirTermination {
    Total,
    Partial,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VirContract {
    pub unit_id: String,
    pub function_id: String,
    pub semantic_profile: SemanticProfile,
    pub semantic_parameters: SemanticParameters,
    pub requires: Vec<VirContractExpr>,
    pub ensures: Vec<VirContractExpr>,
    pub modifies: Vec<String>,
    pub panic: VirPanicPolicy,
    pub termination: VirTermination,
    pub loops: Vec<VirLoopContract>,
    pub contract_hash: LowercaseSha256,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VirImportError {
    StrictJson(StrictJsonError),
    CanonicalJson(CanonicalJsonError),
    InvalidShape(String),
    UnsupportedSchema { found: String },
    SemanticProfile(SemanticProfileError),
    Validation(VirValidationError),
    NonemptyModifies { function_id: String },
}

impl fmt::Display for VirImportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StrictJson(error) => write!(formatter, "invalid strict VIR JSON: {error}"),
            Self::CanonicalJson(error) => {
                write!(formatter, "failed to normalize strict VIR JSON: {error}")
            }
            Self::InvalidShape(message) => write!(formatter, "invalid VIR shape: {message}"),
            Self::UnsupportedSchema { found } => write!(
                formatter,
                "unsupported VIR schema {found:?}; expected {VIR_SCHEMA_VERSION:?}"
            ),
            Self::SemanticProfile(error) => {
                write!(formatter, "invalid VIR semantic context: {error}")
            }
            Self::Validation(error) => write!(formatter, "invalid VIR: {error}"),
            Self::NonemptyModifies { function_id } => write!(
                formatter,
                "function {function_id:?} has a nonempty VIR v0 modifies list"
            ),
        }
    }
}

impl Error for VirImportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::StrictJson(error) => Some(error),
            Self::CanonicalJson(error) => Some(error),
            Self::SemanticProfile(error) => Some(error),
            Self::Validation(error) => Some(error),
            Self::InvalidShape(_)
            | Self::UnsupportedSchema { .. }
            | Self::NonemptyModifies { .. } => None,
        }
    }
}

impl From<StrictJsonError> for VirImportError {
    fn from(error: StrictJsonError) -> Self {
        Self::StrictJson(error)
    }
}

impl From<CanonicalJsonError> for VirImportError {
    fn from(error: CanonicalJsonError) -> Self {
        Self::CanonicalJson(error)
    }
}

impl From<SemanticProfileError> for VirImportError {
    fn from(error: SemanticProfileError) -> Self {
        Self::SemanticProfile(error)
    }
}

impl From<VirValidationError> for VirImportError {
    fn from(error: VirValidationError) -> Self {
        Self::Validation(error)
    }
}
