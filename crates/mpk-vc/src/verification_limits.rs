//! Registered deterministic verification limits for `mpk.verify.limits.v0`.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use crate::program_wp::{
    VC_ASSUMPTIONS_PER_MEMBER_MAX, VC_EXPRESSION_NODES_PER_DOCUMENT_MAX,
    VC_EXPRESSION_NODES_PER_MEMBER_MAX, VC_MEMBERS_PER_DOCUMENT_MAX, VC_MEMBERS_PER_FUNCTION_MAX,
    VC_MEMBER_EXPRESSION_DEPTH_MAX,
};
use crate::vc::{VcDocument, VcTerm, VcTypeTerm};

pub const VC_GROUPED_THEOREM_DEPTH_MAX: u64 = 512;
pub const VC_GENERATED_PROOF_DEPTH_MAX: u64 = 512;
pub const VC_CANONICAL_JSON_BYTES_MAX: u64 = 268_435_456;
pub const VC_CANONICAL_SKELETON_JSON_BYTES_MAX: u64 = 268_435_456;
pub const VC_CANONICAL_CERTIFICATE_BYTES_MAX: u64 = 536_870_912;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VerificationLimitId {
    MembersPerFunction,
    MembersPerDocument,
    AssumptionsPerMember,
    ExpressionNodesPerMember,
    ExpressionNodesPerDocument,
    MemberExpressionDepth,
    GroupedTheoremDepth,
    GeneratedProofDepth,
    CanonicalVcJsonBytes,
    CanonicalSkeletonJsonBytes,
    CanonicalCertificateBytes,
}

impl VerificationLimitId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MembersPerFunction => "members_per_function",
            Self::MembersPerDocument => "members_per_document",
            Self::AssumptionsPerMember => "assumptions_per_member",
            Self::ExpressionNodesPerMember => "expression_nodes_per_member",
            Self::ExpressionNodesPerDocument => "expression_nodes_per_document",
            Self::MemberExpressionDepth => "member_expression_depth",
            Self::GroupedTheoremDepth => "grouped_theorem_depth",
            Self::GeneratedProofDepth => "generated_proof_depth",
            Self::CanonicalVcJsonBytes => "canonical_vc_json_bytes",
            Self::CanonicalSkeletonJsonBytes => "canonical_skeleton_json_bytes",
            Self::CanonicalCertificateBytes => "canonical_certificate_bytes",
        }
    }

    pub const fn maximum(self) -> u64 {
        match self {
            Self::MembersPerFunction => VC_MEMBERS_PER_FUNCTION_MAX as u64,
            Self::MembersPerDocument => VC_MEMBERS_PER_DOCUMENT_MAX as u64,
            Self::AssumptionsPerMember => VC_ASSUMPTIONS_PER_MEMBER_MAX as u64,
            Self::ExpressionNodesPerMember => VC_EXPRESSION_NODES_PER_MEMBER_MAX as u64,
            Self::ExpressionNodesPerDocument => VC_EXPRESSION_NODES_PER_DOCUMENT_MAX as u64,
            Self::MemberExpressionDepth => VC_MEMBER_EXPRESSION_DEPTH_MAX as u64,
            Self::GroupedTheoremDepth => VC_GROUPED_THEOREM_DEPTH_MAX,
            Self::GeneratedProofDepth => VC_GENERATED_PROOF_DEPTH_MAX,
            Self::CanonicalVcJsonBytes => VC_CANONICAL_JSON_BYTES_MAX,
            Self::CanonicalSkeletonJsonBytes => VC_CANONICAL_SKELETON_JSON_BYTES_MAX,
            Self::CanonicalCertificateBytes => VC_CANONICAL_CERTIFICATE_BYTES_MAX,
        }
    }

    pub const fn code(self) -> &'static str {
        match self {
            Self::MembersPerFunction => "VC_LIMIT_MEMBERS_PER_FUNCTION",
            Self::MembersPerDocument => "VC_LIMIT_MEMBERS_PER_DOCUMENT",
            Self::AssumptionsPerMember => "VC_LIMIT_ASSUMPTIONS_PER_MEMBER",
            Self::ExpressionNodesPerMember => "VC_LIMIT_EXPRESSION_NODES_PER_MEMBER",
            Self::ExpressionNodesPerDocument => "VC_LIMIT_EXPRESSION_NODES_PER_DOCUMENT",
            Self::MemberExpressionDepth => "VC_LIMIT_MEMBER_EXPRESSION_DEPTH",
            Self::GroupedTheoremDepth => "VC_LIMIT_GROUPED_THEOREM_DEPTH",
            Self::GeneratedProofDepth => "VC_LIMIT_GENERATED_PROOF_DEPTH",
            Self::CanonicalVcJsonBytes => "VC_LIMIT_CANONICAL_JSON_BYTES",
            Self::CanonicalSkeletonJsonBytes => "VC_LIMIT_CANONICAL_SKELETON_JSON_BYTES",
            Self::CanonicalCertificateBytes => "VC_LIMIT_CANONICAL_CERTIFICATE_BYTES",
        }
    }

    /// Adds to a verification counter using the profile's checked arithmetic.
    #[doc(hidden)]
    pub fn checked_add_count(
        self,
        current: u64,
        increment: u64,
    ) -> Result<u64, VerificationLimitError> {
        checked_add(current, increment, self)
    }
}

