//! CSHARP-03-T05-W03: byte-complete output and cumulative reproduction evidence.
use super::*;
use PracticalJsonValue as J;
fn path(file: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/boundary-output")
        .join(file)
}
fn output_setup(
    b: &ValidatedFoundationBundle,
    case: &str,
) -> (PracticalArtifactContext, CapturedInputSet) {
    let (src, result) = if case == "unit" {
        (
            "namespace Boundary;public static class Entry{public static void Run(){}}\n".into(),
            ty("unit"),
        )
    } else {
        let pairs: Vec<(String, String)> = if case == "product" {
            vec![
                ("int".into(), "Number".into()),
                ("string".into(), "Text".into()),
                ("decimal".into(), "Amount".into()),
                ("int?".into(), "Maybe".into()),
            ]
        } else {
            (0..32)
                .map(|i| {
                    (
                        if i < 16 { "long[]" } else { "string" }.into(),
                        format!("F{i}"),
                    )
                })
                .collect()
        };
        let fields = pairs
            .iter()
            .map(|(t, n)| format!("public readonly {t} {n};"))
            .collect::<String>();
        let parameters = pairs
            .iter()
            .enumerate()
            .map(|(i, (t, _))| format!("{t} p{i}"))
            .collect::<Vec<_>>()
            .join(",");
        let body = pairs
            .iter()
            .enumerate()
            .map(|(i, (_, n))| format!("{n}=p{i};"))
            .collect::<String>();
        let arguments = pairs
            .iter()
            .map(|(_, n)| format!("p.{n}"))
            .collect::<Vec<_>>()
            .join(",");
        (format!("namespace Boundary;public readonly struct Payload{{{fields}public Payload({parameters}){{{body}}}}}public static class Entry{{public static Payload Run(Payload p){{return new Payload({arguments});}}}}\n"),decl("type","Payload","",&[],""))
    };
    let types = if case == "unit" {
        vec![]
    } else {
        vec![result.clone()]
    };
    let root = decl(
        "method",
        "Run",
        &decl("type", "Entry", "", &[], ""),
        &types,
        &result,
    );
    support::context_with_sidecars(
        b,
        &root,
        src.as_bytes(),
        vec![
            "contracts/boundary.json".into(),
            "contracts/method.json".into(),
        ],
        |ctx| {
            let mut output = if case == "unit" {
                vec![]
            } else {
                vec![field("result", &result)]
            };
            if case == "product" {
                set(&mut output[0], "json_name", J::Utf16String(vec![0xd800]));
            }
            let boundary = hashed(
                "MPK-CSHARP-BOUNDARY-CONTRACT-1.0",
                vec![
                    ("schema", j(BOUNDARY_CONTRACT_SCHEMA)),
                    ("semantic_context", ctx.semantic_context().clone()),
                    ("compilation_id", j(ctx.compilation_id())),
                    ("boundary_id", j("boundary.entry")),
                    ("selected_callable_id", j(&root)),
                    (
                        "input_fields",
                        J::Array(types.iter().map(|t| field("field0", t)).collect()),
                    ),
                    ("output_fields", J::Array(output)),
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
                ],
            );
            let method = hashed(
                "MPK-CSHARP-METHOD-CONTRACT-1.0",
                vec![
                    ("schema", j(METHOD_CONTRACT_SCHEMA)),
                    ("semantic_context", ctx.semantic_context().clone()),
                    ("compilation_id", j(ctx.compilation_id())),
                    ("callable_id", j(&root)),
                    (
                        "source_content_sha256",
                        j(&format!("{:x}", Sha256::digest(src.as_bytes()))),
                    ),
                    ("termination", j("total")),
                    ("requires", J::Array(vec![])),
                    ("ensures", J::Array(vec![])),
                    ("exceptional_cases", J::Array(vec![])),
                    ("modifies", J::Array(vec![])),
                    ("loops", J::Array(vec![])),
                ],
            );
            vec![boundary, method]
        },
    )
}
fn output_fixture(
    b: &ValidatedFoundationBundle,
    case: &str,
) -> (PracticalArtifactContext, CapturedInputSet, EmittedDataPhase) {
    let (context, captures) = output_setup(b, case);
    let rows: Vec<Value> =
        serde_json::from_slice(&fs::read(path("source-responses.json")).unwrap()).unwrap();
    let row = rows
        .iter()
        .find(|r| r["id"] == captures.snapshot_sha256())
        .unwrap();
    let requests: Vec<Value> =
        serde_json::from_slice(&fs::read(path("source-requests.json")).unwrap()).unwrap();
    assert_eq!(
        requests
            .iter()
            .find(|r| r["id"] == captures.snapshot_sha256())
            .unwrap(),
        &request(&context, &captures)
    );
    let source = ValidatedDataSource::import_captured_facts(
        b,
        &context,
        &captures,
        &serde_json::to_vec(&row["facts"]).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(b, &context, &captures, &source).unwrap();
    (context, captures, emitted)
}
fn request(context: &PracticalArtifactContext, captures: &CapturedInputSet) -> Value {
    json!({"id":captures.snapshot_sha256(),"compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.kind()==OriginalInputKind::Source {"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()})
}
fn evidence(run: &CapturedBoundaryOutputRun) -> BoundaryOutputEvidence<'_> {
    BoundaryOutputEvidence {
        canonical_document: run.capture().canonical_document(),
        capture: run.capture().artifact().canonical_bytes(),
        manifest: run.manifest().canonical_bytes(),
        artifacts: run.artifacts().canonical_bytes(),
    }
}
#[test]
fn csharp_03_t05_w03_original_source_output_and_cumulative_identity() {
    let b = bundle();
    if let Ok(file) = std::env::var("MPK_W03_REQUESTS_OUT") {
        let rows = ["product", "limits", "unit"].map(|case| {
            let (c, s) = output_setup(&b, case);
            request(&c, &s)
        });
        fs::write(file, serde_json::to_vec(&rows).unwrap()).unwrap();
        return;
    }
    let mut retained = vec![];
    for (case, doc, golden) in [
        (
            "product",
            r#"{"field0":{"Number":7,"Text":"😀\ud800","Amount":"1.25","Maybe":{"tag":"some","payload":9}}}"#,
            r#"{"\ud800":{"Number":7,"Text":"😀\ud800","Amount":"1.25","Maybe":{"tag":"some","payload":9}}}"#,
        ),
        ("unit", "{}", "{}"),
    ] {
        let (ctx, captures, emitted) = output_fixture(&b, case);
        let input = emitted
            .capture_boundary_input(
                &b,
                &ctx,
                &captures,
                input_bytes(&emitted, doc.as_bytes(), doc.as_bytes()),
            )
            .unwrap();
        let returned = if case == "unit" {
            MonomorphicValue::Unit {
                type_id: ty("unit"),
            }
        } else {
            input.arguments()[0].value().clone()
        };
        let run = emitted
            .capture_boundary_output(&b, &ctx, &captures, &input, &returned)
            .unwrap();
        assert_eq!(run.capture().canonical_document(), golden.as_bytes());
        assert_eq!(run.returned_value(), &returned);
        assert_eq!(run.reparsed_value(), &returned);
        let again = emitted
            .import_boundary_output_run(&b, &ctx, &captures, &input, &returned, evidence(&run))
            .unwrap();
        assert_eq!(
            again.artifacts().canonical_bytes(),
            run.artifacts().canonical_bytes()
        );
        for key in ["boundary_inputs", "boundary_outputs"] {
            assert_eq!(
                run.manifest()
                    .value()
                    .get(key)
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .len(),
                1
            );
        }
        assert_eq!(
            run.manifest().value().get("source_map").unwrap(),
            input.manifest().value().get("source_map").unwrap()
        );
        if case == "product" {
            for changed in [
                golden.replace(
                    r#""Number":7,"Text":"😀\ud800""#,
                    r#""Text":"😀\ud800","Number":7"#,
                ),
                golden.replace(r#""tag":"some","payload":9"#, r#""payload":9,"tag":"some"#),
                golden.replace(r#"\ud800"#, r#"\uD800"#),
                golden.replace(r#""1.25""#, r#""1.250""#),
                golden.replace(r#""Number":7"#, r#""Number":8"#),
            ] {
                assert_ne!(changed, golden);
                let mut e = evidence(&run);
                e.canonical_document = changed.as_bytes();
                assert!(emitted
                    .import_boundary_output_run(&b, &ctx, &captures, &input, &returned, e)
                    .is_err());
            }
        }
        retained.push(json!({"case":case,"canonical_document":golden,"capture":std::str::from_utf8(run.capture().artifact().canonical_bytes()).unwrap(),"manifest":std::str::from_utf8(run.manifest().canonical_bytes()).unwrap(),"artifacts":std::str::from_utf8(run.artifacts().canonical_bytes()).unwrap()}));
    }
    let record = json!({"work_item":"CSHARP-03-T05-W03","runs":retained});
    if let Ok(file) = std::env::var("MPK_W03_EVIDENCE_OUT") {
        fs::write(file, serde_json::to_vec_pretty(&record).unwrap()).unwrap();
    } else {
        assert_eq!(
            record,
            serde_json::from_slice::<Value>(&fs::read(path("conformance.json")).unwrap()).unwrap()
        );
    }
}
#[test]
fn csharp_03_t05_w03_hostile_output_cannot_publish_evidence() {
    let b = bundle();
    let (ctx, captures, emitted) = input_fixture(&b, "required");
    let doc = br#"{"field0":7,"field1":"0"}"#;
    let input = emitted
        .capture_boundary_input(&b, &ctx, &captures, input_bytes(&emitted, doc, doc))
        .unwrap();
    let returned = MonomorphicValue::Signed {
        type_id: ty("i32"),
        value: "7".into(),
    };
    let run = emitted
        .capture_boundary_output(&b, &ctx, &captures, &input, &returned)
        .unwrap();
    assert_eq!(run.capture().canonical_document(), br#"{"result":7}"#);
    for bytes in [
        br#"{"result":8}"#.as_slice(),
        br#"{"result":"7"}"#,
        br#"{"result":7.0}"#,
        br#"{"result":07}"#,
        br#"{ "result":7}"#,
        br#"{"\u0072esult":7}"#,
        br#"{"other":7}"#,
        br#"{"result":7,"result":7}"#,
        br#"{"result":{"tag":"some","payload":7}}"#,
        b"\xff",
        b"\xef\xbb\xbf{}",
    ] {
        let mut e = evidence(&run);
        e.canonical_document = bytes;
        assert_eq!(
            emitted
                .import_boundary_output_run(&b, &ctx, &captures, &input, &returned, e)
                .unwrap_err(),
            BoundaryOutputError::Bytes
        );
    }
    for key in [
        "source_value",
        "reparsed_value",
        "source_value_sha256",
        "reparsed_value_sha256",
        "canonical_document_utf8_sha256",
        "boundary_contract_sha256",
        "semantic_context",
    ] {
        let mut mutated = run.capture().artifact().value().clone();
        set(
            &mut mutated,
            key,
            if key.ends_with("sha256") {
                j(&"0".repeat(64))
            } else {
                obj(vec![])
            },
        );
        let J::Object(fields) = &mut mutated else {
            panic!()
        };
        fields.pop();
        let mut h = Sha256::new();
        h.update(b"MPK-CSHARP-BOUNDARY-OUTPUT-1.0\0");
        h.update(canonical_practical_json_bytes(&mutated).unwrap());
        let J::Object(fields) = &mut mutated else {
            panic!()
        };
        fields.push(("capture_sha256".into(), j(&format!("{:x}", h.finalize()))));
        let bytes = canonical_practical_json_bytes(&mutated).unwrap();
        let mut e = evidence(&run);
        e.capture = &bytes;
        assert!(
            emitted
                .import_boundary_output_run(&b, &ctx, &captures, &input, &returned, e)
                .is_err(),
            "{key}"
        );
    }
    let other = emitted
        .capture_boundary_input(
            &b,
            &ctx,
            &captures,
            input_bytes(&emitted, b"other provenance", doc),
        )
        .unwrap();
    assert!(emitted
        .import_boundary_output_run(&b, &ctx, &captures, &other, &returned, evidence(&run))
        .is_err());
    let mut e = evidence(&run);
    e.manifest = input.manifest().canonical_bytes();
    assert!(emitted
        .import_boundary_output_run(&b, &ctx, &captures, &input, &returned, e)
        .is_err());
    let wrong = MonomorphicValue::Signed {
        type_id: ty("i64"),
        value: "7".into(),
    };
    assert_eq!(
        emitted
            .capture_boundary_output(&b, &ctx, &captures, &input, &wrong)
            .unwrap_err(),
        BoundaryOutputError::Value
    );
}
#[test]
fn csharp_03_t05_w03_actual_source_output_resource_boundaries() {
    let b = bundle();
    let (ctx, captures, emitted) = output_fixture(&b, "limits");
    let base = || {
        J::Object(
            (0..32)
                .map(|i| {
                    (
                        format!("F{i}"),
                        if i < 16 { J::Array(vec![]) } else { j("") },
                    )
                })
                .collect(),
        )
    };
    let doc = canonical_practical_json_bytes(&obj(vec![("field0", base())])).unwrap();
    let input = emitted
        .capture_boundary_input(&b, &ctx, &captures, input_bytes(&emitted, &doc, &doc))
        .unwrap();
    for target in [65535, 65536, 65537] {
        let mut returned = input.arguments()[0].value().clone();
        let MonomorphicValue::Product { fields, .. } = &mut returned else {
            panic!()
        };
        let mut remaining = target - 34; // document, product, 32 field roots
        for f in fields.iter_mut().take(16) {
            let n = remaining.min(4096);
            remaining -= n;
            let MonomorphicValue::Sequence { elements, .. } = f.value.as_mut() else {
                panic!()
            };
            *elements = vec![
                MonomorphicValue::Signed {
                    type_id: ty("i64"),
                    value: "0".into()
                };
                n
            ];
        }
        assert_eq!(remaining, 0);
        let result = emitted.capture_boundary_output(&b, &ctx, &captures, &input, &returned);
        assert_eq!(
            result.is_ok(),
            target <= 65536,
            "cells {target}: {:?}",
            result.as_ref().err()
        );
    }
    for target in [1048575, 1048576, 1048577] {
        let mut returned = input.arguments()[0].value().clone();
        let MonomorphicValue::Product { fields, .. } = &mut returned else {
            panic!()
        };
        let mut golden = base();
        let mut remaining = target
            - canonical_practical_json_bytes(&obj(vec![("result", golden.clone())]))
                .unwrap()
                .len();
        for (i, f) in fields.iter_mut().take(16).enumerate() {
            let n = ((remaining + 1) / 23).min(4096);
            if n > 0 {
                let MonomorphicValue::Sequence { elements, .. } = f.value.as_mut() else {
                    panic!()
                };
                *elements = vec![
                    MonomorphicValue::Signed {
                        type_id: ty("i64"),
                        value: "-9223372036854775808".into()
                    };
                    n
                ];
                set(
                    &mut golden,
                    &format!("F{i}"),
                    J::Array(vec![j("-9223372036854775808"); n]),
                );
                remaining -= 23 * n - 1;
            }
        }
        assert!(remaining < 23);
        *fields[16].value = MonomorphicValue::String {
            type_id: ty("string"),
            utf16: vec![b'x' as u16; remaining],
        };
        set(&mut golden, "F16", j(&"x".repeat(remaining)));
        let bytes = canonical_practical_json_bytes(&obj(vec![("result", golden)])).unwrap();
        assert_eq!(bytes.len(), target);
        let result = emitted.capture_boundary_output(&b, &ctx, &captures, &input, &returned);
        assert_eq!(
            result.is_ok(),
            target <= 1048576,
            "bytes {target}: {:?}",
            result.as_ref().err()
        );
        if let Ok(run) = result {
            assert_eq!(run.capture().canonical_document(), bytes);
        }
    }
}
