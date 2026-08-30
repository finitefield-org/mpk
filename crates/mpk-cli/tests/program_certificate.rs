#![cfg(target_os = "linux")]

use mpk_cert::encode::{DeclarationKind, SourceManifest};
use mpk_cli::program_certificate::{
    assemble_program_certificate_alpha, ProgramCertificateErrorKind, ProgramCertificateOutcome,
    PROGRAM_CERTIFICATE_MODULE,
};
use mpk_vc::{
    group_body, GroupedTheoremDeclaration, GroupedTheoremType, VcBinder, VcCertificateSkeletonV1,
    VcDocument, VcTerm,
};

const VC_BYTES: &[u8] =
    include_bytes!("../../../fixtures/program-certificate/internal-v1/module-calls.vc.json");
const SKELETON_BYTES: &[u8] = include_bytes!(
    "../../../fixtures/program-certificate/internal-v1/module-calls.vc-skeleton.json"
);
const SOURCE_MANIFEST_BYTES: &[u8] = include_bytes!(
    "../../../fixtures/program-certificate/alpha-module-calls.source-manifest.certificate.json"
);
const BITVEC_VC_BYTES: &[u8] =
    include_bytes!("../../../fixtures/program-certificate/internal-v1/checked-addition.vc.json");
const BITVEC_SKELETON_BYTES: &[u8] = include_bytes!(
    "../../../fixtures/program-certificate/internal-v1/checked-addition.vc-skeleton.json"
);
const PROGRAM_CERTIFICATE_HEX: &[u8] =
    include_bytes!("../../../fixtures/program-certificate/alpha-module-calls.hex");

fn inputs() -> (VcDocument, VcCertificateSkeletonV1, SourceManifest) {
    (
        serde_json::from_slice(VC_BYTES).expect("fixture VC should parse"),
        serde_json::from_slice(SKELETON_BYTES).expect("fixture skeleton should parse"),
        SourceManifest {
            payload: SOURCE_MANIFEST_BYTES.to_vec(),
        },
    )
}

fn reconstruct_theorem_declarations(vc: &VcDocument) -> Vec<GroupedTheoremDeclaration> {
    vc.functions
        .iter()
        .flat_map(|function| {
            function
                .groups
                .iter()
                .map(|group| GroupedTheoremDeclaration {
                    name: group.declaration_name.clone(),
                    function_id: function.function_id.clone(),
                    group_id: group.id.clone(),
                    group_kind: group.kind,
                    member_ids: group.member_ids.clone(),
                    dependencies: group.dependencies.clone(),
                    theorem_type: GroupedTheoremType {
                        binders: function.parameters.clone(),
                        body: group_body(function, group)
                            .expect("synthetic group should reference existing members"),
                    },
                })
        })
        .collect()
}

fn equality(left: &str, right: &str) -> VcTerm {
    VcTerm::Apply {
        function: "Std.Eq".to_owned(),
        args: vec![
            VcTerm::Var {
                name: left.to_owned(),
            },
            VcTerm::Var {
                name: right.to_owned(),
            },
        ],
    }
}

fn constant(name: &str) -> VcTerm {
    VcTerm::Constant {
        name: name.to_owned(),
    }
}

fn decode_hex(input: &[u8]) -> Vec<u8> {
    let compact = input
        .iter()
        .copied()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect::<Vec<_>>();
    compact
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).expect("fixture hex is ASCII"), 16)
                .expect("fixture uses lowercase hexadecimal bytes")
        })
        .collect()
}