impl TryFrom<&str> for VerificationLimitId {
    type Error = VerificationLimitError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let id = match value {
            "members_per_function" => Self::MembersPerFunction,
            "members_per_document" => Self::MembersPerDocument,
            "assumptions_per_member" => Self::AssumptionsPerMember,
            "expression_nodes_per_member" => Self::ExpressionNodesPerMember,
            "expression_nodes_per_document" => Self::ExpressionNodesPerDocument,
            "member_expression_depth" => Self::MemberExpressionDepth,
            "grouped_theorem_depth" => Self::GroupedTheoremDepth,
            "generated_proof_depth" => Self::GeneratedProofDepth,
            "canonical_vc_json_bytes" => Self::CanonicalVcJsonBytes,
            "canonical_skeleton_json_bytes" => Self::CanonicalSkeletonJsonBytes,
            "canonical_certificate_bytes" => Self::CanonicalCertificateBytes,
            _ => return Err(VerificationLimitError::UnknownLimit(value.to_owned())),
        };
        Ok(id)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerificationLimitError {
    UnknownLimit(String),
    Exceeded {
        limit: VerificationLimitId,
        count: u64,
    },
    CounterOverflow {
        limit: VerificationLimitId,
    },
    MissingMember(String),
}

impl VerificationLimitError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnknownLimit(_) | Self::MissingMember(_) => "VC_SHAPE",
            Self::Exceeded { limit, .. } | Self::CounterOverflow { limit } => limit.code(),
        }
    }
}

impl fmt::Display for VerificationLimitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownLimit(id) => write!(formatter, "unknown verification limit {id:?}"),
            Self::Exceeded { limit, count } => write!(
                formatter,
                "{} count {count} exceeds inclusive maximum {}",
                limit.as_str(),
                limit.maximum()
            ),
            Self::CounterOverflow { limit } => {
                write!(formatter, "{} counter overflow", limit.as_str())
            }
            Self::MissingMember(id) => write!(formatter, "group references missing member {id:?}"),
        }
    }
}

impl Error for VerificationLimitError {}

/// Checks one frozen counter without constructing a potentially huge fixture.
/// This is also the conformance-vector boundary for below/at/above cases.
pub fn validate_verification_limit(
    limit_id: &str,
    count: u64,
) -> Result<(), VerificationLimitError> {
    validate_limit(VerificationLimitId::try_from(limit_id)?, count)
}

