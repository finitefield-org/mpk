//! Language-neutral unresolved expression terms shared by VIR and VC.

use serde::{Deserialize, Serialize};

use crate::type_encode::MpkTypeTerm;

pub const STD_BOOL_TRUE: &str = "Std.Bool.true";
pub const STD_BOOL_FALSE: &str = "Std.Bool.false";
pub const STD_BOOL_NOT: &str = "Std.Bool.not";
pub const STD_BOOL_AND: &str = "Std.Bool.and";
pub const STD_BOOL_OR: &str = "Std.Bool.or";
pub const STD_BOOL_IF: &str = "Std.Bool.if";
pub const STD_EQ: &str = "Std.Eq";
pub const STD_BITVEC_MODULE: &str = "Std.BitVec";

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MpkExprTerm {
    Var {
        name: String,
    },
    Bound {
        index: u32,
    },
    Result {
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
        args: Vec<MpkExprTerm>,
    },
    Convert {
        value: Box<MpkExprTerm>,
        target: MpkTypeTerm,
    },
    Forall {
        binder_type: MpkTypeTerm,
        body: Box<MpkExprTerm>,
    },
}

impl MpkExprTerm {
    pub fn bool_literal(value: bool) -> Self {
        Self::Constant {
            name: if value { STD_BOOL_TRUE } else { STD_BOOL_FALSE }.to_owned(),
        }
    }

    pub fn apply<I>(function: impl Into<String>, args: I) -> Self
    where
        I: IntoIterator<Item = MpkExprTerm>,
    {
        Self::Apply {
            function: function.into(),
            args: args.into_iter().collect(),
        }
    }
}
