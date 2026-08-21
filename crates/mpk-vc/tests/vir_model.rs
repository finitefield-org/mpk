use std::collections::HashSet;

use mpk_vc::{
    import_vir_json, SemanticContext, StrictJsonError, VirConstDecl, VirContract, VirContractExpr,
    VirImportError, VirInstruction, VirInstructionKind, VirModule, VirSafetyCheck,
    VirSafetyCheckKind, VirStructDecl, VirTerminator, VirTerminatorKind, VirType,
};
use serde::Deserialize;
use serde_json::{Map, Value};

const VIR_VECTORS: &[u8] = include_bytes!("../../../develop/specs/vectors/vir-v0.json");
const LEGACY_GIR: &[u8] = include_bytes!("../../../examples/max64/gir.json");

#[test]
fn imports_valid_module_vectors_and_rejects_every_structural_module_case() {
    let vectors = vectors();
    assert_eq!(vectors.module_cases.len(), 7, "module vector count changed");

    for case in &vectors.module_cases {
        let input = materialize_module_case(&vectors, case);
        let result = import_vir_json(&input);
        match case.expect.outcome.as_str() {
            "accept" => {
                let module = result.unwrap_or_else(|error| panic!("{}: {error}", case.id));
                let original: Value = serde_json::from_slice(&input).expect("valid module JSON");
                let encoded = serde_json::to_value(module).expect("VIR model serializes");
                assert_eq!(encoded, original, "{} changed document shape", case.id);
            }
            "reject" => assert!(result.is_err(), "{} unexpectedly imported", case.id),
            outcome => panic!("{} has unknown outcome {outcome:?}", case.id),
        }
    }

    let duplicate = vectors
        .module_cases
        .iter()
        .find(|case| case.id == "module.reject_duplicate_key")
        .expect("duplicate-key vector exists");
    let error = import_vir_json(
        duplicate
            .json_text
            .as_deref()
            .expect("duplicate-key case carries raw JSON")
            .as_bytes(),
    )
    .expect_err("duplicate object name rejects before typed deserialization");
    assert!(matches!(
        error,
        VirImportError::StrictJson(StrictJsonError::DuplicateObjectName { .. })
    ));
}

#[test]
fn structure_vectors_cover_all_instruction_variants_and_forbidden_fields() {
    let vectors = vectors();
    assert_eq!(
        vectors.instruction_cases.len(),
        27,
        "instruction vector count changed"
    );
    let mut kinds = HashSet::new();

    for case in &vectors.instruction_cases {
        let input = case.input.as_ref().expect("instruction case has input");
        let result = serde_json::from_value::<VirInstruction>(input.clone());
        let must_reject_structurally = matches!(
            case.expect.code.as_deref(),
            Some("VIR_INSTRUCTION_SHAPE" | "VIR_INSTRUCTION_KIND" | "VIR_VALUE_SHAPE")
        );

        if must_reject_structurally {
            assert!(result.is_err(), "{} retained an invalid shape", case.id);
        } else {
            let instruction = result.unwrap_or_else(|error| {
                panic!("{} should reach semantic validation: {error}", case.id)
            });
            if case.expect.outcome == "accept" {
                kinds.insert(instruction.kind());
            }
        }
    }

    assert_eq!(
        kinds,
        HashSet::from([
            VirInstructionKind::Const,
            VirInstructionKind::Copy,
            VirInstructionKind::BinOp,
            VirInstructionKind::UnaryOp,
            VirInstructionKind::Convert,
            VirInstructionKind::Field,
            VirInstructionKind::Index,
            VirInstructionKind::MakeStruct,
            VirInstructionKind::MakeArray,
            VirInstructionKind::CallStatic,
        ])
    );
}

#[test]
fn structure_vectors_cover_types_declarations_and_terminators() {
    let vectors = vectors();
    assert_eq!(vectors.type_cases.len(), 26, "type vector count changed");
    assert_eq!(
        vectors.terminator_cases.len(),
        7,
        "terminator vector count changed"
    );

    for case in &vectors.type_cases {
        let input = case.input.as_ref().expect("type case has input");
        match case
            .context
            .as_ref()
            .and_then(|context| context.get("validator"))
            .and_then(Value::as_str)
        {
            Some("struct_decl") => {
                serde_json::from_value::<VirStructDecl>(input.clone()).unwrap_or_else(|error| {
                    panic!("{} should deserialize structurally: {error}", case.id)
                });
            }
            Some("const_decl") => {
                serde_json::from_value::<VirConstDecl>(input.clone()).unwrap_or_else(|error| {
                    panic!("{} should deserialize structurally: {error}", case.id)
                });
            }
            None => {
                let result = serde_json::from_value::<VirType>(input.clone());
                let must_reject_structurally = matches!(
                    case.expect.code.as_deref(),
                    Some(
                        "VIR_TYPE_SHAPE"
                            | "VIR_TYPE_KIND"
                            | "VIR_TYPE_WIDTH"
                            | "VIR_LIMIT_ARRAY_ELEMENTS"
                    )
                );
                assert_eq!(
                    result.is_err(),
                    must_reject_structurally,
                    "{} reached the wrong validation layer",
                    case.id
                );
            }
            Some(validator) => panic!("{} has unknown validator {validator:?}", case.id),
        }
    }

    let mut kinds = HashSet::new();
    for case in &vectors.terminator_cases {
        let input = case.input.as_ref().expect("terminator case has input");
        let result = serde_json::from_value::<VirTerminator>(input.clone());
        let must_reject_structurally = matches!(
            case.expect.code.as_deref(),
            Some("VIR_TERMINATOR_SHAPE" | "VIR_TERMINATOR_KIND")
        );
        if must_reject_structurally {
            assert!(result.is_err(), "{} retained an invalid shape", case.id);
        } else {
            let terminator = result.unwrap_or_else(|error| {
                panic!("{} should reach semantic validation: {error}", case.id)
            });
            if case.expect.outcome == "accept" {
                kinds.insert(terminator.kind());
            }
        }
    }
    assert_eq!(
        kinds,
        HashSet::from([
            VirTerminatorKind::Return,
            VirTerminatorKind::Jump,
            VirTerminatorKind::Branch,
        ])
    );
}

