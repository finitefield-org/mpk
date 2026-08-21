//! Verification-condition obligation skeletons.

use serde::{Deserialize, Serialize};

use crate::expr_encode::MpkExprTerm;
use crate::program_wp::{ProgramVcMember, ProgramVcMemberKind};
use crate::semantic_profile::{SemanticParameters, SemanticProfile};
use crate::type_encode::MpkTypeTerm;

/// Frozen public VC schema selected by the VIR frontend cutover.
pub const VC_SCHEMA_VERSION: &str = "mpk.vc.v1";
pub const VERIFICATION_LIMIT_PROFILE: &str = "mpk.verify.limits.v0";

/// Exact, closed `mpk.vc.v1` artifact model.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VcDocument {
    pub schema: String,
    pub source_ir_schema: String,
    pub source_ir_hash: String,
    pub input_set_hash: String,
    pub semantic_profile: SemanticProfile,
    pub semantic_parameters: SemanticParameters,
    pub verification_limit_profile: String,
    pub functions: Vec<VcFunction>,
    pub vc_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VcFunction {
    pub function_id: String,
    pub contract_hash: String,
    pub parameters: Vec<VcBinder>,
    pub requires: Vec<VcTerm>,
    pub members: Vec<VcMember>,
    pub groups: Vec<VcGroup>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VcBinder {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: VcTypeTerm,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VcMember {
    pub id: String,
    pub function_id: String,
    pub kind: VcMemberKind,
    pub local_binders: Vec<VcTypeTerm>,
    pub assumptions: Vec<VcTerm>,
    pub conclusion: VcTerm,
    pub group_id: String,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VcMemberKind {
    CalleePanicFree,
    CalleePrecondition,
    LoopDecreases,
    LoopExit,
    LoopInitialization,
    LoopPreservation,
    OperationSafety,
    Postcondition,
}

impl VcMemberKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CalleePanicFree => "callee_panic_free",
            Self::CalleePrecondition => "callee_precondition",
            Self::LoopDecreases => "loop_decreases",
            Self::LoopExit => "loop_exit",
            Self::LoopInitialization => "loop_initialization",
            Self::LoopPreservation => "loop_preservation",
            Self::OperationSafety => "operation_safety",
            Self::Postcondition => "postcondition",
        }
    }

    pub const fn required_group(self) -> VcGroupKind {
        match self {
            Self::CalleePanicFree | Self::OperationSafety => VcGroupKind::PanicFree,
            Self::CalleePrecondition
            | Self::LoopDecreases
            | Self::LoopExit
            | Self::LoopInitialization
            | Self::LoopPreservation
            | Self::Postcondition => VcGroupKind::Contract,
        }
    }
}

impl From<ProgramVcMemberKind> for VcMemberKind {
    fn from(value: ProgramVcMemberKind) -> Self {
        match value {
            ProgramVcMemberKind::CalleePanicFree => Self::CalleePanicFree,
            ProgramVcMemberKind::CalleePrecondition => Self::CalleePrecondition,
            ProgramVcMemberKind::LoopDecreases => Self::LoopDecreases,
            ProgramVcMemberKind::LoopExit => Self::LoopExit,
            ProgramVcMemberKind::LoopInitialization => Self::LoopInitialization,
            ProgramVcMemberKind::LoopPreservation => Self::LoopPreservation,
            ProgramVcMemberKind::OperationSafety => Self::OperationSafety,
            ProgramVcMemberKind::Postcondition => Self::Postcondition,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VcGroup {
    pub id: String,
    pub kind: VcGroupKind,
    pub declaration_name: String,
    pub member_ids: Vec<String>,
    pub dependencies: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VcGroupKind {
    Contract,
    PanicFree,
}

impl VcGroupKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Contract => "contract",
            Self::PanicFree => "panic_free",
        }
    }
}

/// Exact type-term union at the VC boundary.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum VcTypeTerm {
    Constant {
        name: String,
    },
    Apply {
        function: String,
        args: Vec<VcTypeTerm>,
    },
    NatLiteral {
        value: u64,
    },
    StringLiteral {
        value: String,
    },
}

