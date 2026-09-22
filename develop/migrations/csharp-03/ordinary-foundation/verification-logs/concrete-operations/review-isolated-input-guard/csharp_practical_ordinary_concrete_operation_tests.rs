//! Preserve original closed recipes and sequents, then exercise outcome routing
//! independently of the unchanged component algorithms.
use super::*;
use core_eval::V;
use mpk_cert::encode::{Certificate, DeclarationKind, TermNode};

fn global(c: &Certificate, name: &str) -> u32 {
    c.declarations
        .iter()
        .position(|d| c.name_table[d.name as usize] == name)
        .unwrap() as u32
}
fn body(c: &Certificate, name: &str) -> u32 {
    match c.declarations[global(c, name) as usize].kind {
        DeclarationKind::Def { value, .. } => value,
        _ => panic!("expected definition: {name}"),
    }
}
fn cube_depth(c: &Certificate, mut t: u32) -> u32 {
    let boolean = global(c, "Std.Bool");
    let mut depth = 0;
    while let TermNode::Pi { ty, body } = c.term_table[t as usize] {
        assert!(
            matches!(c.term_table[ty as usize], TermNode::Const { global, .. } if global == boolean)
        );
        depth += 1;
        t = body;
    }
    assert!(
        matches!(c.term_table[t as usize], TermNode::Const { global, .. } if global == boolean)
    );
    depth
}
fn operands(c: &Certificate, name: &str, depths: &[u32]) -> u32 {
    let mut t = body(c, name);
    for depth in depths {
        let TermNode::Lam { ty, body } = c.term_table[t as usize] else {
            panic!("missing operand: {name}")
        };
        assert_eq!(cube_depth(c, ty), *depth, "{name}");
        t = body;
    }
    t
}
// Normalize only application grouping. No beta reduction or truth replacement
// may conceal a lost operand, guard, comparison, or free-variable index.
fn syntax(c: &Certificate, t: u32) -> Value {
    match &c.term_table[t as usize] {
        TermNode::Const { global, levels } => {
            assert!(levels.is_empty());
            json!([
                "const",
                c.name_table[c.declarations[*global as usize].name as usize]
            ])
        }
        TermNode::Var(i) => json!(["var", i]),
        TermNode::App {
            function,
            arguments,
        } => arguments.iter().fold(syntax(c, *function), |f, a| {
            json!(["app", f, syntax(c, *a)])
        }),
        other => panic!("unexpected lowered formula: {other:?}"),
    }
}
fn formula(
    t: &ContractTerm,
    subjects: &[TypedValueRef],
    symbols: &BTreeMap<String, String>,
) -> Value {
    match t {
        ContractTerm::Var { index, type_id } => {
            assert_eq!(&subjects[*index].type_id, type_id);
            json!(["var", subjects.len() - 1 - index])
        }
        ContractTerm::Const { name, .. } => json!(["const", symbols[name]]),
        ContractTerm::App {
            function, argument, ..
        } => json!([
            "app",
            formula(function, subjects, symbols),
            formula(argument, subjects, symbols)
        ]),
        other => panic!("unexpected original formula: {other:?}"),
    }
}
fn wrapper(c: &Certificate, name: &str, implementation: &str, depths: &[u32], selected: &[usize]) {
    let expected = selected
        .iter()
        .fold(json!(["const", implementation]), |f, i| {
            json!(["app", f, ["var", depths.len() - 1 - i]])
        });
    assert_eq!(syntax(c, operands(c, name, depths)), expected, "{name}");
}
fn roots(v: &Value, out: &mut BTreeSet<String>) {
    match v {
        Value::Object(fields) => {
            for (k, v) in fields {
                if k == "definition" || k.ends_with("_definition") {
                    if let Some(s) = v.as_str() {
                        out.insert(s.into());
                    }
                } else {
                    roots(v, out);
                }
            }
        }
        Value::Array(vs) => {
            for v in vs {
                roots(v, out);
            }
        }
        _ => {}
    }
}

