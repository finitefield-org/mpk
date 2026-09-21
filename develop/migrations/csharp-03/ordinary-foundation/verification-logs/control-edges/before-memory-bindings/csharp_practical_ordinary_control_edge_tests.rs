//! Exact edge guards, ordered failures and guarded slot joins at original nodes.
use super::*;
use std::path::PathBuf;

#[test]
fn csharp_03_t06_w09_control_edge_phi_preserves_existing_declarations() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation");
    let previous = root.join("verification-logs/control-edges/before-phi-joins/control-edges");
    let current = std::env::var_os("MPK_W09_CONTROL_EDGES_OUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("control-edges"));
    let mut count = 0;
    let mut added = 0;
    for file in fs::read_dir(&previous).unwrap() {
        let file = file.unwrap().path();
        if file.extension().and_then(|s| s.to_str()) != Some("hex") {
            continue;
        }
        let decode = |path: &Path| {
            let hex = fs::read_to_string(path).unwrap();
            let bytes = hex
                .trim()
                .as_bytes()
                .chunks_exact(2)
                .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
                .collect::<Vec<_>>();
            mpk_cert::decode_canonical_certificate(&bytes).unwrap()
        };
        let before = decode(&file);
        let after = decode(&current.join(file.file_name().unwrap()));
        assert_eq!(before.module, after.module);
        assert_eq!(before.imports, after.imports);
        assert!(after.declarations.len() >= before.declarations.len());
        assert_eq!(
            before.term_table,
            after.term_table[..before.term_table.len()]
        );
        assert_eq!(
            before.level_table,
            after.level_table[..before.level_table.len()]
        );
        assert_eq!(before.proof_node_table, after.proof_node_table);
        assert_eq!(before.theory_certificates, after.theory_certificates);
        for (before_decl, after_decl) in before.declarations.iter().zip(&after.declarations) {
            let mut remapped = before_decl.clone();
            assert_eq!(
                before.name_table[before_decl.name as usize],
                after.name_table[after_decl.name as usize]
            );
            remapped.name = after_decl.name;
            assert_eq!(&remapped, after_decl);
        }
        let before_meta: Value =
            serde_json::from_slice(&fs::read(file.with_extension("json")).unwrap()).unwrap();
        let mut after_meta: Value = serde_json::from_slice(
            &fs::read(
                current
                    .join(file.file_name().unwrap())
                    .with_extension("json"),
            )
            .unwrap(),
        )
        .unwrap();
        for function in after_meta["functions"].as_array_mut().unwrap() {
            for edge in function["edges"].as_array_mut().unwrap() {
                added += usize::from(edge.as_object_mut().unwrap().remove("phi_join").is_some());
            }
        }
        after_meta["certificate_sha256"] = before_meta["certificate_sha256"].clone();
        assert_eq!(before_meta, after_meta);
        count += 1;
    }
    assert_eq!(count, 17);
    assert!(added > 0);
    eprintln!("preserved {count} source guard/slot programs; added {added} phi joins");
}

