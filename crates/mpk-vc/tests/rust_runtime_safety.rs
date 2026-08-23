use std::fs;
use std::path::{Path, PathBuf};

use mpk_cert::{build_axiom_report, decode_canonical_certificate};
use mpk_vc::vir::{VirContractBinaryExpr, VirContractExpr, VirIntegerLiteral, VirResultRef};
use mpk_vc::{
    canonical_vir_json, contract_hash, emit_validated_vc_skeleton_v1, encode_vir_type,
    generate_program_vcs, generate_vc_v1_from_context, import_vir_json, vir_hash, BitVectorWidth,
    DecimalInteger, MpkExprTerm, PointerWidth, ProgramVcMemberKind, SafetyEvidenceRoute, VcBinder,
    VcGroupKind, VcMember, VcSourceContext, VcSourceFunction, VcTerm, VcTypeTerm,
    VirBinaryOperator, VirImportError, VirInstruction, VirIntLiteral, VirModule, VirSafetyCheck,
    SAFETY_GROUPED_CERTIFICATE_FOUNDATION, VERIFICATION_LIMIT_PROFILE, VIR_SCHEMA_VERSION,
};
use serde_json::Value;

const ARITHMETIC_VIR: &[u8] =
    include_bytes!("../../../rust-tools/rust2vir/testdata/arithmetic/expected-vir.json");
const DIV_REM_VIR: &[u8] =
    include_bytes!("../../../rust-tools/rust2vir/testdata/div-rem/expected-vir.json");
const SHIFT_VIR: &[u8] =
    include_bytes!("../../../rust-tools/rust2vir/testdata/bitwise-shift/expected-vir.json");
const INDEX_VIR: &[u8] =
    include_bytes!("../../../rust-tools/rust2vir/testdata/array-index/expected-vir.json");
const GUARDED_VIR: &[u8] =
    include_bytes!("../../../rust-tools/rust2vir/testdata/runtime-safety/expected-vir.json");

#[test]
fn all_lowered_runtime_safety_instructions_reach_vc_v1_and_grouped_skeletons() {
    let fixtures = [
        ("arithmetic", ARITHMETIC_VIR),
        ("div-rem", DIV_REM_VIR),
        ("shift", SHIFT_VIR),
        ("index", INDEX_VIR),
        ("guarded", GUARDED_VIR),
    ];
    let mut saw_array_index_safety = false;

    for (fixture_id, bytes) in fixtures {
        for module in import_modules(bytes) {
            let expected_safety = instruction_safety_count(&module);
            let first_program = generate_program_vcs(&module)
                .unwrap_or_else(|error| panic!("{fixture_id} program VCs failed: {error}"));
            let second_program = generate_program_vcs(&module).expect("repeat program VCs");
            assert_eq!(first_program, second_program, "{fixture_id} program VCs");

            for member in first_program
                .functions
                .iter()
                .flat_map(|function| &function.members)
                .filter(|member| member.kind == ProgramVcMemberKind::OperationSafety)
            {
                assert_eq!(
                    member.safety_evidence,
                    Some(SafetyEvidenceRoute::GroupedCertificate {
                        foundation: SAFETY_GROUPED_CERTIFICATE_FOUNDATION,
                    }),
                    "{fixture_id} checked safety evidence route"
                );
                if fixture_id == "guarded" {
                    assert_eq!(
                        member.assumptions,
                        vec![MpkExprTerm::Var {
                            name: "arg0".to_owned(),
                        }],
                        "guarded division path condition"
                    );
                }
            }

            let actual_safety = first_program
                .functions
                .iter()
                .flat_map(|function| &function.members)
                .filter(|member| member.kind == ProgramVcMemberKind::OperationSafety)
                .count();
            assert_eq!(
                actual_safety, expected_safety,
                "{fixture_id} safety coverage"
            );

            let first_vc = generate_fixture_vc(&module);
            let second_vc = generate_fixture_vc(&module);
            assert_eq!(first_vc.canonical_bytes(), second_vc.canonical_bytes());
            let first_skeleton =
                emit_validated_vc_skeleton_v1(&first_vc).expect("runtime-safety skeleton emits");
            let second_skeleton =
                emit_validated_vc_skeleton_v1(&second_vc).expect("repeat skeleton emits");
            assert_eq!(
                first_skeleton.canonical_bytes(),
                second_skeleton.canonical_bytes(),
                "{fixture_id} grouped skeleton"
            );

            for function in &first_vc.document().functions {
                let safety = function
                    .members
                    .iter()
                    .filter(|member| member.kind == mpk_vc::VcMemberKind::OperationSafety)
                    .collect::<Vec<_>>();
                for (ordinal, member) in safety.iter().enumerate() {
                    assert_eq!(
                        member.id,
                        format!("{}#operation_safety#{ordinal:06}", function.function_id)
                    );
                    assert_eq!(
                        member.group_id,
                        format!("{}.panic_free", function.function_id)
                    );
                    if fixture_id == "guarded" {
                        assert_eq!(member.assumptions.len(), 1);
                    } else {
                        assert!(member.assumptions.is_empty());
                    }
                }
                if fixture_id == "guarded" && safety.len() == 2 {
                    assert_eq!(safety[0].assumptions, safety[1].assumptions);
                }

                let panic_free = function
                    .groups
                    .iter()
                    .find(|group| group.kind == VcGroupKind::PanicFree)
                    .expect("panic-free group");
                let safety_ids = safety
                    .iter()
                    .map(|member| member.id.as_str())
                    .collect::<Vec<_>>();
                assert_eq!(
                    panic_free
                        .member_ids
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>(),
                    safety_ids,
                    "{} exact panic-free partition",
                    function.function_id
                );
            }

            if module
                .units
                .iter()
                .flat_map(|unit| &unit.functions)
                .flat_map(|function| &function.blocks)
                .flat_map(|block| &block.instructions)
                .any(|instruction| matches!(instruction, VirInstruction::Index { .. }))
            {
                saw_array_index_safety = true;
                let text = std::str::from_utf8(first_vc.canonical_bytes()).unwrap();
                assert!(!text.contains("Std.Program.Base.Array.Index"));
            }
        }
    }
    assert!(saw_array_index_safety);
}