fn component_link(base: &Value, op: &OrdinaryConcreteOperationComponent) {
    for group in ["constructions", "outcomes", "collections", "money"] {
        for d in base[group].as_array().unwrap() {
            if let Some(original) = d["operations"]
                .as_array()
                .unwrap()
                .iter()
                .find(|o| o["operation_id"] == op.operation_id)
            {
                assert_eq!(original["normal_definition"], op.normal_definition);
                assert_eq!(original["argument_type_ids"], json!(op.argument_type_ids));
                assert_eq!(original["result_type_id"], op.result_type_id);
                let failures = original["failures"].as_array().unwrap();
                assert_eq!(failures.len(), op.failures.len());
                for (a, b) in failures.iter().zip(&op.failures) {
                    assert_eq!(a["label"], b.label);
                    assert_eq!(a["definition"], json!(b.definition));
                    let arguments = if group == "money" {
                        json!((0..op.argument_type_ids.len()).collect::<Vec<_>>())
                    } else {
                        a["argument_indices"].clone()
                    };
                    assert_eq!(arguments, json!(b.argument_indices));
                }
                assert_eq!(
                    original["currency_predicate_argument_type_id"],
                    json!(op.currency_predicate_argument_type_id)
                );
                return;
            }
        }
    }
    let (id, suffix) = op.operation_id.rsplit_once('.').unwrap();
    for group in ["sequences", "entries", "transitions"] {
        let Some(rows) = base[group].as_array() else {
            continue;
        };
        for d in rows.iter().filter(|d| d["carrier"]["type_id"] == id) {
            let field = match suffix {
                "equal" => "equality_definition".into(),
                other => format!("{other}_definition"),
            };
            assert_eq!(d[&field], op.normal_definition);
            match (group, suffix) {
                ("sequences", "read") => {
                    assert_eq!(op.failures.len(), 1);
                    assert_eq!(op.failures[0].label, "index_range");
                    assert_eq!(
                        json!(op.failures[0].definition),
                        d["index_range_definition"]
                    );
                    assert_eq!(op.failures[0].argument_indices, [0, 1]);
                }
                ("transitions", "make") => {
                    assert_eq!(op.failures.len(), 1);
                    assert_eq!(op.failures[0].label, "event_bound");
                    assert_eq!(
                        json!(op.failures[0].definition),
                        d["event_bound_definition"]
                    );
                    assert_eq!(
                        json!(op.failures[0].argument_indices),
                        json!([d["event_bound_argument_index"]])
                    );
                }
                _ => assert!(op.failures.is_empty()),
            }
            assert!(op.currency_predicate_argument_type_id.is_none());
            return;
        }
    }
    panic!("unmatched component: {}", op.operation_id);
}

