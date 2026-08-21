use std::collections::{BTreeSet, HashSet};

use mpk_vc::{
    contract_hash, import_vir_json, validate_vir_const_decl_fragment,
    validate_vir_contract_expr_fragment, validate_vir_limit_count, validate_vir_safety_fragment,
    validate_vir_struct_decl_fragment, validate_vir_type_fragment, vir_hash, SemanticProfile,
    SourceLanguage, VirBinding, VirConstDecl, VirContractExpr, VirImportError, VirInstruction,
    VirInstructionKind, VirModule, VirSafetyCheck, VirSafetyOperation, VirStructDecl, VirType,
};
use serde_json::{json, Value};

const VIR_VECTORS: &[u8] = include_bytes!("../../../develop/specs/vectors/vir-v0.json");

#[test]
fn every_type_declaration_and_constant_vector_reaches_its_owned_validation_layer() {
    let vectors = vectors();
    let cases = cases(&vectors, "type_cases");
    assert_eq!(cases.len(), 26);
    let mut seen = HashSet::new();

    for case in cases {
        let id = string(case, "id");
        assert!(seen.insert(id), "duplicate case {id}");
        let expected = expectation(case);
        let input = case.get("input").expect("type case has input");
        let validator = case.pointer("/context/validator").and_then(Value::as_str);
        let result = match validator {
            Some("struct_decl") => {
                let declaration = serde_json::from_value::<VirStructDecl>(input.clone());
                declaration
                    .map_err(|_| "STRUCTURE")
                    .and_then(|declaration| {
                        let language = source_language(case);
                        let unit = case
                            .pointer("/context/unit_id")
                            .and_then(Value::as_str)
                            .expect("unit id");
                        validate_vir_struct_decl_fragment(language, unit, &declaration, &[])
                            .map_err(|error| error.code())
                    })
            }
            Some("const_decl") => {
                let declaration = serde_json::from_value::<VirConstDecl>(input.clone());
                declaration
                    .map_err(|_| "STRUCTURE")
                    .and_then(|declaration| {
                        let language = source_language(case);
                        let unit = case
                            .pointer("/context/unit_id")
                            .and_then(Value::as_str)
                            .expect("unit id");
                        validate_vir_const_decl_fragment(language, unit, &declaration, &[])
                            .map_err(|error| error.code())
                    })
            }
            None => serde_json::from_value::<VirType>(input.clone())
                .map_err(|_| "STRUCTURE")
                .and_then(|r#type| {
                    let declarations = type_declarations(case);
                    validate_vir_type_fragment(&r#type, &declarations).map_err(|error| error.code())
                }),
            Some(other) => panic!("unknown type validator {other}"),
        };
        assert_vector_result(id, expected, result);
    }

    let mut nested = json!({"kind":"bool"});
    for _ in 0..16 {
        nested = json!({"kind":"array","length":0,"element":nested});
    }
    let declaration: VirStructDecl = serde_json::from_value(json!({
        "id":"vector::Deep","name":"Deep","fields":[{"name":"value","type":nested}]
    }))
    .expect("deep declaration decodes");
    let error =
        validate_vir_struct_decl_fragment(SourceLanguage::Rust, "vector", &declaration, &[])
            .expect_err("outer struct contributes to aggregate depth");
    assert_eq!(error.code(), "VIR_LIMIT_AGGREGATE_TYPE_NESTING");
}

#[test]
fn every_instruction_and_terminator_vector_is_executed() {
    let vectors = vectors();
    let instruction_cases = cases(&vectors, "instruction_cases");
    assert_eq!(instruction_cases.len(), 27);
    for case in instruction_cases {
        let id = string(case, "id");
        let expected = expectation(case);
        let instruction = match serde_json::from_value::<VirInstruction>(case["input"].clone()) {
            Ok(instruction) => instruction,
            Err(_) => {
                assert_eq!(expected.outcome, "reject", "{id}");
                assert!(is_shape_code(expected.code), "{id} failed before semantics");
                continue;
            }
        };
        let module = module_for_instruction(case, instruction);
        assert_import_result(id, expected, module);
    }

    let terminator_cases = cases(&vectors, "terminator_cases");
    assert_eq!(terminator_cases.len(), 7);
    for case in terminator_cases {
        let id = string(case, "id");
        let expected = expectation(case);
        if serde_json::from_value::<mpk_vc::VirTerminator>(case["input"].clone()).is_err() {
            assert_eq!(expected.outcome, "reject", "{id}");
            assert!(is_shape_code(expected.code), "{id} failed before semantics");
            continue;
        }
        assert_import_result(id, expected, module_for_terminator(case));
    }
}