#[test]
fn assembles_deterministic_self_contained_dual_checked_candidate() {
    let (vc, skeleton, source_manifest) = inputs();
    let manifest: serde_json::Value =
        serde_json::from_slice(&source_manifest.payload).expect("certificate-stage manifest JSON");
    assert_eq!(manifest["vc_hash"], vc.vc_hash);
    let first = assemble_program_certificate_alpha(
        &vc,
        &skeleton.theorem_declarations,
        source_manifest.clone(),
    )
    .expect("alpha candidate should assemble");
    let second = assemble_program_certificate_alpha(
        &vc,
        &skeleton.theorem_declarations,
        source_manifest.clone(),
    )
    .expect("repeated alpha candidate should assemble");

    let (ProgramCertificateOutcome::Candidate(first), ProgramCertificateOutcome::Candidate(second)) =
        (first, second)
    else {
        panic!("the structurally provable fixture should produce a candidate");
    };

    assert_eq!(first.bytes, second.bytes);
    assert_eq!(first.bytes, decode_hex(PROGRAM_CERTIFICATE_HEX));
    assert_eq!(first.certificate.module, PROGRAM_CERTIFICATE_MODULE);
    assert!(first.certificate.imports.is_empty());
    assert!(first.certificate.proof_node_table.is_empty());
    assert!(first.certificate.theory_certificates.is_empty());
    assert_eq!(first.certificate.axiom_report.summary.total_axiom_count, 0);
    assert_eq!(
        first.certificate.source_manifest,
        Some(source_manifest),
        "the exact supplied source-manifest bytes must be embedded"
    );
    assert!(first.certificate.declarations.iter().all(|declaration| {
        !matches!(
            declaration.kind,
            DeclarationKind::Axiom { .. } | DeclarationKind::TheoryPrimitive { .. }
        )
    }));
    assert_eq!(first.generated_declarations.len(), 6);
    assert_eq!(first.rust_report.axiom_count, 0);
    assert_eq!(first.reference_report.axiom_count, 0);
    assert_eq!(first.rust_report.module, first.reference_report.module);
}

#[test]
fn one_missing_member_keeps_the_complete_valid_interface_pending() {
    let (mut vc, _, source_manifest) = inputs();
    vc.functions.truncate(1);
    let function = vc
        .functions
        .first_mut()
        .expect("fixture should contain a function");
    let parameter_type = function.parameters[0].r#type.clone();
    function.parameters.push(VcBinder {
        id: "arg1".to_owned(),
        r#type: parameter_type,
    });

    let provable_member_id = function.members[0].id.clone();
    let mut missing_member = function.members[0].clone();
    missing_member.id = "synthetic#postcondition#unprovable".to_owned();
    missing_member.conclusion = equality("arg0", "arg1");
    let missing_member_id = missing_member.id.clone();
    function.members.push(missing_member);
    function.groups[0]
        .member_ids
        .push(missing_member_id.clone());

    let declarations = reconstruct_theorem_declarations(&vc);
    let outcome = assemble_program_certificate_alpha(&vc, &declarations, source_manifest)
        .expect("the complete synthetic interface should be supported");

    let ProgramCertificateOutcome::Pending {
        generated_declarations,
        missing_member_ids,
    } = outcome
    else {
        panic!("one unproved sibling must keep the whole interface pending");
    };

    assert_eq!(generated_declarations.len(), declarations.len());
    assert_eq!(missing_member_ids, vec![missing_member_id]);
    assert!(!missing_member_ids.contains(&provable_member_id));
    assert!(generated_declarations
        .iter()
        .all(|declaration| !declaration.declaration_hash.is_empty()));
}

#[test]
fn checks_balanced_hypothesis_projection_and_member_conjunction_intro() {
    let (mut vc, _, source_manifest) = inputs();
    vc.functions.truncate(1);
    let function = vc
        .functions
        .first_mut()
        .expect("fixture should contain a function");
    let parameter_type = function.parameters[0].r#type.clone();
    function.parameters.push(VcBinder {
        id: "arg1".to_owned(),
        r#type: parameter_type,
    });
    function.requires = vec![equality("arg0", "arg0"), equality("arg1", "arg1")];
    function.members[0].conclusion = equality("arg0", "arg0");
    let mut second = function.members[0].clone();
    second.id = "synthetic#postcondition#000001".to_owned();
    second.conclusion = equality("arg1", "arg1");
    function.groups[0].member_ids.push(second.id.clone());
    function.members.push(second);

    let declarations = reconstruct_theorem_declarations(&vc);
    let outcome = assemble_program_certificate_alpha(&vc, &declarations, source_manifest)
        .expect("the balanced structural proof should assemble");

    let ProgramCertificateOutcome::Candidate(candidate) = outcome else {
        panic!("exact conjunction leaves and both sibling members should be provable");
    };
    assert_eq!(candidate.generated_declarations.len(), 2);
    assert_eq!(candidate.rust_report.axiom_count, 0);
    assert_eq!(candidate.reference_report.axiom_count, 0);
}

