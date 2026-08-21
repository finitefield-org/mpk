//! Canonical balanced grouping for VC v1 theorem declarations.

use std::collections::BTreeMap;
use std::fmt;

use crate::vc::{VcFunction, VcGroup, VcMember, VcTerm};

pub const STD_BOOL_TRUE: &str = "Std.Bool.true";
pub const STD_BOOL_AND: &str = "Std.Bool.and";
pub const STD_LOGIC_IMP_V1: &str = "Std.Logic.Imp";

/// Builds the one canonical, order-preserving balanced conjunction.
pub fn conjoin_terms(terms: &[VcTerm]) -> VcTerm {
    match terms {
        [] => VcTerm::Constant {
            name: STD_BOOL_TRUE.to_owned(),
        },
        [term] => term.clone(),
        terms => {
            let split = terms.len() / 2;
            VcTerm::Apply {
                function: STD_BOOL_AND.to_owned(),
                args: vec![
                    conjoin_terms(&terms[..split]),
                    conjoin_terms(&terms[split..]),
                ],
            }
        }
    }
}

pub fn imply(antecedent: VcTerm, consequent: VcTerm) -> VcTerm {
    VcTerm::Apply {
        function: STD_LOGIC_IMP_V1.to_owned(),
        args: vec![antecedent, consequent],
    }
}

/// Wraps member-local binders from last to first, leaving outer function
/// parameters in the skeleton's explicit binder array.
pub fn member_theorem_type(member: &VcMember) -> VcTerm {
    member.local_binders.iter().rev().fold(
        imply(
            conjoin_terms(&member.assumptions),
            member.conclusion.clone(),
        ),
        |body, binder_type| VcTerm::Forall {
            binder_type: binder_type.clone(),
            body: Box::new(body),
        },
    )
}

pub fn group_body(function: &VcFunction, group: &VcGroup) -> Result<VcTerm, GroupingError> {
    let members = function
        .members
        .iter()
        .map(|member| (member.id.as_str(), member))
        .collect::<BTreeMap<_, _>>();
    let grouped_members = group
        .member_ids
        .iter()
        .map(|member_id| {
            members
                .get(member_id.as_str())
                .copied()
                .map(member_theorem_type)
                .ok_or_else(|| GroupingError::MissingMember(member_id.clone()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(imply(
        conjoin_terms(&function.requires),
        conjoin_terms(&grouped_members),
    ))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GroupingError {
    MissingMember(String),
}

impl fmt::Display for GroupingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingMember(member_id) => {
                write!(formatter, "group references missing member {member_id:?}")
            }
        }
    }
}

impl std::error::Error for GroupingError {}