#[test]
fn removing_or_changing_each_frontend_check_fails_independent_validation() {
    for (fixture_id, bytes) in [
        ("arithmetic", ARITHMETIC_VIR),
        ("div-rem", DIV_REM_VIR),
        ("shift", SHIFT_VIR),
        ("index", INDEX_VIR),
        ("guarded", GUARDED_VIR),
    ] {
        for module in import_modules(bytes) {
            let locations = safety_locations(&module);
            for (unit, function, block, instruction, check) in locations {
                let mut removed = module.clone();
                instruction_safety_mut(
                    &mut removed.units[unit].functions[function].blocks[block].instructions
                        [instruction],
                )
                .remove(check);
                assert_safety_mutation_rejects(removed, fixture_id);

                let mut changed = module.clone();
                let slot = &mut instruction_safety_mut(
                    &mut changed.units[unit].functions[function].blocks[block].instructions
                        [instruction],
                )[check];
                *slot = if matches!(slot, VirSafetyCheck::IndexInBounds {}) {
                    VirSafetyCheck::DivisorNonzero {}
                } else {
                    VirSafetyCheck::IndexInBounds {}
                };
                assert_safety_mutation_rejects(changed, fixture_id);
            }
        }
    }
}

#[test]
fn array_index_value_use_remains_fail_closed_without_array_read_semantics() {
    let mut module = import_modules(INDEX_VIR)
        .into_iter()
        .find(|module| module.semantic_parameters.pointer_width() == PointerWidth::Bits64)
        .expect("64-bit array-index fixture");
    {
        let function = &mut module.units[0].functions[0];
        function.contracts.ensures = vec![VirContractExpr::Binary(VirContractBinaryExpr {
            op: VirBinaryOperator::Eq,
            lhs: Box::new(VirContractExpr::Result(VirResultRef { result: 0 })),
            rhs: Box::new(VirContractExpr::Integer(VirIntegerLiteral {
                int: VirIntLiteral {
                    value: DecimalInteger::new("0".to_owned()).unwrap(),
                    width: BitVectorWidth::Bits8,
                    signed: false,
                },
            })),
        })];
        function.contracts.contract_hash =
            contract_hash(&function.contracts).expect("changed contract hashes");
    }
    module.vir_hash = vir_hash(&module).expect("changed VIR hashes");

    let error = generate_program_vcs(&module)
        .expect_err("semantic array reads must stay unavailable before foundation integration");
    assert_eq!(error.code(), "VC_PROGRAM_UNCLOSED_TERM");
}