#[test]
fn csharp_03_t06_w09_concrete_operations_original_source_certificates() {
    let bundle = b();
    let output = std::env::var_os("MPK_W09_CONCRETE_OPERATIONS_OUT").map(std::path::PathBuf::from);
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/concrete-operations");
    if let Some(dir) = &output {
        fs::create_dir_all(dir).unwrap();
    }
    let mut rows = vec![];
    let (mut defined_count, mut pending_count) = (0, 0);
    let mut arities = BTreeSet::new();
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    for (id, row, facts) in sources() {
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let vir = emitted.vir();
        let p = generate_csharp_practical_ordinary_concrete_operations(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let full = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        let vc = full.binding_vcs();
        let operation_table: Value =
            serde_json::from_slice(emitted.operations().operations().canonical_bytes()).unwrap();
        let check_table: Value =
            serde_json::from_slice(emitted.operations().required_checks().canonical_bytes())
                .unwrap();
        assert_eq!(p.instances(), vc.instances());
        let internal = emitted
            .closure()
            .closed()
            .entries()
            .iter()
            .filter(|e| e["template_id"] == "mpk.csharp.semantic.sequence_construction.v1")
            .map(|e| e["instance_id"].as_str().unwrap())
            .collect::<BTreeSet<_>>();
        let mut pending_ids = BTreeSet::new();
        for instance in vc.instances() {
            for recipe in &instance.operation_definitions {
                let op_id = recipe["id"].as_str().unwrap();
                let mut reasons = vec![];
                if recipe["argument_type_ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .chain(std::iter::once(&recipe["normal_result_type_id"]))
                    .any(|v| internal.contains(v.as_str().unwrap()))
                {
                    reasons.push(OrdinaryConcreteOperationPendingReason::InternalConstructionState);
                }
                if recipe["error_precedence"]
                    .as_array()
                    .unwrap()
                    .contains(&json!("ownership"))
                {
                    reasons.push(OrdinaryConcreteOperationPendingReason::SourceOwnership);
                }
                // This prerequisite comes from the closed Money.create contract,
                // independently of the adapter's component metadata.
                let money = emitted.closure().closed().entries().iter().any(|e| {
                    e["instance_id"] == instance.instance_id
                        && e["template_id"] == "mpk.csharp.semantic.money.v1"
                });
                if money && op_id.ends_with(".create") {
                    reasons
                        .push(OrdinaryConcreteOperationPendingReason::ApplicationCurrencyPredicate);
                }
                if reasons.is_empty() {
                    let d = p
                        .definitions()
                        .iter()
                        .find(|d| d.component.operation_id == op_id)
                        .unwrap();
                    assert_eq!(d.instance_id, instance.instance_id);
                    assert_eq!(&d.recipe, recipe);
                    let original = operation_table["operations"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|s| s["id"] == op_id);
                    assert_eq!(d.signature.is_some(), original.is_some());
                    if let (Some(signature), Some(original)) = (&d.signature, original) {
                        assert_eq!(signature.id, op_id);
                        assert_eq!(json!(signature.tag), original["tag"]);
                        assert_eq!(
                            json!(signature.argument_type_ids),
                            original["argument_type_ids"]
                        );
                        assert_eq!(
                            signature.normal_result_type_id,
                            original["normal_result_type_id"]
                        );
                        assert_eq!(
                            json!(signature
                                .ordered_checks
                                .iter()
                                .map(|c| &c.id)
                                .collect::<Vec<_>>()),
                            original["ordered_check_ids"]
                        );
                        for check in &signature.ordered_checks {
                            let original = check_table["checks"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .find(|c| c["id"] == check.id)
                                .unwrap();
                            assert_eq!(json!(check.tag), original["tag"]);
                            assert_eq!(json!(check.failure_type_id), original["failure_type_id"]);
                        }
                    }
                } else {
                    let d = p
                        .pending_operations()
                        .iter()
                        .find(|d| d.component.operation_id == op_id)
                        .unwrap();
                    assert_eq!(d.instance_id, instance.instance_id);
                    assert_eq!(&d.recipe, recipe);
                    assert_eq!(d.reasons, reasons);
                    pending_ids.insert(op_id);
                }
            }
        }
        assert_eq!(
            p.definitions().len() + p.pending_operations().len(),
            vc.instances()
                .iter()
                .map(|i| i.operation_definitions.len())
                .sum::<usize>()
        );
        assert_eq!(
            p.conditions()
                .iter()
                .map(|c| &c.sequent)
                .collect::<Vec<_>>(),
            vc.sequents()
                .iter()
                .filter(|s| s.kind == "concrete_definition_equivalence"
                    && !pending_ids.contains(s.owner_id.as_str()))
                .collect::<Vec<_>>()
        );
        assert_eq!(p.conditions().len(), p.definitions().len());
        assert_eq!(
            p.pending_condition_ids(),
            vc.sequents()
                .iter()
                .filter(|s| s.kind != "concrete_definition_equivalence"
                    || pending_ids.contains(s.owner_id.as_str()))
                .map(|s| s.id.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            p.pending_proof_ids(),
            vc.sequents()
                .iter()
                .map(|s| s.id.clone())
                .collect::<Vec<_>>()
        );
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let independent = generate_csharp_practical_ordinary_structural_public(vir).unwrap();
        let base: Value = serde_json::from_slice(&independent.canonical_bytes()).unwrap();
        assert_eq!(Some(p.public_domains()), independent.public_domains());
        assert_eq!(Some(p.source_clauses()), independent.source_clauses());
        assert_eq!(p.source_observations(), independent.source_observations());
        let before =
            mpk_cert::decode_canonical_certificate(independent.certificate_bytes()).unwrap();
        let mut names = BTreeSet::new();
        roots(&base, &mut names);
        structural_equivalence_tests::same_definition_closure(&before, &cert, &names).unwrap();
        let mut symbols = p
            .public_domains()
            .iter()
            .map(|d| (d.symbol.clone(), d.valid_definition.clone()))
            .collect::<BTreeMap<_, _>>();
        symbols.extend(p.source_observations().iter().map(|d| {
            (
                format!("Mpk.CSharp.Binding.Equal.{}", d.carrier.type_id),
                d.equality_definition.clone(),
            )
        }));
        for (a, b) in [
            ("Not", "not"),
            ("And", "and"),
            ("Or", "or"),
            ("true", "true"),
            ("false", "false"),
        ] {
            symbols.insert(format!("Mpk.CSharp.Bool.{a}"), format!("Std.Bool.{b}"));
        }
        let depths = independent
            .carriers()
            .iter()
            .map(|c| (c.type_id.as_str(), c.depth))
            .collect::<BTreeMap<_, _>>();
        for d in p.definitions() {
            component_link(&base, &d.component);
            symbols.insert(
                d.component.operation_id.clone(),
                d.normal_definition.clone(),
            );
            symbols.insert(d.concrete_symbol.clone(), d.concrete_definition.clone());
            symbols.insert(
                d.all_outcomes_symbol.clone(),
                d.all_outcomes_definition.clone(),
            );
            let args = d
                .component
                .argument_type_ids
                .iter()
                .map(|id| depths[id.as_str()])
                .collect::<Vec<_>>();
            arities.insert(args.len());
            assert_ne!(d.normal_definition, d.concrete_definition);
            for name in [&d.normal_definition, &d.concrete_definition] {
                wrapper(
                    &cert,
                    name,
                    &d.component.normal_definition,
                    &args,
                    &(0..args.len()).collect::<Vec<_>>(),
                );
            }
            assert_eq!(d.failures.len(), d.component.failures.len());
            for (f, link) in d.failures.iter().zip(&d.component.failures) {
                assert_eq!(f.label, link.label);
                assert_ne!(f.failure_definition, f.concrete_failure_definition);
                symbols.insert(f.check_symbol.clone(), f.failure_definition.clone());
                for name in [&f.failure_definition, &f.concrete_failure_definition] {
                    wrapper(
                        &cert,
                        name,
                        link.definition.as_ref().unwrap(),
                        &args,
                        &link.argument_indices,
                    );
                }
            }
        }
        for d in p.pending_operations() {
            component_link(&base, &d.component);
        }
        for c in p.conditions() {
            let args = c
                .sequent
                .subjects
                .iter()
                .map(|s| depths[s.type_id.as_str()])
                .collect::<Vec<_>>();
            for (original, names) in [
                (&c.sequent.assumptions, &c.assumption_definitions),
                (&c.sequent.goals, &c.goal_definitions),
            ] {
                assert_eq!(original.len(), names.len());
                for (t, name) in original.iter().zip(names) {
                    assert_eq!(
                        syntax(&cert, operands(&cert, name, &args)),
                        formula(t, &c.sequent.subjects, &symbols),
                        "{id}: {name}"
                    );
                }
            }
        }
        assert_eq!(
            import_csharp_practical_ordinary_concrete_operations(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        if let Some((metadata, bytes)) = &previous {
            assert!(
                import_csharp_practical_ordinary_concrete_operations(metadata, bytes, vir).is_err()
            );
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        if id == "binding-vc-option" {
            let metadata: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
            for field in metadata.as_object().unwrap().keys() {
                let mut changed = metadata.clone();
                changed[field] = json!("forged");
                assert!(
                    import_csharp_practical_ordinary_concrete_operations(
                        &serde_json::to_vec(&changed).unwrap(),
                        p.certificate_bytes(),
                        vir
                    )
                    .is_err(),
                    "{field}"
                );
            }
            let mut corrupted = p.certificate_bytes().to_vec();
            *corrupted.last_mut().unwrap() ^= 1;
            assert!(import_csharp_practical_ordinary_concrete_operations(
                &p.canonical_bytes(),
                &corrupted,
                vir
            )
            .is_err());
            let oversized = vec![0; 16 * 1024 * 1024 + 1];
            assert!(
                import_csharp_practical_ordinary_concrete_operations(&oversized, &[], vir).is_err()
            );
            assert!(
                import_csharp_practical_ordinary_concrete_operations(&[], &oversized, vir).is_err()
            );
        }
        defined_count += p.definitions().len();
        pending_count += p.pending_operations().len();
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|v| format!("{v:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(dir) = &output {
            fs::write(dir.join(format!("{id}.hex")), hex).unwrap();
        } else {
            assert_eq!(
                fs::read_to_string(fixture.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        rows.push(json!({"id":id,"metadata":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),"terms":cert.term_table.len(),"declarations":cert.declarations.len()}));
        eprintln!("concrete operation definitions complete {id}");
    }
    assert_eq!(rows.len(), 45);
    assert_eq!((defined_count, pending_count), (436, 31));
    assert_eq!(arities, BTreeSet::from([0, 1, 2, 3, 4]));
    let data = json!({"sources":rows,"conditions":defined_count,"pending_operation_conditions":pending_count});
    if let Some(dir) = &output {
        fs::write(
            dir.join("certificates.json"),
            serde_json::to_vec_pretty(&data).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/concrete-operations/certificates.json"),
            data
        );
    }
}

// These mutations exist only in evaluator test copies. The certificates above
// retain the exact algorithms and must pass strict import and both checkers.
fn constant_result(c: &mut Certificate, name: &str, arity: usize, truth: bool) {
    let index = global(c, name) as usize;
    let mut value = body(c, name);
    let mut binders = vec![];
    for _ in 0..arity {
        let TermNode::Lam { ty, body } = c.term_table[value as usize] else {
            panic!()
        };
        binders.push(ty);
        value = body;
    }
    let DeclarationKind::Def { mut ty, .. } = c.declarations[index].kind else {
        panic!()
    };
    for _ in 0..arity {
        let TermNode::Pi { body, .. } = c.term_table[ty as usize] else {
            panic!()
        };
        ty = body;
    }
    while let TermNode::Pi { ty: argument, body } = c.term_table[ty as usize] {
        binders.push(argument);
        ty = body;
    }
    assert_eq!(cube_depth(c, ty), 0);
    let mut value = c.term_table.len() as u32;
    c.term_table.push(TermNode::Const {
        global: global(
            c,
            if truth {
                "Std.Bool.true"
            } else {
                "Std.Bool.false"
            },
        ),
        levels: vec![],
    });
    for ty in binders.into_iter().rev() {
        let body = value;
        value = c.term_table.len() as u32;
        c.term_table.push(TermNode::Lam { ty, body });
    }
    let DeclarationKind::Def { value: old, .. } = &mut c.declarations[index].kind else {
        panic!()
    };
    *old = value;
}
fn zeros(depth: u32) -> V {
    if depth == 0 {
        V::Bit(false)
    } else {
        V::UniformCube(false, depth)
    }
}

#[test]
fn csharp_03_t06_w09_concrete_operations_ordered_outcomes() {
    let bundle = b();
    let (mut operations, mut assignments, mut flips, mut masked) = (0, 0, 0, 0);
    for (id, row, facts) in sources() {
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_concrete_operations(emitted.vir()).unwrap();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        let depths = p
            .public_domains()
            .iter()
            .map(|d| (d.carrier.type_id.as_str(), d.carrier.depth))
            .collect::<BTreeMap<_, _>>();
        for op in p.definitions().iter().filter(|d| !d.failures.is_empty()) {
            operations += 1;
            let n = op.component.argument_type_ids.len();
            let args = op
                .component
                .argument_type_ids
                .iter()
                .map(|id| zeros(depths[id.as_str()]))
                .collect::<Vec<_>>();
            let mut seed = cert.clone();
            for name in [&op.normal_definition, &op.concrete_definition] {
                constant_result(&mut seed, name, n, false);
            }
            for mask in 0..(1usize << op.failures.len()) {
                let first = (0..op.failures.len()).find(|i| mask & (1 << i) != 0);
                let mut same = seed.clone();
                for (i, f) in op.failures.iter().enumerate() {
                    for name in [&f.failure_definition, &f.concrete_failure_definition] {
                        constant_result(&mut same, name, n, mask & (1 << i) != 0);
                    }
                }
                for (i, f) in op.failures.iter().enumerate() {
                    for name in [
                        &f.first_failure_definition,
                        &f.concrete_first_failure_definition,
                    ] {
                        assert_eq!(
                            bit(run(&same, name, args.clone())),
                            first == Some(i),
                            "{id}: {mask}"
                        );
                    }
                }
                for name in [&op.success_definition, &op.concrete_success_definition] {
                    assert_eq!(bit(run(&same, name, args.clone())), first.is_none());
                }
                assert!(
                    bit(run(&same, &op.all_outcomes_definition, args.clone())),
                    "{id}: {mask}"
                );
                assignments += 1;
                for (i, f) in op.failures.iter().enumerate() {
                    let mut changed = same.clone();
                    constant_result(&mut changed, &f.failure_definition, n, mask & (1 << i) == 0);
                    let next_mask = mask ^ (1 << i);
                    let next = (0..op.failures.len()).find(|j| next_mask & (1 << j) != 0);
                    assert_eq!(
                        bit(run(&changed, &op.all_outcomes_definition, args.clone())),
                        first == next,
                        "{id}: {mask}, flip {i}"
                    );
                    flips += 1;
                    masked += usize::from(first == next);
                }
            }
        }
        eprintln!("concrete operation routing complete {id}");
    }
    assert!(operations > 0 && assignments > operations && flips > assignments && masked > 0);
    eprintln!("ordered outcomes: {operations} operations, {assignments} assignments, {flips} single-failure changes, {masked} correctly masked later changes");
}

#[test]
fn csharp_03_t06_w09_concrete_operations_real_values_and_guards() {
    let bundle = b();
    let (mut closed, mut nan, mut guarded, mut normal) = (0, 0, 0, 0);
    for (id, row, facts) in sources().into_iter().filter(|(id, _, _)| {
        [
            "binding-vc-option",
            "float-make-commutation",
            "binding-vc-transition",
            "binding-vc-money",
        ]
        .contains(&id.as_str())
    }) {
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_concrete_operations(emitted.vir()).unwrap();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        let carriers = p
            .public_domains()
            .iter()
            .map(|d| (d.carrier.type_id.clone(), d.carrier.clone()))
            .collect::<BTreeMap<_, _>>();
        for op in p.definitions() {
            let arity = op.component.argument_type_ids.len();
            let is_nan =
                id == "float-make-commutation" && op.component.operation_id.ends_with(".make");
            let selected = arity == 0
                || is_nan
                || (id == "binding-vc-option" && op.component.operation_id.ends_with(".value"))
                || (id == "binding-vc-transition" && op.component.operation_id.ends_with(".make"))
                || (id == "binding-vc-money" && arity == 4);
            if !selected {
                continue;
            }
            let c = p
                .conditions()
                .iter()
                .find(|c| c.sequent.owner_id == op.component.operation_id)
                .unwrap();
            let mut args = op
                .component
                .argument_type_ids
                .iter()
                .map(|id| {
                    let value = relation_tests::sample(
                        id,
                        1,
                        &carriers,
                        &facts,
                        emitted.closure().closed(),
                    );
                    let (depth, ones) = projection_tests::sparse_storage(&value, &carriers);
                    sparse_cube(depth, ones)
                })
                .collect::<Vec<_>>();
            if is_nan {
                assert_eq!(op.component.argument_type_ids, [ty("bool"), ty("f32")]);
                args = vec![
                    V::Bit(false),
                    sparse_cube(
                        5,
                        (0..32).filter(|i| 0x7fc0_1234u32 & (1 << i) != 0).collect(),
                    ),
                ];
                let result = run(&cert, &op.normal_definition, args.clone());
                let eq = p
                    .definitions()
                    .iter()
                    .find(|d| {
                        d.instance_id == op.instance_id
                            && d.component.operation_id.ends_with(".equal")
                    })
                    .unwrap();
                assert!(!bit(run(
                    &cert,
                    &eq.normal_definition,
                    vec![result.clone(), result]
                )));
                nan += 1;
            }
            assert!(
                bit(run(&cert, &op.normal_agreement_definition, args.clone())),
                "{id}"
            );
            assert!(
                bit(run(&cert, &op.all_outcomes_definition, args.clone())),
                "{id}"
            );
            assert!(
                bit(run(&cert, &c.condition_definition, args.clone())),
                "{id}"
            );
            let mut changed = cert.clone();
            constant_result(&mut changed, &op.concrete_definition, arity, true);
            assert!(
                !bit(run(&changed, &op.normal_agreement_definition, args.clone())),
                "{id}: concrete result ignored"
            );
            let assumptions = c
                .assumption_definitions
                .iter()
                .all(|s| bit(run(&cert, s, args.clone())));
            let goals = c
                .goal_definitions
                .iter()
                .all(|s| bit(run(&changed, s, args.clone())));
            assert_eq!(
                bit(run(&changed, &c.condition_definition, args.clone())),
                !assumptions || goals
            );
            if assumptions {
                assert!(!goals, "{id}: changed result did not falsify normal goal");
            }
            if !c.assumption_definitions.is_empty() {
                // Force the original source-domain guard false, while retaining
                // the actual changed normal result and original full sequent.
                let domain = p
                    .public_domains()
                    .iter()
                    .find(|d| d.carrier.type_id == c.sequent.subjects[0].type_id)
                    .unwrap();
                constant_result(&mut changed, &domain.valid_definition, 1, false);
                assert!(!c.assumption_definitions.iter().all(|s| bit(run(
                    &changed,
                    s,
                    args.clone()
                ))));
                assert!(bit(run(&changed, &c.condition_definition, args.clone())));
                guarded += 1;
            }
            normal += 1;
            closed += usize::from(arity == 0);
        }
        eprintln!("concrete operation real values complete {id}");
    }
    assert!(closed > 0 && nan == 1 && guarded > 0 && normal > 3);
    eprintln!("real operation values: {normal} normal observations, {closed} closed operations, {nan} NaN observation, {guarded} rejected-input guards");
}
