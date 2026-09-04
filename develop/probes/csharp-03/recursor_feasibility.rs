//! W09 pre-capacity feasibility probe. This does not implement a foundation.
use mpk_cert::{
    axiom_report_hash_for_report, build_axiom_report, build_export_block,
    decode_canonical_certificate,
    encode::{
        Certificate, CertificateHashes, Declaration, DeclarationKind, DefinitionReducibility,
        LevelNode, TermNode,
    },
    encode_certificate, export_block_hash,
};
use serde_json::{json, Value};

const INPUTS: [&str; 2] = ["proofs/std/bool/std-bool.hex", "proofs/std/nat/std-nat.hex"];
const RECORD: &str = "develop/migrations/csharp-03/probes/recursor-feasibility.json";
const CAPACITY_RECORD: &str = "develop/migrations/csharp-03/probes/checker-capacity.json";
const TERM_LIMIT: u32 = 262_144;
const DECLARATION_LIMIT: u32 = 8_192;
const BINDER_DEPTH_LIMIT: u32 = 256;
const TRANSFORMER_LIMIT: u32 = 16_384;

fn from_hex(hex: &str) -> Vec<u8> {
    let digits: String = hex.chars().filter(|c| !c.is_whitespace()).collect();
    assert_eq!(digits.len() % 2, 0);
    (0..digits.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&digits[i..i + 2], 16).unwrap())
        .collect()
}

fn base() -> Certificate {
    let mut result = decode_canonical_certificate(&from_hex(
        &String::from_utf8(super::read(INPUTS[0])).unwrap(),
    ))
    .unwrap();
    let source = decode_canonical_certificate(&from_hex(
        &String::from_utf8(super::read(INPUTS[1])).unwrap(),
    ))
    .unwrap();
    assert!(result.imports.is_empty() && source.imports.is_empty());
    assert!(result.proof_node_table.is_empty() && source.proof_node_table.is_empty());
    assert!(result.theory_certificates.is_empty() && source.theory_certificates.is_empty());
    let n = result.name_table.len() as u32;
    let l = result.level_table.len() as u32;
    let t = result.term_table.len() as u32;
    let g = result.declarations.len() as u32;
    result.name_table.extend(source.name_table);
    result
        .level_table
        .extend(source.level_table.into_iter().map(|node| match node {
            LevelNode::Zero => LevelNode::Zero,
            LevelNode::Succ(a) => LevelNode::Succ(a + l),
            LevelNode::Max(a, b) => LevelNode::Max(a + l, b + l),
            LevelNode::Param(a) => LevelNode::Param(a + n),
        }));
    result
        .term_table
        .extend(source.term_table.into_iter().map(|node| match node {
            TermNode::Sort(a) => TermNode::Sort(a + l),
            TermNode::Var(a) => TermNode::Var(a),
            TermNode::Const { global, levels } => TermNode::Const {
                global: global + g,
                levels: levels.into_iter().map(|a| a + l).collect(),
            },
            TermNode::App {
                function,
                arguments,
            } => TermNode::App {
                function: function + t,
                arguments: arguments.into_iter().map(|a| a + t).collect(),
            },
            TermNode::Lam { ty, body } => TermNode::Lam {
                ty: ty + t,
                body: body + t,
            },
            TermNode::Pi { ty, body } => TermNode::Pi {
                ty: ty + t,
                body: body + t,
            },
            TermNode::Let { ty, value, body } => TermNode::Let {
                ty: ty + t,
                value: value + t,
                body: body + t,
            },
        }));
    result.declarations.extend(
        source
            .declarations
            .into_iter()
            .map(|declaration| Declaration {
                name: declaration.name + n,
                kind: match declaration.kind {
                    DeclarationKind::Inductive { ty } => DeclarationKind::Inductive { ty: ty + t },
                    DeclarationKind::Constructor {
                        ty,
                        inductive,
                        generated,
                    } => DeclarationKind::Constructor {
                        ty: ty + t,
                        inductive: inductive + g,
                        generated,
                    },
                    DeclarationKind::Recursor {
                        ty,
                        inductive,
                        generated,
                    } => DeclarationKind::Recursor {
                        ty: ty + t,
                        inductive: inductive + g,
                        generated,
                    },
                    _ => {
                        panic!("Nat input changed: audit the input rather than widening the probe")
                    }
                },
            }),
    );
    result.module = "Probe.CSharp03.W09.Recursor".to_owned();
    result.export_block.clear();
    result.hashes = CertificateHashes::default();
    result
}

