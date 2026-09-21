//! Exact edge guards, ordered failures and guarded slot joins at original nodes.
use super::*;
use std::path::PathBuf;

#[test]
fn csharp_03_t06_w09_control_edge_phi_preserves_existing_declarations() {
    preserves_existing_declarations("before-phi-joins", true);
}
#[test]
fn csharp_03_t06_w09_control_memory_preserves_existing_declarations() {
    preserves_existing_declarations("before-memory-bindings", false);
}
#[test]
fn csharp_03_t06_w09_control_projection_preserves_existing_declarations() {
    preserves_existing_declarations("before-public-projection", false);
}
#[test]
fn csharp_03_t06_w09_control_builtin_throw_preserves_existing_declarations() {
    preserves_existing_declarations("before-builtin-throw", false);
}
#[test]
fn csharp_03_t06_w09_control_node_entry_preserves_existing_declarations() {
    preserves_existing_declarations("before-node-entry", false);
}
#[test]
fn csharp_03_t06_w09_control_memory_effect_preserves_existing_declarations() {
    preserves_existing_declarations("before-memory-effects", false);
}
// Compare syntax modulo canonical term/global numbering. Nullable slot joins
// deliberately change carrier types; every other old definition must remain.
fn declaration_bodies(c: &mpk_cert::encode::Certificate) -> BTreeMap<String, Value> {
    use mpk_cert::encode::{DeclarationKind as D, LevelNode as L, TermNode as T};
    use sha2::{Digest, Sha256};
    let hash =
        |value: Value| -> Vec<u8> { Sha256::digest(serde_json::to_vec(&value).unwrap()).to_vec() };
    let global = |id: u32| c.name_table[c.declarations[id as usize].name as usize].clone();
    let mut levels: Vec<Vec<u8>> = vec![];
    for level in &c.level_table {
        levels.push(hash(match level {
            L::Zero => json!(["zero"]),
            L::Succ(a) => json!(["succ", levels[*a as usize]]),
            L::Max(a, b) => json!(["max", levels[*a as usize], levels[*b as usize]]),
            L::Param(n) => json!(["param", c.name_table[*n as usize]]),
        }));
    }
    let mut terms: Vec<Vec<u8>> = vec![];
    for term in &c.term_table {
        terms.push(hash(match term {
            T::Sort(l) => json!(["sort", levels[*l as usize]]),
            T::Var(v) => json!(["var", v]),
            T::Const {
                global: g,
                levels: ls,
            } => json!([
                "const",
                global(*g),
                ls.iter().map(|l| &levels[*l as usize]).collect::<Vec<_>>()
            ]),
            T::App {
                function,
                arguments,
            } => json!([
                "app",
                terms[*function as usize],
                arguments
                    .iter()
                    .map(|a| &terms[*a as usize])
                    .collect::<Vec<_>>()
            ]),
            T::Lam { ty, body } => json!(["lam", terms[*ty as usize], terms[*body as usize]]),
            T::Pi { ty, body } => json!(["pi", terms[*ty as usize], terms[*body as usize]]),
            T::Let { ty, value, body } => json!([
                "let",
                terms[*ty as usize],
                terms[*value as usize],
                terms[*body as usize]
            ]),
        }));
    }
    c.declarations
        .iter()
        .map(|d| {
            let body = match &d.kind {
                D::Axiom { ty } => json!(["axiom", terms[*ty as usize]]),
                D::Def {
                    ty,
                    value,
                    reducibility,
                } => json!([
                    "def",
                    terms[*ty as usize],
                    terms[*value as usize],
                    format!("{reducibility:?}")
                ]),
                D::Theorem { ty, proof } => {
                    json!(["theorem", terms[*ty as usize], terms[*proof as usize]])
                }
                D::Inductive { ty } => json!(["inductive", terms[*ty as usize]]),
                D::Constructor {
                    ty,
                    inductive,
                    generated,
                } => json!([
                    "constructor",
                    terms[*ty as usize],
                    global(*inductive),
                    generated
                ]),
                D::Recursor {
                    ty,
                    inductive,
                    generated,
                } => json!([
                    "recursor",
                    terms[*ty as usize],
                    global(*inductive),
                    generated
                ]),
                D::TheoryPrimitive { ty } => json!(["theory", terms[*ty as usize]]),
            };
            (c.name_table[d.name as usize].clone(), body)
        })
        .collect()
}

