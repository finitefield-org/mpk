//! Untrusted refinement candidates bound to complete reconstructed path types.
//! Structural linkage is not proof acceptance; both kernels must check the bytes.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlPatternRefinementCandidateTheorem {
    pub source_sequent_id: String,
    pub native_edge_id: String,
    pub proposition_definition: String,
    pub theorem: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlPatternRefinementCandidate {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    control_vc_sha256: String,
    original_program_sha256: String,
    original_certificate_sha256: String,
    theorems: Vec<OrdinaryControlPatternRefinementCandidateTheorem>,
    /// A candidate can contain an incorrect proof even after structural checks.
    proof_check_pending: bool,
    /// Native execution, application VCs and other proof families are separate.
    application_scope_pending: bool,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}

impl OrdinaryControlPatternRefinementCandidate {
    pub fn theorems(&self) -> &[OrdinaryControlPatternRefinementCandidateTheorem] {
        &self.theorems
    }
    pub fn proof_check_pending(&self) -> bool {
        self.proof_check_pending
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("untrusted pattern refinement candidate")
    }
}

/// Every path is required, in reconstructed order, with its exact named type.
/// Names bind all source/context metadata, even if ordinary bodies happen to agree.
pub fn csharp_practical_ordinary_pattern_refinement_candidate_theorems(
    program: &OrdinaryControlPredicateProgram,
) -> R<Vec<OrdinaryControlPatternRefinementCandidateTheorem>> {
    if program.pattern_proof_types.is_empty() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let program_hash = format!("{:x}", Sha256::digest(program.canonical_bytes()));
    let mut identities = BTreeSet::new();
    program
        .pattern_proof_types
        .iter()
        .map(|entry| {
            if !entry.pending_definition_reasons.is_empty()
                || !identities.insert((&entry.source_sequent_id, &entry.native_edge_id))
            {
                return Err(OrdinaryCarrierError::Linkage);
            }
            let proposition_definition = entry
                .refinement_proposition_definition
                .clone()
                .ok_or(OrdinaryCarrierError::Linkage)?;
            Ok(OrdinaryControlPatternRefinementCandidateTheorem {
                source_sequent_id: entry.source_sequent_id.clone(),
                native_edge_id: entry.native_edge_id.clone(),
                theorem: name(
                    "PatternSourceRefinementCandidate",
                    &(
                        &program_hash,
                        &entry.source_sequent_id,
                        &entry.native_edge_id,
                        &proposition_definition,
                    ),
                ),
                proposition_definition,
            })
        })
        .collect()
}

fn preserves_prefix(base: &Certificate, candidate: &Certificate) -> R<()> {
    if base.module != candidate.module
        || base.source_manifest != candidate.source_manifest
        || base.imports != candidate.imports
        || candidate.level_table.len() < base.level_table.len()
        || !candidate.term_table.starts_with(&base.term_table)
        || candidate.declarations.len() < base.declarations.len()
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    for (old, new) in base.level_table.iter().zip(&candidate.level_table) {
        let same = match (old, new) {
            (LevelNode::Param(a), LevelNode::Param(b)) => {
                base.name_table[*a as usize] == candidate.name_table[*b as usize]
            }
            _ => old == new,
        };
        if !same {
            return Err(OrdinaryCarrierError::Linkage);
        }
    }
    // Appended names are canonically sorted, so declaration name-table indices
    // may move. Global IDs and every original type/body term remain unchanged.
    for (old, new) in base.declarations.iter().zip(&candidate.declarations) {
        if base.name_table[old.name as usize] != candidate.name_table[new.name as usize]
            || old.kind != new.kind
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
    }
    Ok(())
}

