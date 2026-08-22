use crate::sha256::{digest, hex};
use crate::EXPECTED_RUSTC_COMMIT;
use std::collections::BTreeMap;

pub const MIR_PROFILE_ID: &str = "mpk.rust.mir.4d08223c.v0";
pub const MIR_QUERY: &str = "mir_drops_elaborated_and_const_checked";
pub const MIR_DIALECT_SUMMARY: &str = concat!(
    "compiler=4d08223c054cf5a56d9761ca925fd46ffebe7115\n",
    "query=mir_drops_elaborated_and_const_checked\n",
    "stage=post-borrow-check,post-drop-elaboration,const-checked,unoptimized\n",
    "statement=Assign,StorageLive,StorageDead,Nop\n",
    "rvalue=Use,BinaryOp,UnaryOp,Aggregate,Cast(IntToInt)\n",
    "operand=Copy,Move,Constant\n",
    "projection=Field,Index\n",
    "terminator=Goto,SwitchInt,Return,Call,Assert\n",
    "assert=Overflow,OverflowNeg,DivisionByZero,RemainderByZero,BoundsCheck\n",
    "unwind=Unreachable\n",
    "source=accepted-captured-source-only,no-expansion,no-cleanup\n",
);
pub const MIR_DIALECT_SHA256: &str =
    "6dd18917a34f886319af0284d9a8a1bd8e9634388c6ea56fbee2c52f05917a80";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MirAccessError {
    Profile,
    Compiler,
    Query,
    Dialect,
    UnknownBody,
    DuplicateRequest,
    QueryTheft,
    Incomplete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BodyState {
    Pending,
    Forced,
    Borrowed,
}

#[derive(Debug)]
pub struct MirAccessTracker {
    bodies: BTreeMap<String, BodyState>,
}

impl MirAccessTracker {
    pub fn new(functions: impl IntoIterator<Item = String>) -> Result<Self, MirAccessError> {
        let mut bodies = BTreeMap::new();
        for function in functions {
            if function.is_empty() || bodies.insert(function, BodyState::Pending).is_some() {
                return Err(MirAccessError::DuplicateRequest);
            }
        }
        if bodies.is_empty() {
            return Err(MirAccessError::Incomplete);
        }
        Ok(Self { bodies })
    }

    pub fn force(&mut self, function: &str, query: &str) -> Result<(), MirAccessError> {
        if query != MIR_QUERY {
            return Err(if query == "optimized_mir" {
                MirAccessError::QueryTheft
            } else {
                MirAccessError::Query
            });
        }
        let state = self
            .bodies
            .get_mut(function)
            .ok_or(MirAccessError::UnknownBody)?;
        if *state != BodyState::Pending {
            return Err(MirAccessError::DuplicateRequest);
        }
        *state = BodyState::Forced;
        Ok(())
    }

    pub fn mark_borrowed(&mut self, function: &str) -> Result<(), MirAccessError> {
        let state = self
            .bodies
            .get_mut(function)
            .ok_or(MirAccessError::UnknownBody)?;
        if *state != BodyState::Forced {
            return Err(MirAccessError::QueryTheft);
        }
        *state = BodyState::Borrowed;
        Ok(())
    }

    pub fn finish(self) -> Result<Vec<String>, MirAccessError> {
        if self
            .bodies
            .values()
            .any(|state| *state != BodyState::Borrowed)
        {
            return Err(MirAccessError::Incomplete);
        }
        Ok(self.bodies.into_keys().collect())
    }
}

pub fn compatibility_fingerprint(summary: &str) -> String {
    hex(&digest(summary.as_bytes()))
}

pub fn validate_compatibility(
    profile: &str,
    compiler_commit: &str,
    query: &str,
    summary: &str,
    fingerprint: &str,
) -> Result<(), MirAccessError> {
    if profile != MIR_PROFILE_ID {
        return Err(MirAccessError::Profile);
    }
    if compiler_commit != EXPECTED_RUSTC_COMMIT {
        return Err(MirAccessError::Compiler);
    }
    if query != MIR_QUERY {
        return Err(MirAccessError::Query);
    }
    if summary != MIR_DIALECT_SUMMARY
        || fingerprint != MIR_DIALECT_SHA256
        || compatibility_fingerprint(summary) != MIR_DIALECT_SHA256
    {
        return Err(MirAccessError::Dialect);
    }
    Ok(())
}