fn preserves_existing_declarations(previous_folder: &str, strip_phis: bool) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation");
    let previous = root.join(format!(
        "verification-logs/control-edges/{previous_folder}/control-edges"
    ));
    let current = std::env::var_os("MPK_W09_CONTROL_EDGES_OUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("control-edges"));
    let mut count = 0;
    let mut added = 0;
    let mut memory_records = 0;
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
        for (old, new) in before_meta["functions"]
            .as_array()
            .unwrap()
            .iter()
            .zip(after_meta["functions"].as_array_mut().unwrap())
        {
            for (old_edge, new_edge) in old["edges"]
                .as_array()
                .unwrap()
                .iter()
                .zip(new["edges"].as_array_mut().unwrap())
            {
                if old_edge.get("builtin_throw_source_node_id").is_none() {
                    if let Some(source) = new_edge
                        .as_object_mut()
                        .unwrap()
                        .remove("builtin_throw_source_node_id")
                    {
                        assert_eq!(file.file_stem().unwrap(), "type");
                        assert!(old["source"]["source_graph"]["nodes"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|n| n["id"] == source && n["kind"] == "builtin_throw"));
                        assert_eq!(
                            old_edge["pending_constant_names"].as_array().unwrap().len(),
                            1
                        );
                        assert!(new_edge["pending_constant_names"]
                            .as_array()
                            .unwrap()
                            .is_empty());
                        assert!(
                            new_edge["guard_definition"].is_string()
                                && new_edge["join"].is_object()
                        );
                        for field in ["guard_definition", "join", "pending_constant_names"] {
                            new_edge[field] = old_edge[field].clone();
                        }
                        new_edge.as_object_mut().unwrap().remove("phi_join");
                        if let Some(phi) = old_edge.get("phi_join") {
                            new_edge["phi_join"] = phi.clone();
                        }
                    }
                }
            }
        }
        for (old, new) in before_meta["functions"]
            .as_array()
            .unwrap()
            .iter()
            .zip(after_meta["functions"].as_array_mut().unwrap())
        {
            for (old_edge, new_edge) in old["edges"]
                .as_array()
                .unwrap()
                .iter()
                .zip(new["edges"].as_array_mut().unwrap())
            {
                for field in ["join", "phi_join"] {
                    if let Some(rule) = old_edge[field]["state_rule"].as_str() {
                        if rule.ends_with("node_entry_merge_pending") {
                            assert_eq!(
                                new_edge[field]["state_rule"],
                                rule.replace(
                                    "node_entry_merge_pending",
                                    "node_entry_merge_separate"
                                )
                            );
                            new_edge[field]["state_rule"] = json!(rule);
                        }
                    }
                }
            }
        }
        let overrides = after_meta["functions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f.get("slot_type_overrides").is_some())
            && !before_meta["functions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|f| f.get("slot_type_overrides").is_some());
        if overrides {
            assert_eq!(file.file_stem().unwrap(), "type");
            assert_eq!(before.proof_node_table, after.proof_node_table);
            assert_eq!(before.theory_certificates, after.theory_certificates);
            let old_joins = before_meta["functions"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|f| f["edges"].as_array().unwrap())
                .filter_map(|e| e["join"]["definition"].as_str())
                .collect::<BTreeSet<_>>();
            let new_bodies = declaration_bodies(&after);
            for (name, body) in declaration_bodies(&before) {
                if !old_joins.contains(name.as_str()) {
                    assert_eq!(
                        Some(&body),
                        new_bodies.get(&name),
                        "changed non-slot-join {name}"
                    );
                }
            }
            for (old, new) in before_meta["functions"]
                .as_array()
                .unwrap()
                .iter()
                .zip(after_meta["functions"].as_array_mut().unwrap())
            {
                let lifted = new
                    .as_object_mut()
                    .unwrap()
                    .remove("slot_type_overrides")
                    .unwrap();
                assert_eq!(lifted.as_object().unwrap().len(), 1);
                assert!(lifted["local:0"].is_string());
                let nominal = old["source"]["slots"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|s| s[0] == "local:0")
                    .unwrap()[1]
                    .clone();
                assert_ne!(nominal, lifted["local:0"]);
                for (old_edge, new_edge) in old["edges"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .zip(new["edges"].as_array_mut().unwrap())
                {
                    if let Some(join) = new_edge["join"].as_object_mut() {
                        let n = join["guard_argument_count"].as_u64().unwrap() as usize;
                        let mut changed = 0;
                        for arg in join["arguments"].as_array_mut().unwrap().iter_mut().skip(n) {
                            if arg["kind"] == "current_slot" && arg["value_id"] == "local:0" {
                                assert_eq!(arg["type_id"], lifted["local:0"]);
                                arg["type_id"] = nominal.clone();
                                changed += 1;
                            }
                        }
                        assert_eq!(changed, 2);
                        assert_ne!(join["definition"], old_edge["join"]["definition"]);
                        join.insert("definition".into(), old_edge["join"]["definition"].clone());
                        assert_eq!(new_edge["join"], old_edge["join"]);
                    }
                }
            }
        } else {
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
        }
        for (before_function, function) in before_meta["functions"]
            .as_array()
            .unwrap()
            .iter()
            .zip(after_meta["functions"].as_array_mut().unwrap())
        {
            for field in [
                "node_entries",
                "memory_effects",
                "pending_memory_effect_node_ids",
            ] {
                if before_function.get(field).is_none() {
                    function.as_object_mut().unwrap().remove(field);
                }
            }
            if let Some(mut memory) = function.as_object_mut().unwrap().remove("memory_bindings") {
                memory_records += memory.as_array().unwrap().len();
                if previous_folder == "before-public-projection" {
                    for binding in memory.as_array_mut().unwrap() {
                        binding
                            .as_object_mut()
                            .unwrap()
                            .remove("public_projection_definedness");
                        binding
                            .as_object_mut()
                            .unwrap()
                            .remove("public_projection_definition");
                        binding
                            .as_object_mut()
                            .unwrap()
                            .remove("source_slot_snapshot_definition");
                        binding["public_slot_projection_pending"] = json!(true);
                    }
                }
                if matches!(
                    previous_folder,
                    "before-public-projection"
                        | "before-builtin-throw"
                        | "before-node-entry"
                        | "before-memory-effects"
                ) {
                    function
                        .as_object_mut()
                        .unwrap()
                        .insert("memory_bindings".into(), memory);
                }
            }
            for edge in function["edges"].as_array_mut().unwrap() {
                if strip_phis {
                    added +=
                        usize::from(edge.as_object_mut().unwrap().remove("phi_join").is_some());
                }
            }
        }
        after_meta["certificate_sha256"] = before_meta["certificate_sha256"].clone();
        assert_eq!(before_meta, after_meta);
        count += 1;
    }
    assert_eq!(count, 17);
    assert!(if strip_phis {
        added > 0
    } else {
        memory_records == 4
    });
    eprintln!(
        "preserved {count} programs modulo explicitly checked nullable slot join replacements; added {added} phi joins and {memory_records} memory bindings"
    );
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
                if edge.builtin_throw_source_node_id.is_some() {
                    assert!(a.is_empty());
                    assert_eq!(name, &format!("Mpk.CSharp.Control.ExceptionEdge.{}.exception.closed.System.Runtime.CompilerServices.SwitchExpressionException", edge.source.source_node_id));
                    return 1;
                }
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
fn run_node_entries(
    p: &OrdinaryControlEdgeProgram,
    vir: &mpk_vc::csharp_practical_vir_validation::ValidatedPracticalVir,
    c: &mpk_cert::encode::Certificate,
    depths: &BTreeMap<&str, u32>,
    factored_only: bool,
) -> usize {
    let mut observations = 0;
    for f in p.functions() {
        let native = vir
            .functions()
            .iter()
            .find(|n| n.id == f.source.function_id)
            .unwrap();
        assert_eq!(f.node_entries.len(), native.blocks.len());
        for (entry, block) in f.node_entries.iter().zip(&native.blocks) {
            assert_eq!(entry.node_id, block.node.id);
            assert_eq!(
                entry.entry_state_argument_count,
                2 * f.source.slots.len() + block.phi_values.len()
            );
            let edges = f
                .edges
                .iter()
                .filter(|e| e.source.target_node_id.as_ref() == Some(&block.node.id))
                .collect::<Vec<_>>();
            assert_eq!(entry.incoming.len(), edges.len());
            if factored_only && entry.components.is_empty() {
                continue;
            }
            let n = entry.entry_state_argument_count;
            if entry.components.is_empty() {
                assert!(
                    entry.definition.is_some(),
                    "incoming guards must be defined"
                );
            } else {
                assert!(entry.definition.is_none());
                assert!(entry.arguments.len() > 256);
                assert_eq!(entry.components.len(), 1 + edges.len());
                assert_eq!(entry.components[0].role, "exactly_one_incoming");
                assert_eq!(
                    entry.components[0].argument_indices,
                    entry
                        .incoming
                        .iter()
                        .map(|i| i.selected_argument)
                        .collect::<Vec<_>>()
                );
                if factored_only {
                    // The original count/fill exit has eleven predecessors.
                    // Exhaust the selector truth table independently of the
                    // guards and arrival snapshots.
                    assert_eq!(entry.incoming.len(), 11);
                    for mask in 0u32..(1 << 11) {
                        let selected = (0..11).map(|i| V::Bit(mask & (1 << i) != 0)).collect();
                        assert_eq!(
                            bit(run(c, &entry.components[0].definition, selected)),
                            mask.count_ones() == 1
                        );
                        observations += 1;
                    }
                }
                for (part, incoming) in entry.components[1..].iter().zip(&entry.incoming) {
                    assert_eq!(part.role, format!("selected_arrival:{}", incoming.edge_id));
                    let want = (0..n)
                        .chain(incoming.selected_argument..incoming.state_argument_start + n)
                        .collect::<Vec<_>>();
                    assert_eq!(part.argument_indices, want);
                    assert!(part.argument_indices.len() <= 256);
                }
            }
            let evaluate = |args: Vec<V>| {
                if let Some(definition) = &entry.definition {
                    bit(run(c, definition, args))
                } else {
                    assert!(!entry.components.is_empty());
                    entry.components.iter().all(|part| {
                        bit(run(
                            c,
                            &part.definition,
                            part.argument_indices
                                .iter()
                                .map(|&i| args[i].clone())
                                .collect(),
                        ))
                    })
                }
            };
            for (i, arg) in entry.arguments[..n].iter().enumerate() {
                assert_eq!(arg.node_id, block.node.id);
                assert!(arg.edge_id.is_none());
                if i < 2 * f.source.slots.len() {
                    let (slot, ty) = &f.source.slots[i / 2];
                    assert_eq!(&arg.value_id, slot);
                    assert_eq!(
                        arg.kind,
                        if i % 2 == 0 {
                            "slot_assigned"
                        } else {
                            "current_slot"
                        }
                    );
                    assert_eq!(
                        &arg.type_id,
                        if i % 2 == 0 {
                            "mpk.csharp.value.bool.v1"
                        } else {
                            f.slot_type_overrides.get(slot).unwrap_or(ty)
                        }
                    );
                } else {
                    let phi = &block.phi_values[i - 2 * f.source.slots.len()];
                    assert_eq!(arg.kind, "ssa");
                    assert_eq!(arg.value_id, phi.value.id);
                    assert_eq!(arg.type_id, phi.value.type_id);
                }
            }
            let mut base = entry
                .arguments
                .iter()
                .map(|a| {
                    if a.kind == "incoming_edge_selected" {
                        V::Bit(false)
                    } else if a.kind == "slot_assigned" {
                        V::Bit(true)
                    } else {
                        encoded(depths[a.type_id.as_str()], 0)
                    }
                })
                .collect::<Vec<_>>();
            for incoming in &entry.incoming {
                for i in 0..n {
                    let a = &entry.arguments[incoming.state_argument_start + i];
                    if a.kind != "slot_assigned" {
                        base[incoming.state_argument_start + i] =
                            encoded(depths[a.type_id.as_str()], -1);
                    }
                }
            }
            assert!(!evaluate(base.clone()));
            observations += 1;
            let mut positives = vec![];
            for (incoming, edge) in entry.incoming.iter().zip(edges) {
                assert_eq!(incoming.edge_id, edge.source.id);
                assert!(incoming.pending_constant_names.is_empty());
                let select = &entry.arguments[incoming.selected_argument];
                assert_eq!(select.kind, "incoming_edge_selected");
                assert_eq!(select.edge_id.as_ref(), Some(&edge.source.id));
                assert_eq!(select.node_id, entry.node_id);
                assert_eq!(select.value_id, edge.source.id);
                assert_eq!(
                    incoming.guard_argument_start,
                    incoming.selected_argument + 1
                );
                assert_eq!(
                    incoming.guard_argument_count,
                    edge.source.guard.bindings.len()
                );
                assert_eq!(
                    &entry.arguments[incoming.guard_argument_start..incoming.state_argument_start],
                    edge.source.guard.bindings.as_slice()
                );
                for i in 0..n {
                    let mut want = entry.arguments[i].clone();
                    want.edge_id = Some(edge.source.id.clone());
                    assert_eq!(entry.arguments[incoming.state_argument_start + i], want);
                }
                let mut positive = None;
                for sample in [0, 1, i32::MIN, i32::MAX, -1] {
                    let inputs = vec![sample; incoming.guard_argument_count];
                    // The factored test verifies composition against the exact
                    // independently evaluated guard, without claiming new guard
                    // semantics coverage for the count/fill source.
                    let expected = if factored_only {
                        bit(run(
                            c,
                            edge.guard_definition.as_ref().unwrap(),
                            edge.source
                                .guard
                                .bindings
                                .iter()
                                .zip(&inputs)
                                .map(|(a, v)| {
                                    guard_input(p, &a.type_id, depths[a.type_id.as_str()], *v)
                                })
                                .collect(),
                        ))
                    } else {
                        reference(&edge.source.guard.term, &inputs, p, edge)
                    };
                    let mut args = base.clone();
                    args[incoming.selected_argument] = V::Bit(true);
                    for i in 0..n {
                        args[incoming.state_argument_start + i] = args[i].clone();
                    }
                    for (i, (binding, value)) in
                        edge.source.guard.bindings.iter().zip(&inputs).enumerate()
                    {
                        args[incoming.guard_argument_start + i] = guard_input(
                            p,
                            &binding.type_id,
                            depths[binding.type_id.as_str()],
                            *value,
                        );
                    }
                    assert_eq!(
                        evaluate(args.clone()),
                        expected,
                        "entry {} from {} sample {sample}",
                        entry.node_id,
                        edge.source.id
                    );
                    observations += 1;
                    if expected && positive.is_none() {
                        positive = Some(args);
                    }
                }
                if let Some(args) = positive {
                    let mut varied = args.clone();
                    for (i, a) in entry.arguments[..n].iter().enumerate() {
                        if a.kind != "slot_assigned" {
                            let value = encoded(depths[a.type_id.as_str()], 1 + (i % 2) as i32);
                            varied[i] = value.clone();
                            varied[incoming.state_argument_start + i] = value;
                        }
                    }
                    assert!(evaluate(varied));
                    observations += 1;
                    for i in 0..n {
                        let mut wrong = args.clone();
                        let a = &entry.arguments[i];
                        wrong[i] = if a.kind == "slot_assigned" {
                            V::Bit(false)
                        } else {
                            let depth = depths[a.type_id.as_str()];
                            if depth == 0 {
                                V::Bit(true)
                            } else {
                                sparse_cube(depth, BTreeSet::from([(1usize << depth) - 1]))
                            }
                        };
                        assert!(
                            !evaluate(wrong),
                            "selected state mutation {} arg {i}",
                            entry.node_id
                        );
                        observations += 1;
                    }
                    let mut unassigned = args.clone();
                    for (i, a) in entry.arguments[..n].iter().enumerate() {
                        if a.kind == "slot_assigned" {
                            unassigned[i] = V::Bit(false);
                            unassigned[incoming.state_argument_start + i] = V::Bit(false);
                        }
                        if a.kind == "current_slot" {
                            unassigned[i] = encoded(depths[a.type_id.as_str()], -1);
                        }
                    }
                    assert!(evaluate(unassigned));
                    observations += 1;
                    positives.push((incoming, args));
                }
            }
            if positives.len() >= 2 {
                let (first, mut args) = positives[0].clone();
                let (second, other) = &positives[1];
                let range = second.selected_argument..second.state_argument_start + n;
                args[range.clone()].clone_from_slice(&other[range]);
                args[first.selected_argument] = V::Bit(true);
                assert!(!evaluate(args));
                observations += 1;
            }
        }
    }
    if factored_only {
        assert!(observations > 100, "large entry must be exercised");
        assert_eq!(
            p.functions()
                .iter()
                .flat_map(|f| &f.node_entries)
                .filter(|e| !e.components.is_empty())
                .count(),
            1
        );
    }
    let original: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
    let entries = original["functions"][0]["node_entries"].as_array().unwrap();
    let position = entries
        .iter()
        .position(|e| !e["incoming"].as_array().unwrap().is_empty())
        .unwrap();
    for field in ["node_id", "state_rule", "definition"] {
        let mut altered = original.clone();
        altered["functions"][0]["node_entries"][position][field] = json!("wrong-entry");
        assert!(import_csharp_practical_ordinary_control_edges(
            &serde_json::to_vec(&altered).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
    }
    for field in [
        "edge_id",
        "selected_argument",
        "guard_argument_start",
        "state_argument_start",
    ] {
        let mut altered = original.clone();
        altered["functions"][0]["node_entries"][position]["incoming"][0][field] =
            json!("wrong-input");
        assert!(import_csharp_practical_ordinary_control_edges(
            &serde_json::to_vec(&altered).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
    }
    if factored_only {
        let position = entries
            .iter()
            .position(|e| e.get("components").is_some())
            .unwrap();
        for field in ["components", "argument_indices", "definition", "role"] {
            let mut altered = original.clone();
            if field == "components" {
                altered["functions"][0]["node_entries"][position]
                    .as_object_mut()
                    .unwrap()
                    .remove(field);
            } else if field == "argument_indices" {
                altered["functions"][0]["node_entries"][position]["components"][1][field][0] =
                    json!(1);
            } else {
                altered["functions"][0]["node_entries"][position]["components"][1][field] =
                    json!("wrong-component");
            }
            assert!(import_csharp_practical_ordinary_control_edges(
                &serde_json::to_vec(&altered).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
    }
    let mut altered = original.clone();
    altered["functions"][0]
        .as_object_mut()
        .unwrap()
        .remove("node_entries");
    assert!(import_csharp_practical_ordinary_control_edges(
        &serde_json::to_vec(&altered).unwrap(),
        p.certificate_bytes(),
        vir
    )
    .is_err());
    observations
}

#[test]
fn csharp_03_t06_w09_control_node_entry_large_fan_in() {
    run_selected_cases(&["count_fill"], false, true, false);
}

#[test]
fn csharp_03_t06_w09_control_node_entries_count_fill_source() {
    run_selected_cases_mode(&["count_fill"], false, true, false, false);
}

#[test]
fn csharp_03_t06_w09_control_node_entries_original_sources() {
    run_selected_cases(
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
        true,
        false,
    );
}

fn run_memory_effects(
    p: &OrdinaryControlEdgeProgram,
    vir: &mpk_vc::csharp_practical_vir_validation::ValidatedPracticalVir,
    c: &mpk_cert::encode::Certificate,
    depths: &BTreeMap<&str, u32>,
) -> usize {
    fn array(depth: u32, private: bool, values: &[u32], complete: bool) -> V {
        let length = values.len() as u32;
        let mut bits = (0..32)
            .filter(|i| length & (1 << i) != 0)
            .map(|i| i << (depth - 5))
            .collect::<BTreeSet<_>>();
        for (i, value) in values.iter().enumerate() {
            if private && complete {
                bits.insert(2 | (i << (depth - 14)));
            }
            for b in 0..32 {
                if value & (1 << b) != 0 {
                    bits.insert(if private {
                        1 | (i << 2) | (b << 16)
                    } else {
                        1 | (i << 1) | (b << 13)
                    });
                }
            }
        }
        sparse_cube(depth, bits)
    }
    let mut observations = 0;
    let mut effects = 0;
    for f in p.functions() {
        assert!(f.pending_memory_effect_node_ids.is_empty());
        for effect in &f.memory_effects {
            assert_eq!(effect.arguments.len(), 8);
            assert_eq!(effect.receiver_transfer.kind, "load");
            let native = vir
                .functions()
                .iter()
                .find(|n| n.id == f.source.function_id)
                .unwrap();
            let block = native
                .blocks
                .iter()
                .find(|b| b.node.id == effect.operation.node_id)
                .unwrap();
            let invocation = block.invocation.as_ref().unwrap();
            assert!(invocation.operation_id.ends_with(".rewrite"));
            assert_eq!(effect.source_result, invocation.operands[2]);
            assert_eq!(effect.operation.subjects[3], invocation.result);
            assert_ne!(effect.source_result.type_id, invocation.result.type_id);
            let anchor = native
                .control_protocol
                .as_ref()
                .unwrap()
                .anchors
                .iter()
                .find(|a| a.source_node_id == effect.source_node_id)
                .unwrap();
            assert!(anchor.artifact_node_ids.contains(&block.node.id));
            assert_eq!(effect.arguments[0].node_id, block.node.id);
            assert_eq!(effect.arguments[2].node_id, invocation.normal_successor_id);
            let source = f
                .source
                .source_graph
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|n| n.id == effect.source_node_id)
                .unwrap();
            assert_eq!(source.operation, "update");
            let old = vec![7, 11];
            let new = vec![13, 11];
            let public_depth = depths[effect.arguments[1].type_id.as_str()];
            let private_depth = depths[invocation.result.type_id.as_str()];
            let args = vec![
                V::Bit(true),
                array(public_depth, false, &old, true),
                V::Bit(true),
                array(public_depth, false, &new, true),
                array(private_depth, true, &old, true),
                encoded(5, 0),
                encoded(5, 13),
                array(private_depth, true, &new, true),
            ];
            assert!(bit(run(c, &effect.definition, args.clone())));
            observations += 1;
            for i in [0, 2] {
                let mut wrong = args.clone();
                wrong[i] = V::Bit(false);
                assert!(!bit(run(c, &effect.definition, wrong)));
                observations += 1;
            }
            for (i, v) in [
                (1, array(public_depth, false, &new, true)),
                (3, array(public_depth, false, &old, true)),
                (4, array(private_depth, true, &new, true)),
                (7, array(private_depth, true, &old, true)),
                (4, array(private_depth, true, &old, false)),
                (7, array(private_depth, true, &new, false)),
                (5, encoded(5, -1)),
                (5, encoded(5, 2)),
                (6, encoded(5, 12)),
            ] {
                let mut wrong = args.clone();
                wrong[i] = v;
                assert!(
                    !bit(run(c, &effect.definition, wrong)),
                    "memory effect mutation argument {i}"
                );
                observations += 1;
            }
            // A write to the other element must change the corresponding source
            // snapshot, without confusing the element result with native memory.
            let mut other = args.clone();
            other[5] = encoded(5, 1);
            other[3] = array(public_depth, false, &[7, 13], true);
            other[7] = array(private_depth, true, &[7, 13], true);
            assert!(bit(run(c, &effect.definition, other)));
            observations += 1;
            effects += 1;
        }
    }
    assert_eq!(effects, 1);
    let original: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
    for field in [
        "source_node_id",
        "source_result",
        "receiver_transfer",
        "allocation_origin_id",
        "before_state_id",
        "after_state_id",
        "ownership",
        "snapshot_definition",
        "execution_scope",
    ] {
        let mut changed = original.clone();
        changed["functions"][0]["memory_effects"][0][field] = json!("wrong-binding");
        assert!(import_csharp_practical_ordinary_control_edges(
            &serde_json::to_vec(&changed).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
    }
    let mut changed = original;
    changed["functions"][0]
        .as_object_mut()
        .unwrap()
        .remove("memory_effects");
    assert!(import_csharp_practical_ordinary_control_edges(
        &serde_json::to_vec(&changed).unwrap(),
        p.certificate_bytes(),
        vir
    )
    .is_err());
    observations
}
#[test]
fn csharp_03_t06_w09_control_memory_effect_original_source() {
    run_selected_cases(&["index_update"], false, false, true);
}

#[test]
fn csharp_03_t06_w09_control_memory_effect_count_fill_source() {
    run_selected_cases(&["count_fill"], false, false, true);
}

fn run_cases(ids: &[&str], runtime: bool) {
    run_selected_cases(ids, runtime, false, false);
}
fn run_selected_cases(ids: &[&str], runtime: bool, entries: bool, memory: bool) {
    run_selected_cases_mode(ids, runtime, entries, memory, ids == ["count_fill"]);
}
fn run_selected_cases_mode(
    ids: &[&str],
    runtime: bool,
    entries: bool,
    memory: bool,
    factored_only: bool,
) {
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
        if memory {
            let count = run_memory_effects(&p, vir, &c, &depths);
            eprintln!("control memory effects {id}: {count} observations");
        }
        if entries {
            let count = run_node_entries(&p, vir, &c, &depths, factored_only);
            eprintln!("control node entries {id}: {count} observations");
        }
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
        let ownership_traces = vir.symbolic_construction_ownership().unwrap();
        let mut refreshed_memory_loads = 0;
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
            for binding in &f.memory_bindings {
                let transfer = &binding.transfer;
                assert!(f.source.transfers.contains(transfer));
                assert_eq!(
                    &binding.source_slot_type_id,
                    &f.source
                        .slots
                        .iter()
                        .find(|(s, _)| s == &transfer.slot)
                        .unwrap()
                        .1
                );
                // Captured array temporaries already store private native
                // memory. Only public source slots require the projection.
                let native_slot = binding.source_slot_type_id == transfer.value.type_id;
                assert_eq!(binding.public_slot_projection_pending, native_slot);
                assert_eq!(binding.public_projection_definedness.is_none(), native_slot);
                assert_eq!(binding.public_projection_definition.is_none(), native_slot);
                assert_eq!(
                    binding.source_slot_snapshot_definition.is_none(),
                    native_slot
                );
                let loading = transfer.kind == "load";
                let node = if loading {
                    &transfer.entry_node_id
                } else {
                    &transfer.exit_node_id
                };
                let trace = ownership_traces
                    .iter()
                    .find(|t| t.function_id == f.source.function_id)
                    .unwrap();
                let state = trace.blocks.iter().find(|b| &b.node_id == node).unwrap();
                let live = if loading {
                    &state.before_invocation
                } else {
                    &state.normal
                };
                let expected = &live[&binding.allocation_origin_id];
                assert_eq!(
                    binding.state_id,
                    format!("{node}.{}", if loading { "invoke" } else { "normal" })
                );
                assert_eq!(binding.arguments.len(), 2);
                assert_eq!(binding.arguments[0].kind, "native_slot");
                assert_eq!(binding.arguments[0].value_id, transfer.slot);
                assert_eq!(binding.arguments[1].kind, "ssa");
                assert_eq!(&binding.arguments[1].value_id, expected);
                for a in &binding.arguments {
                    assert_eq!(&a.node_id, node);
                    assert_eq!(a.edge_id, None);
                    assert_eq!(a.type_id, transfer.value.type_id);
                }
                let proof = p
                    .symbolic_ownership_proofs()
                    .iter()
                    .find(|p| p.function_id == f.source.function_id)
                    .unwrap();
                assert_eq!(binding.flow_theorem, proof.flow_theorem);
                refreshed_memory_loads += usize::from(loading && expected != &transfer.value.id);
                let depth = depths[transfer.value.type_id.as_str()];
                // Complete native storage equality, including private capacity
                // beyond the published sequence's 4096-element boundary.
                for length in [0u32, 1, 4096, 4097, 16384] {
                    let mut cells = (0..32)
                        .filter(|b| length & (1 << b) != 0)
                        .map(|b| b << (depth - 5))
                        .collect::<BTreeSet<_>>();
                    if length > 0 {
                        cells.insert(1 | ((length as usize - 1) << 2));
                        cells.insert(2 | ((length as usize - 1) << (depth - 14)));
                    }
                    let current = sparse_cube(depth, cells.clone());
                    assert!(bit(run(
                        &c,
                        &binding.definition,
                        vec![current.clone(), current.clone()]
                    )));
                    observations += 1;
                    for position in [1usize, (1usize << depth) - 1] {
                        let mut stale = cells.clone();
                        if !stale.remove(&position) {
                            stale.insert(position);
                        }
                        let stale = sparse_cube(depth, stale);
                        for reverse in [false, true] {
                            let args = if reverse {
                                vec![current.clone(), stale.clone()]
                            } else {
                                vec![stale.clone(), current.clone()]
                            };
                            assert!(
                                !bit(run(&c, &binding.definition, args)),
                                "{id}: stale or corrupted native memory"
                            );
                            observations += 1;
                        }
                    }
                }
            }
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
                    assert_eq!(phi_join.state_rule, "parallel_phi_inputs_at_source_exit_to_edge_specific_target; node_entry_merge_separate");
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
                            "source_exit_to_edge_specific_target; node_entry_merge_separate"
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
                                        f.slot_type_overrides.get(slot).unwrap_or(ty)
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
                            let mut stored_binding = binding.clone();
                            if binding.kind == "current_slot" {
                                if let Some(storage) = f.slot_type_overrides.get(&binding.value_id)
                                {
                                    stored_binding.type_id = storage.clone();
                                }
                            }
                            assert!(
                                join.arguments[n..]
                                    .chunks_exact(4)
                                    .any(|pair| pair[2..].contains(&stored_binding)),
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
                        if f.slot_type_overrides.contains_key("local:0") {
                            let i = f
                                .source
                                .slots
                                .iter()
                                .position(|(slot, _)| slot == "local:0")
                                .unwrap();
                            let source = n + 4 * i + 1;
                            let target = n + 4 * i + 3;
                            assert_eq!(depths[join.arguments[source].type_id.as_str()], 6);
                            let mut present = args.clone();
                            // Same Some(Box(17)) state is retained across an enabled edge.
                            present[source] = sparse_cube(6, BTreeSet::from([0, 1, 9]));
                            present[target] = present[source].clone();
                            assert!(bit(run(&c, &join.definition, present.clone())));
                            // Keep payload bits, change only nullable presence on target.
                            present[target] = sparse_cube(6, BTreeSet::from([1, 9]));
                            assert_eq!(bit(run(&c, &join.definition, present)), !expected);
                            observations += 2;
                        }
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
        assert_eq!(case_pending, 0);
        let original: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        if id == "type" {
            let functions = original["functions"].as_array().unwrap();
            assert_eq!(functions.len(), 1);
            assert_eq!(
                functions[0]["slot_type_overrides"]
                    .as_object()
                    .unwrap()
                    .len(),
                1
            );
            let mut changed = original.clone();
            changed["functions"][0]
                .as_object_mut()
                .unwrap()
                .remove("slot_type_overrides");
            assert!(import_csharp_practical_ordinary_control_edges(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
            changed = original.clone();
            let nominal = functions[0]["source"]["slots"]
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s[0] == "local:0")
                .unwrap()[1]
                .clone();
            changed["functions"][0]["slot_type_overrides"]["local:0"] = nominal;
            assert!(import_csharp_practical_ordinary_control_edges(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }

        if id == "count_fill" {
            assert_eq!(refreshed_memory_loads, 2);
            let bindings = p
                .functions()
                .iter()
                .flat_map(|f| &f.memory_bindings)
                .collect::<Vec<_>>();
            assert_eq!(bindings.len(), 4);
            assert_eq!(
                bindings
                    .iter()
                    .filter(|b| b.source_slot_type_id == b.transfer.value.type_id)
                    .count(),
                1
            );
        }
        if id == "index_update" {
            assert_eq!(
                p.functions()
                    .iter()
                    .map(|f| f.memory_bindings.len())
                    .sum::<usize>(),
                4
            );
            assert_eq!(refreshed_memory_loads, 3);
            let function_index = original["functions"]
                .as_array()
                .unwrap()
                .iter()
                .position(|f| f["memory_bindings"].is_array())
                .unwrap();
            for field in [
                "transfer",
                "source_slot_type_id",
                "allocation_origin_id",
                "state_id",
                "flow_theorem",
                "arguments",
                "definition",
                "public_slot_projection_pending",
                "public_projection_definedness",
                "public_projection_definition",
                "source_slot_snapshot_definition",
            ] {
                let mut altered = original.clone();
                altered["functions"][function_index]["memory_bindings"][0]
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
            let mut stale = original.clone();
            let binding = &mut stale["functions"][function_index]["memory_bindings"][1];
            binding["arguments"][1]["value_id"] = binding["transfer"]["value"]["id"].clone();
            assert_ne!(stale, original);
            assert!(import_csharp_practical_ordinary_control_edges(
                &serde_json::to_vec(&stale).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
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
    assert!(observations > 100 && pending == 0 && false_guards > 0 && true_exceptions > 0);
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
fn csharp_03_t06_w09_control_edge_count_fill_source_relations() {
    run_cases(&["count_fill"], true);
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
fn csharp_03_t06_w09_control_edge_builtin_throw_is_source_bound() {
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
    assert!(p
        .functions()
        .iter()
        .flat_map(|f| &f.edges)
        .all(|e| e.pending_constant_names.is_empty()));
    let mut throws = 0;
    for (f, e) in p
        .functions()
        .iter()
        .flat_map(|f| f.edges.iter().map(move |e| (f, e)))
        .filter(|(_, e)| e.builtin_throw_source_node_id.is_some())
    {
        let source_id = e.builtin_throw_source_node_id.as_ref().unwrap();
        let node = f
            .source
            .source_graph
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|n| &n.id == source_id)
            .unwrap();
        assert_eq!(node.kind, "builtin_throw");
        assert_eq!(
            node.slot,
            "System.Runtime.CompilerServices.SwitchExpressionException"
        );
        let native = vir
            .functions()
            .iter()
            .find(|n| n.id == f.source.function_id)
            .unwrap();
        let anchor = native
            .control_protocol
            .as_ref()
            .unwrap()
            .anchors
            .iter()
            .find(|a| &a.source_node_id == source_id)
            .unwrap();
        assert_eq!(anchor.exit_node_id, e.source.source_node_id);
        let block = native
            .blocks
            .iter()
            .find(|b| b.node.id == anchor.exit_node_id)
            .unwrap();
        assert!(block.literal_values.iter().any(|l| Some(&l.result.id)
            == block.abrupt_value_id.as_ref()
            && matches!(
                l.value,
                MonomorphicValue::ClosedException {
                    tag: 8,
                    payload: None,
                    source_type_id: None,
                    ..
                }
            )));
        assert!(e.join.is_some() && e.ownership.is_none());
        let certificate = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        assert!(bit(run(
            &certificate,
            e.guard_definition.as_ref().unwrap(),
            vec![]
        )));
        throws += 1;
    }
    assert_eq!(throws, 1);
    for replacement in [Value::Null, json!("wrong-source-anchor")] {
        let mut altered: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        for f in altered["functions"].as_array_mut().unwrap() {
            for e in f["edges"].as_array_mut().unwrap() {
                if e.get("builtin_throw_source_node_id").is_some() {
                    e["builtin_throw_source_node_id"] = replacement.clone();
                }
            }
        }
        assert!(import_csharp_practical_ordinary_control_edges(
            &serde_json::to_vec(&altered).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
    }
    eprintln!("built-in type-pattern throw: {throws} exact source anchor and tag-8 literal");
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
