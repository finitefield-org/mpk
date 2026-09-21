//! Exact edge guards, ordered failures and guarded slot joins at original nodes.
use super::*;
use std::path::PathBuf;

fn reference(
    term: &ContractTerm,
    args: &[i32],
    definitions: &[OrdinaryIntegerDataDefinition],
) -> bool {
    fn eval(t: &ContractTerm, args: &[i32], ds: &[OrdinaryIntegerDataDefinition]) -> i32 {
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
            .map(|t| eval(t, args, ds))
            .collect::<Vec<_>>();
        let result = match name.as_str() {
            "Mpk.CSharp.Bool.true" => true,
            "Mpk.CSharp.Bool.false" => false,
            "Mpk.CSharp.Bool.Not" => a[0] == 0,
            "Mpk.CSharp.Bool.And" => a[0] != 0 && a[1] != 0,
            "Mpk.CSharp.Bool.Or" => a[0] != 0 || a[1] != 0,
            _ => {
                let (d, i) = ds
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
    eval(term, args, definitions) != 0
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
#[test]
fn csharp_03_t06_w09_control_edge_source_relations() {
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
    for id in [
        "while",
        "for",
        "short_circuit",
        "switch",
        "is_binding",
        "guard_order",
        "guard_throw",
        "index_update",
        "total_variable",
    ] {
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
                    pending += 1;
                    case_pending += 1;
                    continue;
                };
                assert!(edge.pending_constant_names.is_empty());
                assert_eq!(
                    edge.join.is_some(),
                    edge.source.target_node_id.is_some() && edge.source.kind != "function_entry"
                );
                for sample in 0..7 {
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
                            _ => {
                                if i % 2 == 0 {
                                    1
                                } else {
                                    0
                                }
                            }
                        })
                        .collect::<Vec<_>>();
                    let expected =
                        reference(&edge.source.guard.term, &inputs, p.integer_definitions());
                    let values = edge
                        .source
                        .guard
                        .bindings
                        .iter()
                        .zip(&inputs)
                        .map(|(b, n)| encoded(depths[b.type_id.as_str()], *n))
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
        assert_eq!(case_pending > 0, id == "index_update");
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
    assert!(observations > 100 && pending > 0 && false_guards > 0 && true_exceptions > 0);
    assert!(
        checked_goal_bindings.len() >= 4,
        "variable decrease must exercise actual W04 edge-goal binding identities"
    );
    assert!(
        checked_decrease_bindings.len() >= 2,
        "actual assignedness goals for entry/backedge variable decreases must be compared"
    );
    eprintln!(
        "control edges: {} distinct original W04 goal bindings checked",
        checked_goal_bindings.len()
    );
}
