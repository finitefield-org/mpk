//! CSHARP-03-T05-W01: strict captured boundary sidecars and typed presence plans.
use mpk_vc::csharp_practical_source_artifacts::*;
use mpk_vc::csharp_practical_vir_model::*;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path};
#[allow(dead_code)]
#[path = "support/csharp_practical_data_context.rs"]
mod support;
fn bundle() -> ValidatedFoundationBundle {
    validate_registered_foundation_bundle(
        registered_foundation_descriptor_transport(),
        registered_foundation_definitions_transport(),
    )
    .unwrap()
}
fn ty(s: &str) -> String {
    format!("mpk.csharp.value.{s}.v1")
}
fn decl(kind: &str, name: &str, owner: &str, params: &[String], result: &str) -> String {
    csharp_practical_declaration_id(&json!({"kind":kind,"namespace":"Boundary","owner":owner,"name":name,"parameter_type_ids":params,"result_type_id":result})).unwrap()
}
fn obj(fields: Vec<(&str, PracticalJsonValue)>) -> PracticalJsonValue {
    PracticalJsonValue::object(fields)
}
fn j(s: &str) -> PracticalJsonValue {
    PracticalJsonValue::string(s)
}
fn hashed(domain: &str, mut fields: Vec<(&str, PracticalJsonValue)>) -> Vec<u8> {
    let bytes = canonical_practical_json_bytes(&obj(fields.clone())).unwrap();
    let mut h = Sha256::new();
    h.update(domain);
    h.update([0]);
    h.update(bytes);
    fields.push(("contract_sha256", j(&format!("{:x}", h.finalize()))));
    canonical_practical_json_bytes(&obj(fields)).unwrap()
}
fn field(id: &str, t: &str) -> PracticalJsonValue {
    use PracticalJsonValue as J;
    obj(vec![
        ("field_id", j(id)),
        ("json_name", j(id)),
        ("type_id", j(t)),
        ("required", J::Bool(true)),
        ("nullable", J::Bool(false)),
        ("missing_rule", obj(vec![("mode", j("reject"))])),
        ("codec_id", J::Null),
        ("codec_parameters", J::Null),
    ])
}
fn set(v: &mut PracticalJsonValue, key: &str, value: PracticalJsonValue) {
    let PracticalJsonValue::Object(fields) = v else {
        panic!()
    };
    fields.iter_mut().find(|(k, _)| k == key).unwrap().1 = value;
}
fn freeze(v: PracticalJsonValue) -> PracticalJsonValue {
    obj(vec![("mode", j("use_frozen_typed_default")), ("value", v)])
}
fn source(presence: bool, nullable: bool) -> String {
    if presence {
        "namespace Boundary;public enum Tag{Missing=0,Null=1,Value=2}public readonly struct Presence{public readonly Tag Tag;public readonly int Value;public Presence(Tag tag,int value){Tag=tag;Value=value;}}public static class Entry{public static int Run(Presence p){return new Presence(p.Tag,p.Value).Value;}}\n".into()
    } else if nullable {
        "namespace Boundary;public static class Entry{public static int Run(int? p){return 0;}}\n"
            .into()
    } else {
        "namespace Boundary;public static class Entry{public static int Run(int n,long timestamp){return n;}}\n".into()
    }
}
fn configuration(b: &ValidatedFoundationBundle, case: &str) -> (String, Vec<String>, String, bool) {
    if matches!(case, "input_limits" | "input_byte_limits") {
        let array = csharp_practical_closed_instance_id(b, &json!({"kind":"instance","template":"bounded_sequence","arguments":[{"kind":"primitive","id":if case=="input_byte_limits" {"i64"} else {"i32"}}]})).unwrap();
        let types = (0..32)
            .map(|i| if i < 16 { array.clone() } else { ty("string") })
            .collect::<Vec<_>>();
        let parameters = (0..32)
            .map(|i| {
                format!(
                    "{} p{i}",
                    if i < 16 {
                        if case == "input_byte_limits" {
                            "long[]"
                        } else {
                            "int[]"
                        }
                    } else {
                        "string"
                    }
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let src = format!("namespace Boundary;public static class Entry{{public static int Run({parameters}){{return 0;}}}}\n");
        let owner = decl("type", "Entry", "", &[], "");
        let root = decl("method", "Run", &owner, &types, &ty("i32"));
        return (src, types, root, false);
    }
    let presence = case.starts_with("presence");
    let nullable = case.starts_with("nullable");
    let types = if case.starts_with("decimal") {
        vec![ty("decimal"), ty("i64")]
    } else if case.starts_with("enum") {
        vec![decl("type", "Color", "", &[], "")]
    } else if presence {
        vec![decl("type", "Presence", "", &[], "")]
    } else if nullable {
        vec![csharp_practical_closed_instance_id(b,&json!({"kind":"instance","template":"option","arguments":[{"kind":"primitive","id":"i32"}]})).unwrap()]
    } else {
        vec![ty("i32"), ty("i64")]
    };
    let owner = decl("type", "Entry", "", &[], "");
    let root = decl("method", "Run", &owner, &types, &ty("i32"));
    let src = if case.starts_with("decimal") {
        "namespace Boundary;public static class Entry{public static int Run(decimal n,long timestamp){return 0;}}\n".into()
    } else if case.starts_with("enum") {
        "namespace Boundary;public enum Color{Red=0,Blue=1}public static class Entry{public static int Run(Color n){return 0;}}\n".into()
    } else {
        source(presence, nullable)
    };
    let src = if case == "presence_instant" {
        "namespace Boundary;public enum Tag{Missing=0,Null=1,Value=2}public readonly struct Instant{public readonly long Milliseconds;public Instant(long milliseconds){Milliseconds=milliseconds;}}public readonly struct Presence{public readonly Tag Tag;public readonly Instant Value;public Presence(Tag tag,Instant value){Tag=tag;Value=value;}}public static class Entry{public static int Run(Presence p){return (int)new Presence(p.Tag,new Instant(p.Value.Milliseconds)).Value.Milliseconds;}}\n".into()
    } else {
        src
    };
    let src = if case == "presence_nonmissing_zero" {
        src.replace("Missing=0,Null=1", "Missing=1,Null=0")
    } else {
        src
    };
    (src, types, root, presence)
}
fn cases() -> Vec<(&'static str, bool)> {
    vec![
        ("required", true),
        ("decimal_default", true),
        ("decimal_wrong_scale", false),
        ("enum_default", true),
        ("enum_unknown", false),
        ("enum_noncanonical", false),
        ("nullable_nonnull", true),
        ("surrogate_name", true),
        ("missing_schema_field", false),
        ("optional_default", true),
        ("raw_instant", true),
        ("nullable_required", true),
        ("nullable_default", true),
        ("nullable_none_default", false),
        ("presence_exposed", true),
        ("presence_required", true),
        ("presence_instant", true),
        ("presence_nonmissing_zero", true),
        ("presence_default", true),
        ("presence_missing_default", true),
        ("presence_inactive_default", false),
        ("presence_bad_tag", false),
        ("presence_wrong_member", false),
        ("unknown_field", false),
        ("duplicate_field", false),
        ("duplicate_name", false),
        ("unknown_missing", false),
        ("required_default", false),
        ("optional_reject", false),
        ("ordinary_expose", false),
        ("nonnull", false),
        ("wrong_type", false),
        ("wrong_order", false),
        ("unknown_codec", false),
        ("codec_parameters", false),
        ("bad_default_type", false),
        ("bad_default_range", false),
        ("string_i32_default", false),
        ("field_limit", false),
        ("bad_id", false),
        ("unknown_root", false),
        ("profile", false),
        ("schema", false),
        ("compilation", false),
        ("context", false),
        ("partial", false),
        ("stale_hash", false),
        ("stale_source", false),
        ("duplicate_json", false),
        ("output_type", false),
        ("output_count", false),
        ("default_depth", false),
        ("default_cells", false),
        ("unknown_root_field", false),
        ("reordered_field", false),
        ("noncanonical", false),
    ]
}
fn setup(
    b: &ValidatedFoundationBundle,
    case: &str,
) -> (PracticalArtifactContext, CapturedInputSet) {
    use PracticalJsonValue as J;
    let (src, types, root, presence) = configuration(b, case);
    let paths = if presence {
        vec![
            "contracts/binding.json".into(),
            "contracts/boundary.json".into(),
            "contracts/method.json".into(),
        ]
    } else {
        vec![
            "contracts/boundary.json".into(),
            "contracts/method.json".into(),
        ]
    };
    support::context_with_sidecars(b, &root, src.as_bytes(), paths.clone(), |ctx| {
        let mut fields = types
            .iter()
            .enumerate()
            .map(|(i, t)| field(&format!("field{i}"), t))
            .collect::<Vec<_>>();
        if case.starts_with("nullable") || presence {
            set(&mut fields[0], "nullable", J::Bool(true));
        }
        match case {
            "decimal_default" | "decimal_wrong_scale" => {
                set(&mut fields[0], "required", J::Bool(false));
                set(&mut fields[0], "codec_id", j("decimal.fixed"));
                set(
                    &mut fields[0],
                    "codec_parameters",
                    obj(vec![("scale", J::U64(3)), ("rounding", j("ToEven"))]),
                );
                set(
                    &mut fields[0],
                    "missing_rule",
                    freeze(j(if case == "decimal_default" {
                        "1.250"
                    } else {
                        "1.25"
                    })),
                );
            }
            "enum_default" | "enum_unknown" | "enum_noncanonical" => {
                set(&mut fields[0], "required", J::Bool(false));
                set(
                    &mut fields[0],
                    "missing_rule",
                    freeze(j(match case {
                        "enum_default" => "1",
                        "enum_unknown" => "9",
                        _ => "01",
                    })),
                );
            }
            "nullable_nonnull" => set(&mut fields[0], "nullable", J::Bool(false)),
            "surrogate_name" => set(&mut fields[0], "json_name", J::Utf16String(vec![0xd800])),
            "optional_default" => {
                set(&mut fields[0], "required", J::Bool(false));
                set(&mut fields[0], "missing_rule", freeze(J::I64(7)));
            }
            "raw_instant" => {
                set(&mut fields[1], "codec_id", j("unix_milliseconds"));
                set(
                    &mut fields[1],
                    "codec_parameters",
                    obj(vec![("scale", J::Null), ("rounding", J::Null)]),
                );
            }
            "nullable_default" | "nullable_none_default" => {
                set(&mut fields[0], "required", J::Bool(false));
                set(
                    &mut fields[0],
                    "missing_rule",
                    freeze(if case == "nullable_default" {
                        obj(vec![("tag", j("some")), ("payload", J::I64(7))])
                    } else {
                        obj(vec![("tag", j("none"))])
                    }),
                );
            }
            "presence_missing_default" => {
                set(&mut fields[0], "required", J::Bool(false));
                set(
                    &mut fields[0],
                    "missing_rule",
                    freeze(obj(vec![("tag", j("missing"))])),
                );
            }
            "presence_exposed" => {
                set(&mut fields[0], "required", J::Bool(false));
                set(
                    &mut fields[0],
                    "missing_rule",
                    obj(vec![("mode", j("expose_missing"))]),
                );
            }
            "presence_default" | "presence_inactive_default" | "presence_bad_tag" => {
                set(&mut fields[0], "required", J::Bool(false));
                set(
                    &mut fields[0],
                    "missing_rule",
                    freeze(obj(vec![
                        (
                            "tag",
                            j(if case == "presence_bad_tag" {
                                "invalid"
                            } else if case == "presence_inactive_default" {
                                "missing"
                            } else {
                                "value"
                            }),
                        ),
                        ("payload", J::I64(7)),
                    ])),
                );
            }
            "unknown_field" => {
                let J::Object(x) = &mut fields[0] else {
                    panic!()
                };
                x.push(("unknown".into(), J::Null));
            }
            "duplicate_field" => {
                let id = fields[0].get("field_id").unwrap().clone();
                set(&mut fields[1], "field_id", id);
            }
            "duplicate_name" => set(&mut fields[1], "json_name", j("field0")),
            "unknown_missing" => set(
                &mut fields[0],
                "missing_rule",
                obj(vec![("mode", j("collapse"))]),
            ),
            "required_default" => set(&mut fields[0], "missing_rule", freeze(J::I64(7))),
            "optional_reject" => set(&mut fields[0], "required", J::Bool(false)),
            "ordinary_expose" => {
                set(&mut fields[0], "required", J::Bool(false));
                set(
                    &mut fields[0],
                    "missing_rule",
                    obj(vec![("mode", j("expose_missing"))]),
                );
            }
            "nonnull" => set(&mut fields[0], "nullable", J::Bool(true)),
            "wrong_type" => set(&mut fields[0], "type_id", j(&ty("bool"))),
            "wrong_order" => fields.swap(0, 1),
            "unknown_codec" => set(&mut fields[0], "codec_id", j("local.parse")),
            "codec_parameters" => set(&mut fields[0], "codec_parameters", obj(vec![])),
            "bad_default_type" | "bad_default_range" | "string_i32_default" => {
                set(&mut fields[0], "required", J::Bool(false));
                set(
                    &mut fields[0],
                    "missing_rule",
                    freeze(match case {
                        "bad_default_type" => J::Bool(true),
                        "bad_default_range" => J::U64(2147483648),
                        _ => j("7"),
                    }),
                );
            }
            "default_depth" | "default_cells" => {
                let mut value = J::I64(7);
                if case == "default_depth" {
                    for _ in 0..33 {
                        value = J::Array(vec![value]);
                    }
                } else {
                    value = J::Array(vec![value; 65536]);
                }
                set(&mut fields[0], "required", J::Bool(false));
                set(&mut fields[0], "missing_rule", freeze(value));
            }
            "field_limit" => fields = vec![fields[0].clone(); 257],
            "bad_id" => set(&mut fields[0], "field_id", j("../host")),
            "reordered_field" => {
                let J::Object(xs) = &mut fields[0] else {
                    panic!()
                };
                xs.swap(0, 1);
            }
            _ => {}
        }
        let mut output = vec![field("result", &ty("i32"))];
        if case == "output_type" {
            set(&mut output[0], "type_id", j(&ty("i64")));
        }
        if case == "output_count" {
            output.clear();
        }
        let mut boundary = vec![
            (
                "schema",
                j(if case == "schema" {
                    "mpk.csharp.boundary.v2"
                } else {
                    BOUNDARY_CONTRACT_SCHEMA
                }),
            ),
            (
                "semantic_context",
                if case == "context" {
                    obj(vec![])
                } else {
                    ctx.semantic_context().clone()
                },
            ),
            (
                "compilation_id",
                j(if case == "compilation" {
                    "other"
                } else {
                    ctx.compilation_id()
                }),
            ),
            ("boundary_id", j("boundary.entry")),
            (
                "selected_callable_id",
                j(if case == "unknown_root" {
                    "mpk.csharp.source.0000000000000000000000000000000000000000000000000000000000000000"
                } else {
                    &root
                }),
            ),
            ("input_fields", J::Array(fields)),
            ("output_fields", J::Array(output)),
            (
                "canonical_json_profile",
                j(if case == "profile" {
                    "other"
                } else {
                    "mpk.csharp.canonical_json.v1"
                }),
            ),
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
        ];
        if case == "missing_schema_field" {
            boundary.retain(|(key, _)| *key != "parse_format_profile");
        }
        if case == "unknown_root_field" {
            boundary.push(("maximum_bytes", J::U64(1)));
        }
        let mut bytes = hashed("MPK-CSHARP-BOUNDARY-CONTRACT-1.0", boundary);
        if case == "stale_hash" {
            let s = String::from_utf8(bytes).unwrap();
            bytes = s.replace("boundary.entry", "boundary.other").into_bytes();
        }
        if case == "duplicate_json" {
            let s = String::from_utf8(bytes).unwrap();
            bytes = s
                .replace("\"required\":true", "\"required\":true,\"required\":true")
                .into_bytes();
        }
        if case == "noncanonical" {
            bytes.push(b'\n');
        }
        let source_hash = format!("{:x}", Sha256::digest(src.as_bytes()));
        let method = hashed(
            "MPK-CSHARP-METHOD-CONTRACT-1.0",
            vec![
                ("schema", j(METHOD_CONTRACT_SCHEMA)),
                ("semantic_context", ctx.semantic_context().clone()),
                ("compilation_id", j(ctx.compilation_id())),
                ("callable_id", j(&root)),
                (
                    "source_content_sha256",
                    j(if case == "stale_source" {
                        "0000000000000000000000000000000000000000000000000000000000000000"
                    } else {
                        &source_hash
                    }),
                ),
                (
                    "termination",
                    j(if case == "partial" {
                        "partial"
                    } else {
                        "total"
                    }),
                ),
                ("requires", J::Array(vec![])),
                ("ensures", J::Array(vec![])),
                ("exceptional_cases", J::Array(vec![])),
                ("modifies", J::Array(vec![])),
                ("loops", J::Array(vec![])),
            ],
        );
        let mut outputs = vec![];
        if presence {
            let preliminary = capture_original_inputs(
                ctx,
                std::iter::once(OriginalInput {
                    kind: OriginalInputKind::Source,
                    path: "src/Entry.cs".into(),
                    bytes: src.as_bytes().to_vec(),
                })
                .chain(paths.iter().map(|path| OriginalInput {
                    kind: OriginalInputKind::Sidecar,
                    path: path.clone(),
                    bytes: b"{}".to_vec(),
                }))
                .collect(),
            )
            .unwrap();
            let tag_id = decl("type", "Tag", "", &[], "");
            let members = [
                ("tag", "Tag", json!({"kind":"source","id":tag_id})),
                (
                    "value",
                    "Value",
                    if case == "presence_instant" {
                        json!({"kind":"source","id":decl("type","Instant","",&[],"")})
                    } else {
                        json!({"kind":"primitive","id":"i32"})
                    },
                ),
            ]
            .into_iter()
            .map(|(role, name, t)| SemanticBindingMember {
                role: role.into(),
                member_id: csharp_practical_stored_member_id(
                    &types[0],
                    if case == "presence_wrong_member" && name == "Value" {
                        "Invented"
                    } else {
                        name
                    },
                    &t,
                    "readonly_field",
                )
                .unwrap(),
            })
            .collect();
            let mut bindings = vec![SemanticBindingInput {
                source_type_id: types[0].clone(),
                source_content_sha256: source_hash.clone(),
                role: "boundary_field".into(),
                member_map: members,
                tag_arms: ["missing", "null", "value"]
                    .into_iter()
                    .enumerate()
                    .map(|(i, arm)| SemanticArmMapping {
                        source_tag: (if case == "presence_nonmissing_zero" && i < 2 {
                            1 - i
                        } else {
                            i
                        })
                        .to_string(),
                        semantic_arm: arm.into(),
                    })
                    .collect(),
                inferred_argument_ids: vec![ty(if case == "presence_instant" {
                    "instant"
                } else {
                    "i32"
                })],
                default_arm: "ineligible".into(),
                bounds: vec![],
                operation_map: vec![],
                enum_arms: BTreeMap::new(),
            }];
            if case == "presence_instant" {
                let id = decl("type", "Instant", "", &[], "");
                bindings.push(SemanticBindingInput {
                    source_type_id: id.clone(),
                    source_content_sha256: source_hash.clone(),
                    role: "instant".into(),
                    member_map: vec![SemanticBindingMember {
                        role: "milliseconds".into(),
                        member_id: csharp_practical_stored_member_id(
                            &id,
                            "Milliseconds",
                            &json!({"kind":"primitive","id":"i64"}),
                            "readonly_field",
                        )
                        .unwrap(),
                    }],
                    tag_arms: vec![],
                    inferred_argument_ids: vec![],
                    default_arm: "ineligible".into(),
                    bounds: vec![],
                    operation_map: vec![],
                    enum_arms: BTreeMap::new(),
                });
            }
            outputs.push(
                build_semantic_bindings(ctx, &preliminary, bindings)
                    .unwrap()
                    .canonical_bytes()
                    .to_vec(),
            );
        }
        outputs.extend([bytes, method]);
        outputs
    })
}
#[test]
fn csharp_03_t05_w01_real_source_boundary_matrix() {
    let b = bundle();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let generation = std::env::var("MPK_CSHARP_BOUNDARY_REQUESTS_OUT").ok();
    let responses: Vec<Value> = if generation.is_some() {
        vec![]
    } else {
        serde_json::from_slice(
            &fs::read(root.join("develop/migrations/csharp-03/boundary-attachment/responses.json"))
                .unwrap(),
        )
        .unwrap()
    };
    let mut requests = vec![];
    let mut accepted = 0;
    for (case, expected) in cases() {
        let (context, captures) = setup(&b, case);
        requests.push(json!({"case":case,"expected_attachment":expected,"id":captures.snapshot_sha256(),"compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.kind()==OriginalInputKind::Source{"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>() }));
        if generation.is_some() {
            continue;
        }
        let row = responses
            .iter()
            .find(|r| r["id"] == captures.snapshot_sha256())
            .unwrap_or_else(|| panic!("missing actual capture: {case}"));
        if row.get("facts").is_none() {
            assert_eq!(case, "presence_wrong_member");
            assert!(!expected);
            assert_eq!(row["reject"], "CSHARP_PRACTICAL_TYPE/business_member");
            assert_eq!(row["artifact_count"], 0);
            continue;
        }
        assert!(row["facts"].is_object(), "{case}: {row}");
        let source = ValidatedDataSource::import_captured_facts(
            &b,
            &context,
            &captures,
            &serde_json::to_vec(&row["facts"]).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&b, &context, &captures, &source);
        assert_eq!(
            emitted.is_ok(),
            expected,
            "{case}: {:?}",
            emitted.as_ref().err()
        );
        if !expected {
            assert!(
                matches!(
                    emitted.as_ref().err(),
                    Some(DataPhaseError::Boundary(_) | DataPhaseError::Sidecar)
                ),
                "wrong rejection owner for {case}: {:?}",
                emitted.as_ref().err()
            );
        }
        if matches!(case, "field_limit" | "default_depth" | "default_cells") {
            assert!(
                matches!(
                    emitted.as_ref().err(),
                    Some(DataPhaseError::Boundary(BoundaryError::Limit))
                ),
                "{case}: {:?}",
                emitted.as_ref().err()
            );
        }
        if let Ok(emitted) = emitted {
            accepted += 1;
            assert_eq!(emitted.boundaries().len(), 1);
            let boundary = &emitted.boundaries()[0];
            assert_eq!(boundary.maximum_document_bytes(), 1_048_576);
            assert_eq!(boundary.maximum_value_depth(), 32);
            assert_eq!(boundary.maximum_value_cells(), 65_536);
            assert_eq!(
                emitted
                    .manifest()
                    .value()
                    .get("boundary_contracts")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .len(),
                1
            );
            assert_eq!(
                emitted
                    .artifacts()
                    .value()
                    .get("boundary_contracts")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .len(),
                1
            );
            assert!(boundary.obligations().iter().all(|o| !o.discharged));
            let f = &boundary.input_fields()[0];
            let r = emitted.closure().roots();
            let c = emitted.closure().closed();
            assert_eq!(
                f.check_state(&b, r, c, &BoundaryValueState::Missing)
                    .is_ok(),
                !f.required()
            );
            assert_eq!(
                f.check_state(&b, r, c, &BoundaryValueState::Null).is_ok(),
                f.nullable()
            );
            let value = if case == "presence_instant" {
                MonomorphicValue::Instant {
                    type_id: ty("instant"),
                    milliseconds: "7".into(),
                }
            } else if case.starts_with("decimal") {
                MonomorphicValue::DecimalBits {
                    type_id: ty("decimal"),
                    negative: false,
                    scale: 3,
                    coefficient: "1250".into(),
                }
            } else if case.starts_with("enum") {
                MonomorphicValue::Enum {
                    type_id: f.payload_type_id().into(),
                    underlying: "i32".into(),
                    carrier: "1".into(),
                }
            } else {
                MonomorphicValue::Signed {
                    type_id: ty("i32"),
                    value: "7".into(),
                }
            };
            assert!(f
                .check_state(&b, r, c, &BoundaryValueState::Value(value))
                .is_ok());
            assert!(f
                .check_state(
                    &b,
                    r,
                    c,
                    &BoundaryValueState::Value(MonomorphicValue::Bool {
                        type_id: ty("bool"),
                        value: true
                    })
                )
                .is_err());
            if case == "raw_instant" {
                assert!(boundary.input_fields()[1].is_raw_instant());
            } else {
                assert!(boundary.input_fields().iter().all(|f| !f.is_raw_instant()));
            }
            assert_eq!(f.presence_binding().is_some(), case.starts_with("presence"));
            if case.starts_with("presence") {
                assert_eq!(
                    f.presence_default_candidate().is_some(),
                    case != "presence_nonmissing_zero"
                );
                assert_eq!(
                    boundary
                        .obligations()
                        .iter()
                        .any(|o| o.kind == "boundary_actual_default_public_invariant"),
                    case != "presence_nonmissing_zero"
                );
            }
        }
    }
    if let Some(path) = generation {
        fs::write(path, serde_json::to_vec(&requests).unwrap()).unwrap();
        return;
    }
    let retained: Value = serde_json::from_slice(
        &fs::read(root.join("develop/migrations/csharp-03/boundary-attachment/requests.json"))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(json!(requests), retained);
    assert_eq!(accepted, 15);
}

#[test]
fn csharp_03_t05_w01_boundary_links_cannot_be_removed_or_moved() {
    use mpk_vc::csharp_practical_vir_validation as v;
    if std::env::var_os("MPK_CSHARP_BOUNDARY_REQUESTS_OUT").is_some() {
        return;
    }
    let b = bundle();
    let (context, captures) = setup(&b, "required");
    let rows: Vec<Value> = serde_json::from_slice(
        &fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../develop/migrations/csharp-03/boundary-attachment/responses.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let row = rows
        .iter()
        .find(|r| r["id"] == captures.snapshot_sha256())
        .unwrap();
    let source = ValidatedDataSource::import_captured_facts(
        &b,
        &context,
        &captures,
        &serde_json::to_vec(&row["facts"]).unwrap(),
    )
    .unwrap();
    let e = emit_data_phase(&b, &context, &captures, &source).unwrap();
    let input = v::PracticalVirImportContext {
        data_source_facts: Some(source.captured_facts()),
        artifact_context: &context,
        captured_inputs: &captures,
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
    for mutation in 0..3 {
        let mut changed = valid.clone();
        match mutation {
            0 => changed
                .data_contracts
                .retain(|s| !s.contains(BOUNDARY_CONTRACT_SCHEMA)),
            1 => {
                let boundary = changed
                    .data_contracts
                    .iter_mut()
                    .find(|s| s.contains(BOUNDARY_CONTRACT_SCHEMA))
                    .unwrap();
                *boundary = boundary.replace("boundary.entry", "boundary.other");
            }
            _ => changed.data_contracts.reverse(),
        }
        if let Ok(bytes) = v::canonical_csharp_practical_vir_transport(input, changed) {
            assert!(v::import_csharp_practical_vir_json(&bytes, input).is_err());
        }
    }
    let (other_context, other_captures) = setup(&b, "optional_default");
    assert!(ValidatedDataSource::import_captured_facts(
        &b,
        &other_context,
        &other_captures,
        source.captured_facts()
    )
    .is_err());
    let input = v::PracticalVirImportContext {
        data_source_facts: None,
        ..input
    };
    assert!(v::import_csharp_practical_vir_json(e.vir().canonical_bytes(), input).is_err());
}

#[test]
fn csharp_03_t05_w01_retained_evidence_and_frozen_limits() {
    if std::env::var_os("MPK_CSHARP_BOUNDARY_REQUESTS_OUT").is_some() {
        return;
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let record: Value = serde_json::from_slice(
        &fs::read(root.join("develop/migrations/csharp-03/boundary-attachment/conformance.json"))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(record["work_item"], "CSHARP-03-T05-W01");
    assert_eq!(record["source_snapshots"], cases().len());
    assert_eq!(
        record["accepted_attachments"],
        cases().iter().filter(|(_, ok)| *ok).count()
    );
    assert_eq!(record["rejected_attachments"], 41);
    assert_eq!(record["deterministic_source_captures"], 55);
    assert_eq!(record["source_rejections"], 1);
    assert_eq!(record["files"].as_array().unwrap().len(), 3);
    for file in record["files"].as_array().unwrap() {
        let bytes = fs::read(root.join(file["path"].as_str().unwrap())).unwrap();
        assert_eq!(file["sha256"], format!("{:x}", Sha256::digest(&bytes)));
        assert_eq!(file["size_bytes"], bytes.len());
    }
    let package: Value = serde_json::from_slice(
        &fs::read(root.join("develop/specs/vectors/csharp-practical-profile-v1.json")).unwrap(),
    )
    .unwrap();
    let limits = package["frozen_contract"]["limits"]["practical"]
        .as_array()
        .unwrap();
    for (id, max) in [
        ("boundary_fields", 256),
        ("boundary_nesting", 32),
        ("boundary_canonical_bytes", 1_048_576),
        ("total_collection_cells", 65_536),
        ("string_utf16_units", 16_384),
    ] {
        assert_eq!(
            limits.iter().find(|r| r["id"] == id).unwrap()["inclusive_maximum"],
            max
        );
    }
}

fn input_fixture(
    b: &ValidatedFoundationBundle,
    case: &str,
) -> (PracticalArtifactContext, CapturedInputSet, EmittedDataPhase) {
    let (context, captures) = setup(b, case);
    if matches!(case, "input_limits" | "input_byte_limits") {
        let requests: Vec<Value> = serde_json::from_slice(
            &fs::read(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../develop/migrations/csharp-03/boundary-input/source-requests.json"),
            )
            .unwrap(),
        )
        .unwrap();
        let request = requests
            .iter()
            .find(|r| r["id"] == captures.snapshot_sha256())
            .unwrap();
        let expected = json!({"id":captures.snapshot_sha256(),"compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.kind()==OriginalInputKind::Source {"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()});
        assert_eq!(request, &expected);
    }
    let path = if matches!(case, "input_limits" | "input_byte_limits") {
        "boundary-input/source-responses.json"
    } else {
        "boundary-attachment/responses.json"
    };
    let rows: Vec<Value> = serde_json::from_slice(
        &fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../develop/migrations/csharp-03")
                .join(path),
        )
        .unwrap(),
    )
    .unwrap();
    let row = rows
        .iter()
        .find(|r| r["id"] == captures.snapshot_sha256())
        .unwrap();
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
fn input_bytes<'a>(
    emitted: &'a EmittedDataPhase,
    raw: &'a [u8],
    document: &'a [u8],
) -> BoundaryInputBytes<'a> {
    BoundaryInputBytes {
        boundary_id: emitted.boundaries()[0]
            .artifact()
            .value()
            .get("boundary_id")
            .unwrap()
            .as_str()
            .unwrap(),
        provenance_id: "test.canonical",
        raw_bytes: raw,
        canonical_document: document,
    }
}
#[test]
fn csharp_03_t05_w02_real_source_input_matrix_and_two_run_identity() {
    let b = bundle();
    let rows = [
        (
            "required",
            vec![
                r#"{"field0":7,"field1":"0"}"#,
                r#"{"field0":-2147483648,"field1":"9223372036854775807"}"#,
            ],
            vec![
                r#"{"field0":"7","field1":"0"}"#,
                r#"{"field0":2147483648,"field1":"0"}"#,
                r#"{"field0":7,"field1":0}"#,
                r#"{"field0":7,"field1":"00"}"#,
                r#"{"field0":null,"field1":"0"}"#,
                r#"{"field1":"0"}"#,
            ],
        ),
        (
            "optional_default",
            vec![r#"{"field1":"0"}"#, r#"{"field0":7,"field1":"0"}"#],
            vec![r#"{"field0":null,"field1":"0"}"#],
        ),
        (
            "raw_instant",
            vec![r#"{"field0":0,"field1":"-9223372036854775808"}"#],
            vec![r#"{"field0":0,"field1":"9223372036854775808"}"#],
        ),
        (
            "nullable_required",
            vec![r#"{"field0":null}"#, r#"{"field0":7}"#],
            vec!["{}", r#"{"field0":{"tag":"none"}}"#],
        ),
        (
            "nullable_default",
            vec!["{}", r#"{"field0":7}"#, r#"{"field0":null}"#],
            vec![r#"{"field0":"7"}"#],
        ),
        (
            "nullable_nonnull",
            vec![r#"{"field0":7}"#],
            vec![r#"{"field0":null}"#],
        ),
        (
            "presence_exposed",
            vec!["{}", r#"{"field0":null}"#, r#"{"field0":7}"#],
            vec![
                r#"{"field0":{"tag":"value","payload":7}}"#,
                r#"{"field0":{"tag":"unknown"}}"#,
            ],
        ),
        (
            "presence_required",
            vec![r#"{"field0":null}"#, r#"{"field0":7}"#],
            vec!["{}"],
        ),
        (
            "presence_default",
            vec!["{}", r#"{"field0":null}"#, r#"{"field0":7}"#],
            vec![r#"{"field0":"7"}"#],
        ),
        (
            "presence_missing_default",
            vec!["{}", r#"{"field0":null}"#],
            vec![r#"{"field0":false}"#],
        ),
        (
            "presence_nonmissing_zero",
            vec![r#"{"field0":null}"#],
            vec!["{}", r#"{"field0":true}"#],
        ),
        (
            "presence_instant",
            vec![r#"{"field0":null}"#, r#"{"field0":"1234"}"#],
            vec!["{}", r#"{"field0":1234}"#],
        ),
        (
            "decimal_default",
            vec![r#"{"field1":"0"}"#, r#"{"field0":"1.250","field1":"0"}"#],
            vec![
                r#"{"field0":"1.25","field1":"0"}"#,
                r#"{"field0":1,"field1":"0"}"#,
            ],
        ),
        (
            "enum_default",
            vec!["{}", r#"{"field0":"0"}"#],
            vec![
                r#"{"field0":"9"}"#,
                r#"{"field0":"Red"}"#,
                r#"{"field0":0}"#,
            ],
        ),
        (
            "surrogate_name",
            vec![r#"{"\ud800":7,"field1":"0"}"#],
            vec![
                r#"{"\uD800":7,"field1":"0"}"#,
                r#"{"\ud800":7,"\ud800":8,"field1":"0"}"#,
            ],
        ),
    ];
    let mut accepted = 0;
    let mut rejected = 0;
    let mut evidence = vec![];
    for (case, good, bad) in rows {
        let (context, captures, emitted) = input_fixture(&b, case);
        for doc in good {
            let input = || input_bytes(&emitted, doc.as_bytes(), doc.as_bytes());
            let first = emitted
                .capture_boundary_input(&b, &context, &captures, input())
                .unwrap_or_else(|e| panic!("{case} {doc}: {e:?}"));
            let second = emitted
                .validate_boundary_input_run(&b, &context, &captures, input(), &first)
                .unwrap();
            assert_eq!(
                first.capture().artifact().canonical_bytes(),
                second.capture().artifact().canonical_bytes()
            );
            assert_eq!(
                first.manifest().canonical_bytes(),
                second.manifest().canonical_bytes()
            );
            assert_eq!(
                first.artifacts().canonical_bytes(),
                second.artifacts().canonical_bytes()
            );
            assert_eq!(
                first.arguments().len(),
                emitted.boundaries()[0].input_fields().len()
            );
            assert_eq!(
                first
                    .manifest()
                    .value()
                    .get("boundary_inputs")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .len(),
                1
            );
            for arg in first.arguments() {
                if let Some(reconstruct) = arg.reconstruction() {
                    assert_eq!(reconstruct.argument_type_ids, [arg.value().type_id()]);
                    assert_eq!(reconstruct.normal_result_type_id, arg.source_type_id());
                    assert_eq!(reconstruct.tag, ClosedOperationTag::BindingReconstruct);
                } else {
                    assert_eq!(arg.source_type_id(), arg.value().type_id());
                }
            }
            assert!(first.obligations().iter().all(|o| !o.discharged));
            evidence.push(json!({"case":case,"source_snapshot_sha256":captures.snapshot_sha256(),"document_utf8":doc,"capture_utf8":std::str::from_utf8(first.capture().artifact().canonical_bytes()).unwrap(),"manifest_sha256":first.manifest().hash(),"artifacts_sha256":first.artifacts().hash(),"arguments":first.arguments().iter().map(|a| json!({"source_type_id":a.source_type_id(),"value":a.value(),"reconstruction":a.reconstruction()})).collect::<Vec<_>>()}));
            accepted += 1;
        }
        for doc in bad {
            assert!(
                emitted
                    .capture_boundary_input(
                        &b,
                        &context,
                        &captures,
                        input_bytes(&emitted, doc.as_bytes(), doc.as_bytes())
                    )
                    .is_err(),
                "{case}: {doc}"
            );
            rejected += 1;
        }
    }
    assert_eq!((accepted, rejected), (29, 28));
    let retained = json!({"work_item":"CSHARP-03-T05-W02","accepted_inputs":accepted,"rejected_inputs":rejected,"source_cases":15,"runs":evidence});
    if let Ok(path) = std::env::var("MPK_W02_INPUT_EVIDENCE_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&retained).unwrap()).unwrap();
    } else {
        let expected: Value = serde_json::from_slice(
            &fs::read(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../develop/migrations/csharp-03/boundary-input/conformance.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(retained, expected);
    }
}
#[test]
fn csharp_03_t05_w02_provenance_linkage_and_byte_parser_cannot_be_bypassed() {
    let b = bundle();
    let (context, captures, emitted) = input_fixture(&b, "optional_default");
    let doc = br#"{"field1":"0"}"#;
    let raw = b"untrusted adapter media";
    let first = emitted
        .capture_boundary_input(&b, &context, &captures, input_bytes(&emitted, raw, doc))
        .unwrap();
    assert_eq!(first.capture().raw_bytes(), raw);
    assert_eq!(first.capture().canonical_document(), doc);
    let explicit = br#"{"field0":7,"field1":"0"}"#;
    let second = emitted
        .capture_boundary_input(
            &b,
            &context,
            &captures,
            input_bytes(&emitted, raw, explicit),
        )
        .unwrap();
    assert_eq!(
        first
            .capture()
            .artifact()
            .value()
            .get("canonical_value_sha256"),
        second
            .capture()
            .artifact()
            .value()
            .get("canonical_value_sha256")
    );
    assert_ne!(
        first.capture().artifact().hash(),
        second.capture().artifact().hash()
    );
    let import = |capture: &[u8], manifest: &[u8], artifacts: &[u8]| {
        emitted.import_boundary_input_run(
            &b,
            &context,
            &captures,
            input_bytes(&emitted, raw, doc),
            BoundaryInputEvidence {
                capture,
                manifest,
                artifacts,
            },
        )
    };
    import(
        first.capture().artifact().canonical_bytes(),
        first.manifest().canonical_bytes(),
        first.artifacts().canonical_bytes(),
    )
    .unwrap();
    for field in [
        "boundary_contract_sha256",
        "canonical_document_utf8_sha256",
        "canonical_value_sha256",
    ] {
        let mut mutated = first.capture().artifact().value().clone();
        set(&mut mutated, field, j(&"0".repeat(64)));
        let PracticalJsonValue::Object(fields) = &mut mutated else {
            panic!()
        };
        fields.pop();
        let preimage = canonical_practical_json_bytes(&mutated).unwrap();
        let mut hash = Sha256::new();
        hash.update(b"MPK-CSHARP-BOUNDARY-INPUT-1.0\0");
        hash.update(preimage);
        let PracticalJsonValue::Object(fields) = &mut mutated else {
            panic!()
        };
        fields.push((
            "capture_sha256".into(),
            j(&format!("{:x}", hash.finalize())),
        ));
        assert!(import(
            &canonical_practical_json_bytes(&mutated).unwrap(),
            first.manifest().canonical_bytes(),
            first.artifacts().canonical_bytes()
        )
        .is_err());
    }
    let mut missing_link = first.manifest().value().clone();
    set(
        &mut missing_link,
        "boundary_inputs",
        PracticalJsonValue::Array(vec![]),
    );
    assert!(import(
        first.capture().artifact().canonical_bytes(),
        &canonical_practical_json_bytes(&missing_link).unwrap(),
        first.artifacts().canonical_bytes()
    )
    .is_err());
    assert!(import(
        first.capture().artifact().canonical_bytes(),
        first.manifest().canonical_bytes(),
        emitted.artifacts().canonical_bytes()
    )
    .is_err());
    let byte_only = build_boundary_input_capture(
        &context,
        &emitted.boundaries()[0].artifact().artifact_ref(),
        "test.canonical",
        raw,
        doc,
    )
    .unwrap();
    assert!(import(
        byte_only.artifact().canonical_bytes(),
        first.manifest().canonical_bytes(),
        first.artifacts().canonical_bytes()
    )
    .is_err());
    let changed_raw = emitted
        .capture_boundary_input(
            &b,
            &context,
            &captures,
            input_bytes(&emitted, b"changed", doc),
        )
        .unwrap();
    assert_ne!(first.manifest().hash(), changed_raw.manifest().hash());
    assert_ne!(first.artifacts().hash(), changed_raw.artifacts().hash());
    assert!(emitted
        .validate_boundary_input_run(
            &b,
            &context,
            &captures,
            input_bytes(&emitted, b"changed", doc),
            &first
        )
        .is_err());
    let mut provenance = input_bytes(&emitted, raw, doc);
    provenance.provenance_id = "test.other";
    assert!(emitted
        .validate_boundary_input_run(&b, &context, &captures, provenance, &first)
        .is_err());
    let mut wrong = input_bytes(&emitted, raw, doc);
    wrong.boundary_id = "other.boundary";
    assert!(emitted
        .capture_boundary_input(&b, &context, &captures, wrong)
        .is_err());
    let (other_context, other_captures, _) = input_fixture(&b, "required");
    assert!(emitted
        .capture_boundary_input(
            &b,
            &other_context,
            &other_captures,
            input_bytes(&emitted, raw, doc)
        )
        .is_err());
    // An adapter object/evidence blob is not a canonical argument document.
    for bad in [
        b"".as_slice(),
        br#"{"field0":7,"field0":8,"field1":"0"}"#,
        br#"{"field1":"0","field0":7}"#,
        br#"{"unknown":0,"field1":"0"}"#,
        br#"{"field1":"0","unknown":0}"#,
        br#"{"field1":"0"} "#,
        br#"{"field1": "0"}"#,
        br#"{"field0":-0,"field1":"0"}"#,
        br#"{"field0":1.0,"field1":"0"}"#,
        br#"{"field0":1e0,"field1":"0"}"#,
        b"\xef\xbb\xbf{}",
        b"\xff",
        first.capture().artifact().canonical_bytes(),
    ] {
        assert!(emitted
            .capture_boundary_input(&b, &context, &captures, input_bytes(&emitted, raw, bad))
            .is_err());
    }
    assert!(validate_contract_artifact(
        &context,
        &captures,
        PracticalArtifactKind::BoundaryInput,
        first.capture().artifact().canonical_bytes()
    )
    .is_err());
    let oversized = vec![b' '; 1_048_577];
    assert_eq!(
        emitted
            .capture_boundary_input(
                &b,
                &context,
                &captures,
                input_bytes(&emitted, raw, &oversized)
            )
            .unwrap_err(),
        BoundaryInputError::Limit
    );
}

#[test]
fn csharp_03_t05_w02_actual_source_aggregate_and_document_limits() {
    use PracticalJsonValue as J;
    let b = bundle();
    if let Ok(path) = std::env::var("MPK_W02_LIMIT_REQUESTS_OUT") {
        let requests = ["input_limits","input_byte_limits"].iter().map(|case| {
            let (context,captures) = setup(&b,case);
            json!({"id":captures.snapshot_sha256(),"compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.kind()==OriginalInputKind::Source {"source"} else {"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()})
        }).collect::<Vec<_>>();
        fs::write(path, serde_json::to_vec(&requests).unwrap()).unwrap();
        return;
    }
    let (context, captures, emitted) = input_fixture(&b, "input_limits");
    let base = || {
        J::Object(
            (0..32)
                .map(|i| {
                    (
                        format!("field{i}"),
                        if i < 16 { J::Array(vec![]) } else { j("") },
                    )
                })
                .collect(),
        )
    };
    for target in [65_535, 65_536, 65_537] {
        let mut doc = base();
        let mut remaining = target - 33;
        for i in 0..16 {
            let n = remaining.min(4096);
            set(&mut doc, &format!("field{i}"), J::Array(vec![J::U64(0); n]));
            remaining -= n;
        }
        assert_eq!(remaining, 0);
        let bytes = canonical_practical_json_bytes(&doc).unwrap();
        let result = emitted.capture_boundary_input(
            &b,
            &context,
            &captures,
            input_bytes(&emitted, &bytes, &bytes),
        );
        assert_eq!(
            result.is_ok(),
            target <= 65_536,
            "cells {target}: {:?}",
            result.as_ref().err()
        );
    }
    let (context, captures, emitted) = input_fixture(&b, "input_byte_limits");
    for target in [1_048_575, 1_048_576, 1_048_577] {
        let mut doc = base();
        let mut remaining = target - canonical_practical_json_bytes(&doc).unwrap().len();
        for i in 0..16 {
            let n = ((remaining + 1) / 23).min(4096);
            if n > 0 {
                set(
                    &mut doc,
                    &format!("field{i}"),
                    J::Array(vec![j("-9223372036854775808"); n]),
                );
                remaining -= 23 * n - 1;
            }
        }
        assert!(remaining < 23);
        set(&mut doc, "field16", j(&"x".repeat(remaining)));
        let bytes = canonical_practical_json_bytes(&doc).unwrap();
        assert_eq!(bytes.len(), target);
        let result = emitted.capture_boundary_input(
            &b,
            &context,
            &captures,
            input_bytes(&emitted, &bytes, &bytes),
        );
        assert_eq!(
            result.is_ok(),
            target <= 1_048_576,
            "bytes {target}: {:?}",
            result.as_ref().err()
        );
    }
}