#[test]
fn every_contract_and_safety_check_vector_is_executed() {
    let vectors = vectors();
    let contract_cases = cases(&vectors, "contract_cases");
    assert_eq!(contract_cases.len(), 17);
    for case in contract_cases {
        let id = string(case, "id");
        let expected = expectation(case);
        match case.pointer("/context/validator").and_then(Value::as_str) {
            Some("expression") => {
                let expression = serde_json::from_value::<VirContractExpr>(case["input"].clone());
                let result = expression.map_err(|_| "STRUCTURE").and_then(|expression| {
                    let context = &case["context"];
                    let profile = semantic_profile(context);
                    let variables = bindings(context, "bindings");
                    let results = bindings(context, "results");
                    let declarations = object_type_declarations(context);
                    validate_vir_contract_expr_fragment(
                        &expression,
                        profile,
                        &variables,
                        &results,
                        &declarations,
                    )
                    .map(|_| ())
                    .map_err(|error| error.code())
                });
                assert_vector_result(id, expected, result);
            }
            Some("contract") if id == "contract.reject_partial_with_decreases" => {
                let mut module = loop_module(SourceLanguage::Go, true);
                module["units"][0]["functions"][0]["id"] = json!("example.com/mpk/vector.Loop");
                module["units"][0]["functions"][0]["name"] = json!("Loop");
                module["units"][0]["functions"][0]["contracts"] = case["input"].clone();
                assert_import_result(id, expected, module);
            }
            Some("contract") => {
                let module = materialize_contract_case(&vectors, case);
                assert_import_result(id, expected, module);
            }
            validator => panic!("unknown contract validator {validator:?}"),
        }
    }

    let safety_cases = cases(&vectors, "safety_check_cases");
    assert_eq!(safety_cases.len(), 17);
    for case in safety_cases {
        let id = string(case, "id");
        let expected = expectation(case);
        let actual = serde_json::from_value::<Vec<VirSafetyCheck>>(case["input"].clone());
        let result = actual.map_err(|_| "STRUCTURE").and_then(|actual| {
            let context = &case["context"];
            let profile = semantic_profile(context);
            let operand_types = context["operand_types"]
                .as_array()
                .expect("operand types")
                .iter()
                .map(|value| serde_json::from_value(value.clone()).expect("operand type"))
                .collect::<Vec<_>>();
            let operation = safety_operation(context);
            validate_vir_safety_fragment(profile, operation, &operand_types, &actual)
                .map_err(|error| error.code())
        });
        assert_vector_result(id, expected, result);
    }
}

#[test]
fn profiles_graphs_and_minimum_collections_are_validated_end_to_end() {
    let vectors = vectors();
    let profile_cases = cases(&vectors, "profile_cases");
    assert_eq!(profile_cases.len(), 32);
    let mut handled = BTreeSet::new();
    for case in profile_cases {
        let id = string(case, "id");
        assert!(handled.insert(id), "duplicate profile vector {id}");
        let expected = expectation(case);
        if matches!(
            id,
            "profile.reject_go_with_rust_profile"
                | "profile.reject_rust_with_go_profile"
                | "profile.reject_go_unknown_parameter"
                | "profile.reject_rust_wrong_overflow_mode"
                | "profile.reject_go_target_width_mismatch"
                | "profile.reject_rust_target_width_mismatch"
                | "profile.rust_i686_target_width"
        ) {
            assert_import_result(id, expected, module_for_semantic_context(&vectors, case));
        } else if id == "profile.go_zero_result_function" {
            let mut module = base_module(&vectors, "module.valid_go_identity");
            make_zero_result(&mut module);
            rehash(&mut module);
            assert_import_result(id, expected, module);
        } else if id == "profile.reject_rust_zero_result_function" {
            let mut module = base_module(&vectors, "module.valid_rust_identity");
            make_zero_result(&mut module);
            assert_import_result(id, expected, module);
        } else if id == "profile.go_cycle_with_cutpoint" {
            let mut module = loop_module(SourceLanguage::Go, false);
            rehash(&mut module);
            assert_import_result(id, expected, module);
        } else if id == "profile.reject_rust_same_cycle" {
            assert_import_result(id, expected, loop_module(SourceLanguage::Rust, false));
        } else if matches!(
            id,
            "profile.reject_rust_unsigned_neg"
                | "profile.reject_rust_convert"
                | "profile.reject_rust_non_usize_index"
        ) {
            assert_import_result(id, expected, module_for_invalid_profile_operation(case));
        } else {
            let input = &case["input"];
            let actual: Vec<VirSafetyCheck> = serde_json::from_value(
                input
                    .get("safety_checks")
                    .cloned()
                    .unwrap_or_else(|| json!([])),
            )
            .expect("profile safety checks");
            let operation = safety_operation(input);
            let operands = profile_operand_types(input);
            let result = validate_vir_safety_fragment(
                semantic_profile(input),
                operation,
                &operands,
                &actual,
            )
            .map_err(|error| error.code());
            assert_vector_result(id, expected, result);
        }
    }
    assert_eq!(handled.len(), profile_cases.len());

    for (field, code) in [
        ("units", "VIR_EMPTY_UNITS"),
        ("functions", "VIR_EMPTY_FUNCTIONS"),
        ("blocks", "VIR_EMPTY_BLOCKS"),
        ("ensures", "VIR_EMPTY_ENSURES"),
    ] {
        let mut module = base_module(&vectors, "module.valid_go_identity");
        match field {
            "units" => module["units"] = json!([]),
            "functions" => module["units"][0]["functions"] = json!([]),
            "blocks" => module["units"][0]["functions"][0]["blocks"] = json!([]),
            "ensures" => module["units"][0]["functions"][0]["contracts"]["ensures"] = json!([]),
            _ => unreachable!(),
        }
        assert_validation_code(&module, code);
    }
}