pub(crate) fn validate_vc_stream_limits(
    document: &VcDocument,
) -> Result<(), VerificationLimitError> {
    let mut document_members = 0_u64;
    let mut document_nodes = 0_u64;

    for function in &document.functions {
        validate_limit(
            VerificationLimitId::MembersPerFunction,
            as_u64(
                function.members.len(),
                VerificationLimitId::MembersPerFunction,
            )?,
        )?;
        document_members = checked_add(
            document_members,
            as_u64(
                function.members.len(),
                VerificationLimitId::MembersPerDocument,
            )?,
            VerificationLimitId::MembersPerDocument,
        )?;
        validate_limit(VerificationLimitId::MembersPerDocument, document_members)?;

        for requirement in &function.requires {
            document_nodes = checked_add(
                document_nodes,
                term_nodes(requirement, VerificationLimitId::ExpressionNodesPerDocument)?,
                VerificationLimitId::ExpressionNodesPerDocument,
            )?;
            validate_limit(
                VerificationLimitId::ExpressionNodesPerDocument,
                document_nodes,
            )?;
        }

        for member in &function.members {
            validate_limit(
                VerificationLimitId::AssumptionsPerMember,
                as_u64(
                    member.assumptions.len(),
                    VerificationLimitId::AssumptionsPerMember,
                )?,
            )?;

            let mut member_nodes = 0_u64;
            for term in member
                .assumptions
                .iter()
                .chain(std::iter::once(&member.conclusion))
            {
                let nodes = term_nodes(term, VerificationLimitId::ExpressionNodesPerMember)?;
                member_nodes = checked_add(
                    member_nodes,
                    nodes,
                    VerificationLimitId::ExpressionNodesPerMember,
                )?;
                validate_limit(VerificationLimitId::ExpressionNodesPerMember, member_nodes)?;
                validate_limit(
                    VerificationLimitId::MemberExpressionDepth,
                    term_depth(term, VerificationLimitId::MemberExpressionDepth)?,
                )?;
            }
            document_nodes = checked_add(
                document_nodes,
                member_nodes,
                VerificationLimitId::ExpressionNodesPerDocument,
            )?;
            validate_limit(
                VerificationLimitId::ExpressionNodesPerDocument,
                document_nodes,
            )?;
        }
    }

    Ok(())
}

