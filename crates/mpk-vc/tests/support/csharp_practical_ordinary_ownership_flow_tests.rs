//! Witness extraction for native linear-token equations; not proof discharge.
use super::*;
use mpk_vc::csharp_practical_vir_validation::{PracticalConstructionAction, ValidatedPracticalVir};
fn check(vir: &ValidatedPracticalVir) -> (usize, usize, usize) {
    let traces = vir.symbolic_construction_ownership().unwrap();
    assert_eq!(traces, vir.symbolic_construction_ownership().unwrap());
    assert!(!traces.is_empty());
    let mut blocks = 0;
    let mut exceptional = 0;
    let mut backedges = 0;
    for trace in &traces {
        let function = vir
            .functions()
            .iter()
            .find(|f| f.id == trace.function_id)
            .unwrap();
        let source = function
            .blocks
            .iter()
            .map(|b| (b.node.id.as_str(), b))
            .collect::<BTreeMap<_, _>>();
        let states = trace
            .blocks
            .iter()
            .map(|b| (b.node_id.as_str(), b))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            source.keys().collect::<Vec<_>>(),
            states.keys().collect::<Vec<_>>()
        );
        assert!(states[function.blocks[0].node.id.as_str()]
            .before_actions
            .is_empty());
        let expected_edges = function
            .blocks
            .iter()
            .flat_map(|b| {
                b.node
                    .normal_successor_ids
                    .iter()
                    .chain(b.node.exceptional_successors.iter().map(|e| &e.target_id))
                    .map(|target| (b.node.id.clone(), target.clone()))
            })
            .collect::<BTreeSet<_>>();
        let edges = trace
            .blocks
            .iter()
            .flat_map(|b| b.incoming.iter())
            .chain(trace.backedges.iter())
            .collect::<Vec<_>>();
        assert_eq!(
            edges
                .iter()
                .map(|e| (e.predecessor_node_id.clone(), e.target_node_id.clone()))
                .collect::<BTreeSet<_>>(),
            expected_edges
        );
        assert_eq!(edges.len(), expected_edges.len());
        for e in edges {
            let predecessor = source[e.predecessor_node_id.as_str()];
            let before = states[e.predecessor_node_id.as_str()];
            if predecessor
                .node
                .normal_successor_ids
                .contains(&e.target_node_id)
            {
                assert_eq!(e.before_cleanup, before.normal);
            } else {
                exceptional += 1;
                assert_eq!(e.before_cleanup, before.before_invocation);
            }
            assert_eq!(
                e.after_phis,
                states[e.target_node_id.as_str()].before_actions
            );
            let target = source[e.target_node_id.as_str()];
            let mut after = e.after_cleanup.clone();
            for phi in &target.phi_values {
                let incoming = phi
                    .incoming
                    .iter()
                    .find(|i| i.predecessor_node_id == e.predecessor_node_id)
                    .unwrap();
                if let Some(origin) = after
                    .iter()
                    .find(|(_, v)| **v == incoming.value_id)
                    .map(|(o, _)| o.clone())
                {
                    after.insert(origin, phi.value.id.clone());
                }
            }
            assert_eq!(after, e.after_phis);
        }
        for b in &trace.blocks {
            blocks += 1;
            for live in [&b.before_actions, &b.before_invocation, &b.normal] {
                assert!(live.keys().all(|origin| trace.allocations.contains(origin)));
                assert_eq!(live.values().collect::<BTreeSet<_>>().len(), live.len());
            }
            let block = source[b.node_id.as_str()];
            let mut before = b.before_actions.clone();
            if !matches!(
                block.node.tag,
                ControlNodeTag::Exit | ControlNodeTag::HandlerEntry | ControlNodeTag::FinallyEntry
            ) {
                for action in &block.construction_actions {
                    let PracticalConstructionAction::Discard {
                        construction_id,
                        actor_id,
                    } = action
                    else {
                        panic!("unexpected native action");
                    };
                    assert_eq!(actor_id, &function.id);
                    let origin = before
                        .iter()
                        .find(|(_, v)| *v == construction_id)
                        .map(|(o, _)| o.clone())
                        .unwrap();
                    before.remove(&origin);
                }
            }
            assert_eq!(before, b.before_invocation);
            let mut normal = before.clone();
            if let Some(i) = &block.invocation {
                if trace.allocations.contains(&i.result.id) {
                    assert!(i.operation_id.ends_with(".allocate"));
                    assert!(normal
                        .insert(i.result.id.clone(), i.result.id.clone())
                        .is_none());
                } else if let Some(receiver) = i.operands.first() {
                    if let Some(origin) = normal
                        .iter()
                        .find(|(_, v)| **v == receiver.id)
                        .map(|(o, _)| o.clone())
                    {
                        match i.operation_id.rsplit('.').next().unwrap() {
                            "fill" | "rewrite" => {
                                normal.insert(origin, i.result.id.clone());
                            }
                            "freeze" => {
                                normal.remove(&origin);
                            }
                            _ => {
                                assert!(
                                    i.operation_id.ends_with(".read")
                                        || i.operation_id.starts_with("construction.complete.")
                                );
                            }
                        }
                    }
                }
            }
            assert_eq!(normal, b.normal);
        }
        backedges += trace.backedges.len();
        for edge in &trace.backedges {
            assert!(function
                .loops
                .iter()
                .any(|r| r.header_node_id == edge.target_node_id
                    && r.backedge_source_ids.contains(&edge.predecessor_node_id)));
        }
    }
    (blocks, exceptional, backedges)
}
#[test]
fn csharp_03_t06_w09_symbolic_ownership_flow_original_sources() {
    verify(None);
}
#[test]
fn csharp_03_t06_w09_ownership_equation_candidates() {
    verify(Some(Mode::Equations(false)));
}
#[test]
fn csharp_03_t06_w09_ownership_equation_runtime() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| verify(Some(Mode::Equations(true))))
        .unwrap()
        .join()
        .unwrap();
}
#[derive(Clone, Copy)]
enum Mode {
    Equations(bool),
    Proofs,
    ConstructionRecords,
}
#[test]
fn csharp_03_t06_w09_construction_ownership_control_records() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| verify(Some(Mode::ConstructionRecords)))
        .unwrap()
        .join()
        .unwrap();
}
#[test]
fn csharp_03_t06_w09_ownership_proof_candidates() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| verify(Some(Mode::Proofs)))
        .unwrap()
        .join()
        .unwrap();
}
fn verify(ordinary: Option<Mode>) {
    let bundle = b();
    let mut rows = vec![];
    let replay = read("data-phase/data-stage-replay.json");
    for id in [
        "27bf71b9691bff9bf271182a00171dcaf676eca9d77b8102e07cd1facfc6dd09",
        "b85dd77635fb03689bd82d5b31d1088a0f49d29f414ec392e08af68e4d0c46c9",
        "dcb1bfeae19deccb577dce44cd239e814c6ca65f71bb32d8005af44a572fa358",
    ] {
        let row = replay
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        rows.push((id.to_owned(), row.clone(), row["outcome"]["facts"].clone()));
    }
    if ordinary.is_some() {
        for (label, id) in [
            (
                "eight-live",
                "168ae92032147c09e6f9f7c159af4a40d7e44419d012c60e8668d53248f6e7fc",
            ),
            (
                "two-origins",
                "ad5fca99c3564cc6840f6aeac11fc0185430ec9f103a211fe73bc74ad91fc0ad",
            ),
        ] {
            let row = replay
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["id"] == id)
                .unwrap();
            rows.push((
                label.to_owned(),
                row.clone(),
                row["outcome"]["facts"].clone(),
            ));
        }
    }
    let requests = read("control-vc/loop-requests.json");
    let responses = read("control-emission/loop-responses.json");
    for id in [
        "index_update",
        "count_fill",
        "sort",
        "dedup",
        "duplicate_replace",
    ] {
        let request = requests
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        rows.push((id.to_owned(), request.clone(), response["facts"].clone()));
    }
    let mut totals = (0, 0, 0);
    let mut traces = vec![];
    for (id, request, facts) in rows {
        let (context, captures) = support::replay_context(&bundle, &request);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let counts = check(emitted.vir());
        if let Some(runtime) = ordinary {
            ordinary_check(emitted.vir(), &id, runtime);
        }
        totals.0 += counts.0;
        totals.1 += counts.1;
        totals.2 += counts.2;
        traces.push(
            json!({"id":id,"traces":emitted.vir().symbolic_construction_ownership().unwrap()}),
        );
    }
    let cases = read("control-emission/construction-sources.json");
    let captured = read("control-emission/construction-capture.json");
    for id in [
        "array_initializer_catch",
        "array_finally_abandon",
        "array_publish_then_catch",
        "array_return_finally",
    ] {
        let row = cases
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        let data = &captured
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap()["facts"];
        let (context, captures) = support::context(
            &bundle,
            data["selected_root_ids"][0].as_str().unwrap(),
            row["source"].as_str().unwrap().as_bytes(),
        );
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(data).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let counts = check(emitted.vir());
        if let Some(runtime) = ordinary {
            ordinary_check(emitted.vir(), id, runtime);
        }
        totals.0 += counts.0;
        totals.1 += counts.1;
        totals.2 += counts.2;
        traces.push(
            json!({"id":id,"traces":emitted.vir().symbolic_construction_ownership().unwrap()}),
        );
    }
    assert_eq!(traces.len(), if ordinary.is_some() { 14 } else { 12 });
    assert!(totals.1 > 0 && totals.2 > 0);
    if ordinary.is_some() {
        if let Ok(selected) = std::env::var("MPK_W09_OWNERSHIP_EQUATIONS_CONTEXT") {
            assert!(
                traces.iter().any(|row| row["id"] == selected),
                "unknown ownership context: {selected}"
            );
        }
        return;
    }
    let bytes = serde_json::to_vec_pretty(&traces).unwrap();
    if let Some(path) = std::env::var_os("MPK_W09_OWNERSHIP_TRACE_OUT") {
        fs::write(path, bytes).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/ownership-flow/traces.json"),
            json!(traces)
        );
    }
    eprintln!("ownership traces: 12 source contexts, {} blocks, {} exceptional edges, {} delayed backedges",totals.0,totals.1,totals.2);
}

