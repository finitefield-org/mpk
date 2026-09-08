//! T05 closure gate. Reuse the complete W01–W05 suites and actual fixture builders.
// Each predecessor suite remains independently runnable with its own fixtures.
#![allow(clippy::duplicate_mod)]
#[path = "csharp_practical_boundary.rs"]
mod boundary;
#[allow(dead_code)]
#[path = "support/csharp_practical_data_context.rs"]
mod support;
#[path = "csharp_practical_transition.rs"]
mod transition;
use mpk_vc::csharp_practical_source_artifacts::*;
use mpk_vc::csharp_practical_vir_model::*;
use mpk_vc::csharp_practical_vir_validation as v;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};
use PracticalJsonValue as J;
fn bundle() -> ValidatedFoundationBundle {
    validate_registered_foundation_bundle(
        registered_foundation_descriptor_transport(),
        registered_foundation_definitions_transport(),
    )
    .unwrap()
}
fn ty(t: &str) -> String {
    format!("mpk.csharp.value.{t}.v1")
}
fn decl(kind: &str, name: &str, owner: &str, params: &[String], result: &str) -> String {
    csharp_practical_declaration_id(&json!({"kind":kind,"namespace":"Business","owner":owner,"name":name,"parameter_type_ids":params,"result_type_id":result})).unwrap()
}
fn src(name: &str) -> String {
    decl("type", name, "", &[], "")
}
fn j(s: &str) -> J {
    J::string(s)
}
fn obj(f: Vec<(&str, J)>) -> J {
    J::object(f)
}
fn set(v: &mut J, k: &str, n: J) {
    let J::Object(f) = v else { panic!() };
    f.iter_mut().find(|(key, _)| key == k).unwrap().1 = n;
}
fn parse(bytes: &[u8]) -> J {
    serde_json::from_slice(bytes).unwrap()
}
fn hashed(domain: &str, mut value: J) -> Vec<u8> {
    let J::Object(f) = &mut value else { panic!() };
    f.retain(|(k, _)| k != "contract_sha256");
    let mut h = Sha256::new();
    h.update(domain);
    h.update([0]);
    h.update(canonical_practical_json_bytes(&value).unwrap());
    let J::Object(f) = &mut value else { panic!() };
    f.push(("contract_sha256".into(), j(&format!("{:x}", h.finalize()))));
    canonical_practical_json_bytes(&value).unwrap()
}
fn field(name: &str, t: &str) -> J {
    obj(vec![
        ("field_id", j(name)),
        ("json_name", j(name)),
        ("type_id", j(t)),
        ("required", J::Bool(true)),
        ("nullable", J::Bool(false)),
        ("missing_rule", obj(vec![("mode", j("reject"))])),
        ("codec_id", J::Null),
        ("codec_parameters", J::Null),
    ])
}
fn boundary_contract(
    ctx: &PracticalArtifactContext,
    root: &str,
    inputs: Vec<J>,
    result: &str,
) -> Vec<u8> {
    hashed(
        "MPK-CSHARP-BOUNDARY-CONTRACT-1.0",
        obj(vec![
            ("schema", j(BOUNDARY_CONTRACT_SCHEMA)),
            ("semantic_context", ctx.semantic_context().clone()),
            ("compilation_id", j(ctx.compilation_id())),
            ("boundary_id", j("boundary.apply")),
            ("selected_callable_id", j(root)),
            ("input_fields", J::Array(inputs)),
            ("output_fields", J::Array(vec![field("result", result)])),
            ("canonical_json_profile", j("mpk.csharp.canonical_json.v1")),
            ("parse_format_profile", j("mpk.csharp.parse_format.v1")),
            (
                "evidence_linkage",
                obj(vec![
                    ("raw_input_domain", j("MPK-CSHARP-BOUNDARY-INPUT-1.0")),
                    (
                        "canonical_value_domain",
                        j("MPK-CSHARP-CANONICAL-VALUE-1.0"),
                    ),
                    (
                        "canonical_output_domain",
                        j("MPK-CSHARP-BOUNDARY-OUTPUT-1.0"),
                    ),
                    ("reparse_equality", j("typed_field_complete")),
                ]),
            ),
        ]),
    )
}
fn codec_claim(b: &ValidatedFoundationBundle) -> J {
    let result=csharp_practical_closed_instance_id(b,&json!({"kind":"instance","template":"result","arguments":[{"kind":"primitive","id":"i32"},{"kind":"primitive","id":"parse_error"}]})).unwrap();
    obj(vec![
        ("tag", j("tagged_is")),
        ("type_id", j(&ty("bool"))),
        (
            "value",
            obj(vec![
                ("tag", j("codec_parse")),
                ("type_id", j(&result)),
                ("codec_id", j("integer.i32")),
                (
                    "codec_parameters",
                    obj(vec![("scale", J::Null), ("rounding", J::Null)]),
                ),
                (
                    "text",
                    obj(vec![
                        ("tag", j("literal")),
                        ("type_id", j(&ty("string"))),
                        ("value", j("42")),
                    ]),
                ),
            ]),
        ),
        ("arm", j("ok")),
    ])
}
fn setup_cross(
    b: &ValidatedFoundationBundle,
    case: &str,
) -> (PracticalArtifactContext, CapturedInputSet) {
    let mut source =
        include_str!("../../../develop/migrations/csharp-03/transition/Entry.cs").to_owned();
    match case {
        "partial_loop" => source = source.replace("        if (state.Version != command.Expected)", "        for (int i = 0; i < 1; i++) { }\n        if (state.Version != command.Expected)"),
        "mpk" => source = format!("using Mpk;\n{source}"),
        "effect" => source = source.replace("context.Effective", "System.DateTime.Now.Ticks"),
        "serializer" => {
            source = source.replace(
                "context.Effective",
                "(long)System.Text.Json.JsonSerializer.Serialize(command).Length",
            )
        }
        "async" => {
            source=source.replace("public static class Entry {","public static class Entry { public static async System.Threading.Tasks.Task<long> Time(long x) { await System.Threading.Tasks.Task.Delay(0); return x; }");
            source = source.replace("context.Effective", "Time(context.Effective).Result");
        }
        "iterator" => {
            source=source.replace("public static class Entry {","public static class Entry { public static System.Collections.Generic.IEnumerable<long> Times(long x) { yield return x; } public static long Time(long x) { foreach(long v in Times(x)) return v; return x; }");
            source = source.replace("context.Effective", "Time(context.Effective)");
        }
        _ => {}
    }
    transition::setup_variant(
        b,
        "w06",
        source,
        vec!["contracts/zzboundary.json".into()],
        |ctx, _, mut rows| {
            let mut method = parse(&rows[1]);
            let apply = method
                .get("callable_id")
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned();
            set(&mut method, "requires", J::Array(vec![codec_claim(b)]));
            if matches!(case, "partial_root" | "partial_loop") {
                set(&mut method, "termination", j("partial"));
            }
            if case == "partial_loop" {
                set(
                    &mut method,
                    "loops",
                    J::Array(vec![obj(vec![
                        ("loop_id", j(&format!("{apply}#loop#0000"))),
                        (
                            "invariants",
                            J::Array(vec![obj(vec![
                                ("tag", j("literal")),
                                ("type_id", j(&ty("bool"))),
                                ("value", J::Bool(true)),
                            ])]),
                        ),
                        ("modifies", J::Array(vec![j("local:1")])),
                        ("decreases", J::Array(vec![])),
                    ])]),
                );
            }
            rows[1] = hashed("MPK-CSHARP-METHOD-CONTRACT-1.0", method);
            let mut contract = parse(&rows[2]);
            let J::Array(mut commands) = contract.get("accepted_commands").unwrap().clone() else {
                panic!()
            };
            set(&mut commands[0], "condition", codec_claim(b));
            set(&mut contract, "accepted_commands", J::Array(commands));
            if case == "binding" {
                set(&mut contract, "transition_binding_id", j("binding.invalid"));
            }
            rows[2] = hashed("MPK-CSHARP-TRANSITION-CONTRACT-1.0", contract);
            let input = vec![
                field("state", &src("State")),
                field(
                    "command",
                    &src(if case == "boundary_type" {
                        "Context"
                    } else {
                        "Command"
                    }),
                ),
                field("context", &src("Context")),
            ];
            rows.push(boundary_contract(ctx, &apply, input, &src("ApplyResult")));
            rows
        },
    )
}
fn setup_totality(
    b: &ValidatedFoundationBundle,
    case: &str,
) -> (PracticalArtifactContext, CapturedInputSet) {
    let source="namespace Business; public readonly struct Value { public readonly int N; public Value(int n) { N=n; } public int Number { get { return N; } } } public static class Entry { public static int Echo(int n) { return new Value(n).Number; } public static int Run(int n) { return Echo(n); } }\n";
    let run = decl("method", "Run", &src("Entry"), &[ty("i32")], &ty("i32"));
    let helper = match case {
        "partial_constructor" => decl(
            "constructor",
            "Value",
            &src("Value"),
            &[ty("i32")],
            &src("Value"),
        ),
        "partial_getter" => decl("method", "get_Number", &src("Value"), &[], &ty("i32")),
        _ => decl("method", "Echo", &src("Entry"), &[ty("i32")], &ty("i32")),
    };
    support::context_with_sidecars(
        b,
        &run,
        source.as_bytes(),
        vec![
            "contracts/boundary.json".into(),
            "contracts/helper.json".into(),
            "contracts/method.json".into(),
        ],
        |ctx| {
            let method = |id: &str, mode: &str| {
                hashed(
                    "MPK-CSHARP-METHOD-CONTRACT-1.0",
                    obj(vec![
                        ("schema", j(METHOD_CONTRACT_SCHEMA)),
                        ("semantic_context", ctx.semantic_context().clone()),
                        ("compilation_id", j(ctx.compilation_id())),
                        ("callable_id", j(id)),
                        (
                            "source_content_sha256",
                            j(&format!("{:x}", Sha256::digest(source.as_bytes()))),
                        ),
                        ("termination", j(mode)),
                        ("requires", J::Array(vec![])),
                        ("ensures", J::Array(vec![])),
                        ("exceptional_cases", J::Array(vec![])),
                        ("modifies", J::Array(vec![])),
                        ("loops", J::Array(vec![])),
                    ]),
                )
            };
            vec![
                boundary_contract(ctx, &run, vec![field("value", &ty("i32"))], &ty("i32")),
                method(&helper, if case == "total" { "total" } else { "partial" }),
                method(&run, "total"),
            ]
        },
    )
}
fn path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/boundary-transition")
        .join(name)
}
fn request(case: &str, ctx: &PracticalArtifactContext, c: &CapturedInputSet) -> Value {
    json!({"case":case,"id":c.snapshot_sha256(),"compilation_id":ctx.compilation_id(),"roots":ctx.selected_root_ids(),"inputs":c.entries().iter().map(|e|json!({"kind":if e.kind()==OriginalInputKind::Source{"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()})
}
#[test]
fn csharp_03_t05_w06_cumulative_emission_and_barriers() {
    let b = bundle();
    let generation = std::env::var("MPK_W06_REQUESTS_OUT").ok();
    let responses: Vec<Value> = if generation.is_some() {
        vec![]
    } else {
        serde_json::from_slice(&fs::read(path("control-responses.json")).unwrap()).unwrap()
    };
    let mut requests = vec![];
    for case in [
        "valid",
        "partial_root",
        "partial_loop",
        "binding",
        "boundary_type",
        "mpk",
        "effect",
        "serializer",
        "async",
        "iterator",
    ] {
        let (ctx, c) = setup_cross(&b, case);
        requests.push(request(case, &ctx, &c));
        if generation.is_some() {
            continue;
        }
        let row = responses
            .iter()
            .find(|r| r["id"] == c.snapshot_sha256())
            .unwrap_or_else(|| panic!("missing {case}"));
        if ["mpk", "effect", "serializer", "async", "iterator"].contains(&case) {
            assert!(row.get("facts").is_none(), "{case}");
            assert_eq!(row["artifact_count"], 0);
            assert_eq!(
                row["reject"],
                match case {
                    "mpk" => "CSHARP_PRACTICAL_DEPENDENCY/mpk_namespace",
                    "effect" => "CSHARP_PRACTICAL_EFFECT/external_effect_or_concurrency",
                    "serializer" => "CSHARP_PRACTICAL_GENERIC/generic_method",
                    "async" => "CSHARP_PRACTICAL_DECLARATION/data_modifier",
                    "iterator" => "CSHARP_PRACTICAL_GENERIC/constructed_type",
                    _ => unreachable!(),
                }
            );
            continue;
        }
        let source = ValidatedDataSource::import_captured_facts(
            &b,
            &ctx,
            &c,
            &serde_json::to_vec(&row["facts"]).unwrap(),
        )
        .unwrap_or_else(|e| panic!("{case}: {e:?}"));
        let emitted = emit_data_phase(&b, &ctx, &c, &source);
        if case != "valid" {
            assert!(
                matches!(
                    emitted.as_ref().err(),
                    Some(DataPhaseError::Boundary(_) | DataPhaseError::Transition(_))
                ),
                "{case}: {:?}",
                emitted.as_ref().err()
            );
            continue;
        }
        let e = emitted.unwrap();
        assert_eq!(e.boundaries().len(), 1);
        assert_eq!(e.transitions().len(), 1);
        verify_closure_and_import(&b, &ctx, &c, &source, &e);
        verify_evidence_and_fuzz(&b, &ctx, &c, &e);
    }
    if let Some(file) = generation {
        fs::write(file, serde_json::to_vec(&requests).unwrap()).unwrap();
    }
}
#[test]
fn csharp_03_t05_w06_data_route_totality() {
    let b = bundle();
    let generation = std::env::var("MPK_W06_DATA_REQUESTS_OUT").ok();
    let mut requests = vec![];
    let responses: Vec<Value> = if generation.is_some() {
        vec![]
    } else {
        serde_json::from_slice(&fs::read(path("data-responses.json")).unwrap()).unwrap()
    };
    for case in [
        "total",
        "partial_callee",
        "partial_constructor",
        "partial_getter",
    ] {
        let (ctx, c) = setup_totality(&b, case);
        requests.push(request(case, &ctx, &c));
        if generation.is_some() {
            continue;
        }
        let row = responses
            .iter()
            .find(|r| r["id"] == c.snapshot_sha256())
            .unwrap_or_else(|| panic!("missing {case}"));
        let source = ValidatedDataSource::import_captured_facts(
            &b,
            &ctx,
            &c,
            &serde_json::to_vec(&row["facts"]).unwrap(),
        )
        .unwrap_or_else(|e| panic!("{case}: {e:?}"));
        assert!(source.control_lowering().is_none());
        let sidecars = DataSidecars::capture(&ctx, &c).unwrap();
        for method in sidecars
            .contracts()
            .iter()
            .filter(|s| s.schema() == METHOD_CONTRACT_SCHEMA)
        {
            let id = method.value().get("callable_id").unwrap().as_str().unwrap();
            assert!(
                source.callables().iter().any(|s| s.id() == id),
                "{case}: unknown callable {id}"
            );
        }
        let e = emit_data_phase(&b, &ctx, &c, &source);
        if case == "total" {
            e.unwrap();
        } else {
            assert_eq!(e.as_ref().err(), Some(&DataPhaseError::Contract), "{case}");
        }
    }
    if let Some(file) = generation {
        fs::write(file, serde_json::to_vec(&requests).unwrap()).unwrap();
    }
}
fn verify_closure_and_import(
    b: &ValidatedFoundationBundle,
    ctx: &PracticalArtifactContext,
    c: &CapturedInputSet,
    source: &ValidatedDataSource,
    e: &EmittedDataPhase,
) {
    let roots: Value = serde_json::from_slice(e.closure().roots().canonical_json()).unwrap();
    let codec_roots = roots["roots"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["origin"] == "codec_result")
        .collect::<Vec<_>>();
    assert_eq!(codec_roots.len(), 2);
    assert_ne!(
        codec_roots[0]["provenance_id"],
        codec_roots[1]["provenance_id"]
    );
    let closed = derive_closed_instances(b, e.closure().roots()).unwrap();
    assert_eq!(
        closed.canonical_json(),
        e.closure().closed().canonical_json()
    );
    assert_eq!(closed.counters(), e.closure().closed().counters());
    assert_eq!(
        closed.counters(),
        &json!({"declarations":9,"operations":54,"recipe_nodes":554})
    );
    assert_eq!(e.closure().roots().root_count(), 33);
    let input = v::PracticalVirImportContext {
        data_source_facts: Some(source.captured_facts()),
        artifact_context: ctx,
        captured_inputs: c,
        foundation_descriptor_transport: registered_foundation_descriptor_transport(),
        foundation_definitions_transport: registered_foundation_definitions_transport(),
        closed_roots_transport: e.closure().roots().canonical_json(),
        closed_instances_transport: e.closure().closed().canonical_json(),
        semantic_bindings_transport: e.closure().bindings().canonical_bytes(),
        required_checks_transport: e.operations().required_checks().canonical_bytes(),
        operations_transport: e.operations().operations().canonical_bytes(),
    };
    v::import_csharp_practical_vir_json(e.vir().canonical_bytes(), input).unwrap();
    let valid = v::PracticalVirContents {
        functions: e.vir().functions().to_vec(),
        binding_projections: e.vir().binding_projections().to_vec(),
        binding_commutations: e.vir().binding_commutations().to_vec(),
        source_exceptions: e.vir().source_exceptions().to_vec(),
        source_obligations: e.vir().source_obligations().to_vec(),
        data_contracts: e.vir().data_contracts().to_vec(),
    };
    for n in 0..5 {
        let mut bad = valid.clone();
        match n {
            0 => bad
                .data_contracts
                .retain(|s| !s.contains(BOUNDARY_CONTRACT_SCHEMA)),
            1 => bad
                .data_contracts
                .retain(|s| !s.contains(TRANSITION_CONTRACT_SCHEMA)),
            2 => bad.binding_projections.clear(),
            3 => bad.source_obligations.clear(),
            _ => bad.functions[0].id = "T<unexpanded>".into(),
        };
        if let Ok(bytes) = v::canonical_csharp_practical_vir_transport(input, bad) {
            assert!(
                v::import_csharp_practical_vir_json(&bytes, input).is_err(),
                "mutation {n}"
            );
        }
    }
    let mut fewer = roots.clone();
    fewer["roots"]
        .as_array_mut()
        .unwrap()
        .retain(|r| r["origin"] != "codec_result");
    let bytes =
        canonical_closed_root_set_transport(b, &fewer["roots"], &fewer["source_types"]).unwrap();
    assert!(v::import_csharp_practical_vir_json(
        e.vir().canonical_bytes(),
        v::PracticalVirImportContext {
            closed_roots_transport: &bytes,
            ..input
        }
    )
    .is_err());
    let again = emit_data_phase(b, ctx, c, source).unwrap();
    assert_eq!(
        again.manifest().canonical_bytes(),
        e.manifest().canonical_bytes()
    );
    assert_eq!(
        again.source_map().canonical_bytes(),
        e.source_map().canonical_bytes()
    );
    assert_eq!(again.vir().canonical_bytes(), e.vir().canonical_bytes());
    // Bounded source-protocol bit mutations must fail original-capture lineage.
    for n in 0..16 {
        let mut facts: Value = serde_json::from_slice(source.captured_facts()).unwrap();
        facts["compilation_id"] = json!(format!("mutated.{n}"));
        assert!(ValidatedDataSource::import_captured_facts(
            b,
            ctx,
            c,
            &serde_json::to_vec(&facts).unwrap()
        )
        .is_err());
    }
}
fn signed(token: &str, n: i32) -> MonomorphicValue {
    MonomorphicValue::Signed {
        type_id: ty(token),
        value: n.to_string(),
    }
}
fn product(
    b: &ValidatedFoundationBundle,
    e: &EmittedDataPhase,
    name: &str,
    values: Vec<MonomorphicValue>,
) -> MonomorphicValue {
    let id = src(name);
    let roots: Value = serde_json::from_slice(e.closure().roots().canonical_json()).unwrap();
    let members = roots["source_types"][&id]["members"].as_array().unwrap();
    assert_eq!(members.len(), values.len());
    let v = MonomorphicValue::Product {
        type_id: id,
        fields: members
            .iter()
            .zip(values)
            .map(|(m, v)| NamedMonomorphicValue {
                name: m["name"].as_str().unwrap().into(),
                value: Box::new(v),
            })
            .collect(),
    };
    validate_monomorphic_value(b, e.closure().roots(), e.closure().closed(), &v).unwrap();
    v
}
fn verify_evidence_and_fuzz(
    b: &ValidatedFoundationBundle,
    ctx: &PracticalArtifactContext,
    c: &CapturedInputSet,
    e: &EmittedDataPhase,
) {
    let doc=br#"{"state":{"Version":"3","Balance":10},"command":{"Expected":"3","Amount":4},"context":{"Effective":"123"}}"#;
    fn input_bytes<'a>(document: &'a [u8], raw: &'a [u8]) -> BoundaryInputBytes<'a> {
        BoundaryInputBytes {
            boundary_id: "boundary.apply",
            provenance_id: "adapter.request",
            raw_bytes: raw,
            canonical_document: document,
        }
    }
    let input = e
        .capture_boundary_input(b, ctx, c, input_bytes(doc, doc))
        .unwrap();
    let state = product(
        b,
        e,
        "State",
        vec![
            MonomorphicValue::Unsigned {
                type_id: ty("u64"),
                value: "4".into(),
            },
            signed("i32", 14),
        ],
    );
    let event = |n| product(b, e, "Event", vec![signed("i32", n), signed("i64", 123)]);
    let events_id=csharp_practical_closed_instance_id(b,&json!({"kind":"instance","template":"bounded_sequence","arguments":[{"kind":"source","id":src("Event") }]})).unwrap();
    let events = MonomorphicValue::Sequence {
        type_id: events_id,
        elements: vec![event(4), event(0)],
    };
    let response = product(b, e, "Response", vec![signed("i32", 14)]);
    let change = product(b, e, "Change", vec![state, events, response]);
    let tag = |name, carrier| MonomorphicValue::Enum {
        type_id: src(name),
        underlying: "i32".into(),
        carrier: String::from(carrier),
    };
    let returned = product(
        b,
        e,
        "ApplyResult",
        vec![tag("Tag", "0"), change, tag("DomainError", "0")],
    );
    let output = e
        .capture_boundary_output(b, ctx, c, &input, &returned)
        .unwrap();
    assert_eq!(output.reparsed_value(), &returned);
    let evidence = || BoundaryOutputEvidence {
        canonical_document: output.capture().canonical_document(),
        capture: output.capture().artifact().canonical_bytes(),
        manifest: output.manifest().canonical_bytes(),
        artifacts: output.artifacts().canonical_bytes(),
    };
    e.import_boundary_output_run(b, ctx, c, &input, &returned, evidence())
        .unwrap();
    for manifest in [input.manifest(), output.manifest()] {
        for key in [
            "boundary_contracts",
            "transition_contracts",
            "boundary_inputs",
        ] {
            assert_eq!(
                manifest.value().get(key).unwrap().as_array().unwrap().len(),
                1
            );
        }
        assert_eq!(
            manifest.value().get("source_map"),
            e.manifest().value().get("source_map")
        );
    }
    assert_eq!(
        output
            .manifest()
            .value()
            .get("boundary_outputs")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    // Deterministic bounded fuzz seeds target JSON syntax, UTF-8 and exact bytes.
    for n in 0..32 {
        let mut bad = doc.to_vec();
        let index = n * 3 % bad.len();
        bad[index] = 0xff;
        assert!(e
            .capture_boundary_input(b, ctx, c, input_bytes(&bad, doc))
            .is_err());
        let mut bad_output = output.capture().canonical_document().to_vec();
        let index = n * 5 % bad_output.len();
        bad_output[index] ^= 0x80;
        let mut ev = evidence();
        ev.canonical_document = &bad_output;
        assert!(e
            .import_boundary_output_run(b, ctx, c, &input, &returned, ev)
            .is_err());
    }
    let sidecar = c.entry("contracts/zzboundary.json").unwrap();
    for n in 0..32 {
        let mut bytes = sidecar.bytes().to_vec();
        let index = n * 17 % bytes.len();
        bytes[index] = 0xff;
        let mutated = capture_original_inputs(
            ctx,
            c.entries()
                .iter()
                .map(|i| OriginalInput {
                    kind: i.kind(),
                    path: i.path().into(),
                    bytes: if i.path() == sidecar.path() {
                        bytes.clone()
                    } else {
                        i.bytes().to_vec()
                    },
                })
                .collect(),
        )
        .unwrap();
        assert!(DataSidecars::capture(ctx, &mutated).is_err());
    }
    if let Ok(file) = std::env::var("MPK_W06_EVIDENCE_OUT") {
        fs::write(file,serde_json::to_vec_pretty(&json!({"closed_counters":e.closure().closed().counters(),"root_count":e.closure().roots().root_count(),"source_type_count":e.closure().roots().source_type_count(),"source_map":serde_json::from_slice::<Value>(e.source_map().canonical_bytes()).unwrap(),"manifest":serde_json::from_slice::<Value>(output.manifest().canonical_bytes()).unwrap(),"input":serde_json::from_slice::<Value>(input.capture().artifact().canonical_bytes()).unwrap(),"output":serde_json::from_slice::<Value>(output.capture().artifact().canonical_bytes()).unwrap(),"fuzz_seeds":112})).unwrap()).unwrap();
    }
}