#[test]
fn public_and_frontend_runtime_safety_fixtures_are_identical() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for relative in [
        "checked.rs",
        "expected.json",
        "sufficient/arithmetic.json",
        "sufficient/div-rem.json",
        "sufficient/shift.json",
        "sufficient/index.json",
        "insufficient/arithmetic.json",
        "insufficient/div-rem.json",
        "insufficient/shift.json",
        "insufficient/index.json",
        "insufficient/guarded-div.json",
    ] {
        assert_eq!(
            fs::read(
                root.join("fixtures/rust-basic/runtime-safety")
                    .join(relative)
            )
            .unwrap(),
            fs::read(
                root.join("rust-tools/rust2vir/testdata/runtime-safety")
                    .join(relative)
            )
            .unwrap(),
            "runtime-safety fixture drift: {relative}"
        );
    }
}

#[test]
fn runtime_safety_foundations_have_no_rust_semantic_axiom() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for relative in [
        "proofs/program/base/std-program-base.hex",
        "proofs/std/bitvec/ground-eval-fixture.hex",
    ] {
        let certificate = decode_canonical_certificate(&decode_hex(&root.join(relative)))
            .unwrap_or_else(|error| panic!("{relative} must be canonical: {error:?}"));
        let report = build_axiom_report(&certificate).expect("axiom report rebuilds");
        assert_eq!(report, certificate.axiom_report);
        assert_eq!(report.summary.total_axiom_count, 0, "{relative}");
        assert!(certificate
            .name_table
            .iter()
            .all(|name| !name.contains("RustSemanticsAxiom")));
    }
}

fn import_modules(bytes: &[u8]) -> Vec<VirModule> {
    let value: Value = serde_json::from_slice(bytes).expect("runtime-safety VIR fixture JSON");
    match value {
        Value::Array(modules) => modules
            .into_iter()
            .map(|module| {
                let bytes = serde_json::to_vec(&module).expect("fixture module serializes");
                import_vir_json(&bytes).expect("fixture passes independent VIR validation")
            })
            .collect(),
        module => vec![import_vir_json(&serde_json::to_vec(&module).unwrap())
            .expect("fixture passes independent VIR validation")],
    }
}

fn generate_fixture_vc(module: &VirModule) -> mpk_vc::ValidatedVcDocument {
    let generated = generate_program_vcs(module).expect("program VCs generate");
    let mut functions = Vec::with_capacity(generated.functions.len());
    for generated_function in &generated.functions {
        let (unit, function) = module
            .units
            .iter()
            .find_map(|unit| {
                unit.functions
                    .iter()
                    .find(|function| function.id == generated_function.function_id)
                    .map(|function| (unit, function))
            })
            .expect("generated function belongs to VIR");
        let parameters = function
            .params
            .iter()
            .map(|parameter| VcBinder {
                id: parameter.id.clone(),
                r#type: VcTypeTerm::from(
                    &encode_vir_type(
                        module.semantic_profile,
                        &module.semantic_parameters,
                        &unit.type_decls,
                        &parameter.r#type,
                    )
                    .expect("parameter type encodes"),
                ),
            })
            .collect();
        let requires = generated_function
            .requires
            .iter()
            .map(VcTerm::try_from)
            .collect::<Result<Vec<_>, _>>()
            .expect("requires close over parameters");
        let regenerated_members = generated_function
            .members
            .iter()
            .map(VcMember::try_from)
            .collect::<Result<Vec<_>, _>>()
            .expect("members close over parameters");
        functions.push(VcSourceFunction {
            function_id: function.id.clone(),
            contract_hash: contract_hash(&function.contracts)
                .expect("contract hashes")
                .as_str()
                .to_owned(),
            direct_callees: generated_function.direct_callees.clone(),
            parameters,
            requires,
            regenerated_members,
        });
    }
    generate_vc_v1_from_context(&VcSourceContext {
        id: "rust.runtime_safety.fixture".to_owned(),
        source_ir_schema: VIR_SCHEMA_VERSION.to_owned(),
        source_ir_hash: module.vir_hash.as_str().to_owned(),
        input_set_hash: "0".repeat(64),
        semantic_profile: module.semantic_profile,
        semantic_parameters: module.semantic_parameters.clone(),
        verification_limit_profile: VERIFICATION_LIMIT_PROFILE.to_owned(),
        functions,
    })
    .expect("runtime-safety VC v1 generates")
}