fn ordinary_output(id: &str, ext: &str, bytes: &[u8]) {
    let file = format!("{id}.{ext}");
    if let Some(root) = std::env::var_os("MPK_W09_OWNERSHIP_EQUATIONS_OUT") {
        fs::create_dir_all(&root).unwrap();
        fs::write(Path::new(&root).join(file), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/ownership-equations");
        assert_eq!(fs::read(root.join(&file)).unwrap(), bytes, "{file}");
    }
}
fn encoded(f: &OrdinaryOwnershipFunction, live: &BTreeMap<String, String>) -> Vec<bool> {
    let mut bits = vec![false; 1usize << f.state_depth];
    let index_bits = f.state_depth - 5;
    for (origin, value) in live {
        let index = f.origins.iter().position(|o| o == origin).unwrap();
        let code = f.symbols.iter().position(|s| s == value).unwrap() + 1;
        for bit in 0..32 {
            bits[index | (bit << index_bits)] = code & (1 << bit) != 0;
        }
    }
    bits
}
fn ordinary_check(vir: &ValidatedPracticalVir, id: &str, mode: Mode) {
    if std::env::var("MPK_W09_OWNERSHIP_EQUATIONS_CONTEXT")
        .ok()
        .is_some_and(|s| s != id)
    {
        return;
    }
    if matches!(mode, Mode::ConstructionRecords) {
        if [
            "count_fill",
            "array_finally_abandon",
            "array_return_finally",
        ]
        .contains(&id)
        {
            construction_record_check(vir, id);
        }
        return;
    }
    let Mode::Equations(runtime) = mode else {
        proof_check(vir, id);
        return;
    };
    let p = generate_csharp_practical_ordinary_ownership_flow(vir)
        .unwrap_or_else(|e| panic!("{id}: {e:?}"));
    assert_eq!(
        p.functions()
            .iter()
            .map(|f| f.source.clone())
            .collect::<Vec<_>>(),
        vir.symbolic_construction_ownership().unwrap()
    );
    assert_eq!(
        p.pending_flow_proofs(),
        p.functions()
            .iter()
            .map(|f| f.flow_definition.clone())
            .collect::<Vec<_>>()
    );
    let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    validate_csharp_practical_certificate_structure(&c).unwrap();
    assert_eq!(
        import_csharp_practical_ordinary_ownership_flow(
            &p.canonical_bytes(),
            p.certificate_bytes(),
            vir
        )
        .unwrap(),
        p
    );
    let mut altered: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
    altered["pending_flow_proofs"] = json!([]);
    assert!(import_csharp_practical_ordinary_ownership_flow(
        &serde_json::to_vec(&altered).unwrap(),
        p.certificate_bytes(),
        vir
    )
    .is_err());
    altered = serde_json::from_slice(&p.canonical_bytes()).unwrap();
    altered["functions"][0]["equations"] = json!([]);
    assert!(import_csharp_practical_ordinary_ownership_flow(
        &serde_json::to_vec(&altered).unwrap(),
        p.certificate_bytes(),
        vir
    )
    .is_err());
    let mut damaged = p.certificate_bytes().to_vec();
    *damaged.last_mut().unwrap() ^= 1;
    assert!(
        import_csharp_practical_ordinary_ownership_flow(&p.canonical_bytes(), &damaged, vir)
            .is_err()
    );
    ordinary_output(id, "json", &p.canonical_bytes());
    ordinary_output(
        id,
        "hex",
        format!(
            "{}\n",
            p.certificate_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        )
        .as_bytes(),
    );
    let mut observations = 0;
    if runtime {
        for f in p.functions() {
            eprintln!(
                "ownership equations start: {id}, {} equations",
                f.equations.len()
            );
            assert!(bit(run(&c, &f.flow_definition, vec![])), "{id}: whole flow");
            observations += 1;
            let states = f
                .states
                .iter()
                .map(|s| (s.id.as_str(), s))
                .collect::<BTreeMap<_, _>>();
            let mut checked_states = BTreeSet::new();
            for state in &f.states {
                if !checked_states.insert(&state.definition) {
                    continue;
                }
                for (address, expected) in encoded(f, &state.live).into_iter().enumerate() {
                    let args = (0..f.state_depth)
                        .map(|axis| V::Bit(address & (1 << axis) != 0))
                        .collect();
                    assert_eq!(
                        bit(run(&c, &state.definition, args)),
                        expected,
                        "{id}: state {} address {address}",
                        state.id
                    );
                    observations += 1;
                }
            }
            for e in &f.equations {
                if e.state_arguments.is_empty() {
                    continue;
                }
                let args = e
                    .state_arguments
                    .iter()
                    .map(|s| V::Cube(encoded(f, &states[s.as_str()].live)))
                    .collect::<Vec<_>>();
                assert!(
                    bit(run(&c, &e.relation_definition, args.clone())),
                    "{id}: {}",
                    e.id
                );
                observations += 1;
                let mut changed = args;
                let V::Cube(bits) = changed.last_mut().unwrap() else {
                    unreachable!()
                };
                bits[0] = !bits[0];
                assert!(
                    !bit(run(&c, &e.relation_definition, changed)),
                    "{id}: forged state {}",
                    e.id
                );
                observations += 1;
            }
            // Identity transitions must reject aliasing even when input and
            // output carry the same forged state. Exercise the highest token
            // bit independently of the low-bit mutations above.
            let identity = f.equations.iter().find(|e| {
                e.id.ends_with(".local")
                    && e.state_arguments.len() == 2
                    && states[e.state_arguments[0].as_str()].live
                        == states[e.state_arguments[1].as_str()].live
                    && states[e.state_arguments[0].as_str()].live.len() >= 2
            });
            if id == "eight-live" {
                assert!(identity.is_some(), "eight simultaneously live origins");
            }
            if let Some(e) = identity {
                let live = &states[e.state_arguments[0].as_str()].live;
                let mut duplicate = live.clone();
                let entries = live.iter().collect::<Vec<_>>();
                duplicate.insert(entries[1].0.clone(), entries[0].1.clone());
                let aliased = V::Cube(encoded(f, &duplicate));
                assert!(
                    !bit(run(
                        &c,
                        &e.relation_definition,
                        vec![aliased.clone(), aliased]
                    )),
                    "{id}: aliased tokens"
                );
                observations += 1;
            }
            let e = f
                .equations
                .iter()
                .find(|e| e.state_arguments.len() == 2)
                .unwrap();
            let mut high = e
                .state_arguments
                .iter()
                .map(|s| V::Cube(encoded(f, &states[s.as_str()].live)))
                .collect::<Vec<_>>();
            let V::Cube(bits) = high.last_mut().unwrap() else {
                unreachable!()
            };
            let high_bit = 31 << (f.state_depth - 5);
            bits[high_bit] = !bits[high_bit];
            assert!(
                !bit(run(&c, &e.relation_definition, high)),
                "{id}: highest token bit"
            );
            observations += 1;
            for point in &f.points {
                assert!(!bit(run(&c, &point.witness_definition, vec![])));
                observations += 1;
                let original = V::Cube(encoded(f, &states[point.state_id.as_str()].live));
                assert!(!bit(run(&c, &point.failure_definition, vec![original])));
                observations += 1;
                let empty = V::Cube(vec![false; 1 << f.state_depth]);
                assert!(bit(run(&c, &point.failure_definition, vec![empty])));
                observations += 1;
            }
            eprintln!("ownership equations end: {id}");
        }
    }
    eprintln!("ownership equations {id}: {} functions, {} equations, {observations} observations, {} certificate bytes",p.functions().len(),p.functions().iter().map(|f|f.equations.len()).sum::<usize>(),p.certificate_bytes().len());
}

fn construction_record_check(vir: &ValidatedPracticalVir, id: &str) {
    let p = generate_csharp_practical_ordinary_construction_data(vir)
        .unwrap_or_else(|e| panic!("{id}: {e:?}"));
    let metadata: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
    assert!(p.pending_ownership().is_empty());
    assert!(!p.ownership_records().is_empty());
    assert_eq!(metadata["application_scope_pending"], true);
    assert_eq!(
        import_csharp_practical_ordinary_construction_data(
            &p.canonical_bytes(),
            p.certificate_bytes(),
            vir
        )
        .unwrap(),
        p
    );
    let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    validate_csharp_practical_certificate_structure(&cert).unwrap();
    for record in p.ownership_records() {
        assert!(
            bit(run(&cert, &record.predicate_definition, vec![])),
            "{id}: {}",
            record.source.id
        );
        let mut changed = metadata.clone();
        changed["ownership_records"][0]["equation_ids"] = json!([]);
        assert!(import_csharp_practical_ordinary_construction_data(
            &serde_json::to_vec(&changed).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
    }
    // These captured functions exercise loop backedges and normal/exceptional
    // entry into finally, beyond the straight-line data fixtures.
    if id == "count_fill" {
        assert!(p
            .symbolic_ownership()
            .iter()
            .any(|f| !f.source.backedges.is_empty()));
    } else {
        assert!(vir
            .functions()
            .iter()
            .flat_map(|f| &f.blocks)
            .any(|b| b.node.tag == ControlNodeTag::FinallyEntry));
    }
    let root = std::env::var_os("MPK_W09_CONSTRUCTION_RECORDS_OUT").map(std::path::PathBuf::from);
    let stored = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../develop/migrations/csharp-03/ordinary-foundation/construction-ownership-records",
    );
    for (extension, bytes) in [
        ("json", p.canonical_bytes()),
        (
            "hex",
            format!(
                "{}\n",
                p.certificate_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            )
            .into_bytes(),
        ),
    ] {
        let file = format!("control-{id}.{extension}");
        if let Some(root) = &root {
            fs::create_dir_all(root).unwrap();
            fs::write(root.join(file), bytes).unwrap();
        } else {
            assert_eq!(fs::read(stored.join(file)).unwrap(), bytes);
        }
    }
    eprintln!(
        "construction ownership control {id}: {} records, {} bytes",
        p.ownership_records().len(),
        p.certificate_bytes().len()
    );
}

fn proof_check(vir: &ValidatedPracticalVir, id: &str) {
    let p = generate_csharp_practical_ordinary_ownership_proofs(vir)
        .unwrap_or_else(|e| panic!("{id}: {e:?}"));
    assert_eq!(
        p.functions(),
        generate_csharp_practical_ordinary_ownership_flow(vir)
            .unwrap()
            .functions()
    );
    assert_eq!(
        import_csharp_practical_ordinary_ownership_proofs(
            &p.canonical_bytes(),
            p.certificate_bytes(),
            vir
        )
        .unwrap(),
        p
    );
    let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    validate_csharp_practical_certificate_structure(&c).unwrap();
    assert!(c.proof_node_table.is_empty() && c.theory_certificates.is_empty());
    assert_eq!(p.proofs().len(), p.functions().len());
    for (f, proof) in p.functions().iter().zip(p.proofs()) {
        assert_eq!(f.source.function_id, proof.function_id);
        assert_eq!(f.flow_definition, proof.flow_definition);
        assert_eq!(f.points.len(), proof.point_theorems.len());
        for name in std::iter::once(&proof.flow_theorem).chain(proof.point_theorems.values()) {
            let d = c
                .declarations
                .iter()
                .find(|d| &c.name_table[d.name as usize] == name)
                .unwrap();
            assert!(matches!(
                d.kind,
                mpk_cert::encode::DeclarationKind::Theorem { .. }
            ));
        }
    }
    let mut changed: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
    changed["application_scope_pending"] = json!(false);
    assert!(import_csharp_practical_ordinary_ownership_proofs(
        &serde_json::to_vec(&changed).unwrap(),
        p.certificate_bytes(),
        vir
    )
    .is_err());
    changed = serde_json::from_slice(&p.canonical_bytes()).unwrap();
    changed["proofs"] = json!([]);
    assert!(import_csharp_practical_ordinary_ownership_proofs(
        &serde_json::to_vec(&changed).unwrap(),
        p.certificate_bytes(),
        vir
    )
    .is_err());
    let root = std::env::var_os("MPK_W09_OWNERSHIP_PROOFS_OUT").map(std::path::PathBuf::from);
    let outputs = [
        ("json", p.canonical_bytes()),
        (
            "hex",
            format!(
                "{}\n",
                p.certificate_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            )
            .into_bytes(),
        ),
    ];
    for (ext, bytes) in outputs {
        let file = format!("{id}.{ext}");
        if let Some(root) = &root {
            fs::create_dir_all(root).unwrap();
            fs::write(root.join(file), bytes).unwrap();
        } else {
            let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../develop/migrations/csharp-03/ordinary-foundation/ownership-proofs")
                .join(file);
            assert_eq!(fs::read(path).unwrap(), bytes);
        }
    }
    if id.starts_with("27bf") {
        proof_mutations(&p, vir);
    }
    eprintln!(
        "ownership proofs {id}: {} functions, {} theorem candidates, {} bytes",
        p.functions().len(),
        p.proofs()
            .iter()
            .map(|p| 1 + p.point_theorems.len())
            .sum::<usize>(),
        p.certificate_bytes().len()
    );
}

fn proof_mutations(p: &OrdinaryOwnershipProofProgram, vir: &ValidatedPracticalVir) {
    use mpk_cert::encode::{DeclarationKind as D, TermNode as T};
    let original = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    let declaration = |name: &str| {
        original
            .declarations
            .iter()
            .position(|d| original.name_table[d.name as usize] == name)
            .unwrap()
    };
    let f = &p.functions()[0];
    let point = &f.points[0];
    let state = f.states.iter().find(|s| s.id == point.state_id).unwrap();
    assert!(!state.live.is_empty());
    let empty = f.states.iter().find(|s| s.live.is_empty()).unwrap();
    let D::Def {
        value: empty_value, ..
    } = original.declarations[declaration(&empty.definition)].kind
    else {
        unreachable!()
    };
    let false_id = original
        .declarations
        .iter()
        .position(|d| original.name_table[d.name as usize].ends_with(".false"))
        .unwrap() as u32;
    let false_term = original
        .term_table
        .iter()
        .position(|t| matches!(t,T::Const {global,..} if *global==false_id))
        .unwrap() as u32;
    for label in [
        "flow-is-false",
        "receiver-state-empty",
        "wrong-point-conclusion",
    ] {
        let mut c = original.clone();
        match label {
            "flow-is-false" => {
                let D::Def { value, .. } =
                    &mut c.declarations[declaration(&f.flow_definition)].kind
                else {
                    unreachable!()
                };
                *value = false_term;
            }
            "receiver-state-empty" => {
                let D::Def { value, .. } = &mut c.declarations[declaration(&state.definition)].kind
                else {
                    unreachable!()
                };
                *value = empty_value;
            }
            _ => {
                let name = &p.proofs()[0].point_theorems[&point.node_id];
                let D::Theorem { ty, .. } = c.declarations[declaration(name)].kind else {
                    unreachable!()
                };
                let true_id = c
                    .declarations
                    .iter()
                    .position(|d| c.name_table[d.name as usize].ends_with(".true"))
                    .unwrap() as u32;
                let true_term = c
                    .term_table
                    .iter()
                    .position(|t| matches!(t,T::Const {global,..} if *global==true_id))
                    .unwrap() as u32;
                let T::App { arguments, .. } = &mut c.term_table[ty as usize] else {
                    unreachable!()
                };
                assert_eq!(arguments[2], false_term);
                arguments[2] = true_term;
            }
        }
        c.export_block = mpk_cert::build_export_block(&c).unwrap();
        c.axiom_report = mpk_cert::build_axiom_report(&c).unwrap();
        c.hashes = Default::default();
        c.hashes.export_hash = mpk_cert::export_block_hash(&c.export_block);
        c.hashes.axiom_report_hash = mpk_cert::axiom_report_hash_for_report(&c.axiom_report);
        let bytes = mpk_cert::encode_certificate(&c);
        mpk_cert::decode_canonical_certificate(&bytes).unwrap();
        assert!(import_csharp_practical_ordinary_ownership_proofs(
            &p.canonical_bytes(),
            &bytes,
            vir
        )
        .is_err());
        let hex = format!(
            "{}\n",
            bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
        );
        let root = std::env::var_os("MPK_W09_OWNERSHIP_PROOFS_OUT")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../develop/migrations/csharp-03/ordinary-foundation/ownership-proofs")
            });
        let path = root.join("mutations").join(format!("{label}.hex"));
        if std::env::var_os("MPK_W09_OWNERSHIP_PROOFS_OUT").is_some() {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, hex).unwrap();
        } else {
            assert_eq!(fs::read_to_string(path).unwrap(), hex);
        }
    }
}
