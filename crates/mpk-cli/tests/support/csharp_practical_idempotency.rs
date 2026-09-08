//! Actual captured W05 sources and sidecars; obligations stay unproved.
use super::*;
fn cases() -> Vec<(&'static str, bool)> {
    vec![
        ("valid", true),
        ("omitted_context_body", true),
        ("history_member", false),
        ("command_projection", false),
        ("context_projection", false),
        ("response_member", false),
        ("key_member", false),
        ("duplicate_record", false),
        ("digest", false),
        ("projection", false),
        ("capacity_override", false),
        ("helper_missing", false),
        ("helper_unreachable", false),
        ("helper_signature", false),
        ("helper_partial", false),
        ("helper_contract_missing", false),
        ("error_order", false),
        ("error_missing", false),
        ("float_direct", false),
        ("double_nested", false),
        ("float_empty_array", false),
    ]
}
fn read(name: &str) -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/idempotency")
            .join(name),
    )
    .unwrap()
}
fn parsed(bytes: &[u8]) -> J {
    serde_json::from_slice(bytes).unwrap()
}
fn rehash(domain: &str, v: J) -> Vec<u8> {
    let J::Object(mut f) = v else { panic!() };
    f.retain(|(k, _)| k != "contract_sha256");
    hash(
        domain,
        f.iter().map(|(k, v)| (k.as_str(), v.clone())).collect(),
    )
}
fn helper() -> String {
    decl(
        "method",
        "SameSnapshot",
        &src("Entry"),
        &[
            src("Command"),
            src("Context"),
            src("Command"),
            src("Context"),
        ],
        &ty("bool"),
    )
}
fn setup(
    b: &ValidatedFoundationBundle,
    case: &str,
) -> (PracticalArtifactContext, CapturedInputSet) {
    let mut source =
        include_str!("../../../../develop/migrations/csharp-03/idempotency/Entry.cs").to_owned();
    match case {
        "omitted_context_body" => source = source.replace(" && x.Effective == y.Effective", ""),
        "helper_unreachable" => source = source.replace("if (SameSnapshot(old.Command, old.Context, command, context))", "if (context.Effective == 0)").replace("if (n == 0)", "if (SameSnapshot(new Command(expected, a[2], a[5], new Presence(tag, a[7])), new Context((long)a[4]), new Command(expected, a[2], a[5], new Presence(tag, a[7])), new Context((long)a[4])) && n == 0)"),
        "float_empty_array" => source = source.replace("public readonly long Effective;\n    public Context(long effective) { Effective = effective; }", "public readonly long Effective;\n    public readonly float[] Extras;\n    public Context(long effective) { Effective = effective; Extras = new float[0]; }"),
        _ => {}
    }
    // Direct float on an extra field avoids changing arithmetic/result types.
    if case == "float_direct" {
        source = include_str!("../../../../develop/migrations/csharp-03/idempotency/Entry.cs").replace("public readonly long Effective;\n    public Context(long effective) { Effective = effective; }", "public readonly long Effective;\n    public readonly float Extra;\n    public Context(long effective) { Effective = effective; Extra = 0.0f; }");
    }
    if case == "double_nested" {
        // Keep the boundary_field<i32> projection, while adding inactive storage.
        source = include_str!("../../../../develop/migrations/csharp-03/idempotency/Entry.cs").replace("public readonly int Value;\n    public Presence", "public readonly int Value;\n    public readonly double Extra;\n    public Presence").replace("{ Tag = tag; Value = value; }", "{ Tag = tag; Value = value; Extra = 0.0d; }");
    }
    let loops: Value = if std::env::var_os("MPK_IDEMPOTENCY_BOOTSTRAP").is_some() {
        json!({})
    } else {
        serde_json::from_slice(&read("loops.json")).unwrap()
    };
    setup_variant(
        b,
        "w05",
        source,
        if case == "helper_contract_missing" {
            vec!["contracts/zzrun.json".into()]
        } else {
            vec![
                "contracts/zzhelper.json".into(),
                "contracts/zzrun.json".into(),
            ]
        },
        |ctx, _sha, mut rows| {
            let mut method = parsed(&rows[1]);
            let apply = method
                .get("callable_id")
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned();
            let run = ctx
                .selected_root_ids()
                .iter()
                .find(|id| **id != apply)
                .unwrap();
            let loop_rows = |id: &str| -> J {
                let Some(v) = loops.get(id) else {
                    return J::Array(vec![]);
                };
                J::Array(
                    v.as_array()
                        .unwrap()
                        .iter()
                        .map(|l| {
                            obj(vec![
                                ("loop_id", j(l["loop_id"].as_str().unwrap())),
                                ("invariants", J::Array(vec![literal("bool", J::Bool(true))])),
                                (
                                    "modifies",
                                    J::Array(
                                        l["modifies"]
                                            .as_array()
                                            .unwrap()
                                            .iter()
                                            .map(|s| j(s.as_str().unwrap()))
                                            .collect(),
                                    ),
                                ),
                                (
                                    "decreases",
                                    J::Array(vec![bin(
                                        "integer.i32.subtract.checked",
                                        "i32",
                                        literal("i32", j("4096")),
                                        var(l["index"].as_str().unwrap(), &ty("i32")),
                                    )]),
                                ),
                            ])
                        })
                        .collect(),
                )
            };
            set(&mut method, "loops", loop_rows(&apply));
            rows[1] = rehash("MPK-CSHARP-METHOD-CONTRACT-1.0", method.clone());
            set(&mut method, "callable_id", j(&helper()));
            set(&mut method, "loops", J::Array(vec![]));
            if case == "helper_partial" {
                set(&mut method, "termination", j("partial"));
            }
            if case == "helper_contract_missing" {
                set(&mut method, "callable_id", j(run));
                set(&mut method, "loops", loop_rows(run));
            }
            rows.push(rehash("MPK-CSHARP-METHOD-CONTRACT-1.0", method.clone()));
            set(&mut method, "callable_id", j(run));
            set(&mut method, "termination", j("total"));
            set(&mut method, "loops", loop_rows(run));
            rows.push(rehash("MPK-CSHARP-METHOD-CONTRACT-1.0", method));
            // Omit the helper sidecar entirely from the captured input set.
            if case == "helper_contract_missing" {
                rows.remove(3);
            }
            let mut contract = parsed(&rows[2]);
            let mut spec = obj(vec![
                ("mode", j("complete_snapshot")),
                (
                    "history_member_id",
                    j(&member("State", "History", array(source_ty("Processed")))),
                ),
                (
                    "command_key_member_id",
                    j(&member("Command", "Key", prim("i32"))),
                ),
                (
                    "record_key_member_id",
                    j(&member("Processed", "Key", prim("i32"))),
                ),
                (
                    "record_command_member_id",
                    j(&member("Processed", "Command", source_ty("Command"))),
                ),
                (
                    "record_context_member_id",
                    j(&member("Processed", "Context", source_ty("Context"))),
                ),
                (
                    "record_response_member_id",
                    j(&member("Processed", "Response", source_ty("Response"))),
                ),
                ("equality_callable_id", j(&helper())),
            ]);
            match case {
                "history_member" => set(
                    &mut spec,
                    "history_member_id",
                    j(&member("State", "Balance", prim("i32"))),
                ),
                "command_projection" => set(
                    &mut spec,
                    "record_command_member_id",
                    j(&member("Processed", "Key", prim("i32"))),
                ),
                "context_projection" => set(
                    &mut spec,
                    "record_context_member_id",
                    j(&member("Processed", "Command", source_ty("Command"))),
                ),
                "response_member" => set(
                    &mut spec,
                    "record_response_member_id",
                    j(&member("Processed", "Context", source_ty("Context"))),
                ),
                "key_member" => set(
                    &mut spec,
                    "command_key_member_id",
                    j(&member("Command", "Expected", prim("u64"))),
                ),
                "duplicate_record" => {
                    let duplicate = spec.get("record_command_member_id").unwrap().clone();
                    set(&mut spec, "record_response_member_id", duplicate);
                }
                "helper_missing" => set(&mut spec, "equality_callable_id", j("missing")),
                "helper_signature" => set(&mut spec, "equality_callable_id", j(run)),
                "digest" | "projection" | "capacity_override" => {
                    let J::Object(f) = &mut spec else { panic!() };
                    f.push((
                        match case {
                            "digest" => "collision_resistance_assumption",
                            "projection" => "snapshot_fields",
                            _ => "maximum_history",
                        }
                        .into(),
                        j("caller_selected"),
                    ));
                }
                _ => {}
            }
            set(&mut contract, "idempotency", spec);
            let J::Array(old_errors) = contract.get("errors").unwrap() else {
                panic!()
            };
            let fixed = |code, tag| {
                obj(vec![
                    ("code", j(code)),
                    ("source_tag", j(tag)),
                    ("condition", J::Null),
                ])
            };
            let mut errors = vec![
                fixed("idempotency_conflict", "3"),
                old_errors[0].clone(),
                fixed("history_capacity", "4"),
                old_errors[1].clone(),
                old_errors[2].clone(),
            ];
            if case == "error_order" {
                errors.swap(1, 2);
            }
            if case == "error_missing" {
                errors.remove(2);
            }
            set(&mut contract, "errors", J::Array(errors));
            rows[2] = rehash("MPK-CSHARP-TRANSITION-CONTRACT-1.0", contract);
            rows
        },
    )
}
fn inputs() -> Vec<Vec<i32>> {
    let base = vec![3, 3, 4, 10, 123, 7, 0, 0, 0, 7, 3, 4, 123, 0, 0];
    let mut result = vec![base.clone()];
    let changed = |changes: &[(usize, i32)]| {
        let mut a = base.clone();
        for &(i, v) in changes {
            a[i] = v;
        }
        a
    };
    // Replay wins over stale versions, capacity, exhaustion and business error.
    result.push(changed(&[(8, 1), (0, 4)]));
    result.push(changed(&[(8, 4096), (0, -1)]));
    result.push(changed(&[(8, 3), (5, 9), (0, 4)]));
    result.push(changed(&[(8, 1), (2, -1), (11, -1), (0, -1)]));
    // Each snapshot field (including inactive payload) can independently mismatch.
    for (index, value) in [(1, 2), (2, 5), (4, 124), (6, 1), (7, 1)] {
        result.push(changed(&[(8, 4096), (0, -1), (index, value)]));
    }
    result.push(changed(&[(8, 4096), (5, 5000), (1, 2), (2, -1)]));
    result.push(changed(&[(8, 4096), (5, 5000), (0, -1), (1, -1), (2, -1)]));
    result.push(changed(&[(0, -1), (1, -1), (2, -1)]));
    result.push(changed(&[(2, -1)]));
    result.push(changed(&[(8, 3), (5, 99)]));
    result.push(changed(&[(8, 1), (6, 1), (13, 1)]));
    result
}
fn runs() -> Vec<Value> {
    inputs()
        .iter()
        .flat_map(|a| (0..13).map(move |n| json!({"n":n,"a":a,"s":""})))
        .collect()
}
#[test]
fn csharp_03_t05_w05_complete_snapshots_and_precedence() {
    let b = bundle();
    let generation = std::env::var("MPK_IDEMPOTENCY_REQUESTS_OUT").ok();
    let responses: Vec<Value> = if generation.is_some() {
        vec![]
    } else {
        serde_json::from_slice(&read("responses.json")).unwrap()
    };
    let mut requests = vec![];
    for (case, accepted) in cases() {
        let (context, captures) = setup(&b, case);
        let mut request = json!({"case":case,"expected_attachment":accepted,"id":captures.snapshot_sha256(),"compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.kind()==OriginalInputKind::Source{"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()});
        if accepted {
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
        assert!(row.get("facts").is_some(), "{case}: {row}");
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
            accepted,
            "{case}: {:?}",
            emitted.as_ref().err()
        );
        let Ok(e) = emitted else {
            if case == "helper_partial" {
                assert_eq!(emitted.as_ref().err(), Some(&DataPhaseError::Contract));
            } else {
                assert!(
                    matches!(emitted.as_ref().err(), Some(DataPhaseError::Transition(_))),
                    "{case}: {:?}",
                    emitted.as_ref().err()
                );
            }
            if case.starts_with("float_") || case == "double_nested" {
                assert_eq!(
                    emitted.as_ref().err(),
                    Some(&DataPhaseError::Transition(TransitionError::NonReflexive)),
                    "{case}"
                );
            }
            continue;
        };
        let plan = &e.transitions()[0];
        verify_plan(plan);
        if case == "valid" {
            verify_import(&context, &captures, &source, &e);
        }
        let again = emit_data_phase(&b, &context, &captures, &source).unwrap();
        assert_eq!(e.vir().canonical_bytes(), again.vir().canonical_bytes());
        assert_eq!(
            e.manifest().canonical_bytes(),
            again.manifest().canonical_bytes()
        );
        verify_runs(case, row["runs"].as_array().unwrap());
    }
    if let Some(path) = generation {
        fs::write(path, serde_json::to_vec(&requests).unwrap()).unwrap();
    }
}
fn verify_plan(plan: &ValidatedTransitionContract) {
    use TransitionCheck::*;
    assert_eq!(
        plan.check_order(),
        &[
            RetainedKeyLookup,
            SnapshotEquality,
            ExpectedVersion,
            HistoryCapacity,
            VersionExhaustion,
            BusinessErrors,
            NewSuccess
        ]
    );
    let i = plan.idempotency().unwrap();
    assert_eq!(i.maximum_history(), 4096);
    assert_eq!(i.record_type_id(), src("Processed"));
    assert_eq!(i.equality_callable_id(), helper());
    let mut expected = BTreeMap::from([
        (
            src("Command"),
            vec![
                member("Command", "Expected", prim("u64")),
                member("Command", "Amount", prim("i32")),
                member("Command", "Key", prim("i32")),
                member("Command", "Note", source_ty("Presence")),
            ],
        ),
        (
            src("Context"),
            vec![member("Context", "Effective", prim("i64"))],
        ),
        (
            src("Presence"),
            vec![
                member("Presence", "Tag", source_ty("PresenceTag")),
                member("Presence", "Value", prim("i32")),
            ],
        ),
    ]);
    assert_eq!(i.snapshot_obligations().len(), 2);
    for snapshot in i.snapshot_obligations() {
        assert!(!snapshot.discharged());
        assert_eq!(
            snapshot.canonical_encoding_relation(),
            "canonical_source_field_encoding"
        );
        for node in snapshot.nodes() {
            if let Some(fields) = expected.remove(&node.recipe().type_id) {
                assert_eq!(node.member_ids(), fields);
            }
        }
    }
    assert!(expected.is_empty());
    use TransitionObligationKind::*;
    for (path, kinds) in [
        (
            None,
            vec![
                SnapshotEqualityEquivalence,
                RetainedKeyUniqueness,
                ErrorPrecedence,
            ],
        ),
        (
            Some(TransitionPath::Replay),
            vec![
                UnchangedInputState,
                ReplayNoEvents,
                ReplayStoredResponse,
                EventAndValueBounds,
            ],
        ),
        (
            Some(TransitionPath::NewSuccess),
            vec![
                AppendCompleteSnapshot,
                PreserveRetainedHistory,
                EventRelation,
                CheckedVersionIncrement,
            ],
        ),
    ] {
        for kind in kinds {
            assert!(plan
                .obligations()
                .iter()
                .any(|o| o.path() == path.as_ref() && o.kind() == &kind));
        }
    }
    assert!(plan.obligations().iter().all(|o| !o.discharged()));
    assert_eq!(
        plan.errors().iter().map(|e| e.code()).collect::<Vec<_>>(),
        [
            "idempotency_conflict",
            "version_conflict",
            "history_capacity",
            "version_exhausted",
            "negative"
        ]
    );
    for error in plan.errors() {
        assert!(plan.obligations().iter().any(|o| o.path()
            == Some(&TransitionPath::Error(error.code().into()))
            && o.kind() == &UnchangedInputState));
    }
}
fn verify_runs(case: &str, actual: &[Value]) {
    assert_eq!(actual.len(), inputs().len() * 13);
    let mut mismatches = 0;
    for (a, observed) in inputs().iter().zip(actual.chunks_exact(13)) {
        let retained = a[5] >= a[9] && a[5] < a[9] + a[8];
        let equal =
            a[1] == a[10] && a[2] == a[11] && a[4] == a[12] && a[6] == a[13] && a[7] == a[14];
        let code = if retained {
            if equal {
                0
            } else {
                1
            }
        } else if a[0] != a[1] {
            2
        } else if a[8] == 4096 {
            3
        } else if a[0] == -1 {
            4
        } else if a[2] < 0 {
            5
        } else {
            0
        };
        let new = code == 0 && !retained;
        let replay = code == 0 && retained;
        let expected = [
            code,
            if new { a[0] + 1 } else { a[0] },
            if new { a[3] + a[2] } else { a[3] },
            if new { 2 } else { 0 },
            if replay {
                77 + a[5] - a[9]
            } else if new {
                a[3] + a[2]
            } else {
                a[3]
            },
            a[8] + i32::from(new),
            if a[8] > 0 {
                a[9]
            } else if new {
                a[5]
            } else {
                -1
            },
            if new {
                a[5]
            } else if a[8] > 0 {
                a[9] + a[8] - 1
            } else {
                -1
            },
            a[3],
            a[8],
            if new { a[2] } else { -1 },
            if new { 0 } else { -1 },
            if a[8] > 0 { 77 } else { -1 },
        ];
        for (n, (row, value)) in observed.iter().zip(expected).enumerate() {
            assert_eq!(row["n"], n);
            assert_eq!(row["a"], json!(a));
            assert_eq!(row["error"], "", "{case}: {row}");
            if row["value"] != value {
                mismatches += 1;
                if case == "valid" {
                    panic!("{a:?} n={n} expected={value} actual={row}");
                }
            }
        }
    }
    assert_eq!(mismatches > 0, case == "omitted_context_body");
}