fn reference(
    term: &ContractTerm,
    args: &[i32],
    program: &OrdinaryControlEdgeProgram,
    edge: &OrdinaryControlEdgeDefinition,
) -> bool {
    fn eval(
        t: &ContractTerm,
        args: &[i32],
        program: &OrdinaryControlEdgeProgram,
        edge: &OrdinaryControlEdgeDefinition,
    ) -> i32 {
        if let ContractTerm::Var { index, .. } = t {
            return args[*index];
        }
        let mut root = t;
        let mut operands = vec![];
        while let ContractTerm::App {
            function, argument, ..
        } = root
        {
            operands.push(argument.as_ref());
            root = function;
        }
        operands.reverse();
        let ContractTerm::Const { name, .. } = root else {
            panic!("unexpected edge term");
        };
        let a = operands
            .iter()
            .map(|t| eval(t, args, program, edge))
            .collect::<Vec<_>>();
        let result = match name.as_str() {
            "Mpk.CSharp.Bool.true" => true,
            "Mpk.CSharp.Bool.false" => false,
            "Mpk.CSharp.Bool.Not" => a[0] == 0,
            "Mpk.CSharp.Bool.And" => a[0] != 0 && a[1] != 0,
            "Mpk.CSharp.Bool.Or" => a[0] != 0 || a[1] != 0,
            _ => {
                if let Some((d, i)) = program.construction_definitions().iter().find_map(|d| {
                    d.source
                        .failure_names
                        .iter()
                        .position(|n| n == name)
                        .map(|i| (d, i))
                }) {
                    return i32::from(match d.source.signature.ordered_checks[i].id.as_str() {
                        "negative_length" => a[0] < 0,
                        "index_range" => (a[1] as u32) >= (a[0] as u32) || (a[1] as u32) >= 16384,
                        "ownership" => {
                            let binding = edge
                                .ownership
                                .as_ref()
                                .expect("source-scoped ownership required");
                            assert_eq!(&binding.source_failure_name, name);
                            assert_eq!(binding.node_id, edge.source.source_node_id);
                            false
                        }
                        check => panic!("missing construction guard oracle: {check}"),
                    });
                }
                if let Some((d, i)) = program.string_definitions().iter().find_map(|d| {
                    d.source
                        .failure_names
                        .iter()
                        .position(|n| n == name)
                        .map(|i| (d, i))
                }) {
                    let nullable =
                        d.source.signature.argument_type_ids[0] != "mpk.csharp.value.string.v1";
                    let absent = nullable && a[0] == i32::MIN;
                    let length = if absent { 0 } else { a[0] as u32 };
                    return i32::from(match d.source.signature.ordered_checks[i].id.as_str() {
                        "exception.null_receiver" => absent,
                        "index_range" => a[1] < 0 || (a[1] as u32) >= length,
                        check => panic!("missing string guard oracle: {check}"),
                    });
                }
                if let Some(d) = program
                    .sequence_definitions()
                    .iter()
                    .find(|d| d.source.failure_names.iter().any(|n| n == name))
                {
                    assert_eq!(d.source.signature.ordered_checks.len(), 1);
                    assert_eq!(d.source.signature.ordered_checks[0].id, "index_range");
                    return i32::from((a[1] as u32) >= (a[0] as u32) || (a[1] as u32) >= 4096);
                }
                if let Some(d) = program
                    .reference_definitions()
                    .iter()
                    .find(|d| d.source.failure_names.iter().any(|n| n == name))
                {
                    assert_eq!(
                        d.source.signature.ordered_checks[0].id,
                        "exception.null_receiver"
                    );
                    return i32::from(a[0] == i32::MIN);
                }
                if let Some(d) = program
                    .option_definitions()
                    .iter()
                    .find(|d| d.source.failure_names.iter().any(|n| n == name))
                {
                    assert_eq!(d.source.signature.ordered_checks[0].id, "invalid_operation");
                    return i32::from(a[0] == i32::MIN);
                }
                let (d, i) = program
                    .integer_definitions()
                    .iter()
                    .find_map(|d| {
                        d.source
                            .failure_names
                            .iter()
                            .position(|n| n == name)
                            .map(|i| (d, i))
                    })
                    .unwrap_or_else(|| panic!("missing independent guard oracle: {name}"));
                let zero = d.source.signature.ordered_checks[i].id == "exception.division_by_zero";
                let op = d.source.signature.id.as_str();
                assert!(
                    op.starts_with("integer.i32."),
                    "uncovered scalar type: {op}"
                );
                if zero {
                    assert!(op.contains(".divide.") || op.contains(".remainder."));
                    a[1] == 0
                } else {
                    assert_eq!(
                        d.source.signature.ordered_checks[i].id,
                        "exception.overflow"
                    );
                    match op.split('.').nth(2).unwrap() {
                        "add" => a[0].overflowing_add(a[1]).1,
                        "subtract" => a[0].overflowing_sub(a[1]).1,
                        "multiply" => a[0].overflowing_mul(a[1]).1,
                        "negate" => a[0] == i32::MIN,
                        "divide" | "remainder" => a[0] == i32::MIN && a[1] == -1,
                        _ => panic!("uncovered scalar guard: {op}"),
                    }
                }
            }
        };
        i32::from(result)
    }
    eval(term, args, program, edge) != 0
}
fn encoded(depth: u32, n: i32) -> V {
    if depth == 0 {
        V::Bit(n != 0)
    } else {
        sparse_cube(
            depth,
            (0..(1usize << depth).min(32))
                .filter(|i| (n as u32) & (1 << i) != 0)
                .collect(),
        )
    }
}
fn string_kind(p: &OrdinaryControlEdgeProgram, ty: &str) -> Option<bool> {
    if ty == "mpk.csharp.value.string.v1" {
        return Some(false);
    }
    p.string_definitions()
        .iter()
        .any(|d| d.source.signature.argument_type_ids[0] == ty)
        .then_some(true)
}
fn length_input(p: &OrdinaryControlEdgeProgram, ty: &str) -> bool {
    string_kind(p, ty).is_some()
        || p.sequence_definitions()
            .iter()
            .any(|d| d.sequence.carrier.type_id == ty)
        || p.construction_definitions()
            .iter()
            .any(|d| d.construction.carrier.type_id == ty)
}
fn guard_input(p: &OrdinaryControlEdgeProgram, ty: &str, depth: u32, n: i32) -> V {
    if let Some(nullable) = string_kind(p, ty) {
        if nullable && n == i32::MIN {
            return sparse_cube(depth, BTreeSet::new());
        }
        let shift = depth - u32::from(nullable) - 5;
        let mut ones = (0..32)
            .filter(|i| (n as u32) & (1 << i) != 0)
            .map(|i| {
                if nullable {
                    1 + 2 * (i << shift)
                } else {
                    i << shift
                }
            })
            .collect::<BTreeSet<_>>();
        if nullable {
            ones.insert(0);
        }
        return sparse_cube(depth, ones);
    }
    if p.reference_definitions()
        .iter()
        .any(|d| d.source.signature.argument_type_ids[0] == ty)
        || p.option_definitions()
            .iter()
            .any(|d| d.source.signature.argument_type_ids[0] == ty)
    {
        return sparse_cube(
            depth,
            if n == i32::MIN {
                BTreeSet::new()
            } else {
                BTreeSet::from([0])
            },
        );
    }
    if length_input(p, ty) {
        return sparse_cube(
            depth,
            (0..32)
                .filter(|i| (n as u32) & (1 << i) != 0)
                .map(|i| i << (depth - 5))
                .collect(),
        );
    }
    encoded(depth, n)
}
fn output(id: &str, ext: &str, bytes: &[u8]) {
    let file = format!("{id}.{ext}");
    if let Some(root) = std::env::var_os("MPK_W09_CONTROL_EDGES_OUT").map(PathBuf::from) {
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(file), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/control-edges");
        assert_eq!(fs::read(root.join(&file)).unwrap(), bytes, "{file}");
    }
}
fn run_cases(ids: &[&str], runtime: bool) {
    let bundle = b();
    let mut requests = read("control-vc/loop-requests.json");
    let mut responses = read("control-emission/loop-responses.json");
    requests.as_array_mut().unwrap().extend(
        read("control-vc/measure-requests.json")
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["id"] == "total_variable")
            .cloned(),
    );
    responses.as_array_mut().unwrap().extend(
        read("control-vc/measure-responses.json")
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["id"] == "total_variable")
            .cloned(),
    );
    let patterns = read("control-emission/source-cases.json");
    let mut observations = 0;
    let mut true_exceptions = 0;
    let mut false_guards = 0;
    let mut pending = 0;
    let mut checked_goal_bindings = BTreeSet::new();
    let mut checked_decrease_bindings = BTreeSet::new();
    for &id in ids {
        let (context, captures, facts) =
            if let Some(request) = requests.as_array().unwrap().iter().find(|r| r["id"] == id) {
                let response = responses
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| r["id"] == id)
                    .unwrap();
                let (context, captures) = support::replay_context(&bundle, request);
                (context, captures, response["facts"].clone())
            } else {
                let row = patterns
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| r["stage"] == "patterns" && r["source_case"]["id"] == id)
                    .unwrap();
                assert_eq!(row["accepted"], true);
                let source = &row["source_case"];
                let (context, captures) = support::context(
                    &bundle,
                    source["root"].as_str().unwrap(),
                    source["source"].as_str().unwrap().as_bytes(),
                );
                (context, captures, row["data"].clone())
            };
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let vir = emitted.vir();
        let vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        let p = generate_csharp_practical_ordinary_control_edges(vir).unwrap();
        let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
        let depths = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.as_str(), c.depth))
            .collect::<BTreeMap<_, _>>();
        assert!(!p.functions().is_empty());
        assert_eq!(
            import_csharp_practical_ordinary_control_edges(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&c).unwrap();
        if !runtime {
            output(id, "json", &p.canonical_bytes());
            let hex = p
                .certificate_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
                + "\n";
            output(id, "hex", hex.as_bytes());
            let unresolved = p
                .functions()
                .iter()
                .flat_map(|f| &f.edges)
                .filter(|e| !e.pending_constant_names.is_empty())
                .count();
            eprintln!("control edges {id}: generated/imported exact candidate; {unresolved} pending guards");
            continue;
        }
        eprintln!("control edges {id}: runtime start");
        let mut case_pending = 0;
        for f in p.functions() {
            assert_eq!(
                &f.source,
                vc.control_vcs()
                    .functions()
                    .iter()
                    .find(|s| s.function_id == f.source.function_id)
                    .unwrap()
            );
            assert_eq!(
                f.edges.iter().map(|e| e.source.clone()).collect::<Vec<_>>(),
                f.source.edges
            );
            for edge in &f.edges {
                let Some(guard) = &edge.guard_definition else {
                    assert!(edge.join.is_none() && !edge.pending_constant_names.is_empty());
                    assert!(edge.phi_join.is_none());
                    pending += 1;
                    case_pending += 1;
                    continue;
                };
                assert!(edge.pending_constant_names.is_empty());
                let target = edge.source.target_node_id.as_ref().map(|node| {
                    vir.functions()
                        .iter()
                        .find(|native| native.id == f.source.function_id)
                        .unwrap()
                        .blocks
                        .iter()
                        .find(|b| &b.node.id == node)
                        .unwrap()
                });
                assert_eq!(
                    edge.phi_join.is_some(),
                    target.is_some_and(|b| !b.phi_values.is_empty())
                );
                if let Some(phi_join) = &edge.phi_join {
                    let target = target.unwrap();
                    let n = phi_join.guard_argument_count;
                    assert_eq!(n, edge.source.guard.bindings.len());
                    assert_eq!(phi_join.arguments[..n], edge.source.guard.bindings);
                    assert_eq!(phi_join.arguments.len(), n + 2 * target.phi_values.len());
                    assert_eq!(phi_join.state_rule, "parallel_phi_inputs_at_source_exit_to_edge_specific_target; node_entry_merge_pending");
                    for (phi, args) in target
                        .phi_values
                        .iter()
                        .zip(phi_join.arguments[n..].chunks_exact(2))
                    {
                        let incoming = phi
                            .incoming
                            .iter()
                            .find(|i| i.predecessor_node_id == edge.source.source_node_id)
                            .unwrap();
                        assert_eq!(args[0].node_id, incoming.predecessor_node_id);
                        assert_eq!(args[0].value_id, incoming.value_id);
                        assert_eq!(args[1].node_id, target.node.id);
                        assert_eq!(args[1].value_id, phi.value.id);
                        for arg in args {
                            assert_eq!(arg.kind, "ssa");
                            assert_eq!(arg.edge_id.as_deref(), Some(edge.source.id.as_str()));
                            assert_eq!(arg.type_id, phi.value.type_id);
                        }
                    }
                }
                assert_eq!(
                    edge.join.is_some(),
                    edge.source.target_node_id.is_some() && edge.source.kind != "function_entry"
                );
                if let Some(binding) = &edge.ownership {
                    assert_eq!(binding.function_id, f.source.function_id);
                    assert_eq!(binding.node_id, edge.source.source_node_id);
                    let flow = p
                        .symbolic_ownership()
                        .iter()
                        .find(|o| o.source.function_id == binding.function_id)
                        .unwrap();
                    let point = flow
                        .points
                        .iter()
                        .find(|o| o.node_id == binding.node_id)
                        .unwrap();
                    let proof = p
                        .symbolic_ownership_proofs()
                        .iter()
                        .find(|o| o.function_id == binding.function_id)
                        .unwrap();
                    assert_eq!(binding.receiver_id, point.receiver_id);
                    assert_eq!(binding.state_id, point.state_id);
                    assert_eq!(binding.flow_theorem, proof.flow_theorem);
                    assert_eq!(
                        binding.receiver_theorem,
                        proof.point_theorems[&binding.node_id]
                    );
                    assert_eq!(edge.source.guard.bindings[0].value_id, binding.receiver_id);
                }
                let extended = id == "index_update"
                    || !p.string_definitions().is_empty()
                    || !p.sequence_definitions().is_empty();
                let capacity = if !p.sequence_definitions().is_empty() {
                    4096
                } else {
                    16384
                };
                for sample in 0..if extended { 13 } else { 7 } {
                    let inputs = (0..edge.source.guard.bindings.len())
                        .map(|i| match sample {
                            0 => 0,
                            1 => 1,
                            2 => i32::MAX,
                            3 => i32::MIN,
                            4 => {
                                if i % 2 == 0 {
                                    i32::MAX
                                } else {
                                    1
                                }
                            }
                            5 => {
                                if i % 2 == 0 {
                                    i32::MIN
                                } else {
                                    -1
                                }
                            }
                            6 => {
                                if i % 2 == 0 {
                                    1
                                } else {
                                    0
                                }
                            }
                            7..=10 => {
                                if length_input(&p, &edge.source.guard.bindings[i].type_id) {
                                    2
                                } else {
                                    [0, 1, 2, -1][sample - 7]
                                }
                            }
                            11..=12 => {
                                if length_input(&p, &edge.source.guard.bindings[i].type_id) {
                                    capacity
                                } else {
                                    [capacity - 1, capacity][sample - 11]
                                }
                            }
                            _ => unreachable!(),
                        })
                        .collect::<Vec<_>>();
                    let expected = reference(&edge.source.guard.term, &inputs, &p, edge);
                    let values = edge
                        .source
                        .guard
                        .bindings
                        .iter()
                        .zip(&inputs)
                        .map(|(b, n)| guard_input(&p, &b.type_id, depths[b.type_id.as_str()], *n))
                        .collect::<Vec<_>>();
                    assert_eq!(
                        bit(run(&c, guard, values.clone())),
                        expected,
                        "{id}: {} {inputs:?}",
                        edge.source.id
                    );
                    observations += 1;
                    false_guards += usize::from(!expected);
                    true_exceptions += usize::from(expected && edge.source.kind == "exception");
                    if let Some(join) = &edge.phi_join {
                        let n = join.guard_argument_count;
                        let mut args = values.clone();
                        for (i, pair) in join.arguments[n..].chunks_exact(2).enumerate() {
                            // Different values distinguish simultaneous copies
                            // from collapsing all phis into a single assignment.
                            let depth = depths[pair[0].type_id.as_str()];
                            let marker = if depth == 0 {
                                i as i32 % 2
                            } else {
                                i as i32 + 1
                            };
                            let v = encoded(depth, marker);
                            args.extend([v.clone(), v]);
                        }
                        assert!(bit(run(&c, &join.definition, args.clone())));
                        observations += 1;
                        for (i, a) in join.arguments[n..].iter().enumerate() {
                            let depth = depths[a.type_id.as_str()];
                            let marker = if depth == 0 {
                                (i / 2) as i32 % 2
                            } else {
                                (i / 2) as i32 + 1
                            };
                            let mut wrong = args.clone();
                            wrong[n + i] = encoded(depth, marker ^ 1);
                            assert_eq!(
                                bit(run(&c, &join.definition, wrong)),
                                !expected,
                                "{id}: phi input/output mismatch {} argument {i}",
                                edge.source.id
                            );
                            observations += 1;
                            if depth > 0 {
                                let mut wrong = args.clone();
                                let mut ones = (0..(1usize << depth).min(32))
                                    .filter(|i| (marker as u32) & (1 << i) != 0)
                                    .collect::<BTreeSet<_>>();
                                let high = (1usize << depth) - 1;
                                if !ones.remove(&high) {
                                    ones.insert(high);
                                }
                                wrong[n + i] = sparse_cube(depth, ones);
                                assert_eq!(
                                    bit(run(&c, &join.definition, wrong)),
                                    !expected,
                                    "{id}: phi high physical bit {} argument {i}",
                                    edge.source.id
                                );
                                observations += 1;
                            }
                        }
                    }
                    if let Some(join) = &edge.join {
                        let n = join.guard_argument_count;
                        assert_eq!(n, edge.source.guard.bindings.len());
                        assert_eq!(join.arguments[..n], edge.source.guard.bindings);
                        assert_eq!(join.arguments.len(), n + 4 * f.source.slots.len());
                        assert_eq!(
                            join.state_rule,
                            "source_exit_to_edge_specific_target; node_entry_merge_pending"
                        );
                        for (i, (slot, ty)) in f.source.slots.iter().enumerate() {
                            for (j, a) in
                                join.arguments[n + 4 * i..n + 4 * i + 4].iter().enumerate()
                            {
                                assert_eq!(&a.value_id, slot);
                                assert_eq!(
                                    a.kind,
                                    if j % 2 == 0 {
                                        "slot_assigned"
                                    } else {
                                        "current_slot"
                                    }
                                );
                                assert_eq!(
                                    a.type_id,
                                    if j % 2 == 0 {
                                        "mpk.csharp.value.bool.v1"
                                    } else {
                                        ty
                                    }
                                );
                                assert_eq!(a.edge_id.as_deref(), Some(edge.source.id.as_str()));
                                assert_eq!(
                                    &a.node_id,
                                    if j < 2 {
                                        &edge.source.source_node_id
                                    } else {
                                        edge.source.target_node_id.as_ref().unwrap()
                                    }
                                );
                            }
                        }
                        for binding in vc
                            .control_vcs()
                            .sequents()
                            .iter()
                            .filter(|s| s.function_id == f.source.function_id)
                            .flat_map(|s| s.assumptions.iter().chain(&s.goals))
                            .flat_map(|p| &p.bindings)
                            .filter(|b| {
                                b.edge_id.as_deref() == Some(edge.source.id.as_str())
                                    && matches!(b.kind.as_str(), "current_slot" | "slot_assigned")
                            })
                        {
                            assert!(
                                join.arguments[n..]
                                    .chunks_exact(4)
                                    .any(|pair| pair[2..].contains(binding)),
                                "{id}: missing original W04 edge-goal binding {binding:?}"
                            );
                            checked_goal_bindings.insert(
                                serde_json::to_string(&(&f.source.function_id, binding)).unwrap(),
                            );
                            if id == "total_variable" && binding.kind == "slot_assigned" {
                                checked_decrease_bindings
                                    .insert(serde_json::to_string(binding).unwrap());
                            }
                        }
                        let mut args = values;
                        for a in &join.arguments[n..] {
                            args.push(encoded(
                                depths[a.type_id.as_str()],
                                i32::from(a.kind == "slot_assigned"),
                            ));
                        }
                        assert!(bit(run(&c, &join.definition, args.clone())));
                        observations += 1;
                        if !f.source.slots.is_empty() {
                            let mut wrong = args.clone();
                            wrong[n + 2] = V::Bit(false);
                            assert_eq!(bit(run(&c, &join.definition, wrong)), !expected);
                            observations += 1;
                            let mut wrong = args.clone();
                            let depth = depths[join.arguments[n + 3].type_id.as_str()];
                            wrong[n + 3] = if depth == 0 {
                                V::Bit(true)
                            } else {
                                sparse_cube(depth, BTreeSet::from([(1usize << depth) - 1]))
                            };
                            assert_eq!(bit(run(&c, &join.definition, wrong)), !expected);
                            observations += 1;
                            let mut inactive = args;
                            for (i, a) in join.arguments.iter().enumerate().skip(n) {
                                inactive[i] = if a.kind == "slot_assigned" {
                                    V::Bit(false)
                                } else {
                                    encoded(depths[a.type_id.as_str()], i32::from((i - n) % 4 >= 2))
                                };
                            }
                            assert!(bit(run(&c, &join.definition, inactive)));
                            observations += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(case_pending, if id == "type" { 1 } else { 0 });
        let original: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        for field in [
            "source_ir_sha256",
            "control_vc_sha256",
            "integer_definitions",
            "application_scope_pending",
        ] {
            let mut altered = original.clone();
            altered.as_object_mut().unwrap().remove(field);
            assert!(import_csharp_practical_ordinary_control_edges(
                &serde_json::to_vec(&altered).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        for field in [
            "source",
            "guard_definition",
            "join",
            "pending_constant_names",
        ] {
            let mut altered = original.clone();
            altered["functions"][0]["edges"][1]
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(import_csharp_practical_ordinary_control_edges(
                &serde_json::to_vec(&altered).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let join_edge = original["functions"][0]["edges"]
            .as_array()
            .unwrap()
            .iter()
            .position(|e| e["join"].is_object())
            .unwrap();
        for (function_index, f) in original["functions"].as_array().unwrap().iter().enumerate() {
            if let Some((edge_index, edge)) = f["edges"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .find(|(_, e)| e["phi_join"].is_object())
            {
                let n = edge["phi_join"]["guard_argument_count"].as_u64().unwrap() as usize;
                for field in ["edge_id", "node_id", "value_id", "type_id"] {
                    let mut altered = original.clone();
                    altered["functions"][function_index]["edges"][edge_index]["phi_join"]
                        ["arguments"][n + 1][field] = json!("wrong-phi-binding");
                    assert!(import_csharp_practical_ordinary_control_edges(
                        &serde_json::to_vec(&altered).unwrap(),
                        p.certificate_bytes(),
                        vir
                    )
                    .is_err());
                }
                let mut altered = original.clone();
                altered["functions"][function_index]["edges"][edge_index]
                    .as_object_mut()
                    .unwrap()
                    .remove("phi_join");
                assert!(import_csharp_practical_ordinary_control_edges(
                    &serde_json::to_vec(&altered).unwrap(),
                    p.certificate_bytes(),
                    vir
                )
                .is_err());
            }
        }
        for alter_entry in [false, true] {
            let mut altered = original.clone();
            let join = &mut altered["functions"][0]["edges"][join_edge]["join"];
            if alter_entry {
                let n = join["guard_argument_count"].as_u64().unwrap() as usize;
                join["arguments"][n + 2]["edge_id"] = Value::Null;
            } else {
                join.as_object_mut().unwrap().remove("state_rule");
            }
            assert!(import_csharp_practical_ordinary_control_edges(
                &serde_json::to_vec(&altered).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        if id == "index_update" {
            let owned = original["functions"][0]["edges"]
                .as_array()
                .unwrap()
                .iter()
                .position(|e| e["ownership"].is_object())
                .unwrap();
            assert_eq!(
                p.functions()
                    .iter()
                    .flat_map(|f| &f.edges)
                    .filter(|e| e.ownership.is_some())
                    .count(),
                4
            );
            for field in [
                "function_id",
                "node_id",
                "receiver_id",
                "state_id",
                "source_failure_name",
                "scoped_failure_definition",
                "scoped_failure_theorem",
                "flow_theorem",
                "receiver_theorem",
            ] {
                let mut altered = original.clone();
                altered["functions"][0]["edges"][owned]["ownership"][field] =
                    Value::String("wrong-source-point".into());
                assert!(import_csharp_practical_ordinary_control_edges(
                    &serde_json::to_vec(&altered).unwrap(),
                    p.certificate_bytes(),
                    vir
                )
                .is_err());
            }
            for field in [
                "construction_definitions",
                "symbolic_ownership",
                "symbolic_ownership_proofs",
            ] {
                let mut altered = original.clone();
                altered.as_object_mut().unwrap().remove(field);
                assert!(import_csharp_practical_ordinary_control_edges(
                    &serde_json::to_vec(&altered).unwrap(),
                    p.certificate_bytes(),
                    vir
                )
                .is_err());
            }
        }
        let mut damaged = p.certificate_bytes().to_vec();
        *damaged.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_control_edges(
            &p.canonical_bytes(),
            &damaged,
            vir
        )
        .is_err());
        output(id, "json", &p.canonical_bytes());
        output(
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
        eprintln!(
            "control edges {id}: {case_pending} pending, {observations} cumulative observations"
        );
    }
    if !runtime {
        return;
    }
    assert!(
        observations > 100
            && pending == if ids.contains(&"type") { 1 } else { 0 }
            && false_guards > 0
            && true_exceptions > 0
    );
    if ids.contains(&"total_variable") {
        assert!(
            checked_goal_bindings.len() >= 4,
            "variable decrease must exercise actual W04 edge-goal binding identities"
        );
        assert!(
            checked_decrease_bindings.len() >= 2,
            "actual assignedness goals for entry/backedge variable decreases must be compared"
        );
    }
    eprintln!(
        "control edges: {} distinct original W04 goal bindings checked",
        checked_goal_bindings.len()
    );
}

#[test]
fn csharp_03_t06_w09_control_edge_source_relations() {
    run_cases(
        &[
            "while",
            "for",
            "short_circuit",
            "switch",
            "is_binding",
            "guard_order",
            "guard_throw",
            "index_update",
            "total_variable",
        ],
        true,
    );
}
#[test]
fn csharp_03_t06_w09_control_edge_construction_guards() {
    run_cases(&["index_update"], true);
}

#[test]
fn csharp_03_t06_w09_control_edge_other_sources_unchanged() {
    run_cases(
        &[
            "while",
            "for",
            "short_circuit",
            "switch",
            "is_binding",
            "guard_order",
            "guard_throw",
            "total_variable",
            "index_update",
        ],
        false,
    );
}

#[test]
fn csharp_03_t06_w09_control_edge_missing_pattern_guards_stay_pending() {
    let bundle = b();
    let rows = read("control-emission/source-cases.json");
    let row = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["stage"] == "patterns" && r["source_case"]["id"] == "type")
        .unwrap();
    let source_case = &row["source_case"];
    let (context, captures) = support::context(
        &bundle,
        source_case["root"].as_str().unwrap(),
        source_case["source"].as_str().unwrap().as_bytes(),
    );
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(&row["data"]).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    let vir = emitted.vir();
    let p = generate_csharp_practical_ordinary_control_edges(vir).unwrap();
    assert!(!p.functions().is_empty());
    let mut pending = 0;
    for e in p
        .functions()
        .iter()
        .flat_map(|f| &f.edges)
        .filter(|e| !e.pending_constant_names.is_empty())
    {
        assert!(e.guard_definition.is_none() && e.join.is_none() && e.ownership.is_none());
        pending += 1;
    }
    assert_eq!(pending, 1);
    let mut altered: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
    for f in altered["functions"].as_array_mut().unwrap() {
        for e in f["edges"].as_array_mut().unwrap() {
            e["pending_constant_names"] = json!([]);
        }
    }
    assert!(import_csharp_practical_ordinary_control_edges(
        &serde_json::to_vec(&altered).unwrap(),
        p.certificate_bytes(),
        vir
    )
    .is_err());
    eprintln!("unsupported type-pattern: {pending} explicit pending guards");
}

#[test]
fn csharp_03_t06_w09_control_edge_collection_candidates() {
    run_cases(
        &[
            "foreach_string",
            "foreach_string_var",
            "foreach_array",
            "foreach_array_var",
            "lookup",
            "governing_throw",
            "type",
            "string_property",
        ],
        false,
    );
}

#[test]
fn csharp_03_t06_w09_control_edge_collection_source_relations() {
    run_cases(
        &[
            "foreach_string",
            "foreach_string_var",
            "foreach_array",
            "foreach_array_var",
            "lookup",
            "governing_throw",
            "type",
            "string_property",
        ],
        true,
    );
}

#[test]
fn csharp_03_t06_w09_control_edge_option_source_relations() {
    run_cases(&["type"], true);
}