fn term(c: &mut Certificate, node: TermNode) -> u32 {
    let id = c.term_table.len() as u32;
    c.term_table.push(node);
    id
}

fn constant(c: &mut Certificate, name: &str) -> u32 {
    let global = c
        .declarations
        .iter()
        .position(|d| c.name_table[d.name as usize] == name)
        .unwrap() as u32;
    term(
        c,
        TermNode::Const {
            global,
            levels: vec![],
        },
    )
}

fn app(c: &mut Certificate, function: u32, arguments: &[u32]) -> u32 {
    term(
        c,
        TermNode::App {
            function,
            arguments: arguments.to_vec(),
        },
    )
}

fn seal(mut c: Certificate) -> Vec<u8> {
    let old_names = c.name_table.clone();
    c.name_table.sort();
    for d in &mut c.declarations {
        d.name = c
            .name_table
            .binary_search(&old_names[d.name as usize])
            .unwrap() as u32;
    }
    for node in &mut c.level_table {
        if let LevelNode::Param(id) = node {
            *id = c
                .name_table
                .binary_search(&old_names[*id as usize])
                .unwrap() as u32;
        }
    }
    c.export_block = build_export_block(&c).unwrap();
    c.axiom_report = build_axiom_report(&c).unwrap();
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    c.hashes.export_hash = export_block_hash(&c.export_block);
    c.hashes.axiom_report_hash = axiom_report_hash_for_report(&c.axiom_report);
    let bytes = encode_certificate(&c);
    decode_canonical_certificate(&bytes).unwrap();
    bytes
}

fn finish(mut c: Certificate, ty: u32, value: u32) -> Vec<u8> {
    let name = c.name_table.len() as u32;
    c.name_table.push("Probe.CSharp03.W09.Result".to_owned());
    c.declarations.push(Declaration {
        name,
        kind: DeclarationKind::Def {
            ty,
            value,
            reducibility: DefinitionReducibility::Reducible,
        },
    });
    seal(c)
}

