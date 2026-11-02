use mpk_vc::go_practical_context::{
    GoPrivateCandidateRegistry, GoPrivateCandidateSelection, GoPrivateContextError,
};
use mpk_vc::{
    hash_canonical_json, parse_strict_json, HashDomain, StrictJsonLimits, StrictJsonValue,
};
use serde_json::{json, Value};

fn strict(value: &Value) -> StrictJsonValue {
    parse_strict_json(
        &serde_json::to_vec(value).unwrap(),
        StrictJsonLimits::new(4_194_304, 1_000_000, 64, 1_000_000),
    )
    .unwrap()
}

fn registry_value(registry: &GoPrivateCandidateRegistry) -> Value {
    serde_json::from_slice(registry.canonical_bytes()).unwrap()
}

fn seal(value: &mut Value, field: &str, domain: &'static str) {
    value.as_object_mut().unwrap().remove(field);
    let digest = hash_canonical_json(HashDomain::new(domain), &strict(value))
        .unwrap()
        .to_hex();
    value[field] = json!(digest);
}

fn context(registry: &Value, profile: &str, parameters: Value) -> Value {
    let entry = registry["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["semantic_profile"] == profile)
        .unwrap();
    json!({
        "schema":"mpk.semantic_context.v3",
        "profile_registry":{
            "schema":registry["schema"], "id":registry["id"],
            "revision":registry["revision"], "registry_sha256":registry["registry_sha256"],
        },
        "profile_entry_sha256":entry["entry_sha256"],
        "source_language":entry["source_language"],
        "semantic_profile":profile,
        "semantic_parameters":{"schema":entry["semantic_parameters_schema"], "value":parameters},
        "foundation_descriptor":entry["foundation_descriptor"],
    })
}

fn request(context: Value, selection: Value) -> Value {
    let mut request = json!({
        "schema":"mpk.validated_semantic_request.v3",
        "semantic_context":context,
        "selection":selection,
    });
    seal(
        &mut request,
        "request_sha256",
        "MPK-VALIDATED-SEMANTIC-REQUEST-3.0",
    );
    request
}

fn go_parameters() -> Value {
    json!({
        "target_id":"linux/amd64", "pointer_width":64, "goos":"linux", "goarch":"amd64",
        "goamd64":"v1", "compiler_release":"go1.25.0",
        "effective_languages":["go1.22","go1.23","go1.24","go1.25"],
        "limits_id":"mpk.go.practical.limits.v1",
        "limits_sha256":"49ab79bb15643fa34b5b0d9b15ce4c833d098e38b70e89f0af3206bdac570551",
    })
}

fn go_selection() -> Value {
    json!({"schema":"mpk.selection.go_functions.v1", "value":{"roots":[
        {"package":"example.test/quote", "function":"Quote", "receiver":null}
    ]}})
}

fn contexts(registry: &Value) -> Vec<(&'static str, Value, Value)> {
    vec![
        (
            "go_fixed",
            context(
                registry,
                "mpk.go.fixed.v0",
                json!({"target_id":"linux/amd64", "pointer_width":64}),
            ),
            json!({"schema":"mpk.selection.go_function.v0", "value":{"package":"quote", "function":"quote.Quote"}}),
        ),
        (
            "go_practical",
            context(registry, "mpk.go.practical.v1", go_parameters()),
            go_selection(),
        ),
        (
            "rust32",
            context(
                registry,
                "mpk.rust.checked.v0",
                json!({"target_id":"i686-unknown-linux-gnu", "pointer_width":32, "overflow_mode":"checked", "panic_mode":"abort"}),
            ),
            json!({"schema":"mpk.selection.rust_function.v0", "value":{"package":"quote", "crate":"quote", "kind":"bin", "function":"quote::quote"}}),
        ),
        (
            "rust64",
            context(
                registry,
                "mpk.rust.checked.v0",
                json!({"target_id":"x86_64-unknown-linux-gnu", "pointer_width":64, "overflow_mode":"checked", "panic_mode":"abort"}),
            ),
            json!({"schema":"mpk.selection.rust_function.v0", "value":{"package":"quote", "crate":"quote", "kind":"lib", "function":"quote::quote"}}),
        ),
    ]
}

#[test]
fn candidate_registry_and_four_contexts_keep_actual_retained_policy() {
    let registry = GoPrivateCandidateRegistry::new().unwrap();
    let wire = registry_value(&registry);
    assert_eq!(wire["profiles"].as_array().unwrap().len(), 3);
    assert_eq!(
        wire["registry_sha256"],
        "6c300215b21893014f393940416cebc7aa83ac74443918db5fa33a7c7615a7bc"
    );
    registry.import_registry(&strict(&wire)).unwrap();
    let mut checked = Vec::new();
    for (name, candidate, selection) in contexts(&wire) {
        let context = registry.import_context(&strict(&candidate)).unwrap();
        let packet = request(candidate, selection);
        let imported = registry.import_request(&strict(&packet)).unwrap();
        assert_eq!(imported.context(), &context, "{name}");
        assert_eq!(
            imported.request_sha256(),
            packet["request_sha256"].as_str().unwrap()
        );
        if name == "go_practical" {
            assert!(matches!(
                imported.selection(),
                GoPrivateCandidateSelection::Practical(_)
            ));
            // Independent frozen Python canonical/hash observer, after the
            // Go/Rust-only scope amendment and the candidate descriptor link.
            assert_eq!(
                imported.request_sha256(),
                "03d66cea6a8f161a4d528b6a72183ff771aec76d3054762fa939accd3eb3c8ea"
            );
        } else {
            assert!(matches!(
                imported.selection(),
                GoPrivateCandidateSelection::Retained(_)
            ));
        }
        context.require_same(&context).unwrap();
        checked.push((name, context));
    }
    assert_eq!(checked.len(), 4);
    let rust32 = &checked
        .iter()
        .find(|(name, _)| *name == "rust32")
        .unwrap()
        .1;
    let rust64 = &checked
        .iter()
        .find(|(name, _)| *name == "rust64")
        .unwrap()
        .1;
    assert_eq!(
        rust32.require_same(rust64),
        Err(GoPrivateContextError::Linkage)
    );
}

#[test]
fn rehashed_caller_registry_and_context_mutations_cannot_grant_authority() {
    let registry = GoPrivateCandidateRegistry::new().unwrap();
    let original = registry_value(&registry);
    let mut changed = original.clone();
    changed["profiles"][0]["contracts"]["vir"] = json!("mpk.profile.vir.caller.v2");
    for entry in changed["profiles"].as_array_mut().unwrap() {
        seal(entry, "entry_sha256", "MPK-SEMANTIC-PROFILE-ENTRY-3.0");
    }
    seal(
        &mut changed,
        "registry_sha256",
        "MPK-SEMANTIC-PROFILE-REGISTRY-3.0",
    );
    assert_eq!(
        registry.import_registry(&strict(&changed)),
        Err(GoPrivateContextError::Registry)
    );
    let original_context = context(&original, "mpk.go.practical.v1", go_parameters());
    let changes = [
        (
            "entry",
            vec!["profile_entry_sha256"],
            json!(original["profiles"][0]["entry_sha256"]),
        ),
        ("language", vec!["source_language"], json!("other")),
        ("foundation", vec!["foundation_descriptor"], Value::Null),
        (
            "registry revision",
            vec!["profile_registry", "revision"],
            json!(true),
        ),
        (
            "target width",
            vec!["semantic_parameters", "value", "pointer_width"],
            json!(32),
        ),
        (
            "toolchain",
            vec!["semantic_parameters", "value", "compiler_release"],
            json!("go1.25.1"),
        ),
        (
            "language order",
            vec!["semantic_parameters", "value", "effective_languages"],
            json!(["go1.25", "go1.24", "go1.23", "go1.22"]),
        ),
        (
            "limits",
            vec!["semantic_parameters", "value", "limits_sha256"],
            json!("0".repeat(64)),
        ),
    ];
    for (name, path, replacement) in changes {
        let mut changed = original_context.clone();
        let mut cursor = &mut changed;
        for key in &path[..path.len() - 1] {
            cursor = &mut cursor[*key];
        }
        cursor[path[path.len() - 1]] = replacement;
        assert!(
            registry.import_context(&strict(&changed)).is_err(),
            "{name}"
        );
    }
    for field in [
        "schema",
        "profile_registry",
        "profile_entry_sha256",
        "source_language",
        "semantic_profile",
        "semantic_parameters",
        "foundation_descriptor",
    ] {
        let mut missing = original_context.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(
            registry.import_context(&strict(&missing)).is_err(),
            "missing {field}"
        );
    }
}

#[test]
fn retained_parameter_and_selection_policies_are_not_replaced_by_the_candidate() {
    let registry = GoPrivateCandidateRegistry::new().unwrap();
    let wire = registry_value(&registry);
    for (name, mut candidate, mut selection) in contexts(&wire) {
        if name == "go_practical" {
            continue;
        }
        if name == "go_fixed" {
            candidate["semantic_parameters"]["value"]["pointer_width"] = json!(32);
            selection["value"]["function"] = json!("foreign.Quote");
        } else {
            candidate["semantic_parameters"]["value"]["overflow_mode"] = json!("wrapping");
            selection["value"]["kind"] = json!("test");
        }
        assert!(
            registry.import_context(&strict(&candidate)).is_err(),
            "{name} altered parameters"
        );
        let original = contexts(&wire)
            .into_iter()
            .find(|(n, _, _)| *n == name)
            .unwrap()
            .1;
        assert!(
            registry
                .import_request(&strict(&request(original, selection)))
                .is_err(),
            "{name} altered selection"
        );
    }
}

#[test]
fn request_hash_covers_full_context_and_selection_and_rejects_duplicate_json() {
    let registry = GoPrivateCandidateRegistry::new().unwrap();
    let candidate = context(
        &registry_value(&registry),
        "mpk.go.practical.v1",
        go_parameters(),
    );
    let mut packet = request(candidate, go_selection());
    packet["selection"]["value"]["roots"][0]["function"] = json!("Other");
    assert_eq!(
        registry.import_request(&strict(&packet)),
        Err(GoPrivateContextError::RequestHash)
    );
    seal(
        &mut packet,
        "request_sha256",
        "MPK-VALIDATED-SEMANTIC-REQUEST-3.0",
    );
    packet["selection"]["value"]["roots"][0]["receiver"] =
        json!({"package":"other", "type_name":"Ledger"});
    seal(
        &mut packet,
        "request_sha256",
        "MPK-VALIDATED-SEMANTIC-REQUEST-3.0",
    );
    assert_eq!(
        registry.import_request(&strict(&packet)),
        Err(GoPrivateContextError::Selection)
    );
    let duplicate = StrictJsonValue::Object(vec![
        ("schema".into(), StrictJsonValue::Null),
        ("schema".into(), StrictJsonValue::Null),
    ]);
    assert_eq!(
        registry.import_context(&duplicate),
        Err(GoPrivateContextError::Json)
    );
}

#[test]
fn imports_the_exact_request_emitted_by_the_go_candidate_producer() {
    let raw = include_bytes!("fixtures/go-practical-candidate-request.json");
    let parsed = parse_strict_json(
        raw,
        StrictJsonLimits::new(4_194_304, 1_000_000, 64, 1_000_000),
    )
    .unwrap();
    let registry = GoPrivateCandidateRegistry::new().unwrap();
    let request = registry.import_request(&parsed).unwrap();
    assert_eq!(request.canonical_bytes(), raw.strip_suffix(b"\n").unwrap());
    assert_eq!(
        request.request_sha256(),
        "03d66cea6a8f161a4d528b6a72183ff771aec76d3054762fa939accd3eb3c8ea"
    );
    assert!(matches!(
        request.selection(),
        GoPrivateCandidateSelection::Practical(_)
    ));
}