#[test]
fn safety_check_and_contract_expression_unions_fail_closed() {
    let vectors = vectors();
    assert_eq!(
        vectors.safety_check_cases.len(),
        17,
        "safety-check vector count changed"
    );
    assert!(
        serde_json::from_value::<VirSafetyCheck>(serde_json::json!({
            "kind": "divisor_nonzero",
            "operation": "div"
        }))
        .is_err(),
        "a field on a fieldless safety-check variant must reject"
    );
    let mut kinds = HashSet::new();
    for case in &vectors.safety_check_cases {
        let input = case.input.as_ref().expect("safety-check case has input");
        let result = serde_json::from_value::<Vec<VirSafetyCheck>>(input.clone());
        if case.expect.code.as_deref() == Some("VIR_SAFETY_CHECK_KIND") {
            assert!(result.is_err(), "{} accepted an unknown kind", case.id);
        } else {
            let checks = result.unwrap_or_else(|error| {
                panic!("{} should reach check-set validation: {error}", case.id)
            });
            if case.expect.outcome == "accept" {
                kinds.extend(checks.iter().map(VirSafetyCheck::kind));
            }
        }
    }
    assert_eq!(
        kinds,
        HashSet::from([
            VirSafetyCheckKind::IntegerNoOverflow,
            VirSafetyCheckKind::DivisorNonzero,
            VirSafetyCheckKind::SignedDivremRepresentable,
            VirSafetyCheckKind::ShiftCountNonnegative,
            VirSafetyCheckKind::ShiftCountLessThanWidth,
            VirSafetyCheckKind::IndexInBounds,
        ])
    );

    for case in &vectors.contract_cases {
        let Some(input) = &case.input else {
            continue;
        };
        let validator = case
            .context
            .as_ref()
            .and_then(|context| context.get("validator"))
            .and_then(Value::as_str)
            .expect("contract case names its validator");
        match validator {
            "expression" => {
                let result = serde_json::from_value::<VirContractExpr>(input.clone());
                if case.id == "contract.reject_field_selection" {
                    assert!(result.is_err(), "unknown contract operator was accepted");
                } else {
                    result.unwrap_or_else(|error| {
                        panic!("{} should reach semantic validation: {error}", case.id)
                    });
                }
            }
            "contract" => {
                serde_json::from_value::<VirContract>(input.clone()).unwrap_or_else(|error| {
                    panic!("{} should reach semantic validation: {error}", case.id)
                });
            }
            other => panic!("{} has unknown validator {other:?}", case.id),
        }
    }
}

#[test]
fn profile_parameters_are_closed_and_language_pairing_is_explicit() {
    let vectors = vectors();
    let cases = [
        "profile.reject_go_with_rust_profile",
        "profile.reject_rust_with_go_profile",
        "profile.reject_go_unknown_parameter",
        "profile.reject_rust_wrong_overflow_mode",
        "profile.rust_i686_target_width",
    ];

    for id in cases {
        let case = vectors
            .profile_cases
            .iter()
            .find(|candidate| candidate.id == id)
            .unwrap_or_else(|| panic!("missing profile vector {id}"));
        let input = case.input.as_ref().expect("profile case has input");
        let decoded = serde_json::from_value::<SemanticContext>(input.clone());
        match id {
            "profile.reject_go_with_rust_profile" | "profile.reject_rust_with_go_profile" => {
                assert!(
                    decoded.is_err(),
                    "wrong language/profile pairing deserialized"
                );
            }
            "profile.reject_go_unknown_parameter" | "profile.reject_rust_wrong_overflow_mode" => {
                assert!(decoded.is_err(), "{id} accepted an open parameter object");
            }
            "profile.rust_i686_target_width" => decoded
                .expect("registered-width shape deserializes")
                .validate()
                .expect("valid Rust context pairs"),
            _ => unreachable!(),
        }
    }

    assert!(
        serde_json::from_value::<SemanticContext>(serde_json::json!({
            "source_language": "go",
            "semantic_profile": "mpk.go.fixed.v0",
            "semantic_parameters": {
                "target_id": "x86_64-unknown-linux-gnu",
                "pointer_width": 64,
                "overflow_mode": "checked",
                "panic_mode": "abort"
            }
        }))
        .is_err(),
        "parameters for another profile must not deserialize into a semantic context"
    );
}

