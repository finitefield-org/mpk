//! Original native-array SSA clauses and source-scoped symbolic ownership.
use super::*;
fn output(name: &str, bytes: &[u8]) {
    if let Some(root) = std::env::var_os("MPK_W09_CONSTRUCTION_DATA_OUT") {
        fs::create_dir_all(&root).unwrap();
        fs::write(Path::new(&root).join(name), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/construction-data");
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
    }
}
fn sources() -> Vec<(String, Value, Value)> {
    let replay = read("data-phase/data-stage-replay.json");
    [
        (
            "dynamic-string",
            "48991e93f108e2f8bd42cf33eb996e4dc4c30e980273f53037085f6b09ca61cf",
        ),
        (
            "initializer",
            "27bf71b9691bff9bf271182a00171dcaf676eca9d77b8102e07cd1facfc6dd09",
        ),
        (
            "default-write",
            "b85dd77635fb03689bd82d5b31d1088a0f49d29f414ec392e08af68e4d0c46c9",
        ),
        (
            "return-array",
            "622f058418ac2d8cff0a249155d472a5357b285a8da8d7c5aefeccb81703a2f3",
        ),
        (
            "string-fill",
            "dcb1bfeae19deccb577dce44cd239e814c6ca65f71bb32d8005af44a572fa358",
        ),
        (
            "increment",
            "473b17526e8ed8c802f53965971643a1ad27b085ef532cb244f4d9230b3e0858",
        ),
    ]
    .into_iter()
    .map(|(label, id)| {
        let row = replay
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        assert!(row["outcome"].get("reject").is_none());
        (label.into(), row.clone(), row["outcome"]["facts"].clone())
    })
    .collect()
}
fn construction_marks(
    depth: u32,
    child: u32,
    length: u32,
    initialized: &[u32],
    cells: &[(u32, u32)],
) -> BTreeSet<usize> {
    let mut ones = BTreeSet::new();
    for b in 0..32 {
        if length & (1 << b) != 0 {
            ones.insert(b << (depth - 5));
        }
    }
    for &i in initialized {
        ones.insert(2 | ((i as usize) << (2 + child)));
    }
    for &(i, value) in cells {
        for b in 0..32 {
            if value & (1 << b) != 0 {
                ones.insert(1 | ((i as usize) << 2) | (b << 16));
            }
        }
    }
    ones
}
fn construction(
    depth: u32,
    child: u32,
    length: u32,
    initialized: &[u32],
    cells: &[(u32, u32)],
) -> V {
    sparse_cube(
        depth,
        construction_marks(depth, child, length, initialized, cells),
    )
}
fn sequence(depth: u32, length: u32, cells: &[(u32, u32)]) -> V {
    let mut ones = BTreeSet::new();
    for b in 0..32 {
        if length & (1 << b) != 0 {
            ones.insert(b << (depth - 5));
        }
    }
    for &(i, value) in cells {
        for b in 0..32 {
            if value & (1 << b) != 0 {
                ones.insert(1 | ((i as usize) << 1) | (b << 13));
            }
        }
    }
    sparse_cube(depth, ones)
}
fn word(n: u32) -> V {
    V::Cube((0..32).map(|b| n & (1 << b) != 0).collect())
}
#[test]
fn csharp_03_t06_w09_construction_data_candidates() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| verify(false))
        .unwrap()
        .join()
        .unwrap();
}
#[test]
fn csharp_03_t06_w09_construction_data_original_source() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| verify(true))
        .unwrap()
        .join()
        .unwrap();
}
fn verify(runtime: bool) {
    let bundle = b();
    let mut manifest = vec![];
    let mut observed = 0;
    let mut kinds = BTreeSet::new();
    let filter = std::env::var("MPK_W09_CONSTRUCTION_DATA_CONTEXT").ok();
    let op_filter = std::env::var("MPK_W09_CONSTRUCTION_DATA_OPERATION").ok();
    for (id, request, facts) in sources() {
        let (context, captures) = support::replay_context(&bundle, &request);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let vir = emitted.vir();
        let p = generate_csharp_practical_ordinary_construction_data(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        let data = vc.data_vcs();
        assert!(!p.definitions().is_empty());
        assert!(p.pending_ownership().is_empty());
        assert_eq!(
            p.ownership_records()
                .iter()
                .map(|r| &r.source)
                .collect::<Vec<_>>(),
            data.ownership().iter().collect::<Vec<_>>()
        );
        let foundations = generate_csharp_practical_ordinary_constructions(vir).unwrap();
        let available = foundations
            .definitions()
            .iter()
            .flat_map(|d| {
                d.operations
                    .iter()
                    .map(|o| o.operation_id.clone())
                    .chain(std::iter::once(format!(
                        "construction.complete.{}",
                        d.carrier.type_id
                    )))
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            p.definitions()
                .iter()
                .map(|d| &d.source)
                .collect::<Vec<_>>(),
            data.definitions()
                .iter()
                .filter(|d| available.contains(&d.signature.id))
                .collect::<Vec<_>>()
        );
        for d in p.definitions() {
            assert_eq!(
                &d.construction,
                foundations
                    .definitions()
                    .iter()
                    .find(|f| f.carrier.type_id == d.construction.carrier.type_id)
                    .unwrap()
            );
        }
        let ids = p
            .definitions()
            .iter()
            .map(|d| d.source.id.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            p.definitions()
                .iter()
                .map(|d| &d.source)
                .collect::<Vec<_>>(),
            data.definitions()
                .iter()
                .filter(|d| ids.contains(d.id.as_str()))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            p.pending_definition_ids(),
            data.definitions()
                .iter()
                .filter(|d| !ids.contains(d.id.as_str()))
                .map(|d| d.id.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            p.operations().iter().map(|o| &o.source).collect::<Vec<_>>(),
            data.operations()
                .iter()
                .filter(|o| ids.contains(o.definition_id.as_str()))
                .collect::<Vec<_>>()
        );
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let types = generate_csharp_practical_ordinary_carriers(vir)
            .unwrap()
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        for d in p.definitions() {
            let s = &d.source.signature;
            let kind = if s.id.starts_with("construction.complete.") {
                "complete"
            } else {
                s.id.rsplit('.').next().unwrap()
            };
            kinds.insert(kind.to_owned());
            let owner = s.ordered_checks.iter().position(|c| c.id == "ownership");
            for o in p
                .operations()
                .iter()
                .filter(|o| o.source.definition_id == d.source.id)
            {
                assert!(o.predicates.contains_key("success_relation"));
                assert_eq!(
                    o.ownership.is_some(),
                    owner.is_some(),
                    "{id}: {}",
                    o.source.id
                );
                let unresolved = owner.is_some() && o.ownership.is_none();
                assert_eq!(
                    o.pending_predicates.contains_key("success_guard"),
                    unresolved
                );
                assert_eq!(
                    o.pending_predicates.contains_key("success_goal"),
                    unresolved
                );
                for (i, c) in o.source.checks.iter().enumerate() {
                    assert_eq!(
                        d.failure_definitions[i].is_none(),
                        c.check.id == "ownership"
                    );
                    assert_eq!(
                        o.pending_predicates
                            .contains_key(&format!("check.{i}.failed")),
                        c.check.id == "ownership" && unresolved
                    );
                    if c.check.id == "ownership" {
                        let binding = o.ownership.as_ref().unwrap();
                        assert_eq!(binding.function_id, o.source.function_id);
                        assert_eq!(binding.node_id, o.source.node_id);
                        assert_eq!(binding.receiver_id, o.source.subjects[0].id);
                        assert_eq!(binding.source_failure_name, d.source.failure_names[i]);
                        let f = p
                            .symbolic_ownership()
                            .iter()
                            .find(|f| f.source.function_id == binding.function_id)
                            .unwrap();
                        let point = f
                            .points
                            .iter()
                            .find(|point| point.node_id == binding.node_id)
                            .unwrap();
                        assert_eq!(point.receiver_id, binding.receiver_id);
                        assert_eq!(point.state_id, binding.state_id);
                        let proof = p
                            .symbolic_ownership_proofs()
                            .iter()
                            .find(|p| p.function_id == binding.function_id)
                            .unwrap();
                        assert_eq!(proof.flow_theorem, binding.flow_theorem);
                        assert_eq!(
                            proof.point_theorems[&binding.node_id],
                            binding.receiver_theorem
                        );
                        // The generic operation remains unresolved. Only this
                        // exact source point receives the scoped interpretation.
                        assert!(d.failure_definitions[i].is_none());
                        if runtime {
                            let args = o
                                .source
                                .subjects
                                .iter()
                                .map(|s| sparse_cube(types[&s.type_id].depth, BTreeSet::new()))
                                .collect::<Vec<_>>();
                            assert!(!bit(run(
                                &cert,
                                &o.predicates[&format!("check.{i}.failed")],
                                args.clone()
                            )));
                            assert!(bit(run(
                                &cert,
                                &o.predicates[&format!("check.{i}.static_goal")],
                                args
                            )));
                            observed += 2;
                        }
                    }
                }
            }
            if !runtime
                || filter.as_ref().is_some_and(|f| f != &id)
                || op_filter.as_ref().is_some_and(|f| f != kind)
            {
                continue;
            }
            if kind == "complete" && d.construction.element_type_id != "mpk.csharp.value.i32.v1" {
                let depth = d.construction.carrier.depth;
                let child = types[&d.construction.element_type_id].depth;
                for initialized in [vec![], vec![0], vec![0, 1]] {
                    let complete = initialized.len() == 2;
                    for result in [false, true] {
                        let args = vec![
                            construction(depth, child, 2, &initialized, &[]),
                            V::Bit(result),
                        ];
                        assert_eq!(
                            bit(run(&cert, &d.relation_definition, args.clone())),
                            complete == result
                        );
                        for o in p
                            .operations()
                            .iter()
                            .filter(|o| o.source.definition_id == d.source.id)
                        {
                            assert!(bit(run(
                                &cert,
                                &o.predicates["success_guard"],
                                args.clone()
                            )));
                            assert_eq!(
                                bit(run(&cert, &o.predicates["success_goal"], args.clone())),
                                complete == result
                            );
                        }
                        observed += 1;
                    }
                }
                continue;
            }
            // Physical update equality is valid for every child bit pattern;
            // representation/public domains remain separate obligations.
            if matches!(kind, "fill" | "rewrite")
                && d.construction.element_type_id != "mpk.csharp.value.i32.v1"
            {
                let depth = d.construction.carrier.depth;
                let child = types[&d.construction.element_type_id].depth;
                let initialized = if kind == "fill" { vec![0] } else { vec![0, 1] };
                let before = construction_marks(depth, child, 2, &initialized, &[(0, 7)]);
                let replacement = BTreeSet::from([0, 7, (1usize << child) - 1]);
                let mut after = before.clone();
                after.insert(2 | (1 << (2 + child)));
                for leaf in &replacement {
                    after.insert(1 | (1 << 2) | (leaf << 16));
                }
                for dirty in [None, Some(3), Some(1 | (11 << 2))] {
                    let mut expected = after.clone();
                    if let Some(address) = dirty {
                        expected.insert(address);
                    }
                    let a = vec![
                        sparse_cube(depth, before.clone()),
                        word(1),
                        sparse_cube(child, replacement.clone()),
                        sparse_cube(depth, expected),
                    ];
                    assert_eq!(bit(run(&cert, &d.relation_definition, a)), dirty.is_none());
                    observed += 1;
                }
                continue;
            }
            if kind == "freeze" && d.construction.element_type_id != "mpk.csharp.value.i32.v1" {
                let depth = d.construction.carrier.depth;
                let child = types[&d.construction.element_type_id].depth;
                let source = construction(depth, child, 2, &[0, 1], &[(0, 7), (1, 9), (4096, 31)]);
                for last in [9, 8] {
                    let published = sequence(
                        types[&s.normal_result_type_id].depth,
                        2,
                        &[(0, 7), (1, last)],
                    );
                    assert_eq!(
                        bit(run(
                            &cert,
                            &d.relation_definition,
                            vec![source.clone(), published]
                        )),
                        last == 9
                    );
                    observed += 1;
                }
                continue;
            }
            // Other wide-element bodies remain covered by the construction foundation.
            if d.construction.element_type_id != "mpk.csharp.value.i32.v1" {
                continue;
            }
            eprintln!("construction data start: {id} {kind}");
            let depth = d.construction.carrier.depth;
            let state = || construction(depth, 5, 2, &[0, 1], &[(0, 7), (1, 9)]);
            let (args, result): (Vec<V>, V) = match kind {
                "allocate" => (
                    vec![word(2), V::Bit(false)],
                    construction(depth, 5, 2, &[], &[]),
                ),
                "read" => (vec![state(), word(1)], word(9)),
                "fill" => (
                    vec![construction(depth, 5, 2, &[0], &[(0, 7)]), word(1), word(9)],
                    state(),
                ),
                "rewrite" => (
                    vec![state(), word(0), word(11)],
                    construction(depth, 5, 2, &[0, 1], &[(0, 11), (1, 9)]),
                ),
                "freeze" => (
                    vec![state()],
                    sequence(types[&s.normal_result_type_id].depth, 2, &[(0, 7), (1, 9)]),
                ),
                "complete" => (vec![state()], V::Bit(true)),
                _ => panic!("unknown construction operation"),
            };
            let mut subjects = args.clone();
            subjects.push(result);
            assert!(
                bit(run(&cert, &d.relation_definition, subjects.clone())),
                "{id} {kind}"
            );
            observed += 1;
            let result_depth = types[&s.normal_result_type_id].depth;
            let wrong = if result_depth == 0 {
                V::Bit(false)
            } else if result_depth == 5 {
                word(8)
            } else {
                sparse_cube(result_depth, BTreeSet::new())
            };
            *subjects.last_mut().unwrap() = wrong;
            assert!(
                !bit(run(&cert, &d.relation_definition, subjects.clone())),
                "wrong result {id} {kind}"
            );
            observed += 1;
            for o in p
                .operations()
                .iter()
                .filter(|o| o.source.definition_id == d.source.id)
            {
                assert!(!bit(run(
                    &cert,
                    &o.predicates["success_relation"],
                    subjects.clone()
                )));
                observed += 1;
            }
            for (i, failure) in d.failure_definitions.iter().enumerate() {
                if let Some(failure) = failure {
                    assert!(
                        !bit(run(&cert, failure, args.clone())),
                        "valid {kind} {}",
                        s.ordered_checks[i].id
                    );
                    observed += 1;
                }
            }
            if matches!(kind, "fill" | "rewrite") {
                let cells = if kind == "fill" {
                    vec![(0, 7), (1, 9)]
                } else {
                    vec![(0, 11), (1, 9)]
                };
                let expected = construction_marks(depth, 5, 2, &[0, 1], &cells);
                for address in [3, 4, 6, 1 | (23 << 2) | (17 << 16), 1 | (1 << 2)] {
                    let mut changed = expected.clone();
                    if !changed.insert(address) {
                        changed.remove(&address);
                    }
                    let mut a = args.clone();
                    a.push(sparse_cube(depth, changed));
                    assert!(
                        !bit(run(&cert, &d.relation_definition, a)),
                        "{id} dirty address {address}"
                    );
                    observed += 1;
                }
                for index in [0u32, 1, 8192, 16383] {
                    let before = construction(depth, 5, 16384, &[], &[]);
                    let after = construction(depth, 5, 16384, &[index], &[(index, 17)]);
                    let a = vec![before, word(index), word(17), after];
                    assert!(
                        bit(run(&cert, &d.relation_definition, a)),
                        "boundary index {index}"
                    );
                    observed += 1;
                }
                for index in [1u32 << 14, u32::MAX] {
                    let mut a = args.clone();
                    a[1] = word(index);
                    a.push(args[0].clone());
                    assert!(
                        bit(run(&cert, &d.relation_definition, a)),
                        "high-index body must not wrap"
                    );
                    observed += 1;
                }
            }
            if kind == "freeze" {
                let source =
                    construction(depth, 5, 4096, &[], &[(4095, 17), (4096, 31), (16383, 63)]);
                let published =
                    sequence(types[&s.normal_result_type_id].depth, 4096, &[(4095, 17)]);
                assert!(bit(run(
                    &cert,
                    &d.relation_definition,
                    vec![source.clone(), published]
                )));
                observed += 1;
                let wrong = sequence(types[&s.normal_result_type_id].depth, 4096, &[(4095, 16)]);
                assert!(!bit(run(
                    &cert,
                    &d.relation_definition,
                    vec![source, wrong]
                )));
                observed += 1;
            }
            // Ordered check wrappers must forward their selected operands,
            // including negative/high indices and incomplete initialization.
            for (i, failure) in d.failure_definitions.iter().enumerate() {
                let Some(failure) = failure else {
                    continue;
                };
                let label = s.ordered_checks[i].id.as_str();
                let bad = match label {
                    "negative_length" => vec![word(u32::MAX), V::Bit(false)],
                    "construction_bound" => vec![word(16385), V::Bit(false)],
                    "index_range" => {
                        let mut a = args.clone();
                        a[1] = word(1 << 14);
                        a
                    }
                    "uninitialized" => vec![construction(depth, 5, 2, &[0], &[(0, 7)]), word(1)],
                    "already_initialized" => vec![state(), word(1), word(9)],
                    "incomplete" => {
                        let mut a = args.clone();
                        a[0] = construction(depth, 5, 2, &[0], &[(0, 7)]);
                        a
                    }
                    "publication_bound" => vec![construction(depth, 5, 4097, &[], &[])],
                    _ => panic!("unexpected check {label}"),
                };
                assert!(bit(run(&cert, failure, bad)), "{id} {kind} {label}");
                observed += 1;
            }
            if kind == "complete" {
                let a = vec![construction(depth, 5, 2, &[0], &[(0, 7)]), V::Bit(false)];
                assert!(bit(run(&cert, &d.relation_definition, a)));
                observed += 1;
            }
            if kind == "allocate" {
                for (length, negative, bound) in [
                    (u32::MAX, true, true),
                    (16384, false, false),
                    (16385, false, true),
                ] {
                    let a = vec![word(length), V::Bit(false)];
                    assert_eq!(
                        bit(run(
                            &cert,
                            d.failure_definitions[0].as_ref().unwrap(),
                            a.clone()
                        )),
                        negative
                    );
                    assert_eq!(
                        bit(run(&cert, d.failure_definitions[1].as_ref().unwrap(), a)),
                        bound
                    );
                    observed += 2;
                }
            }
            eprintln!("construction data end: {id} {kind}");
        }
        for record in p.ownership_records() {
            let f = p
                .symbolic_ownership()
                .iter()
                .find(|f| f.source.function_id == record.source.function_id)
                .unwrap();
            assert!(record
                .equation_ids
                .contains(&format!("{}.local", record.source.node_id)));
            assert!(record
                .equation_ids
                .contains(&format!("{}.invocation", record.source.node_id)));
            assert!(record
                .equation_ids
                .iter()
                .all(|id| f.equations.iter().any(|e| &e.id == id)));
            assert_eq!(
                record.flow_theorem,
                p.symbolic_ownership_proofs()
                    .iter()
                    .find(|p| p.function_id == record.source.function_id)
                    .unwrap()
                    .flow_theorem
            );
            if runtime && filter.as_deref().is_none_or(|selected| selected == id) {
                assert!(bit(run(&cert, &record.predicate_definition, vec![])));
                observed += 1;
            }
        }
        let meta: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_construction_data(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        for key in [
            "source_ir_sha256",
            "foundation_sha256",
            "data_vc_sha256",
            "certificate_sha256",
            "definitions",
            "operations",
            "pending_ownership",
            "ownership_records",
            "symbolic_ownership",
            "symbolic_ownership_proofs",
            "pending_concrete_ownership_functions",
            "application_scope_pending",
        ] {
            let mut changed = meta.clone();
            changed[key] = json!("forged");
            assert!(import_csharp_practical_ordinary_construction_data(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        for field in [
            "node_id",
            "receiver_id",
            "flow_theorem",
            "scoped_failure_definition",
        ] {
            let mut changed = meta.clone();
            if let Some(o) = changed["operations"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|o| !o["ownership"].is_null())
            {
                o["ownership"][field] = json!("other-source-point");
                assert!(import_csharp_practical_ordinary_construction_data(
                    &serde_json::to_vec(&changed).unwrap(),
                    p.certificate_bytes(),
                    vir
                )
                .is_err());
            }
        }
        for field in [
            "source",
            "equation_ids",
            "predicate_definition",
            "theorem",
            "flow_theorem",
        ] {
            let mut changed = meta.clone();
            if let Some(record) = changed["ownership_records"]
                .as_array_mut()
                .unwrap()
                .first_mut()
            {
                record[field] = json!("other-source-record");
                assert!(import_csharp_practical_ordinary_construction_data(
                    &serde_json::to_vec(&changed).unwrap(),
                    p.certificate_bytes(),
                    vir
                )
                .is_err());
            }
        }
        let mut changed = meta.clone();
        let point = changed["operations"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|o| !o["pending_predicates"].as_object().unwrap().is_empty());
        if let Some(point) = point {
            point["pending_predicates"] = json!({});
            assert!(import_csharp_practical_ordinary_construction_data(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut damaged = p.certificate_bytes().to_vec();
        *damaged.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_construction_data(
            &p.canonical_bytes(),
            &damaged,
            vir
        )
        .is_err());
        output(&format!("{id}.json"), &p.canonical_bytes());
        output(
            &format!("{id}.hex"),
            format!(
                "{}\n",
                p.certificate_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            )
            .as_bytes(),
        );
        manifest.push(json!({"id":id,"definitions":p.definitions().len(),"use_points":p.operations().len(),"certificate_sha256":meta["certificate_sha256"],"bytes":p.certificate_bytes().len(),"pending_predicates":p.operations().iter().map(|o| o.pending_predicates.len()).sum::<usize>()}));
        eprintln!(
            "construction context {id}: {} definitions, {} SSA points",
            p.definitions().len(),
            p.operations().len()
        );
    }
    assert_eq!(
        kinds,
        ["allocate", "read", "fill", "rewrite", "freeze", "complete"]
            .map(str::to_owned)
            .into_iter()
            .collect()
    );
    if runtime {
        assert!(observed > 0);
    }
    output(
        "certificates.json",
        &serde_json::to_vec_pretty(&manifest).unwrap(),
    );
    eprintln!(
        "construction data: {} contexts, {observed} observations",
        manifest.len()
    );
}