/// Link a complete ordinary candidate to the independently generated program.
/// This performs no proof search or proof checking and never reports acceptance.
pub fn link_csharp_practical_ordinary_pattern_refinement_candidate(
    program: &OrdinaryControlPredicateProgram,
    certificate: &[u8],
) -> R<OrdinaryControlPatternRefinementCandidate> {
    if certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let theorems = csharp_practical_ordinary_pattern_refinement_candidate_theorems(program)?;
    let base = decode_canonical_certificate(program.certificate_bytes())
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    let candidate =
        decode_canonical_certificate(certificate).map_err(|_| OrdinaryCarrierError::Linkage)?;
    if candidate.declarations.len() < base.declarations.len() + theorems.len() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    crate::csharp_practical_vc_model::validate_csharp_practical_certificate_structure(&candidate)
        .map_err(|_| OrdinaryCarrierError::Limit)?;
    preserves_prefix(&base, &candidate)?;
    let required = theorems
        .iter()
        .map(|t| (t.theorem.as_str(), t))
        .collect::<BTreeMap<_, _>>();
    let mut found = BTreeSet::new();
    let mut declaration_names = base
        .declarations
        .iter()
        .map(|d| base.name_table[d.name as usize].as_str())
        .collect::<BTreeSet<_>>();
    let mut expected_names = base.name_table.clone();
    for declaration in &candidate.declarations[base.declarations.len()..] {
        let declared_name = candidate.name_table[declaration.name as usize].as_str();
        if !declaration_names.insert(declared_name)
            || !matches!(
                declaration.kind,
                DeclarationKind::Def { .. } | DeclarationKind::Theorem { .. }
            )
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        expected_names.push(declared_name.into());
        let Some(expected) = required.get(declared_name) else {
            // Ordinary auxiliary definitions and checked lemma candidates are
            // permitted. Their bodies still require verification by both kernels.
            continue;
        };
        found.insert(declared_name);
        let DeclarationKind::Theorem { ty, .. } = declaration.kind else {
            return Err(OrdinaryCarrierError::Linkage);
        };
        let Some(TermNode::Const { global, levels }) = candidate.term_table.get(ty as usize) else {
            return Err(OrdinaryCarrierError::Linkage);
        };
        let source = base
            .declarations
            .get(*global as usize)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        if !levels.is_empty()
            || base.name_table[source.name as usize] != expected.proposition_definition
            || !matches!(source.kind, DeclarationKind::Def { .. })
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
    }
    if found.len() != required.len() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    for level in &candidate.level_table {
        if let LevelNode::Param(name) = level {
            expected_names.push(candidate.name_table[*name as usize].clone());
        }
    }
    expected_names.sort();
    expected_names.dedup();
    if candidate.name_table != expected_names {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(OrdinaryControlPatternRefinementCandidate {
        schema: "mpk.csharp.ordinary_pattern_refinement_candidate.v1".into(),
        source_ir_sha256: program.source_ir_sha256.clone(),
        foundation_sha256: program.foundation_sha256.clone(),
        control_vc_sha256: program.control_vc_sha256.clone(),
        original_program_sha256: format!("{:x}", Sha256::digest(program.canonical_bytes())),
        original_certificate_sha256: program.certificate_sha256.clone(),
        theorems,
        proof_check_pending: true,
        application_scope_pending: true,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(certificate)),
        certificate: certificate.to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::super::pattern_proofs::{conjunctions, projection};
    use super::*;

    fn sample() -> (Builder, OrdinaryControlPredicateProgram) {
        let mut b = Builder::new().unwrap();
        super::super::super::super::ownership_proofs::equality(&mut b).unwrap();
        super::super::super::super::ownership_proofs::logic(&mut b).unwrap();
        let mut truths = |offset: u32| {
            (0..3u32)
                .map(|i| {
                    let value = b.var(2 - i + offset).unwrap();
                    let yes = bit(&mut b, true).unwrap();
                    let boolean = b.boolean;
                    call(&mut b, "Std.Eq", vec![boolean, value, yes]).unwrap()
                })
                .collect::<Vec<_>>()
        };
        let terms = truths(0);
        let shifted = truths(1);
        let minor = truths(2);
        let scope = conjunctions(&mut b, &terms).unwrap()[0];
        let mut scope_ty = b.sort;
        let mut scope_body = scope;
        for _ in 0..3 {
            scope_ty = b.pi(b.boolean, scope_ty).unwrap();
            scope_body = b.lam(b.boolean, scope_body).unwrap();
        }
        b.define("Mpk.Candidate.Scope", scope_ty, scope_body)
            .unwrap();
        let arguments = (0..3)
            .map(|i| ControlBinding {
                kind: "ssa".into(),
                edge_id: None,
                node_id: "sample".into(),
                value_id: format!("value:{i}"),
                type_id: SOURCE_BOOL.into(),
            })
            .collect::<Vec<_>>();
        let mut components = vec![];
        for i in 0..3 {
            let predicate = format!("Mpk.Candidate.Goal.P{i}");
            let body = b.var(2 - i as u32).unwrap();
            define(&mut b, &predicate, &[0, 0, 0], 0, body).unwrap();
            components.push(OrdinaryControlStepComponent {
                role: format!("source:{i}"),
                definition: predicate,
                argument_indices: vec![0, 1, 2],
            });
        }
        let mut rows = vec![];
        for i in 0..2 {
            let mut ty = b.pi(scope, shifted[i]).unwrap();
            let proof = projection(&mut b, &shifted, &minor, i).unwrap();
            let mut proof = b.lam(scope, proof).unwrap();
            for _ in 0..3 {
                ty = b.pi(b.boolean, ty).unwrap();
                proof = b.lam(b.boolean, proof).unwrap();
            }
            let proposition = format!("Mpk.Candidate.Refinement.P{i}");
            b.define(&proposition, b.sort, ty).unwrap();
            super::super::super::super::ownership_proofs::publish_theorem(
                &mut b,
                &format!("Mpk.Candidate.Projection.P{i}"),
                ty,
                proof,
            )
            .unwrap();
            rows.push(OrdinaryControlPatternProofType {
                source_sequent_id: format!("sample.source:{i}"),
                function_id: "sample".into(),
                native_edge_id: format!("sample.edge:{i}"),
                arguments: arguments.clone(),
                components: components.clone(),
                goal_argument_indices: vec![0, 1, 2],
                goal_definition: components[i].definition.clone(),
                packed_environment: None,
                scope_proposition_definition: Some("Mpk.Candidate.Scope".into()),
                refinement_proposition_definition: Some(proposition),
                premise_proofs: vec![],
                pending_definition_reasons: vec![],
                source_refinement_proof_pending: true,
                execution_establishment_proof_pending: true,
            });
        }
        let certificate = b.clone().finish().unwrap();
        let program = OrdinaryControlPredicateProgram {
            schema: "mpk.csharp.ordinary_control_predicates.v1".into(),
            source_ir_sha256: "0".repeat(64),
            foundation_sha256: "0".repeat(64),
            control_vc_sha256: "0".repeat(64),
            native_source_program_sha256: None,
            native_source_certificate_sha256: None,
            pattern_scopes: vec![],
            pattern_captures: vec![],
            pattern_capture_scopes: vec![],
            pattern_sources: vec![],
            pattern_proof_types: rows,
            measures: vec![],
            sequents: vec![],
            unresolved_regions: vec![],
            application_scope_pending: true,
            certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
            certificate,
        };
        (b, program)
    }

    fn candidate(
        mut b: Builder,
        program: &OrdinaryControlPredicateProgram,
        wrong: bool,
    ) -> Builder {
        let expected =
            csharp_practical_ordinary_pattern_refinement_candidate_theorems(program).unwrap();
        for (i, theorem) in expected.iter().enumerate() {
            let ty = b.constant(&theorem.proposition_definition).unwrap();
            let selected = if wrong { 1 - i } else { i };
            let proof = b
                .constant(&format!("Mpk.Candidate.Projection.P{selected}"))
                .unwrap();
            super::super::super::super::ownership_proofs::publish_theorem(
                &mut b,
                &theorem.theorem,
                ty,
                proof,
            )
            .unwrap();
        }
        b
    }

    #[test]
    fn pattern_refinement_candidate_linkage_preserves_context_and_original_types() {
        let (base, program) = sample();
        assert_eq!(
            link_csharp_practical_ordinary_pattern_refinement_candidate(
                &program,
                program.certificate_bytes()
            ),
            Err(OrdinaryCarrierError::Linkage)
        );
        let complete = candidate(base, &program, false);
        let bytes = complete.clone().finish().unwrap();
        let linked =
            link_csharp_practical_ordinary_pattern_refinement_candidate(&program, &bytes).unwrap();
        assert_eq!(linked.theorems().len(), 2);
        assert!(linked.proof_check_pending());
        assert_eq!(
            mpk_kernel::verify_certificate_bytes(&bytes)
                .unwrap()
                .axiom_count,
            0
        );

        let mut other_context = program.clone();
        other_context.source_ir_sha256 = "1".repeat(64);
        assert!(link_csharp_practical_ordinary_pattern_refinement_candidate(
            &other_context,
            &bytes
        )
        .is_err());

        let mut missing = complete.clone();
        missing.c.declarations.pop();
        let mut wrong_type = complete.clone();
        let first_ty = match wrong_type.c.declarations[wrong_type.c.declarations.len() - 2].kind {
            DeclarationKind::Theorem { ty, .. } => ty,
            _ => unreachable!(),
        };
        let DeclarationKind::Theorem { ty, .. } =
            &mut wrong_type.c.declarations.last_mut().unwrap().kind
        else {
            unreachable!()
        };
        *ty = first_ty;

        let mut changed_source = complete.clone();
        let first = changed_source.globals["Mpk.Candidate.Goal.P0"] as usize;
        let second = changed_source.globals["Mpk.Candidate.Goal.P1"] as usize;
        let DeclarationKind::Def {
            value: replacement, ..
        } = changed_source.c.declarations[second].kind
        else {
            unreachable!()
        };
        let DeclarationKind::Def { value, .. } = &mut changed_source.c.declarations[first].kind
        else {
            unreachable!()
        };
        *value = replacement;
        for modified in [missing, wrong_type, changed_source] {
            let bytes = modified.finish().unwrap();
            assert_eq!(
                link_csharp_practical_ordinary_pattern_refinement_candidate(&program, &bytes),
                Err(OrdinaryCarrierError::Linkage)
            );
        }

        // Real refinements may share ordinary auxiliary lemmas. The required
        // theorem remains bound to its original type and needs kernel checking.
        let (mut auxiliary, _) = sample();
        let helper_type = auxiliary.constant("Mpk.Candidate.Refinement.P0").unwrap();
        let helper_proof = auxiliary.constant("Mpk.Candidate.Projection.P0").unwrap();
        super::super::super::super::ownership_proofs::publish_theorem(
            &mut auxiliary,
            "Mpk.Candidate.Auxiliary",
            helper_type,
            helper_proof,
        )
        .unwrap();
        let mut with_helpers = candidate(auxiliary, &program, false);
        let helper = with_helpers.constant("Mpk.Candidate.Auxiliary").unwrap();
        let index = with_helpers.c.declarations.len() - 2;
        let DeclarationKind::Theorem { proof, .. } = &mut with_helpers.c.declarations[index].kind
        else {
            unreachable!()
        };
        *proof = helper;
        let boolean = with_helpers.boolean;
        let yes = bit(&mut with_helpers, true).unwrap();
        with_helpers
            .define("Mpk.Candidate.AuxiliaryValue", boolean, yes)
            .unwrap();
        let bytes = with_helpers.finish().unwrap();
        assert!(
            link_csharp_practical_ordinary_pattern_refinement_candidate(&program, &bytes)
                .unwrap()
                .proof_check_pending()
        );
        assert_eq!(
            mpk_kernel::verify_certificate_bytes(&bytes)
                .unwrap()
                .axiom_count,
            0
        );
    }

    #[test]
    fn pattern_refinement_candidate_linkage_does_not_accept_wrong_proofs() {
        let (base, program) = sample();
        let good = candidate(base.clone(), &program, false).finish().unwrap();
        let wrong = candidate(base, &program, true).finish().unwrap();
        let linked =
            link_csharp_practical_ordinary_pattern_refinement_candidate(&program, &wrong).unwrap();
        assert!(linked.proof_check_pending());
        assert_eq!(
            mpk_kernel::verify_certificate_bytes(&wrong)
                .unwrap_err()
                .kind(),
            mpk_kernel::VerificationErrorKind::CoreCheck
        );
        if let Ok(path) = std::env::var("MPK_W09_PATTERN_CANDIDATE_UNIT_OUTPUT") {
            std::fs::create_dir_all(&path).unwrap();
            std::fs::write(std::path::Path::new(&path).join("candidate.mpcert"), good).unwrap();
            std::fs::write(
                std::path::Path::new(&path).join("wrong-candidate.mpcert"),
                wrong,
            )
            .unwrap();
            std::fs::write(
                std::path::Path::new(&path).join("wrong-candidate.json"),
                linked.canonical_bytes(),
            )
            .unwrap();
        }
    }
}