#[test]
fn legacy_gir_and_wrong_language_module_do_not_convert_to_vir() {
    assert!(
        import_vir_json(LEGACY_GIR).is_err(),
        "a GIR document must not deserialize through the VIR importer"
    );

    let vectors = vectors();
    let mut module = vectors
        .module_cases
        .iter()
        .find(|case| case.id == "module.valid_go_identity")
        .and_then(|case| case.input.clone())
        .expect("valid Go module vector exists");
    let root = module.as_object_mut().expect("module is an object");
    root.insert(
        "semantic_profile".to_owned(),
        Value::String("mpk.rust.checked.v0".to_owned()),
    );
    root.insert(
        "semantic_parameters".to_owned(),
        serde_json::json!({
            "target_id": "x86_64-unknown-linux-gnu",
            "pointer_width": 64,
            "overflow_mode": "checked",
            "panic_mode": "abort"
        }),
    );
    let bytes = serde_json::to_vec(&module).expect("serialize wrong-language module");
    assert!(
        serde_json::from_value::<VirModule>(module).is_err(),
        "the typed VIR boundary must reject a wrong-language profile"
    );
    import_vir_json(&bytes).expect_err("strict VIR import rejects a wrong-language profile");
}

fn vectors() -> VectorFile {
    let vectors: VectorFile = serde_json::from_slice(VIR_VECTORS).expect("parse VIR vectors");
    assert_eq!(vectors.schema, "mpk.vir.conformance.v0");
    assert_eq!(vectors.spec_schema, "mpk.vir.v0");
    assert_eq!(
        vectors.owner_tests,
        [
            "crates/mpk-vc/tests/vir_model.rs",
            "crates/mpk-vc/tests/vir_validation.rs"
        ]
    );
    assert_eq!(vectors.limit_cases.len(), 34, "limit vector count changed");
    vectors
}

fn materialize_module_case(vectors: &VectorFile, case: &VectorCase) -> Vec<u8> {
    if let Some(input) = &case.input {
        return serde_json::to_vec(input).expect("serialize module input");
    }
    if let Some(json_text) = &case.json_text {
        return json_text.as_bytes().to_vec();
    }

    let construction = case
        .construction
        .as_ref()
        .and_then(Value::as_object)
        .expect("module construction is an object");
    assert!(construction.get("pointer").is_none());
    let base_id = construction
        .get("base")
        .and_then(Value::as_str)
        .expect("module construction names a base");
    let mut value = vectors
        .module_cases
        .iter()
        .find(|candidate| candidate.id == base_id)
        .and_then(|candidate| candidate.input.clone())
        .expect("module construction base resolves directly");
    for patch in construction
        .get("patches")
        .and_then(Value::as_array)
        .expect("module construction has patches")
    {
        apply_root_patch(&mut value, patch);
    }
    serde_json::to_vec(&value).expect("serialize constructed module")
}

fn apply_root_patch(value: &mut Value, patch: &Value) {
    let patch = patch.as_object().expect("patch is an object");
    let path = patch
        .get("path")
        .and_then(Value::as_str)
        .and_then(|path| path.strip_prefix('/'))
        .expect("patch has a root-member path");
    assert!(
        !path.contains('/'),
        "module patch must target one root field"
    );
    let root: &mut Map<String, Value> = value.as_object_mut().expect("module is an object");
    match patch.get("op").and_then(Value::as_str) {
        Some("add") => {
            assert!(root
                .insert(path.to_owned(), patch["value"].clone())
                .is_none());
        }
        Some("replace") => {
            assert!(root
                .insert(path.to_owned(), patch["value"].clone())
                .is_some());
        }
        Some("remove") => {
            assert!(root.remove(path).is_some());
        }
        operation => panic!("unsupported patch operation {operation:?}"),
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VectorFile {
    schema: String,
    spec_schema: String,
    owner_tests: Vec<String>,
    module_cases: Vec<VectorCase>,
    type_cases: Vec<VectorCase>,
    instruction_cases: Vec<VectorCase>,
    terminator_cases: Vec<VectorCase>,
    contract_cases: Vec<VectorCase>,
    safety_check_cases: Vec<VectorCase>,
    profile_cases: Vec<VectorCase>,
    limit_cases: Vec<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VectorCase {
    id: String,
    #[serde(default)]
    input: Option<Value>,
    #[serde(default)]
    json_text: Option<String>,
    #[serde(default)]
    construction: Option<Value>,
    #[serde(default)]
    context: Option<Value>,
    expect: VectorExpectation,
}

#[derive(Debug, Deserialize)]
struct VectorExpectation {
    outcome: String,
    #[serde(default)]
    code: Option<String>,
}