fn instruction_safety_count(module: &VirModule) -> usize {
    module
        .units
        .iter()
        .flat_map(|unit| &unit.functions)
        .flat_map(|function| &function.blocks)
        .flat_map(|block| &block.instructions)
        .map(|instruction| match instruction {
            VirInstruction::Const { safety_checks, .. }
            | VirInstruction::Copy { safety_checks, .. }
            | VirInstruction::BinOp { safety_checks, .. }
            | VirInstruction::UnaryOp { safety_checks, .. }
            | VirInstruction::Convert { safety_checks, .. }
            | VirInstruction::Field { safety_checks, .. }
            | VirInstruction::Index { safety_checks, .. }
            | VirInstruction::MakeStruct { safety_checks, .. }
            | VirInstruction::MakeArray { safety_checks, .. }
            | VirInstruction::CallStatic { safety_checks, .. } => safety_checks.len(),
        })
        .sum()
}

fn safety_locations(module: &VirModule) -> Vec<(usize, usize, usize, usize, usize)> {
    let mut locations = Vec::new();
    for (unit_index, unit) in module.units.iter().enumerate() {
        for (function_index, function) in unit.functions.iter().enumerate() {
            for (block_index, block) in function.blocks.iter().enumerate() {
                for (instruction_index, instruction) in block.instructions.iter().enumerate() {
                    for check_index in 0..instruction_safety(instruction).len() {
                        locations.push((
                            unit_index,
                            function_index,
                            block_index,
                            instruction_index,
                            check_index,
                        ));
                    }
                }
            }
        }
    }
    locations
}

fn assert_safety_mutation_rejects(mut module: VirModule, fixture_id: &str) {
    module.vir_hash = vir_hash(&module).expect("mutated VIR hashes");
    let bytes = canonical_vir_json(&module).expect("mutated VIR canonicalizes");
    let error = import_vir_json(&bytes).expect_err("mutated safety metadata must reject");
    let VirImportError::Validation(error) = error else {
        panic!("{fixture_id} mutation rejected outside independent validation: {error}")
    };
    assert!(
        error.code().starts_with("VIR_SAFETY_CHECK_"),
        "{fixture_id} unexpected mutation code: {}",
        error.code()
    );
}

fn instruction_safety(instruction: &VirInstruction) -> &[VirSafetyCheck] {
    match instruction {
        VirInstruction::Const { safety_checks, .. }
        | VirInstruction::Copy { safety_checks, .. }
        | VirInstruction::BinOp { safety_checks, .. }
        | VirInstruction::UnaryOp { safety_checks, .. }
        | VirInstruction::Convert { safety_checks, .. }
        | VirInstruction::Field { safety_checks, .. }
        | VirInstruction::Index { safety_checks, .. }
        | VirInstruction::MakeStruct { safety_checks, .. }
        | VirInstruction::MakeArray { safety_checks, .. }
        | VirInstruction::CallStatic { safety_checks, .. } => safety_checks,
    }
}

fn instruction_safety_mut(instruction: &mut VirInstruction) -> &mut Vec<VirSafetyCheck> {
    match instruction {
        VirInstruction::Const { safety_checks, .. }
        | VirInstruction::Copy { safety_checks, .. }
        | VirInstruction::BinOp { safety_checks, .. }
        | VirInstruction::UnaryOp { safety_checks, .. }
        | VirInstruction::Convert { safety_checks, .. }
        | VirInstruction::Field { safety_checks, .. }
        | VirInstruction::Index { safety_checks, .. }
        | VirInstruction::MakeStruct { safety_checks, .. }
        | VirInstruction::MakeArray { safety_checks, .. }
        | VirInstruction::CallStatic { safety_checks, .. } => safety_checks,
    }
}

fn decode_hex(path: &Path) -> Vec<u8> {
    let input = fs::read_to_string(path).expect("hex fixture");
    let input = input.trim();
    assert_eq!(input.len() % 2, 0);
    input
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).expect("ASCII hex"), 16)
                .expect("valid hex")
        })
        .collect()
}
