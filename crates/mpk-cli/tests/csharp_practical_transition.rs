//! W04 actual-source attachment, pending obligations, and pinned CLR observations.
use mpk_vc::csharp_practical_source_artifacts::*;
use mpk_vc::csharp_practical_vir_model::*;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path};
use PracticalJsonValue as J;
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
    csharp_practical_declaration_id(&json!({"kind":kind,"namespace":"Business","owner":owner,"name":name,"parameter_type_ids":params,"result_type_id":result})).unwrap()
}
fn src(name: &str) -> String {
    decl("type", name, "", &[], "")
}
fn source_ty(name: &str) -> Value {
    json!({"kind":"source","id":src(name)})
}
fn prim(name: &str) -> Value {
    json!({"kind":"primitive","id":name})
}
fn array(t: Value) -> Value {
    json!({"kind":"instance","template":"bounded_sequence","arguments":[t]})
}
fn member(owner: &str, name: &str, t: Value) -> String {
    csharp_practical_stored_member_id(&src(owner), name, &t, "readonly_field").unwrap()
}
fn j(s: &str) -> J {
    J::string(s)
}
fn obj(v: Vec<(&str, J)>) -> J {
    J::object(v)
}
fn set(v: &mut J, key: &str, value: J) {
    let J::Object(fields) = v else { panic!() };
    fields.iter_mut().find(|(k, _)| k == key).unwrap().1 = value;
}
fn hash(domain: &str, mut fields: Vec<(&str, J)>) -> Vec<u8> {
    let mut h = Sha256::new();
    h.update(domain);
    h.update([0]);
    h.update(canonical_practical_json_bytes(&obj(fields.clone())).unwrap());
    fields.push(("contract_sha256", j(&format!("{:x}", h.finalize()))));
    canonical_practical_json_bytes(&obj(fields)).unwrap()
}
fn var(id: &str, t: &str) -> J {
    obj(vec![
        ("tag", j("variable")),
        ("type_id", j(t)),
        ("binding_id", j(id)),
    ])
}
fn literal(t: &str, value: J) -> J {
    obj(vec![
        ("tag", j("literal")),
        ("type_id", j(&ty(t))),
        ("value", value),
    ])
}
fn field(v: J, owner: &str, name: &str, t: &str) -> J {
    obj(vec![
        ("tag", j("field")),
        ("type_id", j(&ty(t))),
        ("receiver", v),
        ("member_id", j(&member(owner, name, prim(t)))),
    ])
}
fn bin(op: &str, t: &str, left: J, right: J) -> J {
    obj(vec![
        ("tag", j("binary")),
        ("type_id", j(&ty(t))),
        ("operation_id", j(op)),
        ("left", left),
        ("right", right),
    ])
}
fn eq(t: &str, l: J, r: J) -> J {
    bin(&format!("integer.{t}.equal.checked"), "bool", l, r)
}
fn and(l: J, r: J) -> J {
    bin("boolean.and", "bool", l, r)
}
fn amount() -> J {
    field(var("command", &src("Command")), "Command", "Amount", "i32")
}
fn balance(v: &str) -> J {
    field(var(v, &src("State")), "State", "Balance", "i32")
}
fn cases() -> Vec<(&'static str, bool)> {
    vec![
        ("valid", true),
        ("negative_time", true),
        ("mutant_invariant", true),
        ("mutant_version", true),
        ("mutant_event", true),
        ("mutant_response", true),
        ("mutant_precedence", true),
        ("binding_hash", false),
        ("transition_binding", false),
        ("error_binding", false),
        ("state_type", false),
        ("signature", false),
        ("instance_signature", false),
        ("not_apply", false),
        ("partial", false),
        ("missing_method", false),
        ("version_type", false),
        ("increment", false),
        ("time_type", false),
        ("time_codec", false),
        ("time_extra", false),
        ("invariant_scope", false),
        ("invariant_type", false),
        ("events_type", false),
        ("command_duplicate", false),
        ("command_empty", false),
        ("command_scope", false),
        ("error_order", false),
        ("error_duplicate", false),
        ("error_missing", false),
        ("error_unknown", false),
        ("error_condition", false),
        ("extra", false),
        ("idempotency", false),
        ("effect_clock", false),
        ("effect_identity", false),
        ("effect_io", false),
    ]
}
fn setup(
    b: &ValidatedFoundationBundle,
    case: &str,
) -> (PracticalArtifactContext, CapturedInputSet) {
    setup_variant(
        b,
        case,
        include_str!("../../../develop/migrations/csharp-03/transition/Entry.cs").to_owned(),
        vec![],
        |_, _, rows| rows,
    )
}
pub(crate) fn setup_variant(
    b: &ValidatedFoundationBundle,
    case: &str,
    mut source: String,
    extra_paths: Vec<String>,
    amend: impl FnOnce(&PracticalArtifactContext, &str, Vec<Vec<u8>>) -> Vec<Vec<u8>>,
) -> (PracticalArtifactContext, CapturedInputSet) {
    match case {
        "mutant_invariant" => source = source.replace("state.Balance + command.Amount", "-1"),
        "mutant_version" => source = source.replace("state.Version + 1UL", "state.Version + 2UL"),
        "mutant_event" => {
            source = source.replace(
                "new Event(command.Amount, context.Effective)",
                "new Event(command.Amount + 1, context.Effective)",
            )
        }
        "mutant_response" => {
            source = source.replace(
                "new Response(next.Balance)",
                "new Response(next.Balance + 1)",
            )
        }
        "mutant_precedence" => {
            source = source.replace(
                "if (state.Version != command.Expected)",
                "if (state.Version != command.Expected && command.Amount >= 0)",
            )
        }
        "effect_clock" => source = source.replace("context.Effective", "System.DateTime.Now.Ticks"),
        "effect_identity" => {
            source = source.replace("context.Effective", "System.Guid.NewGuid().GetHashCode()")
        }
        "effect_io" => {
            source = source.replace(
                "context.Effective",
                "System.IO.File.ReadAllText(\"input\").Length",
            )
        }
        _ => {}
    }
    if case == "instance_signature" {
        let start = source.find("    public static ApplyResult Apply").unwrap();
        let end = source.find("    // The pinned CLR harness").unwrap();
        let apply = source[start..end].replace(
            "public static ApplyResult Apply(State state, Command command, Context context) {",
            "public ApplyResult Apply(Command command, Context context) { State state = this;",
        );
        source.replace_range(start..end, "");
        let close = source.find("\n}").unwrap();
        source.insert_str(close, &format!("\n{apply}"));
        source = source.replace("Apply(state, new Command", "state.Apply(new Command");
    }
    let apply = decl(
        "method",
        "Apply",
        &src(if case == "instance_signature" {
            "State"
        } else {
            "Entry"
        }),
        &if case == "instance_signature" {
            vec![src("Command"), src("Context")]
        } else {
            vec![src("State"), src("Command"), src("Context")]
        },
        &src("ApplyResult"),
    );
    let run = decl(
        "method",
        "Run",
        &src("Entry"),
        &[
            ty("i32"),
            csharp_practical_closed_instance_id(b, &array(prim("i32"))).unwrap(),
            ty("string"),
        ],
        &ty("i32"),
    );
    let mut roots = vec![apply.clone(), run.clone()];
    roots.sort();
    let mut paths = vec![
        "contracts/bindings.json".into(),
        "contracts/method.json".into(),
        "contracts/transition.json".into(),
    ];
    paths.extend(extra_paths);
    support::context_with_sidecars_for_roots(b, &roots, source.as_bytes(), paths.clone(), |ctx| {
        let sha = format!("{:x}", Sha256::digest(source.as_bytes()));
        let pre = capture_original_inputs(
            ctx,
            std::iter::once(OriginalInput {
                kind: OriginalInputKind::Source,
                path: "src/Entry.cs".into(),
                bytes: source.as_bytes().to_vec(),
            })
            .chain(paths.iter().map(|p| OriginalInput {
                kind: OriginalInputKind::Sidecar,
                path: p.clone(),
                bytes: vec![],
            }))
            .collect(),
        )
        .unwrap();
        let transition_type=csharp_practical_closed_instance_id(b,&json!({"kind":"instance","template":"transition","arguments":[source_ty("State"),source_ty("Event"),source_ty("Response")]})).unwrap();
        let binding = |name: &str,
                       role: &str,
                       members: Vec<(&str, &str, Value)>,
                       args: Vec<String>| SemanticBindingInput {
            source_type_id: src(name),
            source_content_sha256: sha.clone(),
            role: role.into(),
            member_map: members
                .into_iter()
                .map(|(role, field, t)| SemanticBindingMember {
                    role: role.into(),
                    member_id: member(name, field, t),
                })
                .collect(),
            tag_arms: if role == "result" {
                vec![
                    SemanticArmMapping {
                        semantic_arm: "ok".into(),
                        source_tag: "0".into(),
                    },
                    SemanticArmMapping {
                        semantic_arm: "error".into(),
                        source_tag: "1".into(),
                    },
                ]
            } else if role == "boundary_field" {
                ["missing", "null", "value"]
                    .iter()
                    .enumerate()
                    .map(|(i, arm)| SemanticArmMapping {
                        semantic_arm: (*arm).into(),
                        source_tag: i.to_string(),
                    })
                    .collect()
            } else {
                vec![]
            },
            inferred_argument_ids: args,
            default_arm: "ineligible".into(),
            bounds: if role == "transition" {
                vec![SemanticBound {
                    id: "events".into(),
                    maximum: 4096,
                }]
            } else {
                vec![]
            },
            operation_map: vec![],
            enum_arms: BTreeMap::new(),
        };
        let mut binding_rows = vec![
            binding(
                "Change",
                "transition",
                vec![
                    ("state", "State", source_ty("State")),
                    ("events", "Events", array(source_ty("Event"))),
                    ("response", "Response", source_ty("Response")),
                ],
                vec![src("State"), src("Event"), src("Response")],
            ),
            binding(
                "ApplyResult",
                "result",
                vec![
                    ("tag", "Tag", source_ty("Tag")),
                    ("value", "Value", source_ty("Change")),
                    ("error", "Error", source_ty("DomainError")),
                ],
                vec![transition_type, src("DomainError")],
            ),
        ];
        if source.contains("enum PresenceTag") {
            binding_rows.push(binding(
                "Presence",
                "boundary_field",
                vec![
                    ("tag", "Tag", source_ty("PresenceTag")),
                    ("value", "Value", prim("i32")),
                ],
                vec![ty("i32")],
            ));
        }
        let bindings = build_semantic_bindings(ctx, &pre, binding_rows).unwrap();
        let binding_id = |name: &str| {
            format!(
                "binding.{}",
                bindings
                    .value()
                    .get("bindings")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| r.get("source_type_id").unwrap().as_str() == Some(src(name).as_str()))
                    .unwrap()
                    .get("binding_sha256")
                    .unwrap()
                    .as_str()
                    .unwrap()
            )
        };
        let method = hash(
            "MPK-CSHARP-METHOD-CONTRACT-1.0",
            vec![
                ("schema", j(METHOD_CONTRACT_SCHEMA)),
                ("semantic_context", ctx.semantic_context().clone()),
                ("compilation_id", j(ctx.compilation_id())),
                (
                    "callable_id",
                    j(if case == "missing_method" {
                        &run
                    } else {
                        &apply
                    }),
                ),
                ("source_content_sha256", j(&sha)),
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
        let mut invariant = bin(
            "integer.i32.greater_equal.checked",
            "bool",
            balance("state"),
            literal("i32", j("0")),
        );
        let mut commands = vec![obj(vec![
            ("case_id", j("deposit")),
            ("condition", literal("bool", J::Bool(true))),
        ])];
        let mut errors = vec![
            obj(vec![
                ("code", j("version_conflict")),
                ("source_tag", j("0")),
                ("condition", J::Null),
            ]),
            obj(vec![
                ("code", j("version_exhausted")),
                ("source_tag", j("1")),
                ("condition", J::Null),
            ]),
            obj(vec![
                ("code", j("negative")),
                ("source_tag", j("2")),
                (
                    "condition",
                    bin(
                        "integer.i32.less.checked",
                        "bool",
                        amount(),
                        literal("i32", j("0")),
                    ),
                ),
            ]),
        ];
        let mut time = obj(vec![
            ("member_id", j(&member("Context", "Effective", prim("i64")))),
            ("codec_id", j("unix_milliseconds")),
        ]);
        let mut rule = obj(vec![
            (
                "state_member_id",
                j(&member("State", "Version", prim("u64"))),
            ),
            (
                "expected_member_id",
                j(&member("Command", "Expected", prim("u64"))),
            ),
            ("increment", j("checked_u64_one")),
            ("effective_time", time.clone()),
        ]);
        match case {
            "version_type" => set(
                &mut rule,
                "state_member_id",
                j(&member("State", "Balance", prim("i32"))),
            ),
            "increment" => set(&mut rule, "increment", j("unchecked_u64_one")),
            "time_type" => set(
                &mut time,
                "member_id",
                j(&member("Command", "Expected", prim("u64"))),
            ),
            "time_codec" => set(&mut time, "codec_id", j("date")),
            "time_extra" => {
                let J::Object(v) = &mut time else { panic!() };
                v.push(("extra".into(), J::Null));
            }
            "invariant_scope" => invariant = eq("i32", amount(), literal("i32", j("0"))),
            "invariant_type" => invariant = balance("state"),
            "command_duplicate" => commands.push(commands[0].clone()),
            "command_empty" => commands.clear(),
            "command_scope" => set(
                &mut commands[0],
                "condition",
                eq("i32", balance("next_state"), literal("i32", j("0"))),
            ),
            "error_order" => errors.swap(0, 1),
            "error_duplicate" => set(&mut errors[2], "source_tag", j("0")),
            "error_missing" => {
                errors.pop();
            }
            "error_unknown" => set(&mut errors[2], "source_tag", j("9")),
            "error_condition" => set(&mut errors[0], "condition", literal("bool", J::Bool(true))),
            _ => {}
        }
        set(&mut rule, "effective_time", time);
        let seq = csharp_practical_closed_instance_id(b, &array(source_ty("Event"))).unwrap();
        let events = var("events", &seq);
        let element = |i: &str| {
            obj(vec![
                ("tag", j("sequence_index")),
                ("type_id", j(&src("Event"))),
                ("sequence", events.clone()),
                ("index", literal("i32", j(i))),
            ])
        };
        let mut event_relation = and(
            eq(
                "i32",
                obj(vec![
                    ("tag", j("sequence_length")),
                    ("type_id", j(&ty("i32"))),
                    ("sequence", events.clone()),
                ]),
                literal("i32", j("2")),
            ),
            and(
                eq(
                    "i32",
                    field(element("0"), "Event", "Amount", "i32"),
                    amount(),
                ),
                and(
                    eq(
                        "i32",
                        field(element("1"), "Event", "Amount", "i32"),
                        literal("i32", j("0")),
                    ),
                    eq(
                        "i64",
                        field(element("0"), "Event", "Effective", "i64"),
                        field(
                            var("context", &src("Context")),
                            "Context",
                            "Effective",
                            "i64",
                        ),
                    ),
                ),
            ),
        );
        if case == "events_type" {
            event_relation = eq("i32", var("events", &ty("i32")), literal("i32", j("0")));
        }
        let response_relation = eq(
            "i32",
            field(
                var("response", &src("Response")),
                "Response",
                "Balance",
                "i32",
            ),
            balance("next_state"),
        );
        let mut fields = vec![
            ("schema", j(TRANSITION_CONTRACT_SCHEMA)),
            ("semantic_context", ctx.semantic_context().clone()),
            ("compilation_id", j(ctx.compilation_id())),
            ("transition_id", j("transition.deposit")),
            (
                "selected_callable_id",
                j(if case == "not_apply" { &run } else { &apply }),
            ),
            (
                "state_type_id",
                j(&src(if case == "state_type" {
                    "Command"
                } else {
                    "State"
                })),
            ),
            (
                "command_type_id",
                j(&src(if case == "signature" {
                    "Context"
                } else {
                    "Command"
                })),
            ),
            ("context_type_id", j(&src("Context"))),
            (
                "apply_result_binding_id",
                j(&binding_id(if case == "binding_hash" {
                    "Change"
                } else {
                    "ApplyResult"
                })),
            ),
            (
                "transition_binding_id",
                j(&binding_id(if case == "transition_binding" {
                    "ApplyResult"
                } else {
                    "Change"
                })),
            ),
            (
                "domain_error_binding_id",
                j(&src(if case == "error_binding" {
                    "Tag"
                } else {
                    "DomainError"
                })),
            ),
            ("state_invariant", invariant),
            ("version_rule", rule),
            (
                "idempotency",
                obj(vec![(
                    "mode",
                    j(if case == "idempotency" {
                        "complete_snapshot"
                    } else {
                        "disabled"
                    }),
                )]),
            ),
            ("accepted_commands", J::Array(commands)),
            ("event_relation", event_relation),
            ("response_relation", response_relation),
            ("errors", J::Array(errors)),
        ];
        if case == "extra" {
            fields.push(("extra", J::Null));
        }
        amend(
            ctx,
            &sha,
            vec![
                bindings.canonical_bytes().to_vec(),
                method,
                hash("MPK-CSHARP-TRANSITION-CONTRACT-1.0", fields),
            ],
        )
    })
}
fn runs() -> Vec<Value> {
    [
        [3, 3, 4, 10, 123],
        [3, 2, -1, 10, 123],
        [-1, -1, -1, 10, 123],
        [3, 3, -1, 10, 123],
        [0, 0, 0, 0, -1],
        [3, 3, 4, 10, -123],
    ]
    .into_iter()
    .flat_map(|a| (0..11).map(move |n| json!({"n":n,"a":a,"s":""})))
    .collect()
}
#[test]
fn csharp_03_t05_w04_actual_source_matrix_and_finite_clr_traces() {
    let b = bundle();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let generation = std::env::var("MPK_TRANSITION_REQUESTS_OUT").ok();
    let responses: Vec<Value> = if generation.is_some() {
        vec![]
    } else {
        serde_json::from_slice(
            &fs::read(root.join("develop/migrations/csharp-03/transition/responses.json")).unwrap(),
        )
        .unwrap()
    };
    let mut requests = vec![];
    let mut counterexamples = vec![];
    for (case, expected) in cases() {
        let (context, captures) = setup(&b, case);
        let mut request = json!({"case":case,"expected_attachment":expected,"id":captures.snapshot_sha256(),"compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.kind()==OriginalInputKind::Source{"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()});
        if expected {
            request["runs"] = json!(runs());
        }
        requests.push(request);
        if generation.is_some() {
            continue;
        }
        let row = responses
            .iter()
            .find(|r| r["id"] == captures.snapshot_sha256())
            .unwrap_or_else(|| panic!("missing {case}"));
        if row.get("facts").is_none() {
            assert!(!expected, "{case}: {row}");
            assert!(case.starts_with("effect_"), "{case}: {row}");
            assert_eq!(row["artifact_count"], 0);
            assert_eq!(
                row["reject"],
                if case == "effect_identity" {
                    "CSHARP_PRACTICAL_TYPE/framework_api"
                } else {
                    "CSHARP_PRACTICAL_EFFECT/external_effect_or_concurrency"
                }
            );
            continue;
        }
        let source = ValidatedDataSource::import_captured_facts(
            &b,
            &context,
            &captures,
            &serde_json::to_vec(&row["facts"]).unwrap(),
        )
        .unwrap_or_else(|e| panic!("{case}: {e:?}"));
        let emitted = emit_data_phase(&b, &context, &captures, &source);
        assert_eq!(
            emitted.is_ok(),
            expected,
            "{case}: {:?}",
            emitted.as_ref().err()
        );
        if case == "instance_signature" {
            assert_eq!(
                emitted.as_ref().err(),
                Some(&DataPhaseError::Transition(TransitionError::Method))
            );
        }
        let Ok(emitted) = emitted else {
            assert!(
                matches!(
                    emitted.as_ref().err(),
                    Some(
                        DataPhaseError::Transition(_)
                            | DataPhaseError::Sidecar
                            | DataPhaseError::Contract
                            | DataPhaseError::LaterOwner("CSHARP-03-T05-W05")
                    )
                ),
                "unexpected rejection owner for {case}: {:?}",
                emitted.as_ref().err()
            );
            continue;
        };
        let replay = emit_data_phase(&b, &context, &captures, &source).unwrap();
        assert_eq!(
            emitted.vir().canonical_bytes(),
            replay.vir().canonical_bytes()
        );
        assert_eq!(
            emitted.manifest().canonical_bytes(),
            replay.manifest().canonical_bytes()
        );
        assert_eq!(emitted.transitions().len(), 1);
        let plan = &emitted.transitions()[0];
        assert!(plan.idempotency().is_none());
        assert_eq!(
            plan.check_order(),
            &[
                TransitionCheck::ExpectedVersion,
                TransitionCheck::VersionExhaustion,
                TransitionCheck::BusinessErrors,
                TransitionCheck::NewSuccess
            ]
        );
        assert_eq!(plan.state_type_id(), src("State"));
        assert_eq!(plan.command_type_id(), src("Command"));
        assert_eq!(plan.context_type_id(), src("Context"));
        assert_eq!(plan.event_type_id(), src("Event"));
        assert_eq!(plan.response_type_id(), src("Response"));
        assert_eq!(plan.error_type_id(), src("DomainError"));
        assert!(plan.version_rule().is_raw_instant());
        assert_eq!(plan.version_rule().increment(), 1);
        assert_eq!(plan.maximum_events(), 4096);
        assert_eq!(
            plan.errors().iter().map(|e| e.code()).collect::<Vec<_>>(),
            ["version_conflict", "version_exhausted", "negative"]
        );
        assert_eq!(plan.obligations().len(), 12);
        assert!(plan.obligations().iter().all(|o| !o.discharged()));
        for error in plan.errors() {
            assert!(plan.obligations().iter().any(|o| o.path()
                == Some(&TransitionPath::Error(error.code().into()))
                && o.kind() == &TransitionObligationKind::UnchangedInputState));
        }
        for artifact in [emitted.manifest(), emitted.artifacts()] {
            assert_eq!(
                artifact
                    .value()
                    .get("transition_contracts")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .len(),
                1
            );
        }
        let actual = row["runs"].as_array().unwrap();
        assert_eq!(actual.len(), runs().len());
        verify_observed_obligations(case, plan, actual);
        let mut mismatches = 0;
        for (input, observed) in runs().iter().zip(actual) {
            assert_eq!(observed["n"], input["n"]);
            assert_eq!(observed["a"], input["a"]);
            assert_eq!(observed["error"], "", "{case}: {observed}");
            let a = input["a"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap())
                .collect::<Vec<_>>();
            let error = if a[0] != a[1] {
                Some(0)
            } else if a[0] < 0 {
                Some(1)
            } else if a[2] < 0 {
                Some(2)
            } else {
                None
            };
            let n = input["n"].as_i64().unwrap();
            let expected = match n {
                0 => i64::from(error.is_some()),
                1 => error.unwrap_or(0),
                8 => a[3],
                9 => a[0],
                _ if error.is_some() => -1,
                2 => a[0] + 1,
                3 | 7 => a[3] + a[2],
                4 => 2,
                5 => a[2],
                6 => a[4],
                10 => 0,
                _ => unreachable!(),
            };
            if observed["value"] != expected {
                mismatches += 1;
            }
        }
        if case.starts_with("mutant_") {
            assert!(mismatches > 0, "counterexample not observed: {case}");
            counterexamples.push(case);
        } else {
            assert_eq!(mismatches, 0, "{case}");
        }
    }
    if let Some(path) = generation {
        fs::write(path, serde_json::to_vec(&requests).unwrap()).unwrap();
        return;
    }
    assert_eq!(counterexamples.len(), 5);
}

// Finite observation oracle for the expression subset used by this fixture.
// This is test-only: no production evaluation, proof, or certificate authority.
fn observe_expression(e: &J, scope: &BTreeMap<String, Value>) -> Value {
    let text = |key| e.get(key).unwrap().as_str().unwrap();
    let child = |key| observe_expression(e.get(key).unwrap(), scope);
    match text("tag") {
        "variable" => scope[text("binding_id")].clone(),
        "literal" => match e.get("value").unwrap() {
            J::Bool(v) => json!(v),
            v => json!(v.as_str().unwrap().parse::<i64>().unwrap()),
        },
        "field" => child("receiver")[text("member_id")].clone(),
        "sequence_length" => json!(child("sequence").as_array().unwrap().len()),
        "sequence_index" => child("sequence")[child("index").as_u64().unwrap() as usize].clone(),
        "binary" => {
            let left = child("left");
            let right = child("right");
            match text("operation_id") {
                "boolean.and" => json!(left.as_bool().unwrap() && right.as_bool().unwrap()),
                "integer.i32.equal.checked" | "integer.i64.equal.checked" => json!(left == right),
                "integer.i32.less.checked" => {
                    json!(left.as_i64().unwrap() < right.as_i64().unwrap())
                }
                "integer.i32.greater_equal.checked" => {
                    json!(left.as_i64().unwrap() >= right.as_i64().unwrap())
                }
                op => panic!("unexpected fixture operation {op}"),
            }
        }
        tag => panic!("unexpected fixture expression {tag}"),
    }
}
fn verify_observed_obligations(case: &str, plan: &ValidatedTransitionContract, actual: &[Value]) {
    let mut violations = std::collections::BTreeSet::new();
    for trace in actual.chunks_exact(11) {
        let values: Vec<i64> = trace.iter().map(|r| r["value"].as_i64().unwrap()).collect();
        let a: Vec<i64> = trace[0]["a"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_i64().unwrap())
            .collect();
        // Complete original source State has exactly these two fields.
        assert_eq!(values[8], a[3]);
        assert_eq!(values[9], a[0]);
        if values[0] != 0 {
            continue;
        }
        let scope = BTreeMap::from([
            (
                "state".into(),
                json!({member("State","Balance",prim("i32")):a[3]}),
            ),
            (
                "command".into(),
                json!({member("Command","Amount",prim("i32")):a[2]}),
            ),
            (
                "context".into(),
                json!({member("Context","Effective",prim("i64")):a[4]}),
            ),
            (
                "next_state".into(),
                json!({member("State","Balance",prim("i32")):values[3]}),
            ),
            (
                "response".into(),
                json!({member("Response","Balance",prim("i32")):values[7]}),
            ),
            (
                "events".into(),
                json!([
                    {member("Event","Amount",prim("i32")):values[5],member("Event","Effective",prim("i64")):values[6]},
                    {member("Event","Amount",prim("i32")):values[10],member("Event","Effective",prim("i64")):values[6]}
                ]),
            ),
        ]);
        for obligation in plan.obligations() {
            if let Some(predicate) = obligation.predicate() {
                if observe_expression(predicate.value(), &scope) != true {
                    violations.insert(format!("{:?}", obligation.kind()));
                }
            }
        }
        if values[2] != a[0] + 1 {
            violations.insert("CheckedVersionIncrement".into());
        }
        assert_eq!(values[4], 2);
    }
    let expected = match case {
        "mutant_invariant" => vec!["NewStateInvariant"],
        "mutant_version" => vec!["CheckedVersionIncrement"],
        "mutant_event" => vec!["EventRelation"],
        "mutant_response" => vec!["ResponseRelation"],
        _ => vec![],
    };
    assert_eq!(
        violations.into_iter().collect::<Vec<_>>(),
        expected,
        "{case}"
    );
}

#[test]
fn csharp_03_t05_w04_independent_import_requires_original_transition_contract() {
    use mpk_vc::csharp_practical_vir_validation as v;
    if std::env::var_os("MPK_TRANSITION_REQUESTS_OUT").is_some() {
        return;
    }
    let b = bundle();
    let (context, captures) = setup(&b, "valid");
    let rows: Vec<Value> = serde_json::from_slice(
        &fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../develop/migrations/csharp-03/transition/responses.json"),
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
        let mut bad = valid.clone();
        match mutation {
            0 => bad
                .data_contracts
                .retain(|s| !s.contains(TRANSITION_CONTRACT_SCHEMA)),
            1 => {
                let contract = bad
                    .data_contracts
                    .iter_mut()
                    .find(|s| s.contains(TRANSITION_CONTRACT_SCHEMA))
                    .unwrap();
                *contract = contract.replace("transition.deposit", "transition.other");
            }
            _ => bad.binding_projections.clear(),
        }
        if let Ok(bytes) = v::canonical_csharp_practical_vir_transport(input, bad) {
            assert!(v::import_csharp_practical_vir_json(&bytes, input).is_err());
        }
    }
    let (other_context, other_captures) = setup(&b, "mutant_version");
    assert!(ValidatedDataSource::import_captured_facts(
        &b,
        &other_context,
        &other_captures,
        source.captured_facts()
    )
    .is_err());
}

#[path = "support/csharp_practical_idempotency.rs"]
mod idempotency;
