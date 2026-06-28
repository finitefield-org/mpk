//! Structural proof-node profile helpers.

use mpk_cert::encode::ProofNode;

pub(crate) fn is_core_bootstrap_node(node: &ProofNode) -> bool {
    matches!(
        node,
        ProofNode::Exact { .. }
            | ProofNode::Apply { .. }
            | ProofNode::Intro { .. }
            | ProofNode::Refl { .. }
            | ProofNode::Conv { .. }
    )
}

pub(crate) fn is_mvp_structural_node(node: &ProofNode) -> bool {
    is_core_bootstrap_node(node)
        || matches!(
            node,
            ProofNode::LetProof { .. }
                | ProofNode::Rewrite { .. }
                | ProofNode::EqRec { .. }
                | ProofNode::Constructor { .. }
                | ProofNode::Recursor { .. }
        )
}

pub(crate) fn is_mvp_strict_node(node: &ProofNode) -> bool {
    is_mvp_structural_node(node) || matches!(node, ProofNode::Theory { .. })
}