#[test]
fn every_shared_limit_vector_freezes_the_validator_boundary() {
    let vectors = vectors();
    let limits = cases(&vectors, "limit_cases");
    assert_eq!(limits.len(), 34);
    let mut seen = BTreeSet::new();
    for case in limits {
        let id = string(case, "id");
        assert!(seen.insert(id), "duplicate limit {id}");
        let limit_id = string(case, "limit_id");
        let Some(maximum) = case.get("maximum").and_then(Value::as_u64) else {
            assert!(limit_id.starts_with("minimum_"), "{id}");
            assert_eq!(case["minimum"], 1, "{id}");
            let mut at = base_module(&vectors, "module.valid_go_identity");
            rehash(&mut at);
            import_vir_json(&serde_json::to_vec(&at).expect("at-minimum module serializes"))
                .unwrap_or_else(|error| panic!("{id} at minimum rejected: {error}"));
            let mut below = at;
            match limit_id {
                "minimum_units" => below["units"] = json!([]),
                "minimum_functions" => below["units"][0]["functions"] = json!([]),
                "minimum_blocks" => below["units"][0]["functions"][0]["blocks"] = json!([]),
                "minimum_ensures" => {
                    below["units"][0]["functions"][0]["contracts"]["ensures"] = json!([])
                }
                other => panic!("unknown minimum limit {other}"),
            }
            assert_validation_code(
                &below,
                case.pointer("/below/expect/code")
                    .and_then(Value::as_str)
                    .expect("below-minimum code"),
            );
            continue;
        };
        validate_vir_limit_count(limit_id, maximum)
            .unwrap_or_else(|error| panic!("{id} at limit failed: {error}"));
        let error = match validate_vir_limit_count(limit_id, maximum + 1) {
            Ok(()) => panic!("{id} above limit passed"),
            Err(error) => error,
        };
        assert_eq!(
            error.code(),
            case.pointer("/above/expect/code")
                .and_then(Value::as_str)
                .expect("above-limit code"),
            "{id}"
        );
    }
    assert_eq!(seen.len(), limits.len());
}

#[test]
fn identity_reference_cfg_call_and_hash_mutations_fail_closed() {
    let vectors = vectors();
    let base = base_module(&vectors, "module.valid_go_identity");
    for (path, replacement, code) in [
        (
            "/units/0/functions/0/blocks/0/terminator/values/0/var",
            json!("missing"),
            "VIR_UNKNOWN_VALUE",
        ),
        (
            "/units/0/functions/0/blocks/0/label",
            json!("entry"),
            "VIR_BLOCK_ID",
        ),
        (
            "/units/0/functions/0/blocks/0/terminator/values",
            json!([]),
            "VIR_RETURN_TYPE",
        ),
        (
            "/units/0/functions/0/contracts/function_id",
            json!("example.com/mpk/vector.Other"),
            "VIR_CONTRACT_IDENTITY",
        ),
        (
            "/vir_hash",
            json!("0000000000000000000000000000000000000000000000000000000000000000"),
            "VIR_HASH_MISMATCH",
        ),
    ] {
        let mut module = base.clone();
        *module.pointer_mut(path).expect("mutation path exists") = replacement;
        assert_validation_code(&module, code);
    }

    for forbidden in [
        "/tmp/vector",
        "file://vector",
        "vector\\host",
        "../vector",
        "vector/../host",
    ] {
        let mut module = base.clone();
        module["units"][0]["id"] = json!(forbidden);
        assert_validation_code(&module, "VIR_IDENTIFIER");
    }

    let mut long_reference = base.clone();
    long_reference["units"][0]["functions"][0]["blocks"][0]["terminator"]["values"][0]["var"] =
        json!("x".repeat(1_025));
    assert_validation_code(&long_reference, "VIR_LIMIT_IDENTIFIER_BYTES");

    let mut rust_units = base_module(&vectors, "module.valid_rust_identity");
    let duplicate = rust_units["units"][0].clone();
    rust_units["units"]
        .as_array_mut()
        .expect("units")
        .push(duplicate);
    assert_validation_code(&rust_units, "VIR_RUST_UNIT_COUNT");

    let mut recursive = base_module(&vectors, "module.valid_rust_identity");
    let contract_hash = recursive["units"][0]["functions"][0]["contracts"]["contract_hash"].clone();
    recursive["units"][0]["functions"][0]["blocks"][0]["instructions"] = json!([{
        "id":"t0","kind":"CallStatic","type":{"kind":"bv","width":8,"signed":true},
        "function":"vector::identity","contract_hash":contract_hash,
        "args":[{"var":"arg0"}],"safety_checks":[]
    }]);
    recursive["units"][0]["functions"][0]["blocks"][0]["terminator"]["values"] =
        json!([{"var":"t0"}]);
    recursive["units"][0]["functions"][0]["features_used"] = json!(["call_static"]);
    assert_validation_code(&recursive, "VIR_CALL_CYCLE");

    let mut ordered_parameters = block_parameter_order_module(&vectors);
    rehash(&mut ordered_parameters);
    import_vir_json(&serde_json::to_vec(&ordered_parameters).expect("module serializes"))
        .expect("successor arguments use serialized parameter order beyond p9");

    let mut missing_check = base_module(&vectors, "module.valid_rust_identity");
    missing_check["units"][0]["functions"][0]["blocks"][0]["instructions"] = json!([{
        "id":"t0","kind":"BinOp","op":"bv_add",
        "type":{"kind":"bv","width":8,"signed":true},
        "lhs":{"var":"arg0"},"rhs":{"int":{"value":"1","width":8,"signed":true}},
        "safety_checks":[]
    }]);
    missing_check["units"][0]["functions"][0]["blocks"][0]["terminator"]["values"] =
        json!([{"var":"t0"}]);
    missing_check["units"][0]["functions"][0]["contracts"]["contract_hash"] =
        json!("0000000000000000000000000000000000000000000000000000000000000000");
    assert_validation_code(&missing_check, "VIR_SAFETY_CHECK_MISSING");

    let mut shared_structs = branching_struct_module(&vectors);
    rehash(&mut shared_structs);
    import_vir_json(&serde_json::to_vec(&shared_structs).expect("module serializes"))
        .expect("shared nested struct DAG validates without recursive re-expansion");
}