fn probe(kind: &str, index: u32) -> Vec<u8> {
    let mut c = base();
    let bool_ty = constant(&mut c, "Std.Bool");
    let false_value = constant(&mut c, "Std.Bool.false");
    let true_value = constant(&mut c, "Std.Bool.true");
    let bool_rec = constant(&mut c, "Std.Bool.rec");
    let nat_ty = constant(&mut c, "Std.Nat");
    let zero = constant(&mut c, "Std.Nat.zero");
    let succ = constant(&mut c, "Std.Nat.succ");
    let nat_rec = constant(&mut c, "Std.Nat.rec");
    let var = term(&mut c, TermNode::Var(0));
    let mut numeral = zero;
    for _ in 0..index {
        numeral = app(&mut c, succ, &[numeral]);
    }
    let (ty, value) = match kind {
        "nat_to_nat" => {
            let next = app(&mut c, succ, &[var]);
            let inner = term(
                &mut c,
                TermNode::Lam {
                    ty: nat_ty,
                    body: next,
                },
            );
            let step = term(
                &mut c,
                TermNode::Lam {
                    ty: nat_ty,
                    body: inner,
                },
            );
            (nat_ty, app(&mut c, nat_rec, &[zero, step, numeral]))
        }
        "nat_to_bool" => {
            let inner = term(
                &mut c,
                TermNode::Lam {
                    ty: bool_ty,
                    body: var,
                },
            );
            let step = term(
                &mut c,
                TermNode::Lam {
                    ty: nat_ty,
                    body: inner,
                },
            );
            (bool_ty, app(&mut c, nat_rec, &[false_value, step, numeral]))
        }
        "bool_to_bool" => (
            bool_ty,
            app(
                &mut c,
                bool_rec,
                &[
                    false_value,
                    true_value,
                    if index == 0 { false_value } else { true_value },
                ],
            ),
        ),
        "bool_to_tree" => {
            let tree = term(
                &mut c,
                TermNode::Pi {
                    ty: nat_ty,
                    body: bool_ty,
                },
            );
            let leaf = term(
                &mut c,
                TermNode::Lam {
                    ty: nat_ty,
                    body: false_value,
                },
            );
            (
                tree,
                app(
                    &mut c,
                    bool_rec,
                    &[
                        leaf,
                        leaf,
                        if index == 0 { false_value } else { true_value },
                    ],
                ),
            )
        }
        "pointwise_bool_tree" => {
            let tree = term(
                &mut c,
                TermNode::Pi {
                    ty: nat_ty,
                    body: bool_ty,
                },
            );
            let selected = app(
                &mut c,
                bool_rec,
                &[
                    false_value,
                    true_value,
                    if index == 0 { false_value } else { true_value },
                ],
            );
            (
                tree,
                term(
                    &mut c,
                    TermNode::Lam {
                        ty: nat_ty,
                        body: selected,
                    },
                ),
            )
        }
        "binary_bool_cube" => {
            // C(2) = Bool -> Bool -> Bool.  Product/address selection is
            // performed only after both address binders have exposed a Bool
            // leaf, so every Bool.rec argument and result is exactly Bool.
            let tree = term(
                &mut c,
                TermNode::Pi {
                    ty: bool_ty,
                    body: bool_ty,
                },
            );
            let cube = term(
                &mut c,
                TermNode::Pi {
                    ty: bool_ty,
                    body: tree,
                },
            );
            let coordinate = term(&mut c, TermNode::Var(0));
            let selector = term(&mut c, TermNode::Var(1));
            let not_coordinate = app(&mut c, bool_rec, &[true_value, false_value, coordinate]);
            let selected = app(&mut c, bool_rec, &[not_coordinate, coordinate, selector]);
            let coordinate_lambda = term(
                &mut c,
                TermNode::Lam {
                    ty: bool_ty,
                    body: selected,
                },
            );
            (
                cube,
                term(
                    &mut c,
                    TermNode::Lam {
                        ty: bool_ty,
                        body: coordinate_lambda,
                    },
                ),
            )
        }
        "static_tree_fold" => {
            // A concrete function-valued state is advanced by ordinary
            // lambda/application/let composition.  Bool.rec is used only at
            // the exposed Bool coordinate, never with the tree as its result.
            let tree = term(
                &mut c,
                TermNode::Pi {
                    ty: bool_ty,
                    body: bool_ty,
                },
            );
            let transform = term(
                &mut c,
                TermNode::Pi {
                    ty: tree,
                    body: tree,
                },
            );
            let base_coordinate = term(&mut c, TermNode::Var(0));
            let base = term(
                &mut c,
                TermNode::Lam {
                    ty: bool_ty,
                    body: base_coordinate,
                },
            );

            // step = lambda(state). lambda(coordinate).
            //          if condition then state(not coordinate)
            //                       else state(coordinate)
            let coordinate = term(&mut c, TermNode::Var(0));
            let state = term(&mut c, TermNode::Var(1));
            let current = app(&mut c, state, &[coordinate]);
            let other_coordinate = app(&mut c, bool_rec, &[true_value, false_value, coordinate]);
            let other = app(&mut c, state, &[other_coordinate]);
            let condition = if index == 0 { false_value } else { true_value };
            let next_leaf = app(&mut c, bool_rec, &[current, other, condition]);
            let next_tree = term(
                &mut c,
                TermNode::Lam {
                    ty: bool_ty,
                    body: next_leaf,
                },
            );
            let step = term(
                &mut c,
                TermNode::Lam {
                    ty: tree,
                    body: next_tree,
                },
            );

            // compose = lambda(f). lambda(g). lambda(state). g(f(state))
            let compose_state = term(&mut c, TermNode::Var(0));
            let compose_g = term(&mut c, TermNode::Var(1));
            let compose_f = term(&mut c, TermNode::Var(2));
            let after_f = app(&mut c, compose_f, &[compose_state]);
            let after_g = app(&mut c, compose_g, &[after_f]);
            let state_lambda = term(
                &mut c,
                TermNode::Lam {
                    ty: tree,
                    body: after_g,
                },
            );
            let g_lambda = term(
                &mut c,
                TermNode::Lam {
                    ty: transform,
                    body: state_lambda,
                },
            );
            let compose = term(
                &mut c,
                TermNode::Lam {
                    ty: transform,
                    body: g_lambda,
                },
            );
            let composed = app(&mut c, compose, &[step, step]);
            let bound_transform = term(&mut c, TermNode::Var(0));
            let folded = app(&mut c, bound_transform, &[base]);
            (
                tree,
                term(
                    &mut c,
                    TermNode::Let {
                        ty: transform,
                        value: composed,
                        body: folded,
                    },
                ),
            )
        }
        _ => panic!("unknown probe"),
    };
    finish(c, ty, value)
}

