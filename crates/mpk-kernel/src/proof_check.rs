//! Profile-gated proof-node checking for canonical certificates.

use crate::decl_driver::{
    check_declarations_with_context, CheckedDeclarationContext, DeclarationCheckError,
    DeclarationCheckErrorKind,
};
use crate::proof_structural::{is_core_bootstrap_node, is_mvp_structural_node};

use mpk_cert::encode::{Certificate, ProofNode};
use mpk_core::{
    CoreError, CoreErrorCode, CoreLocation, DeclarationKind as CoreDeclarationKind, GlobalId,
    LocalContext, TermId, TermNode as CoreTermNode,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum ProofCheckProfile {
    CoreBootstrap,
    MvpStructural,
}

impl ProofCheckProfile {
    pub fn canonical_name(self) -> &'static str {
        match self {
            Self::CoreBootstrap => "core-bootstrap",
            Self::MvpStructural => "mvp-structural",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofCheckReport {
    pub proof_node_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofCheckError {
    kind: ProofCheckErrorKind,
    detail: String,
}

impl ProofCheckError {
    pub fn kind(&self) -> ProofCheckErrorKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    fn new(kind: ProofCheckErrorKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    fn from_declaration(proof_node: Option<u32>, error: DeclarationCheckError) -> Self {
        let kind = match error.kind() {
            DeclarationCheckErrorKind::UnsupportedDeclarationKind => {
                ProofCheckErrorKind::UnsupportedDeclarationKind
            }
            DeclarationCheckErrorKind::MissingName => ProofCheckErrorKind::MissingName,
            DeclarationCheckErrorKind::MissingGlobal => ProofCheckErrorKind::MissingGlobal,
            DeclarationCheckErrorKind::OutOfOrderDeclarationDependency => {
                ProofCheckErrorKind::OutOfOrderDeclarationDependency
            }
            DeclarationCheckErrorKind::CoreCheck => ProofCheckErrorKind::CoreCheck,
            DeclarationCheckErrorKind::InternalInvariant => ProofCheckErrorKind::InternalInvariant,
        };
        let detail = match proof_node {
            Some(index) => format!(
                "proof node {index} failed certificate translation: {}",
                error.detail()
            ),
            None => format!("declaration check failed: {}", error.detail()),
        };
        Self::new(kind, detail)
    }

    fn core(proof_node: u32, error: CoreError) -> Self {
        Self::new(
            ProofCheckErrorKind::CoreCheck,
            format!(
                "proof node {proof_node} failed core checking: {}",
                error.to_deterministic_json()
            ),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum ProofCheckErrorKind {
    UnsupportedDeclarationKind,
    UnsupportedProofNodeKind,
    MissingName,
    MissingGlobal,
    MissingProofNode,
    OutOfOrderDeclarationDependency,
    CoreCheck,
    InternalInvariant,
}

pub fn check_proof_nodes(certificate: &Certificate) -> Result<ProofCheckReport, ProofCheckError> {
    check_proof_nodes_with_profile(certificate, ProofCheckProfile::MvpStructural)
}

pub fn check_proof_nodes_with_profile(
    certificate: &Certificate,
    profile: ProofCheckProfile,
) -> Result<ProofCheckReport, ProofCheckError> {
    let mut declarations = check_declarations_with_context(certificate)
        .map_err(|error| ProofCheckError::from_declaration(None, error))?;
    check_proof_nodes_with_context(&mut declarations, profile)
}

pub(crate) fn check_proof_nodes_with_context(
    declarations: &mut CheckedDeclarationContext<'_>,
    profile: ProofCheckProfile,
) -> Result<ProofCheckReport, ProofCheckError> {
    ProofDriver {
        declarations,
        profile,
    }
    .check()
}

struct ProofDriver<'context, 'certificate> {
    declarations: &'context mut CheckedDeclarationContext<'certificate>,
    profile: ProofCheckProfile,
}

impl ProofDriver<'_, '_> {
    fn check(&mut self) -> Result<ProofCheckReport, ProofCheckError> {
        let referenced = self.referenced_nodes()?;
        for (index, is_referenced) in referenced.into_iter().enumerate() {
            if !is_referenced {
                let proof_node = proof_node_index(index)?;
                self.check_node(proof_node, &LocalContext::new())?;
            }
        }

        Ok(ProofCheckReport {
            proof_node_count: self.declarations.certificate().proof_node_table.len(),
        })
    }

    fn referenced_nodes(&self) -> Result<Vec<bool>, ProofCheckError> {
        let table = &self.declarations.certificate().proof_node_table;
        let mut referenced = vec![false; table.len()];

        for (index, node) in table.iter().enumerate() {
            let proof_node = proof_node_index(index)?;
            self.ensure_profile_allows(proof_node, node)?;
            for child in child_proofs(node) {
                let child_index = usize::try_from(child).expect("u32 id fits in usize");
                let Some(slot) = referenced.get_mut(child_index) else {
                    return Err(ProofCheckError::new(
                        ProofCheckErrorKind::MissingProofNode,
                        format!("proof node {proof_node} references missing child {child}"),
                    ));
                };
                *slot = true;
            }
        }

        Ok(referenced)
    }

    fn ensure_profile_allows(
        &self,
        proof_node: u32,
        node: &ProofNode,
    ) -> Result<(), ProofCheckError> {
        let allowed = match self.profile {
            ProofCheckProfile::CoreBootstrap => is_core_bootstrap_node(node),
            ProofCheckProfile::MvpStructural => is_mvp_structural_node(node),
        };
        if allowed {
            return Ok(());
        };

        Err(ProofCheckError::new(
            ProofCheckErrorKind::UnsupportedProofNodeKind,
            format!(
                "profile {} does not permit proof node {proof_node} tag {}",
                self.profile.canonical_name(),
                proof_node_name(node)
            ),
        ))
    }

    fn check_node(
        &mut self,
        proof_node: u32,
        context: &LocalContext,
    ) -> Result<TermId, ProofCheckError> {
        let node = self
            .declarations
            .certificate()
            .proof_node_table
            .get(usize::try_from(proof_node).expect("u32 id fits in usize"))
            .cloned()
            .ok_or_else(|| {
                ProofCheckError::new(
                    ProofCheckErrorKind::MissingProofNode,
                    format!("missing proof node {proof_node}"),
                )
            })?;
        self.ensure_profile_allows(proof_node, &node)?;

        match node {
            ProofNode::Exact {
                term,
                expected_type,
            } => {
                let expected_type = self.expected_type(proof_node, expected_type, context)?;
                let term = self.translate_term(proof_node, term)?;
                self.check_term(proof_node, context, term, expected_type)?;
                Ok(term)
            }
            ProofNode::Apply {
                function_proof,
                argument_proofs,
                expected_type,
            } => {
                let expected_type = self.expected_type(proof_node, expected_type, context)?;
                let function = self.check_node(function_proof, context)?;
                let arguments = argument_proofs
                    .into_iter()
                    .map(|argument| self.check_node(argument, context))
                    .collect::<Result<Vec<_>, _>>()?;
                let term = {
                    let (_, terms, _) = self.declarations.core_parts();
                    terms.app(function, arguments)
                };
                self.check_term(proof_node, context, term, expected_type)?;
                Ok(term)
            }
            ProofNode::Intro {
                domain_type,
                body_proof,
                expected_type,
            } => {
                let domain_type = self.translate_term(proof_node, domain_type)?;
                self.expect_type_is_sort(proof_node, context, domain_type)?;
                let expected_type = self.expected_type(proof_node, expected_type, context)?;

                let mut body_context = context.clone();
                body_context.push_binder(domain_type);
                let body = self.check_node(body_proof, &body_context)?;
                let term = {
                    let (_, terms, _) = self.declarations.core_parts();
                    terms.lam(domain_type, body)
                };
                self.check_term(proof_node, context, term, expected_type)?;
                Ok(term)
            }
            ProofNode::Refl {
                term,
                expected_type,
            } => {
                // Core equality is not a primitive in KERN-003, so Refl cannot
                // introduce new evidence. It is accepted only as an exact proof.
                let expected_type = self.expected_type(proof_node, expected_type, context)?;
                let term = self.translate_term(proof_node, term)?;
                self.check_term(proof_node, context, term, expected_type)?;
                Ok(term)
            }
            ProofNode::Conv {
                proof,
                expected_type,
                defeq_witness,
            } => {
                if let Some(defeq_witness) = defeq_witness {
                    let _ = self.translate_term(proof_node, defeq_witness)?;
                }
                let expected_type = self.expected_type(proof_node, expected_type, context)?;
                let term = self.check_node(proof, context)?;
                self.check_term(proof_node, context, term, expected_type)?;
                Ok(term)
            }
            ProofNode::LetProof {
                value,
                body_proof,
                expected_type,
            } => {
                let expected_type = self.expected_type(proof_node, expected_type, context)?;
                let value = self.translate_term(proof_node, value)?;
                let value_type = self.infer_term(proof_node, context, value)?;
                self.expect_type_is_sort(proof_node, context, value_type)?;

                let mut body_context = context.clone();
                body_context.push_definition(value_type, value);
                let body = self.check_node(body_proof, &body_context)?;
                let term = {
                    let (_, terms, _) = self.declarations.core_parts();
                    terms.let_term(value_type, value, body)
                };
                self.check_term(proof_node, context, term, expected_type)?;
                Ok(term)
            }
            ProofNode::Rewrite {
                eq_proof,
                target_proof,
                expected_type,
            } => {
                let expected_type = self.expected_type(proof_node, expected_type, context)?;
                let _ = self.check_node(eq_proof, context)?;
                let target = self.check_node(target_proof, context)?;
                self.check_term(proof_node, context, target, expected_type)?;
                Ok(target)
            }
            ProofNode::EqRec {
                motive,
                eq_proof,
                base_proof,
                expected_type,
            } => {
                let expected_type = self.expected_type(proof_node, expected_type, context)?;
                let motive = self.translate_term(proof_node, motive)?;
                let _ = self.infer_term(proof_node, context, motive)?;
                let _ = self.check_node(eq_proof, context)?;
                let base = self.check_node(base_proof, context)?;
                self.check_term(proof_node, context, base, expected_type)?;
                Ok(base)
            }
            ProofNode::Constructor {
                constructor,
                argument_proofs,
                expected_type,
            } => {
                let expected_type = self.expected_type(proof_node, expected_type, context)?;
                let constructor = self.generated_constructor(proof_node, constructor)?;
                let arguments = argument_proofs
                    .into_iter()
                    .map(|argument| self.check_node(argument, context))
                    .collect::<Result<Vec<_>, _>>()?;
                let term = self.apply_global(constructor, arguments);
                self.check_term(proof_node, context, term, expected_type)?;
                Ok(term)
            }
            ProofNode::Recursor {
                recursor,
                motive,
                minor_proofs,
                major_proof,
                expected_type,
            } => {
                let expected_type = self.expected_type(proof_node, expected_type, context)?;
                let recursor = self.generated_recursor(proof_node, recursor)?;
                let motive = self.translate_term(proof_node, motive)?;
                let _ = self.infer_term(proof_node, context, motive)?;
                let mut arguments = minor_proofs
                    .into_iter()
                    .map(|minor| self.check_node(minor, context))
                    .collect::<Result<Vec<_>, _>>()?;
                arguments.push(self.check_node(major_proof, context)?);
                let term = self.apply_global(recursor, arguments);
                self.check_term(proof_node, context, term, expected_type)?;
                Ok(term)
            }
            ProofNode::Theory { .. } => unreachable!("profile gate rejects unsupported nodes"),
        }
    }

    fn expected_type(
        &mut self,
        proof_node: u32,
        expected_type: u32,
        context: &LocalContext,
    ) -> Result<TermId, ProofCheckError> {
        let expected_type = self.translate_term(proof_node, expected_type)?;
        self.expect_type_is_sort(proof_node, context, expected_type)?;
        Ok(expected_type)
    }

    fn translate_term(&mut self, proof_node: u32, term: u32) -> Result<TermId, ProofCheckError> {
        self.declarations
            .translate_term(term)
            .map_err(|error| ProofCheckError::from_declaration(Some(proof_node), error))
    }

    fn expect_type_is_sort(
        &mut self,
        proof_node: u32,
        context: &LocalContext,
        term: TermId,
    ) -> Result<(), ProofCheckError> {
        let inferred = {
            let (levels, terms, env, cache) = self.declarations.cached_core_parts();
            cache
                .infer(levels, terms, context, env, term)
                .map_err(|error| ProofCheckError::core(proof_node, error))?
        };
        let is_sort = {
            let (_, terms, _) = self.declarations.core_parts();
            matches!(terms.node(inferred), CoreTermNode::Sort(_))
        };
        if is_sort {
            return Ok(());
        }

        Err(ProofCheckError::core(
            proof_node,
            CoreError::new(
                CoreErrorCode::TypeMismatch,
                CoreLocation::root()
                    .with_field("proof_check")
                    .with_field("proof_nodes")
                    .with_index(proof_node)
                    .with_field("expected_type"),
            )
            .with_detail("kind", "proof_expected_type_not_sort")
            .with_detail("term_index", term.index().to_string())
            .with_detail("inferred_term_index", inferred.index().to_string()),
        ))
    }

    fn infer_term(
        &mut self,
        proof_node: u32,
        context: &LocalContext,
        term: TermId,
    ) -> Result<TermId, ProofCheckError> {
        let (levels, terms, env, cache) = self.declarations.cached_core_parts();
        cache
            .infer(levels, terms, context, env, term)
            .map_err(|error| ProofCheckError::core(proof_node, error))
    }

    fn check_term(
        &mut self,
        proof_node: u32,
        context: &LocalContext,
        term: TermId,
        expected_type: TermId,
    ) -> Result<(), ProofCheckError> {
        let (levels, terms, env, cache) = self.declarations.cached_core_parts();
        cache
            .check(levels, terms, context, env, term, expected_type)
            .map_err(|error| ProofCheckError::core(proof_node, error))
    }

    fn generated_constructor(
        &mut self,
        proof_node: u32,
        global: u32,
    ) -> Result<GlobalId, ProofCheckError> {
        self.generated_global(proof_node, global, "constructor")
    }

    fn generated_recursor(
        &mut self,
        proof_node: u32,
        global: u32,
    ) -> Result<GlobalId, ProofCheckError> {
        self.generated_global(proof_node, global, "recursor")
    }

    fn generated_global(
        &mut self,
        proof_node: u32,
        global: u32,
        expected: &'static str,
    ) -> Result<GlobalId, ProofCheckError> {
        let global_id = self
            .declarations
            .global_by_index(global)
            .map_err(|error| ProofCheckError::from_declaration(Some(proof_node), error))?;
        let Some(declaration) = self.declarations.core_parts().2.lookup(global_id) else {
            return Err(ProofCheckError::new(
                ProofCheckErrorKind::MissingGlobal,
                format!("proof node {proof_node} references missing {expected} global {global}"),
            ));
        };

        match (expected, declaration.kind()) {
            (
                "constructor",
                CoreDeclarationKind::Constructor {
                    generated: true, ..
                },
            )
            | (
                "recursor",
                CoreDeclarationKind::Recursor {
                    generated: true, ..
                },
            ) => Ok(global_id),
            ("constructor", CoreDeclarationKind::Constructor { .. })
            | ("recursor", CoreDeclarationKind::Recursor { .. }) => Err(ProofCheckError::new(
                ProofCheckErrorKind::UnsupportedProofNodeKind,
                format!(
                    "proof node {proof_node} references non-generated {expected} global {global}"
                ),
            )),
            _ => Err(ProofCheckError::new(
                ProofCheckErrorKind::CoreCheck,
                format!(
                    "proof node {proof_node} expected {expected} global {global} but found {}",
                    core_declaration_kind_name(declaration.kind())
                ),
            )),
        }
    }

    fn apply_global(&mut self, global: GlobalId, arguments: Vec<TermId>) -> TermId {
        let (_, terms, _) = self.declarations.core_parts();
        let constant = terms.constant(global, []);
        if arguments.is_empty() {
            constant
        } else {
            terms.app(constant, arguments)
        }
    }
}

fn child_proofs(node: &ProofNode) -> Vec<u32> {
    match node {
        ProofNode::Exact { .. } | ProofNode::Refl { .. } | ProofNode::Theory { .. } => Vec::new(),
        ProofNode::Apply {
            function_proof,
            argument_proofs,
            ..
        } => {
            let mut children = Vec::with_capacity(argument_proofs.len() + 1);
            children.push(*function_proof);
            children.extend(argument_proofs.iter().copied());
            children
        }
        ProofNode::Intro { body_proof, .. }
        | ProofNode::LetProof { body_proof, .. }
        | ProofNode::Conv {
            proof: body_proof, ..
        } => vec![*body_proof],
        ProofNode::Rewrite {
            eq_proof,
            target_proof,
            ..
        } => vec![*eq_proof, *target_proof],
        ProofNode::EqRec {
            eq_proof,
            base_proof,
            ..
        } => vec![*eq_proof, *base_proof],
        ProofNode::Constructor {
            argument_proofs, ..
        } => argument_proofs.clone(),
        ProofNode::Recursor {
            minor_proofs,
            major_proof,
            ..
        } => {
            let mut children = Vec::with_capacity(minor_proofs.len() + 1);
            children.extend(minor_proofs.iter().copied());
            children.push(*major_proof);
            children
        }
    }
}

fn proof_node_index(index: usize) -> Result<u32, ProofCheckError> {
    u32::try_from(index).map_err(|_| {
        ProofCheckError::new(
            ProofCheckErrorKind::InternalInvariant,
            format!("proof node index {index} exceeds u32"),
        )
    })
}

fn proof_node_name(node: &ProofNode) -> &'static str {
    match node {
        ProofNode::Exact { .. } => "exact",
        ProofNode::Apply { .. } => "apply",
        ProofNode::Intro { .. } => "intro",
        ProofNode::LetProof { .. } => "let_proof",
        ProofNode::Refl { .. } => "refl",
        ProofNode::Rewrite { .. } => "rewrite",
        ProofNode::EqRec { .. } => "eq_rec",
        ProofNode::Constructor { .. } => "constructor",
        ProofNode::Recursor { .. } => "recursor",
        ProofNode::Conv { .. } => "conv",
        ProofNode::Theory { .. } => "theory",
    }
}

fn core_declaration_kind_name(kind: CoreDeclarationKind) -> &'static str {
    match kind {
        CoreDeclarationKind::Axiom { .. } => "axiom",
        CoreDeclarationKind::Definition { .. } => "definition",
        CoreDeclarationKind::Theorem { .. } => "theorem",
        CoreDeclarationKind::Inductive { .. } => "inductive",
        CoreDeclarationKind::Constructor { .. } => "constructor",
        CoreDeclarationKind::Recursor { .. } => "recursor",
    }
}

#[cfg(test)]
mod tests {
    use mpk_cert::encode::{
        AxiomReport, Certificate, CertificateHashes, Declaration, DeclarationKind, LevelNode,
        ProofNode, TermNode,
    };

    use super::{
        check_proof_nodes, check_proof_nodes_with_profile, ProofCheckErrorKind, ProofCheckProfile,
    };

    fn bootstrap_certificate() -> Certificate {
        Certificate {
            module: "Example.ProofBootstrap".to_owned(),
            imports: Vec::new(),
            name_table: vec!["Example.ProofBootstrap.x".to_owned()],
            level_table: vec![LevelNode::Zero, LevelNode::Succ(0)],
            term_table: vec![
                TermNode::Sort(0),
                TermNode::Sort(1),
                TermNode::Const {
                    global: 0,
                    levels: Vec::new(),
                },
                TermNode::Var(0),
                TermNode::Lam { ty: 0, body: 3 },
                TermNode::Pi { ty: 0, body: 0 },
            ],
            proof_node_table: vec![
                ProofNode::Exact {
                    term: 2,
                    expected_type: 0,
                },
                ProofNode::Exact {
                    term: 4,
                    expected_type: 5,
                },
                ProofNode::Apply {
                    function_proof: 1,
                    argument_proofs: vec![0],
                    expected_type: 0,
                },
                ProofNode::Exact {
                    term: 3,
                    expected_type: 0,
                },
                ProofNode::Intro {
                    domain_type: 0,
                    body_proof: 3,
                    expected_type: 5,
                },
                ProofNode::Refl {
                    term: 2,
                    expected_type: 0,
                },
                ProofNode::Conv {
                    proof: 0,
                    expected_type: 0,
                    defeq_witness: Some(0),
                },
            ],
            declarations: vec![Declaration {
                name: 0,
                kind: DeclarationKind::Axiom { ty: 0 },
            }],
            theory_certificates: Vec::new(),
            export_block: Vec::new(),
            axiom_report: AxiomReport::default(),
            source_manifest: None,
            hashes: CertificateHashes::default(),
        }
    }

    fn structural_certificate() -> Certificate {
        Certificate {
            module: "Example.ProofStructural".to_owned(),
            imports: Vec::new(),
            name_table: vec![
                "Example.ProofStructural.Bool".to_owned(),
                "Example.ProofStructural.Bool.false".to_owned(),
                "Example.ProofStructural.Bool.true".to_owned(),
                "Example.ProofStructural.Bool.rec".to_owned(),
            ],
            level_table: vec![LevelNode::Zero, LevelNode::Succ(0)],
            term_table: vec![
                TermNode::Sort(0),
                TermNode::Sort(1),
                TermNode::Const {
                    global: 0,
                    levels: Vec::new(),
                },
                TermNode::Const {
                    global: 1,
                    levels: Vec::new(),
                },
                TermNode::Const {
                    global: 2,
                    levels: Vec::new(),
                },
                TermNode::Pi { ty: 2, body: 2 },
                TermNode::Pi { ty: 2, body: 5 },
                TermNode::Pi { ty: 2, body: 6 },
                TermNode::Var(0),
            ],
            proof_node_table: vec![
                ProofNode::Constructor {
                    constructor: 1,
                    argument_proofs: Vec::new(),
                    expected_type: 2,
                },
                ProofNode::Constructor {
                    constructor: 2,
                    argument_proofs: Vec::new(),
                    expected_type: 2,
                },
                ProofNode::Recursor {
                    recursor: 3,
                    motive: 2,
                    minor_proofs: vec![0, 1],
                    major_proof: 0,
                    expected_type: 2,
                },
                ProofNode::LetProof {
                    value: 3,
                    body_proof: 4,
                    expected_type: 2,
                },
                ProofNode::Exact {
                    term: 8,
                    expected_type: 2,
                },
                ProofNode::Rewrite {
                    eq_proof: 1,
                    target_proof: 0,
                    expected_type: 2,
                },
                ProofNode::EqRec {
                    motive: 2,
                    eq_proof: 1,
                    base_proof: 0,
                    expected_type: 2,
                },
            ],
            declarations: vec![
                Declaration {
                    name: 0,
                    kind: DeclarationKind::Inductive { ty: 0 },
                },
                Declaration {
                    name: 1,
                    kind: DeclarationKind::Constructor {
                        ty: 2,
                        inductive: 0,
                        generated: true,
                    },
                },
                Declaration {
                    name: 2,
                    kind: DeclarationKind::Constructor {
                        ty: 2,
                        inductive: 0,
                        generated: true,
                    },
                },
                Declaration {
                    name: 3,
                    kind: DeclarationKind::Recursor {
                        ty: 7,
                        inductive: 0,
                        generated: true,
                    },
                },
            ],
            theory_certificates: Vec::new(),
            export_block: Vec::new(),
            axiom_report: AxiomReport::default(),
            source_manifest: None,
            hashes: CertificateHashes::default(),
        }
    }

    #[test]
    fn checks_core_bootstrap_nodes_with_local_intro_body() {
        let report = check_proof_nodes_with_profile(
            &bootstrap_certificate(),
            ProofCheckProfile::CoreBootstrap,
        )
        .expect("proof nodes check");

        assert_eq!(report.proof_node_count, 7);
    }

    #[test]
    fn core_bootstrap_profile_rejects_structural_node() {
        let mut certificate = bootstrap_certificate();
        certificate.proof_node_table.push(ProofNode::LetProof {
            value: 2,
            body_proof: 0,
            expected_type: 0,
        });

        let error = check_proof_nodes_with_profile(&certificate, ProofCheckProfile::CoreBootstrap)
            .unwrap_err();

        assert_eq!(error.kind(), ProofCheckErrorKind::UnsupportedProofNodeKind);
    }

    #[test]
    fn checks_structural_nodes_under_mvp_structural_profile() {
        let report = check_proof_nodes(&structural_certificate()).expect("structural nodes check");

        assert_eq!(report.proof_node_count, 7);
    }

    #[test]
    fn core_bootstrap_profile_still_rejects_structural_nodes() {
        let error = check_proof_nodes_with_profile(
            &structural_certificate(),
            ProofCheckProfile::CoreBootstrap,
        )
        .unwrap_err();

        assert_eq!(error.kind(), ProofCheckErrorKind::UnsupportedProofNodeKind);
    }

    #[test]
    fn structural_profile_rejects_theory_nodes() {
        let mut certificate = structural_certificate();
        certificate.proof_node_table.push(ProofNode::Theory {
            theory_certificate: 0,
            expected_type: 2,
        });

        let error = check_proof_nodes(&certificate).unwrap_err();

        assert_eq!(error.kind(), ProofCheckErrorKind::UnsupportedProofNodeKind);
    }

    #[test]
    fn structural_constructor_node_requires_generated_constructor() {
        let mut certificate = structural_certificate();
        certificate.declarations[1].kind = DeclarationKind::Constructor {
            ty: 2,
            inductive: 0,
            generated: false,
        };

        let error = check_proof_nodes(&certificate).unwrap_err();

        assert_eq!(error.kind(), ProofCheckErrorKind::UnsupportedProofNodeKind);
    }

    #[test]
    fn rejects_bad_exact_node() {
        let mut certificate = bootstrap_certificate();
        certificate.proof_node_table = vec![ProofNode::Exact {
            term: 2,
            expected_type: 1,
        }];

        let error = check_proof_nodes(&certificate).unwrap_err();

        assert_eq!(error.kind(), ProofCheckErrorKind::CoreCheck);
    }
}