#[test]
fn program_bool_false_without_an_outer_bool_parameter_has_a_valid_pending_interface() {
    let (mut vc, _, source_manifest) = inputs();
    vc.functions.truncate(1);
    let function = vc
        .functions
        .first_mut()
        .expect("fixture should contain a function");
    let missing_member_id = function.members[0].id.clone();
    function.members[0].conclusion = constant("Std.Program.Base.Bool.false");

    let declarations = reconstruct_theorem_declarations(&vc);
    let outcome = assemble_program_certificate_alpha(&vc, &declarations, source_manifest)
        .expect("Program.Bool false has a complete registered interface");

    let ProgramCertificateOutcome::Pending {
        missing_member_ids, ..
    } = outcome
    else {
        panic!("false is a valid but unproved Program.Bool proposition");
    };
    assert_eq!(missing_member_ids, [missing_member_id]);
}

#[test]
fn equality_reflexivity_uses_registered_std_bool_definitional_reduction() {
    let (mut vc, _, source_manifest) = inputs();
    vc.functions.truncate(1);
    let function = vc
        .functions
        .first_mut()
        .expect("fixture should contain a function");
    function.members[0].conclusion = VcTerm::Apply {
        function: "Std.Eq".to_owned(),
        args: vec![
            VcTerm::Apply {
                function: "Std.Bool.not".to_owned(),
                args: vec![constant("Std.Bool.true")],
            },
            constant("Std.Bool.false"),
        ],
    };

    let declarations = reconstruct_theorem_declarations(&vc);
    let outcome = assemble_program_certificate_alpha(&vc, &declarations, source_manifest)
        .expect("both checkers should accept refl after Std.Bool normalization");
    assert!(matches!(outcome, ProgramCertificateOutcome::Candidate(_)));
}

#[test]
fn boolean_proposition_uses_registered_std_bool_definitional_reduction() {
    let (mut vc, _, source_manifest) = inputs();
    vc.functions.truncate(1);
    let function = vc
        .functions
        .first_mut()
        .expect("fixture should contain a function");
    function.members[0].conclusion = VcTerm::Apply {
        function: "Std.Bool.not".to_owned(),
        args: vec![constant("Std.Bool.false")],
    };

    let declarations = reconstruct_theorem_declarations(&vc);
    let outcome = assemble_program_certificate_alpha(&vc, &declarations, source_manifest)
        .expect("both checkers should accept a reduced true Boolean proposition");
    assert!(matches!(outcome, ProgramCertificateOutcome::Candidate(_)));
}

#[test]
fn substituted_skeleton_theorem_fails_before_certificate_assembly() {
    let (vc, mut skeleton, source_manifest) = inputs();
    skeleton.theorem_declarations[0].theorem_type.body = constant("Std.Bool.false");

    let error =
        assemble_program_certificate_alpha(&vc, &skeleton.theorem_declarations, source_manifest)
            .expect_err("a skeleton theorem substituted after VC validation must fail closed");

    assert_eq!(error.kind(), ProgramCertificateErrorKind::Skeleton);
    assert!(error.detail().contains("independently reconstructed"));
}

#[test]
fn unregistered_bitvec_interface_fails_closed_before_certificate_evidence() {
    let vc: VcDocument =
        serde_json::from_slice(BITVEC_VC_BYTES).expect("BitVec fixture VC should parse");
    let skeleton: VcCertificateSkeletonV1 = serde_json::from_slice(BITVEC_SKELETON_BYTES)
        .expect("BitVec fixture skeleton should parse");

    let error = assemble_program_certificate_alpha(
        &vc,
        &skeleton.theorem_declarations,
        SourceManifest {
            payload: SOURCE_MANIFEST_BYTES.to_vec(),
        },
    )
    .expect_err("unregistered BitVec lowering must fail closed");

    assert_eq!(error.kind(), ProgramCertificateErrorKind::Interface);
    assert!(error.detail().contains("bitvector-literal"));
}