fn cases() -> Vec<Value> {
    let mut rows = Vec::new();
    for (kind, accepted, count) in [
        ("bool_to_bool", true, 2),
        ("nat_to_nat", true, 3),
        ("bool_to_tree", false, 2),
        ("nat_to_bool", false, 3),
        ("pointwise_bool_tree", true, 2),
        ("binary_bool_cube", true, 1),
        ("static_tree_fold", true, 2),
    ] {
        for index in 0..count {
            let bytes = probe(kind, index);
            let c = decode_canonical_certificate(&bytes).unwrap();
            rows.push(json!({"id": format!("{kind}.{index}"), "expected": if accepted {"accepted"} else {"type_mismatch"}, "certificate_hex": bytes.iter().map(|b| format!("{b:02x}")).collect::<String>(), "raw_sha256": super::sha(&bytes), "size_bytes": bytes.len(), "declarations": c.declarations.len(), "terms": c.term_table.len(), "nat_recursor_applications": direct_constant_applications(&c, "Std.Nat.rec"), "proof_nodes": 0, "theory_certificates": 0, "axioms": 0}));
        }
    }
    rows
}

fn type_text(c: &Certificate, id: u32) -> String {
    match &c.term_table[id as usize] {
        TermNode::Const { global, levels } if levels.is_empty() => {
            c.name_table[c.declarations[*global as usize].name as usize].clone()
        }
        TermNode::Pi { ty, body } => format!("({} -> {})", type_text(c, *ty), type_text(c, *body)),
        _ => panic!("checked standard recursor signature changed"),
    }
}

fn recursor_types() -> Value {
    let c = base();
    let mut types = serde_json::Map::new();
    for name in ["Std.Bool.rec", "Std.Nat.rec"] {
        let declaration = c
            .declarations
            .iter()
            .find(|d| c.name_table[d.name as usize] == name)
            .unwrap();
        let DeclarationKind::Recursor {
            ty,
            generated: true,
            ..
        } = declaration.kind
        else {
            panic!("checked standard recursor is not generated");
        };
        types.insert(name.to_owned(), json!(type_text(&c, ty)));
    }
    assert_eq!(
        types["Std.Bool.rec"],
        "(Std.Bool -> (Std.Bool -> (Std.Bool -> Std.Bool)))"
    );
    assert_eq!(
        types["Std.Nat.rec"],
        "(Std.Nat -> ((Std.Nat -> (Std.Nat -> Std.Nat)) -> (Std.Nat -> Std.Nat)))"
    );
    Value::Object(types)
}

fn direct_constant_applications(c: &Certificate, name: &str) -> usize {
    c.term_table
        .iter()
        .filter(|node| {
            let TermNode::App { function, .. } = node else {
                return false;
            };
            let TermNode::Const { global, .. } = &c.term_table[*function as usize] else {
                return false;
            };
            c.name_table[c.declarations[*global as usize].name as usize] == name
        })
        .count()
}

fn balanced_bool_network(
    c: &mut Certificate,
    bool_rec: u32,
    false_value: u32,
    true_value: u32,
    nodes: u32,
) -> u32 {
    if nodes == 0 {
        return false_value;
    }
    let left_nodes = (nodes - 1) / 2;
    let right_nodes = nodes - 1 - left_nodes;
    let left = balanced_bool_network(c, bool_rec, false_value, true_value, left_nodes);
    let right = balanced_bool_network(c, bool_rec, false_value, true_value, right_nodes);
    app(
        c,
        bool_rec,
        &[
            left,
            right,
            if nodes.is_multiple_of(2) {
                false_value
            } else {
                true_value
            },
        ],
    )
}

fn capacity_term_case(target: u32) -> Vec<u8> {
    let mut c = base();
    c.module = format!("Probe.CSharp03.W09.Capacity.Term{target}");
    let bool_ty = constant(&mut c, "Std.Bool");
    let false_value = constant(&mut c, "Std.Bool.false");
    let true_value = constant(&mut c, "Std.Bool.true");
    let bool_rec = constant(&mut c, "Std.Bool.rec");
    let current = c.term_table.len() as u32;
    assert!(target >= current);
    let value = balanced_bool_network(&mut c, bool_rec, false_value, true_value, target - current);
    assert_eq!(c.term_table.len() as u32, target);
    finish(c, bool_ty, value)
}