fn verify_import(
    context: &PracticalArtifactContext,
    captures: &CapturedInputSet,
    source: &ValidatedDataSource,
    e: &EmittedDataPhase,
) {
    use mpk_vc::csharp_practical_vir_validation as v;
    let input = v::PracticalVirImportContext {
        data_source_facts: Some(source.captured_facts()),
        artifact_context: context,
        captured_inputs: captures,
        foundation_descriptor_transport: registered_foundation_descriptor_transport(),
        foundation_definitions_transport: registered_foundation_definitions_transport(),
        closed_roots_transport: e.closure().roots().canonical_json(),
        closed_instances_transport: e.closure().closed().canonical_json(),
        semantic_bindings_transport: e.closure().bindings().canonical_bytes(),
        required_checks_transport: e.operations().required_checks().canonical_bytes(),
        operations_transport: e.operations().operations().canonical_bytes(),
    };
    v::import_csharp_practical_vir_json(e.vir().canonical_bytes(), input).unwrap();
    let mut bad = v::PracticalVirContents {
        functions: e.vir().functions().to_vec(),
        binding_projections: e.vir().binding_projections().to_vec(),
        binding_commutations: e.vir().binding_commutations().to_vec(),
        source_exceptions: e.vir().source_exceptions().to_vec(),
        source_obligations: e.vir().source_obligations().to_vec(),
        data_contracts: e.vir().data_contracts().to_vec(),
    };
    let contract = bad
        .data_contracts
        .iter_mut()
        .find(|s| s.contains(TRANSITION_CONTRACT_SCHEMA))
        .unwrap();
    let mut value = parsed(contract.as_bytes());
    set(
        &mut value,
        "idempotency",
        obj(vec![("mode", j("disabled"))]),
    );
    *contract = String::from_utf8(rehash("MPK-CSHARP-TRANSITION-CONTRACT-1.0", value)).unwrap();
    if let Ok(bytes) = v::canonical_csharp_practical_vir_transport(input, bad) {
        assert!(v::import_csharp_practical_vir_json(&bytes, input).is_err());
    }
}