#[derive(Clone, Copy)]
struct Expected<'a> {
    outcome: &'a str,
    code: Option<&'a str>,
}

fn vectors() -> Value {
    let vectors: Value = serde_json::from_slice(VIR_VECTORS).expect("VIR vectors parse");
    assert_object_fields(
        &vectors,
        &[
            "schema",
            "spec_schema",
            "owner_tests",
            "module_cases",
            "type_cases",
            "instruction_cases",
            "terminator_cases",
            "contract_cases",
            "safety_check_cases",
            "profile_cases",
            "limit_cases",
        ],
    );
    assert_eq!(vectors["schema"], "mpk.vir.conformance.v0");
    assert_eq!(vectors["spec_schema"], "mpk.vir.v0");
    assert_eq!(
        vectors["owner_tests"],
        json!([
            "crates/mpk-vc/tests/vir_model.rs",
            "crates/mpk-vc/tests/vir_validation.rs"
        ])
    );
    for section in [
        "module_cases",
        "type_cases",
        "instruction_cases",
        "terminator_cases",
        "contract_cases",
        "safety_check_cases",
        "profile_cases",
    ] {
        for case in cases(&vectors, section) {
            assert_allowed_object_fields(
                case,
                &[
                    "id",
                    "input",
                    "json_text",
                    "construction",
                    "context",
                    "expect",
                ],
            );
        }
    }
    for case in cases(&vectors, "limit_cases") {
        assert_allowed_object_fields(
            case,
            &[
                "id", "limit_id", "maximum", "minimum", "at", "above", "below",
            ],
        );
    }
    vectors
}

fn assert_object_fields(value: &Value, expected: &[&str]) {
    let actual: BTreeSet<_> = value
        .as_object()
        .expect("value is an object")
        .keys()
        .map(String::as_str)
        .collect();
    let expected: BTreeSet<_> = expected.iter().copied().collect();
    assert_eq!(actual, expected, "object field set changed");
}

fn assert_allowed_object_fields(value: &Value, allowed: &[&str]) {
    let allowed: BTreeSet<_> = allowed.iter().copied().collect();
    for name in value.as_object().expect("case is an object").keys() {
        assert!(allowed.contains(name.as_str()), "unknown case field {name}");
    }
}

fn cases<'a>(vectors: &'a Value, name: &str) -> &'a [Value] {
    vectors[name].as_array().expect("vector section is array")
}

fn string<'a>(value: &'a Value, name: &str) -> &'a str {
    value[name]
        .as_str()
        .unwrap_or_else(|| panic!("missing {name}"))
}

fn expectation(case: &Value) -> Expected<'_> {
    Expected {
        outcome: string(&case["expect"], "outcome"),
        code: case.pointer("/expect/code").and_then(Value::as_str),
    }
}

fn assert_vector_result(id: &str, expected: Expected<'_>, result: Result<(), &'static str>) {
    match expected.outcome {
        "accept" => result.unwrap_or_else(|code| panic!("{id} rejected with {code}")),
        "reject" => {
            let code = match result {
                Ok(()) => panic!("{id} unexpectedly accepted"),
                Err(code) => code,
            };
            if code != "STRUCTURE" {
                assert_eq!(Some(code), expected.code, "{id}");
            } else {
                assert!(
                    is_shape_code(expected.code),
                    "{id}: unexpected structural failure"
                );
            }
        }
        outcome => panic!("{id}: unknown outcome {outcome}"),
    }
}