fn capacity_declaration_case(target: u32) -> Vec<u8> {
    let mut c = base();
    c.module = format!("Probe.CSharp03.W09.Capacity.Declaration{target}");
    let bool_ty = constant(&mut c, "Std.Bool");
    let false_value = constant(&mut c, "Std.Bool.false");
    for index in 0..target {
        let name = c.name_table.len() as u32;
        c.name_table
            .push(format!("Probe.CSharp03.W09.Capacity.Generated.D{index:05}"));
        c.declarations.push(Declaration {
            name,
            kind: DeclarationKind::Def {
                ty: bool_ty,
                value: false_value,
                reducibility: DefinitionReducibility::Reducible,
            },
        });
    }
    seal(c)
}

fn capacity_binder_case(depth: u32) -> Vec<u8> {
    let mut c = base();
    c.module = format!("Probe.CSharp03.W09.Capacity.Binder{depth}");
    let bool_ty = constant(&mut c, "Std.Bool");
    let mut ty = bool_ty;
    let mut value = constant(&mut c, "Std.Bool.false");
    for _ in 0..depth {
        ty = term(
            &mut c,
            TermNode::Pi {
                ty: bool_ty,
                body: ty,
            },
        );
        value = term(
            &mut c,
            TermNode::Lam {
                ty: bool_ty,
                body: value,
            },
        );
    }
    finish(c, ty, value)
}

fn balanced_composition(
    c: &mut Certificate,
    compose: u32,
    left_step: u32,
    right_step: u32,
    count: u32,
) -> u32 {
    if count == 1 {
        return left_step;
    }
    let left_count = count / 2;
    let right_count = count - left_count;
    let left = balanced_composition(c, compose, left_step, right_step, left_count);
    let right = balanced_composition(c, compose, right_step, left_step, right_count);
    app(c, compose, &[left, right])
}

fn capacity_transformer_case(count: u32) -> Vec<u8> {
    assert!(count > 0);
    let mut c = base();
    c.module = format!("Probe.CSharp03.W09.Capacity.Transformer{count}");
    let bool_ty = constant(&mut c, "Std.Bool");
    let false_value = constant(&mut c, "Std.Bool.false");
    let true_value = constant(&mut c, "Std.Bool.true");
    let bool_rec = constant(&mut c, "Std.Bool.rec");
    let state_ty = term(
        &mut c,
        TermNode::Pi {
            ty: bool_ty,
            body: bool_ty,
        },
    );
    let transform_ty = term(
        &mut c,
        TermNode::Pi {
            ty: state_ty,
            body: state_ty,
        },
    );

    let base_coordinate = term(&mut c, TermNode::Var(0));
    let initial = term(
        &mut c,
        TermNode::Lam {
            ty: bool_ty,
            body: base_coordinate,
        },
    );

    let coordinate = term(&mut c, TermNode::Var(0));
    let state = term(&mut c, TermNode::Var(1));
    let current = app(&mut c, state, &[coordinate]);
    let other_coordinate = app(&mut c, bool_rec, &[true_value, false_value, coordinate]);
    let other = app(&mut c, state, &[other_coordinate]);
    let next_leaf = app(&mut c, bool_rec, &[current, other, true_value]);
    let next_state = term(
        &mut c,
        TermNode::Lam {
            ty: bool_ty,
            body: next_leaf,
        },
    );
    let forward = term(
        &mut c,
        TermNode::Lam {
            ty: state_ty,
            body: next_state,
        },
    );
    let reverse_leaf = app(&mut c, bool_rec, &[other, current, false_value]);
    let reverse_state = term(
        &mut c,
        TermNode::Lam {
            ty: bool_ty,
            body: reverse_leaf,
        },
    );
    let reverse = term(
        &mut c,
        TermNode::Lam {
            ty: state_ty,
            body: reverse_state,
        },
    );

    let compose_state = term(&mut c, TermNode::Var(0));
    let compose_right = term(&mut c, TermNode::Var(1));
    let compose_left = term(&mut c, TermNode::Var(2));
    let after_left = app(&mut c, compose_left, &[compose_state]);
    let after_right = app(&mut c, compose_right, &[after_left]);
    let state_lambda = term(
        &mut c,
        TermNode::Lam {
            ty: state_ty,
            body: after_right,
        },
    );
    let right_lambda = term(
        &mut c,
        TermNode::Lam {
            ty: transform_ty,
            body: state_lambda,
        },
    );
    let compose = term(
        &mut c,
        TermNode::Lam {
            ty: transform_ty,
            body: right_lambda,
        },
    );
    let network = balanced_composition(&mut c, compose, forward, reverse, count);
    let value = app(&mut c, network, &[initial]);
    finish(c, state_ty, value)
}