impl From<&MpkTypeTerm> for VcTypeTerm {
    fn from(value: &MpkTypeTerm) -> Self {
        match value {
            MpkTypeTerm::Constant { name } => Self::Constant { name: name.clone() },
            MpkTypeTerm::Apply { function, args } => Self::Apply {
                function: function.clone(),
                args: args.iter().map(Self::from).collect(),
            },
            MpkTypeTerm::NatLiteral { value } => Self::NatLiteral { value: *value },
            MpkTypeTerm::StringLiteral { value } => Self::StringLiteral {
                value: value.clone(),
            },
        }
    }
}

/// Exact expression-term union at the VC boundary. The removed `result`
/// variant is intentionally absent, so serde rejects it during shape checks.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum VcTerm {
    Var {
        name: String,
    },
    Bound {
        index: u32,
    },
    Constant {
        name: String,
    },
    BitVecLiteral {
        value: String,
        width: u32,
        signed: bool,
    },
    Apply {
        function: String,
        args: Vec<VcTerm>,
    },
    Convert {
        value: Box<VcTerm>,
        target: VcTypeTerm,
    },
    Forall {
        binder_type: VcTypeTerm,
        body: Box<VcTerm>,
    },
}

impl TryFrom<&MpkExprTerm> for VcTerm {
    type Error = VcTermConversionError;

    fn try_from(value: &MpkExprTerm) -> Result<Self, Self::Error> {
        match value {
            MpkExprTerm::Var { name } => Ok(Self::Var { name: name.clone() }),
            MpkExprTerm::Bound { index } => Ok(Self::Bound { index: *index }),
            MpkExprTerm::Result { .. } => Err(VcTermConversionError::RetiredResult),
            MpkExprTerm::Constant { name } => Ok(Self::Constant { name: name.clone() }),
            MpkExprTerm::BitVecLiteral {
                value,
                width,
                signed,
            } => Ok(Self::BitVecLiteral {
                value: value.clone(),
                width: *width,
                signed: *signed,
            }),
            MpkExprTerm::Apply { function, args } => Ok(Self::Apply {
                function: function.clone(),
                args: args.iter().map(Self::try_from).collect::<Result<_, _>>()?,
            }),
            MpkExprTerm::Convert { value, target } => Ok(Self::Convert {
                value: Box::new(Self::try_from(value.as_ref())?),
                target: VcTypeTerm::from(target),
            }),
            MpkExprTerm::Forall { binder_type, body } => Ok(Self::Forall {
                binder_type: VcTypeTerm::from(binder_type),
                body: Box::new(Self::try_from(body.as_ref())?),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VcTermConversionError {
    RetiredResult,
}

impl std::fmt::Display for VcTermConversionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RetiredResult => formatter.write_str("retired result term reached VC v1"),
        }
    }
}

impl std::error::Error for VcTermConversionError {}

impl TryFrom<&ProgramVcMember> for VcMember {
    type Error = VcTermConversionError;

    fn try_from(value: &ProgramVcMember) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.clone(),
            function_id: value.function_id.clone(),
            kind: value.kind.into(),
            local_binders: value.local_binders.iter().map(VcTypeTerm::from).collect(),
            assumptions: value
                .assumptions
                .iter()
                .map(VcTerm::try_from)
                .collect::<Result<_, _>>()?,
            conclusion: VcTerm::try_from(&value.conclusion)?,
            group_id: value.group_id.clone(),
        })
    }
}

/// Vector-only linked source projection from VC_V1.md section 11. Production
/// generation constructs the same projection from validated VIR and manifest
/// values before using the common validator.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VcSourceContext {
    pub id: String,
    pub source_ir_schema: String,
    pub source_ir_hash: String,
    pub input_set_hash: String,
    pub semantic_profile: SemanticProfile,
    pub semantic_parameters: SemanticParameters,
    pub verification_limit_profile: String,
    pub functions: Vec<VcSourceFunction>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VcSourceFunction {
    pub function_id: String,
    pub contract_hash: String,
    pub direct_callees: Vec<String>,
    pub parameters: Vec<VcBinder>,
    pub requires: Vec<VcTerm>,
    pub regenerated_members: Vec<VcMember>,
}