pub(crate) fn validate_grouped_theorem_limits(
    document: &VcDocument,
) -> Result<(), VerificationLimitError> {
    for function in &document.functions {
        let members = function
            .members
            .iter()
            .map(|member| (member.id.as_str(), member))
            .collect::<BTreeMap<_, _>>();
        for group in &function.groups {
            let mut member_depths = Vec::with_capacity(group.member_ids.len());
            for member_id in &group.member_ids {
                let member = members
                    .get(member_id.as_str())
                    .ok_or_else(|| VerificationLimitError::MissingMember(member_id.clone()))?;
                let assumptions = conjoin_depth(
                    &member
                        .assumptions
                        .iter()
                        .map(|term| term_depth(term, VerificationLimitId::GroupedTheoremDepth))
                        .collect::<Result<Vec<_>, _>>()?,
                    VerificationLimitId::GroupedTheoremDepth,
                )?;
                let mut depth = one_plus_max(
                    assumptions,
                    term_depth(&member.conclusion, VerificationLimitId::GroupedTheoremDepth)?,
                    VerificationLimitId::GroupedTheoremDepth,
                )?;
                for binder in member.local_binders.iter().rev() {
                    depth = one_plus_max(
                        type_depth(binder, VerificationLimitId::GroupedTheoremDepth)?,
                        depth,
                        VerificationLimitId::GroupedTheoremDepth,
                    )?;
                }
                member_depths.push(depth);
            }

            let requires = conjoin_depth(
                &function
                    .requires
                    .iter()
                    .map(|term| term_depth(term, VerificationLimitId::GroupedTheoremDepth))
                    .collect::<Result<Vec<_>, _>>()?,
                VerificationLimitId::GroupedTheoremDepth,
            )?;
            let grouped_members =
                conjoin_depth(&member_depths, VerificationLimitId::GroupedTheoremDepth)?;
            let mut depth = one_plus_max(
                requires,
                grouped_members,
                VerificationLimitId::GroupedTheoremDepth,
            )?;
            for parameter in function.parameters.iter().rev() {
                depth = one_plus_max(
                    type_depth(&parameter.r#type, VerificationLimitId::GroupedTheoremDepth)?,
                    depth,
                    VerificationLimitId::GroupedTheoremDepth,
                )?;
            }
            validate_limit(VerificationLimitId::GroupedTheoremDepth, depth)?;
        }
    }
    Ok(())
}

fn validate_limit(limit: VerificationLimitId, count: u64) -> Result<(), VerificationLimitError> {
    if count > limit.maximum() {
        Err(VerificationLimitError::Exceeded { limit, count })
    } else {
        Ok(())
    }
}

fn as_u64(value: usize, limit: VerificationLimitId) -> Result<u64, VerificationLimitError> {
    u64::try_from(value).map_err(|_| VerificationLimitError::CounterOverflow { limit })
}

fn checked_add(
    left: u64,
    right: u64,
    limit: VerificationLimitId,
) -> Result<u64, VerificationLimitError> {
    left.checked_add(right)
        .ok_or(VerificationLimitError::CounterOverflow { limit })
}

fn term_nodes(term: &VcTerm, limit: VerificationLimitId) -> Result<u64, VerificationLimitError> {
    let mut total = 1_u64;
    match term {
        VcTerm::Apply { args, .. } => {
            for child in args {
                total = checked_add(total, term_nodes(child, limit)?, limit)?;
            }
        }
        VcTerm::Convert { value, .. } => {
            total = checked_add(total, term_nodes(value, limit)?, limit)?
        }
        VcTerm::Forall { body, .. } => total = checked_add(total, term_nodes(body, limit)?, limit)?,
        VcTerm::Var { .. }
        | VcTerm::Bound { .. }
        | VcTerm::Constant { .. }
        | VcTerm::BitVecLiteral { .. } => {}
    }
    Ok(total)
}

fn term_depth(term: &VcTerm, limit: VerificationLimitId) -> Result<u64, VerificationLimitError> {
    match term {
        VcTerm::Apply { args, .. } => {
            let child = args
                .iter()
                .map(|term| term_depth(term, limit))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .max()
                .unwrap_or(0);
            add_depth(child, limit)
        }
        VcTerm::Convert { value, target } => {
            one_plus_max(term_depth(value, limit)?, type_depth(target, limit)?, limit)
        }
        VcTerm::Forall { binder_type, body } => one_plus_max(
            type_depth(binder_type, limit)?,
            term_depth(body, limit)?,
            limit,
        ),
        VcTerm::Var { .. }
        | VcTerm::Bound { .. }
        | VcTerm::Constant { .. }
        | VcTerm::BitVecLiteral { .. } => Ok(1),
    }
}

fn type_depth(
    term: &VcTypeTerm,
    limit: VerificationLimitId,
) -> Result<u64, VerificationLimitError> {
    match term {
        VcTypeTerm::Apply { args, .. } => {
            let child = args
                .iter()
                .map(|term| type_depth(term, limit))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .max()
                .unwrap_or(0);
            add_depth(child, limit)
        }
        VcTypeTerm::Constant { .. }
        | VcTypeTerm::NatLiteral { .. }
        | VcTypeTerm::StringLiteral { .. } => Ok(1),
    }
}

fn conjoin_depth(
    depths: &[u64],
    limit: VerificationLimitId,
) -> Result<u64, VerificationLimitError> {
    match depths {
        [] => Ok(1),
        [only] => Ok(*only),
        many => {
            let split = many.len() / 2;
            let left = conjoin_depth(&many[..split], limit)?;
            let right = conjoin_depth(&many[split..], limit)?;
            add_depth(left.max(right), limit)
        }
    }
}

fn one_plus_max(
    left: u64,
    right: u64,
    limit: VerificationLimitId,
) -> Result<u64, VerificationLimitError> {
    add_depth(left.max(right), limit)
}

fn add_depth(value: u64, limit: VerificationLimitId) -> Result<u64, VerificationLimitError> {
    value
        .checked_add(1)
        .ok_or(VerificationLimitError::CounterOverflow { limit })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_registered_limit_accepts_its_inclusive_maximum() {
        let limits = [
            VerificationLimitId::MembersPerFunction,
            VerificationLimitId::MembersPerDocument,
            VerificationLimitId::AssumptionsPerMember,
            VerificationLimitId::ExpressionNodesPerMember,
            VerificationLimitId::ExpressionNodesPerDocument,
            VerificationLimitId::MemberExpressionDepth,
            VerificationLimitId::GroupedTheoremDepth,
            VerificationLimitId::GeneratedProofDepth,
            VerificationLimitId::CanonicalVcJsonBytes,
            VerificationLimitId::CanonicalSkeletonJsonBytes,
            VerificationLimitId::CanonicalCertificateBytes,
        ];
        for limit in limits {
            validate_verification_limit(limit.as_str(), limit.maximum())
                .expect("inclusive maximum accepts");
            let error = validate_verification_limit(limit.as_str(), limit.maximum() + 1)
                .expect_err("maximum plus one rejects");
            assert_eq!(error.code(), limit.code());
        }
    }
}