fn capacity_case(family: &str, value: u32) -> Vec<u8> {
    match family {
        "ordinary_term_nodes" => capacity_term_case(value),
        "generated_declarations" => capacity_declaration_case(value),
        "binder_depth" => capacity_binder_case(value),
        "static_transformers" => capacity_transformer_case(value),
        _ => panic!("unknown capacity family"),
    }
}

fn capacity_manifest(directory: &std::path::Path) -> Value {
    let mut rows = Vec::new();
    for (family, limit) in [
        ("binder_depth", BINDER_DEPTH_LIMIT),
        ("generated_declarations", DECLARATION_LIMIT),
        ("ordinary_term_nodes", TERM_LIMIT),
        ("static_transformers", TRANSFORMER_LIMIT),
    ] {
        for (offset, boundary) in [(-1_i32, "below"), (0, "at"), (1, "above")] {
            let value = (limit as i64 + offset as i64) as u32;
            let bytes = capacity_case(family, value);
            let certificate = decode_canonical_certificate(&bytes).unwrap();
            let id = format!("{family}.{boundary}");
            std::fs::write(directory.join(format!("{id}.mpcert")), &bytes).unwrap();
            rows.push(json!({
                "id": id,
                "family": family,
                "boundary": boundary,
                "counter_value": value,
                "inclusive_limit": limit,
                "profile_expected": if offset <= 0 { "accepted" } else { "limit_exceeded" },
                "checker_expected": "accepted",
                "raw_sha256": super::sha(&bytes),
                "size_bytes": bytes.len(),
                "certificate_declarations": certificate.declarations.len(),
                "certificate_terms": certificate.term_table.len(),
                "proof_nodes": certificate.proof_node_table.len(),
                "theory_certificates": certificate.theory_certificates.len(),
                "axioms": certificate.axiom_report.summary.total_axiom_count
            }));
        }
    }
    rows.sort_by(|left, right| left["id"].as_str().cmp(&right["id"].as_str()));
    json!({
        "schema": "mpk.csharp_practical.checker_capacity_probe.v1",
        "limits": {
            "binder_depth": BINDER_DEPTH_LIMIT,
            "generated_declarations": DECLARATION_LIMIT,
            "ordinary_term_nodes": TERM_LIMIT,
            "static_transformers": TRANSFORMER_LIMIT
        },
        "cases": rows
    })
}