fn is_shape_code(code: Option<&str>) -> bool {
    code.is_some_and(|code| {
        code.contains("SHAPE")
            || code.ends_with("_KIND")
            || code == "VIR_UNKNOWN_FIELD"
            || code == "VIR_CONTRACT_OPERATOR"
            || code == "VIR_TYPE_WIDTH"
            || code == "VIR_LIMIT_ARRAY_ELEMENTS"
    })
}

fn assert_import_result(id: &str, expected: Expected<'_>, module: Value) {
    let bytes = serde_json::to_vec(&module).expect("module serializes");
    match (expected.outcome, import_vir_json(&bytes)) {
        ("accept", Ok(_)) => {}
        ("accept", Err(error)) => panic!("{id} rejected: {error}"),
        ("reject", Ok(_)) => panic!("{id} unexpectedly accepted"),
        ("reject", Err(VirImportError::Validation(error))) => {
            assert_eq!(Some(error.code()), expected.code, "{id}")
        }
        ("reject", Err(error)) if is_shape_code(expected.code) => {
            assert!(
                matches!(error, VirImportError::InvalidShape(_)),
                "{id}: {error}"
            )
        }
        ("reject", Err(error)) => panic!("{id} failed at wrong layer: {error}"),
        (outcome, _) => panic!("{id}: unknown outcome {outcome}"),
    }
}

fn assert_validation_code(module: &Value, code: &str) {
    let bytes = serde_json::to_vec(module).expect("module serializes");
    let error = import_vir_json(&bytes).expect_err("mutation must reject");
    match error {
        VirImportError::Validation(error) => assert_eq!(error.code(), code),
        other => panic!("expected {code}, got {other}"),
    }
}

fn source_language(case: &Value) -> SourceLanguage {
    serde_json::from_value(case["context"]["source_language"].clone()).expect("source language")
}

fn semantic_profile(context: &Value) -> SemanticProfile {
    serde_json::from_value(context["semantic_profile"].clone()).expect("semantic profile")
}

fn bindings(context: &Value, name: &str) -> Vec<VirBinding> {
    serde_json::from_value(context.get(name).cloned().unwrap_or_else(|| json!([])))
        .expect("bindings")
}

fn object_type_declarations(context: &Value) -> Vec<VirStructDecl> {
    context
        .get("type_decls")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|value| value.is_object())
        .map(|value| serde_json::from_value(value.clone()).expect("type declaration"))
        .collect()
}

fn type_declarations(case: &Value) -> Vec<VirStructDecl> {
    case.pointer("/context/type_decls")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|value| match value {
            Value::String(id) => VirStructDecl {
                id: id.clone(),
                name: id
                    .rsplit_once("::")
                    .map_or(id.as_str(), |(_, name)| name)
                    .to_owned(),
                fields: Vec::new(),
            },
            _ => serde_json::from_value(value.clone()).expect("type declaration"),
        })
        .collect()
}

fn safety_operation(context: &Value) -> VirSafetyOperation {
    match context
        .get("instruction_kind")
        .and_then(Value::as_str)
        .unwrap_or("BinOp")
    {
        "BinOp" => VirSafetyOperation::Binary(
            serde_json::from_value(context["op"].clone()).expect("binary operator"),
        ),
        "UnaryOp" => VirSafetyOperation::Unary(
            serde_json::from_value(context["op"].clone()).expect("unary operator"),
        ),
        "Index" => VirSafetyOperation::Index,
        "Const" => VirSafetyOperation::None(VirInstructionKind::Const),
        "Convert" => VirSafetyOperation::None(VirInstructionKind::Convert),
        kind => panic!("unknown instruction kind {kind}"),
    }
}