#[test]
fn csharp_03_t01_w09_recursor_probe_bytes_are_reproducible() {
    let value = json!({
        "schema": "mpk.csharp_practical.recursor_probe.v1",
        "inputs": INPUTS.iter().map(|path| json!({"path": path, "raw_sha256": super::sha(&super::read(path))})).collect::<Vec<_>>(),
        "recursor_types": recursor_types(),
        "replacement": {
            "carrier": "C(0)=Bool; C(d+1)=Pi(Bool,C(d))",
            "conditionals": "pointwise Bool.rec at C(0)",
            "folds": "static balanced composition of concrete state transformers using Lam/App/Let",
            "nat_recursor_in_replacement": false
        },
        "cases": cases()
    });
    if let Some(path) = std::env::var_os("MPK_W09_RECURSOR_EXPORT") {
        let mut bytes = serde_json::to_vec(&value).unwrap();
        bytes.push(b'\n');
        std::fs::write(path, bytes).unwrap();
        return;
    }
    let record = super::document(RECORD);
    assert_eq!(
        record["schema"],
        "mpk.csharp_practical.recursor_feasibility.v1"
    );
    assert_eq!(record["work_item"], "CSHARP-03-T01-W09");
    assert_eq!(record["probe"], value);
    assert_eq!(record["status"], "replacement_type_feasible");
    assert_eq!(record["runs"].as_array().unwrap().len(), 2);
    assert_eq!(
        record["runs"][0]["observations"],
        record["runs"][1]["observations"]
    );
    assert_eq!(
        record["capacity_measurement"]["path"],
        "develop/migrations/csharp-03/probes/checker-capacity.json"
    );
    assert_eq!(
        record["capacity_measurement"]["raw_sha256"],
        super::sha(&super::read(
            "develop/migrations/csharp-03/probes/checker-capacity.json"
        ))
    );
    assert_eq!(record["release_gate"], false);
    assert_eq!(
        record["claim"],
        "The retained cross-result applications still reject, while the Bool-addressed pointwise carrier and static cross-coordinate concrete-transformer fold typecheck unchanged in both checkers without using Nat.rec in the replacement."
    );
    let mut source_bytes = serde_json::to_vec(&record["source_inventory"]).unwrap();
    source_bytes.push(b'\n');
    assert_eq!(record["source_inventory_sha256"], super::sha(&source_bytes));
    for input in record["source_inventory"].as_array().unwrap() {
        assert_eq!(
            input["raw_sha256"],
            super::sha(&super::read(input["path"].as_str().unwrap()))
        );
    }
    for run in record["runs"].as_array().unwrap() {
        assert_eq!(run["observations"].as_array().unwrap().len(), 15);
        for (case, observed) in value["cases"]
            .as_array()
            .unwrap()
            .iter()
            .zip(run["observations"].as_array().unwrap())
        {
            assert_eq!(observed["id"], case["id"]);
            assert_eq!(observed["rust"]["result"], case["expected"]);
            assert_eq!(observed["reference"]["result"], case["expected"]);
        }
    }
    assert_eq!(
        value["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|case| case["expected"] == "accepted")
            .count(),
        10
    );
    assert_eq!(
        value["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|case| case["expected"] == "type_mismatch")
            .count(),
        5
    );
    for case in value["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        if id.starts_with("pointwise_bool_tree.")
            || id.starts_with("binary_bool_cube.")
            || id.starts_with("static_tree_fold.")
        {
            assert_eq!(case["nat_recursor_applications"], 0, "{id}");
        }
    }
}

#[test]
fn csharp_03_t01_w09_checker_capacity_bytes_are_reproducible() {
    if let Some(path) = std::env::var_os("MPK_W09_CAPACITY_EXPORT") {
        let directory = std::path::PathBuf::from(path);
        std::fs::create_dir_all(&directory).unwrap();
        let manifest = capacity_manifest(&directory);
        let mut bytes = serde_json::to_vec(&manifest).unwrap();
        bytes.push(b'\n');
        std::fs::write(directory.join("manifest.json"), bytes).unwrap();
        return;
    }
    let record = super::document(CAPACITY_RECORD);
    assert_eq!(
        record["schema"],
        "mpk.csharp_practical.checker_capacity_evidence.v1"
    );
    assert_eq!(record["work_item"], "CSHARP-03-T01-W09");
    assert_eq!(record["release_gate"], false);
    assert_eq!(record["activation"], "candidate_only");
    assert_eq!(
        record["probe"]["limits"]["binder_depth"],
        BINDER_DEPTH_LIMIT
    );
    assert_eq!(
        record["probe"]["limits"]["generated_declarations"],
        DECLARATION_LIMIT
    );
    assert_eq!(record["probe"]["limits"]["ordinary_term_nodes"], TERM_LIMIT);
    assert_eq!(
        record["probe"]["limits"]["static_transformers"],
        TRANSFORMER_LIMIT
    );
    assert_eq!(record["runs"].as_array().unwrap().len(), 2);
    for run in record["runs"].as_array().unwrap() {
        assert_eq!(run["observations"].as_array().unwrap().len(), 12);
        for observation in run["observations"].as_array().unwrap() {
            assert_eq!(observation["rust"]["result"], "accepted");
            assert_eq!(observation["reference"]["result"], "accepted");
            assert!(observation["rust"]["elapsed_ms"].as_u64().unwrap() < 60_000);
            assert!(observation["reference"]["elapsed_ms"].as_u64().unwrap() < 60_000);
        }
    }
    for input in record["source_inventory"].as_array().unwrap() {
        assert_eq!(
            input["raw_sha256"],
            super::sha(&super::read(input["path"].as_str().unwrap()))
        );
    }
    let mut source_bytes = serde_json::to_vec(&record["source_inventory"]).unwrap();
    source_bytes.push(b'\n');
    assert_eq!(record["source_inventory_sha256"], super::sha(&source_bytes));
}