fn profile_operand_types(input: &Value) -> Vec<VirType> {
    if let Some(r#type) = input.get("type") {
        let r#type: VirType = serde_json::from_value(r#type.clone()).expect("operation type");
        return match input.get("instruction_kind").and_then(Value::as_str) {
            Some("UnaryOp") => vec![r#type],
            _ => vec![r#type.clone(), r#type],
        };
    }
    if let (Some(lhs), Some(rhs)) = (input.get("lhs_type"), input.get("rhs_type")) {
        return vec![
            serde_json::from_value(lhs.clone()).expect("lhs type"),
            serde_json::from_value(rhs.clone()).expect("rhs type"),
        ];
    }
    if let (Some(base), Some(index)) = (input.get("base_type"), input.get("index_type")) {
        return vec![
            serde_json::from_value(base.clone()).expect("base type"),
            serde_json::from_value(index.clone()).expect("index type"),
        ];
    }
    Vec::new()
}

fn base_module(vectors: &Value, id: &str) -> Value {
    cases(vectors, "module_cases")
        .iter()
        .find(|case| case["id"] == id)
        .and_then(|case| case.get("input"))
        .cloned()
        .unwrap_or_else(|| panic!("missing base module {id}"))
}

fn module_for_instruction(case: &Value, mut instruction: VirInstruction) -> Value {
    let vectors = vectors();
    let context = &case["context"];
    let profile = semantic_profile(context);
    let base_id = match profile {
        SemanticProfile::GoFixedV0 => "module.valid_go_identity",
        SemanticProfile::RustCheckedV0 => "module.valid_rust_identity",
    };
    let mut module = base_module(&vectors, base_id);
    module["units"][0]["type_decls"] = context
        .get("type_decls")
        .cloned()
        .unwrap_or_else(|| json!([]));
    module["units"][0]["const_decls"] = context
        .get("const_decls")
        .cloned()
        .unwrap_or_else(|| json!([]));
    let function = &mut module["units"][0]["functions"][0];
    function["params"] = context["bindings"].clone();
    function["locals"] = context["locals"].clone();
    function["results"] = json!([{"id":"result0","type":instruction_type(&instruction)}]);
    function["blocks"][0]["instructions"] =
        serde_json::to_value([&instruction]).expect("instruction serializes");
    function["blocks"][0]["terminator"] = json!({"kind":"Return","values":[{"var":"t0"}]});
    function["contracts"]["requires"] = json!([]);
    function["contracts"]["ensures"] = json!([{"bool":true}]);
    function["features_used"] = json!(instruction_features(&instruction, context));

    if let Some(callees) = context.get("callees").and_then(Value::as_array) {
        if let Some(callee) = callees.first() {
            let mut callee_function = function.clone();
            let callee_id = string(callee, "function");
            let callee_name = callee_id
                .rsplit_once("::")
                .or_else(|| callee_id.rsplit_once('.'))
                .map_or(callee_id, |(_, name)| name);
            callee_function["id"] = json!(callee_id);
            callee_function["name"] = json!(callee_name);
            callee_function["params"] = json!(callee["params"]
                .as_array()
                .expect("callee params")
                .iter()
                .enumerate()
                .map(|(index, ty)| json!({"id":format!("arg{index}"),"type":ty}))
                .collect::<Vec<_>>());
            callee_function["results"] = json!([{"id":"result0","type":callee["result"]}]);
            callee_function["blocks"][0]["instructions"] = json!([]);
            callee_function["blocks"][0]["terminator"] =
                json!({"kind":"Return","values":[{"var":"arg0"}]});
            callee_function["contracts"]["function_id"] = json!(callee_id);
            callee_function["contracts"]["ensures"] = json!([{"bool":true}]);
            callee_function["features_used"] = json!([]);
            rehash_contract(&mut callee_function["contracts"]);
            let actual_hash = callee_function["contracts"]["contract_hash"].clone();
            let expected_placeholder = &callee["contract_hash"];
            if instruction_contract_hash(&instruction)
                .is_some_and(|hash| hash == *expected_placeholder)
            {
                set_instruction_contract_hash(&mut instruction, actual_hash);
                function["blocks"][0]["instructions"] =
                    serde_json::to_value([&instruction]).expect("instruction serializes");
            }
            function["id"] = json!("vector::caller");
            function["name"] = json!("caller");
            function["contracts"]["function_id"] = json!("vector::caller");
            module["units"][0]["functions"] = json!([callee_function, function.clone()]);
        }
    }
    rehash(&mut module);
    module
}

fn instruction_type(instruction: &VirInstruction) -> Value {
    let r#type = match instruction {
        VirInstruction::Const { r#type, .. }
        | VirInstruction::Copy { r#type, .. }
        | VirInstruction::BinOp { r#type, .. }
        | VirInstruction::UnaryOp { r#type, .. }
        | VirInstruction::Convert { r#type, .. }
        | VirInstruction::Field { r#type, .. }
        | VirInstruction::Index { r#type, .. }
        | VirInstruction::MakeStruct { r#type, .. }
        | VirInstruction::MakeArray { r#type, .. }
        | VirInstruction::CallStatic { r#type, .. } => r#type,
    };
    serde_json::to_value(r#type).expect("type serializes")
}

fn instruction_features(instruction: &VirInstruction, context: &Value) -> Vec<&'static str> {
    let mut features = BTreeSet::new();
    if context["locals"]
        .as_array()
        .is_some_and(|values| !values.is_empty())
    {
        features.insert("mutable_local");
    }
    match instruction {
        VirInstruction::Copy { value, .. } => {
            features.insert("mutable_local");
            if matches!(value, mpk_vc::VirValue::Constant(_)) {
                features.insert("constant_decl");
            }
        }
        VirInstruction::Convert { .. } => {
            features.insert("conversion");
        }
        VirInstruction::Field { .. } | VirInstruction::MakeStruct { .. } => {
            features.insert("struct");
        }
        VirInstruction::Index { .. } | VirInstruction::MakeArray { .. } => {
            features.insert("array");
        }
        VirInstruction::CallStatic { .. } => {
            features.insert("call_static");
        }
        _ => {}
    }
    features.into_iter().collect()
}

fn instruction_contract_hash(instruction: &VirInstruction) -> Option<Value> {
    match instruction {
        VirInstruction::CallStatic { contract_hash, .. } => Some(json!(contract_hash.as_str())),
        _ => None,
    }
}

fn set_instruction_contract_hash(instruction: &mut VirInstruction, value: Value) {
    if let VirInstruction::CallStatic { contract_hash, .. } = instruction {
        *contract_hash = serde_json::from_value(value).expect("contract hash");
    }
}

fn module_for_terminator(case: &Value) -> Value {
    let vectors = vectors();
    let context = &case["context"];
    let result_types = context["function_results"]
        .as_array()
        .expect("result types");
    let base_id = if result_types.is_empty() {
        "module.valid_go_identity"
    } else {
        "module.valid_rust_identity"
    };
    let mut module = base_module(&vectors, base_id);
    let function = &mut module["units"][0]["functions"][0];
    function["params"] = context["bindings"].clone();
    function["results"] = json!(result_types
        .iter()
        .enumerate()
        .map(|(index, ty)| json!({"id":format!("result{index}"),"type":ty}))
        .collect::<Vec<_>>());
    function["contracts"]["ensures"] = json!([{"bool":true}]);
    let mut blocks = vec![json!({
        "label":"bb0","parameters":[],"instructions":[],"terminator":case["input"]
    })];
    for block in context["blocks"].as_array().expect("blocks") {
        blocks.push(json!({
            "label":block["label"],"parameters":block["parameters"],"instructions":[],
            "terminator":{"kind":"Return","values":[]}
        }));
    }
    function["blocks"] = json!(blocks);
    function["features_used"] = if case["input"]["kind"] == "Branch" {
        json!(["branch"])
    } else {
        json!([])
    };
    rehash(&mut module);
    module
}

fn materialize_contract_case(vectors: &Value, case: &Value) -> Value {
    let construction = &case["construction"];
    let mut module = base_module(vectors, string(construction, "base"));
    let pointer = string(construction, "pointer");
    let target = module.pointer_mut(pointer).expect("contract pointer");
    apply_relative_patches(target, &construction["patches"]);
    module
}

fn apply_relative_patches(target: &mut Value, patches: &Value) {
    for patch in patches.as_array().expect("patches") {
        let path = string(patch, "path");
        match string(patch, "op") {
            "replace" => *target.pointer_mut(path).expect("replace path") = patch["value"].clone(),
            "add" => {
                let name = path.strip_prefix('/').expect("root member patch");
                target
                    .as_object_mut()
                    .expect("object target")
                    .insert(name.to_owned(), patch["value"].clone());
            }
            operation => panic!("unsupported patch {operation}"),
        }
    }
}

fn module_for_semantic_context(vectors: &Value, case: &Value) -> Value {
    let input = &case["input"];
    let language = input["source_language"].as_str().unwrap_or("rust");
    let base_id = if language == "go" {
        "module.valid_go_identity"
    } else {
        "module.valid_rust_identity"
    };
    let mut module = base_module(vectors, base_id);
    for name in ["source_language", "semantic_profile", "semantic_parameters"] {
        if let Some(value) = input.get(name) {
            module[name] = value.clone();
        }
    }
    if expectation(case).outcome == "accept" {
        let profile = module["semantic_profile"].clone();
        let parameters = module["semantic_parameters"].clone();
        let contract = &mut module["units"][0]["functions"][0]["contracts"];
        contract["semantic_profile"] = profile;
        contract["semantic_parameters"] = parameters;
        rehash(&mut module);
    }
    module
}

fn make_zero_result(module: &mut Value) {
    let function = &mut module["units"][0]["functions"][0];
    function["results"] = json!([]);
    function["blocks"][0]["terminator"] = json!({"kind":"Return","values":[]});
    function["contracts"]["ensures"] = json!([{"bool":true}]);
}

fn loop_module(language: SourceLanguage, partial_with_decreases: bool) -> Value {
    let vectors = vectors();
    let base_id = match language {
        SourceLanguage::Go => "module.valid_go_identity",
        SourceLanguage::Rust => "module.valid_rust_identity",
    };
    let mut module = base_module(&vectors, base_id);
    let function = &mut module["units"][0]["functions"][0];
    function["blocks"] = json!([
        {"label":"bb0","parameters":[],"instructions":[],
         "terminator":{"kind":"Jump","label":"bb1","args":[]}},
        {"label":"bb1","parameters":[],"instructions":[],
         "terminator":{"kind":"Branch","cond":{"bool":true},
          "then_label":"bb3","then_args":[],"else_label":"bb2","else_args":[]}},
        {"label":"bb2","parameters":[],"instructions":[],
         "terminator":{"kind":"Return","values":[{"var":"arg0"}]}},
        {"label":"bb3","parameters":[],"instructions":[],
         "terminator":{"kind":"Jump","label":"bb1","args":[]}}
    ]);
    function["features_used"] = json!(["branch", "cyclic_cfg"]);
    if language == SourceLanguage::Go {
        function["contracts"]["loops"] = json!([{
            "header":"bb1","invariants":[{"bool":true}],
            "decreases":[{"int":{"value":"1","width":8,"signed":false}}]
        }]);
        function["contracts"]["termination"] = json!(if partial_with_decreases {
            "partial"
        } else {
            "total"
        });
    } else {
        function["contracts"]["loops"] = json!([]);
    }
    module
}

fn module_for_invalid_profile_operation(case: &Value) -> Value {
    let vectors = vectors();
    let input = &case["input"];
    let mut module = base_module(&vectors, "module.valid_rust_identity");
    let (params, instruction, result_type) = match string(input, "instruction_kind") {
        "UnaryOp" => (
            json!([{"id":"arg0","type":input["type"]}]),
            json!({"id":"t0","kind":"UnaryOp","op":input["op"],"type":input["type"],
                   "value":{"var":"arg0"},"safety_checks":input["safety_checks"]}),
            input["type"].clone(),
        ),
        "Convert" => (
            json!([{"id":"arg0","type":input["source_type"]}]),
            json!({"id":"t0","kind":"Convert","type":input["type"],
                   "value":{"var":"arg0"},"safety_checks":input["safety_checks"]}),
            input["type"].clone(),
        ),
        "Index" => (
            json!([
                {"id":"arg0","type":input["base_type"]},
                {"id":"arg1","type":input["index_type"]}
            ]),
            json!({"id":"t0","kind":"Index","type":input["base_type"]["element"],
                   "base":{"var":"arg0"},"index":{"var":"arg1"},
                   "safety_checks":input["safety_checks"]}),
            input["base_type"]["element"].clone(),
        ),
        kind => panic!("unsupported invalid profile operation {kind}"),
    };
    let function = &mut module["units"][0]["functions"][0];
    function["params"] = params;
    function["results"] = json!([{"id":"result0","type":result_type}]);
    function["blocks"][0]["instructions"] = json!([instruction]);
    function["blocks"][0]["terminator"] = json!({"kind":"Return","values":[{"var":"t0"}]});
    function["contracts"]["ensures"] = json!([{"bool":true}]);
    function["features_used"] = match string(input, "instruction_kind") {
        "Convert" => json!(["conversion"]),
        "Index" => json!(["array"]),
        _ => json!([]),
    };
    module
}

fn block_parameter_order_module(vectors: &Value) -> Value {
    let mut module = base_module(vectors, "module.valid_go_identity");
    let params: Vec<_> = (0..12)
        .map(|index| {
            let r#type = if index % 2 == 0 {
                json!({"kind":"bv","width":8,"signed":true})
            } else {
                json!({"kind":"bool"})
            };
            json!({"id":format!("arg{index}"),"type":r#type})
        })
        .collect();
    let block_params: Vec<_> = params
        .iter()
        .enumerate()
        .map(|(index, binding)| json!({"id":format!("p{index}"),"type":binding["type"]}))
        .collect();
    let args: Vec<_> = (0..12)
        .map(|index| json!({"var":format!("arg{index}")}))
        .collect();
    let function = &mut module["units"][0]["functions"][0];
    function["params"] = json!(params);
    function["blocks"] = json!([
        {"label":"bb0","parameters":[],"instructions":[],
         "terminator":{"kind":"Jump","label":"bb1","args":args}},
        {"label":"bb1","parameters":block_params,"instructions":[],
         "terminator":{"kind":"Return","values":[{"var":"p0"}]}}
    ]);
    function["contracts"]["ensures"] = json!([{"bool":true}]);
    module
}

fn branching_struct_module(vectors: &Value) -> Value {
    let mut module = base_module(vectors, "module.valid_rust_identity");
    let mut declarations = Vec::new();
    for level in 0..16 {
        let name = format!("S{level:02}");
        let fields = if level == 0 {
            Vec::new()
        } else {
            (0..64)
                .map(|field| {
                    json!({
                        "name":format!("f{field}"),
                        "type":{"kind":"struct","id":format!("vector::S{:02}", level - 1)}
                    })
                })
                .collect()
        };
        declarations.push(json!({
            "id":format!("vector::{name}"),"name":name,"fields":fields
        }));
    }
    module["units"][0]["type_decls"] = json!(declarations);
    let root_type = json!({"kind":"struct","id":"vector::S15"});
    let function = &mut module["units"][0]["functions"][0];
    function["params"][0]["type"] = root_type.clone();
    function["results"][0]["type"] = root_type;
    function["features_used"] = json!(["struct"]);
    module
}

fn rehash(module: &mut Value) {
    let functions = module["units"][0]["functions"]
        .as_array_mut()
        .expect("functions");
    for function in functions {
        rehash_contract(&mut function["contracts"]);
    }
    let typed: VirModule = serde_json::from_value(module.clone()).expect("module decodes for hash");
    module["vir_hash"] = json!(vir_hash(&typed).expect("VIR hashes").as_str());
}

fn rehash_contract(contract: &mut Value) {
    let typed = serde_json::from_value(contract.clone()).expect("contract decodes for hash");
    contract["contract_hash"] = json!(contract_hash(&typed).expect("contract hashes").as_str());
}
