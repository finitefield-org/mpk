//! W04 identities, binding projections, original decrease terms and hostile import.
use super::*;
use mpk_vc::csharp_practical_vir_validation::ValidatedPracticalVir;

type PredicateResult = Result<OrdinaryControlPredicateProgram, OrdinaryCarrierError>;
type Generate = fn(&ValidatedPracticalVir) -> PredicateResult;
type Import = fn(&[u8], &[u8], &ValidatedPracticalVir) -> PredicateResult;

fn output(id: &str, suffix: &str, bytes: &[u8], integrated: bool) {
    let env = if integrated {
        "MPK_W09_CONTROL_PREDICATE_INTEGRATED_OUTPUT"
    } else {
        "MPK_W09_CONTROL_PREDICATE_OUTPUT"
    };
    if let Some(dir) = std::env::var_os(env) {
        let dir = PathBuf::from(dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(format!("{id}.{suffix}")), bytes).unwrap();
    } else {
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/control-predicates");
        if integrated {
            dir.push("with-execution");
        }
        assert_eq!(
            fs::read(dir.join(format!("{id}.{suffix}"))).unwrap(),
            bytes,
            "{id}"
        );
    }
}
fn names(term: &ContractTerm, set: &mut BTreeSet<String>) {
    match term {
        ContractTerm::Const { name, .. } => {
            set.insert(name.clone());
        }
        ContractTerm::App {
            function, argument, ..
        } => {
            names(function, set);
            names(argument, set);
        }
        ContractTerm::Lam { body, .. } => names(body, set),
        ContractTerm::Let { value, body, .. } => {
            names(value, set);
            names(body, set);
        }
        ContractTerm::Var { .. } => {}
    }
}
fn integer(term: &ContractTerm, values: &[i32]) -> Option<i32> {
    match term {
        ContractTerm::Var { index, type_id } if type_id == "mpk.csharp.value.i32.v1" => {
            Some(values[*index])
        }
        _ => None,
    }
}
// Evaluate the original mathematical formula directly, without its ordinary
// names, Boolean-cube circuits or argument-index map.
fn mathematical(term: &ContractTerm, values: &[i32]) -> Option<bool> {
    let mut args = vec![];
    let mut f = term;
    while let ContractTerm::App {
        function, argument, ..
    } = f
    {
        args.push(argument.as_ref());
        f = function;
    }
    args.reverse();
    let ContractTerm::Const { name, .. } = f else {
        return None;
    };
    Some(match name.as_str() {
        "Mpk.CSharp.Bool.true" => true,
        "Mpk.CSharp.Bool.false" => false,
        "Mpk.CSharp.Bool.And" => mathematical(args[0], values)? && mathematical(args[1], values)?,
        "Mpk.CSharp.Bool.Or" => mathematical(args[0], values)? || mathematical(args[1], values)?,
        n if n.starts_with("Mpk.CSharp.Integer.MathLess.") => {
            integer(args[0], values)? < integer(args[1], values)?
        }
        n if n.starts_with("Mpk.CSharp.Integer.MathEqual.") => {
            integer(args[0], values)? == integer(args[1], values)?
        }
        n if n.starts_with("Mpk.CSharp.Integer.NonNegative.") => integer(args[0], values)? >= 0,
        _ => return None,
    })
}

#[test]
fn csharp_03_t06_w09_control_predicates_original_sources() {
    verify(false);
}
#[test]
fn csharp_03_t06_w09_control_predicates_with_execution_original_sources() {
    verify(true);
}
fn verify(integrated: bool) {
    let generate: Generate = if integrated {
        generate_csharp_practical_ordinary_control_predicates_with_execution
    } else {
        generate_csharp_practical_ordinary_control_predicates
    };
    let import: Import = if integrated {
        import_csharp_practical_ordinary_control_predicates_with_execution
    } else {
        import_csharp_practical_ordinary_control_predicates
    };
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
    let mut total_sequents = 0;
    let mut compiled = 0;
    let mut pending = 0;
    let mut measure_observations = 0;
    let mut implication_observations = 0;
    let mut distinct_snapshots = 0;
    let mut native_guards = 0;
    let mut ownership_guards = 0;
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    for id in [
        "count_fill",
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
        let p = generate(vir).unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let metadata: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        assert_eq!(metadata["application_scope_pending"], true);
        assert_eq!(metadata["control_vc_sha256"], vc.control_vcs().hash());
        assert_eq!(
            p.unresolved_regions(),
            vc.control_vcs().unresolved_regions()
        );
        assert_eq!(p.sequents().len(), vc.control_vcs().sequents().len());
        assert_eq!(
            import(&p.canonical_bytes(), p.certificate_bytes(), vir,).unwrap(),
            p
        );
        let mut altered = metadata.clone();
        altered["application_scope_pending"] = json!(false);
        assert!(import(
            &serde_json::to_vec(&altered).unwrap(),
            p.certificate_bytes(),
            vir,
        )
        .is_err());
        let mut corrupt = p.certificate_bytes().to_vec();
        *corrupt.last_mut().unwrap() ^= 1;
        assert!(import(&p.canonical_bytes(), &corrupt, vir,).is_err());
        if let Some((old_metadata, old_certificate)) = &previous {
            assert!(import(old_metadata, old_certificate, vir,).is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        let native = if integrated {
            let native = generate_csharp_practical_ordinary_control_edges(vir).unwrap();
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../develop/migrations/csharp-03/ordinary-foundation/control-edges");
            assert_eq!(
                fs::read(root.join(format!("{id}.json"))).unwrap(),
                native.canonical_bytes()
            );
            let hex = native
                .certificate_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
                + "\n";
            assert_eq!(
                fs::read(root.join(format!("{id}.hex"))).unwrap(),
                hex.as_bytes()
            );
            let prior = mpk_cert::decode_canonical_certificate(native.certificate_bytes()).unwrap();
            // Canonical encoding sorts names, so added predicates can renumber
            // the old table. Resolve globals and terms before comparing bodies.
            let current = declaration_bodies(&cert);
            for (name, body) in declaration_bodies(&prior) {
                assert!(
                    current.get(&name) == Some(&body),
                    "{id}: changed native declaration {name}"
                );
            }
            assert_eq!(
                metadata["native_source_certificate_sha256"],
                mpk_cert::hash_hex(&mpk_cert::certificate_hash(native.certificate_bytes()))
            );
            assert_eq!(
                metadata["native_source_program_sha256"],
                format!("{:x}", sha2::Sha256::digest(native.canonical_bytes()))
            );
            Some(native)
        } else {
            None
        };
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        assert!(cert.proof_node_table.is_empty());
        assert!(cert.theory_certificates.is_empty());
        assert!(!cert
            .declarations
            .iter()
            .any(|d| matches!(d.kind, mpk_cert::encode::DeclarationKind::Axiom { .. })));
        let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
        let depths = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.as_str(), c.depth))
            .collect::<BTreeMap<_, _>>();
        for (s, original) in p.sequents().iter().zip(vc.control_vcs().sequents()) {
            assert_eq!(&s.source, original);
            assert_eq!(s.assumptions.len(), original.assumptions.len());
            assert_eq!(s.goals.len(), original.goals.len());
            total_sequents += 1;
            for (lowered, source) in s
                .assumptions
                .iter()
                .chain(&s.goals)
                .zip(original.assumptions.iter().chain(&original.goals))
            {
                assert_eq!(&lowered.source, source);
                if let Some(dependency) = &lowered.native_guard_dependency {
                    assert!(integrated);
                    native_guards += 1;
                    ownership_guards += usize::from(dependency.ownership.is_some());
                    assert_eq!(dependency.function_id, s.source.function_id);
                    let flow = native
                        .as_ref()
                        .unwrap()
                        .functions()
                        .iter()
                        .find(|f| f.source.function_id == dependency.function_id)
                        .unwrap();
                    let edge = flow
                        .edges
                        .iter()
                        .find(|e| e.source.id == dependency.edge_id)
                        .unwrap();
                    assert_eq!(edge.source.guard, lowered.source);
                    assert_eq!(edge.source.source_node_id, s.source.source_node_id);
                    assert_eq!(edge.source.target_node_id, s.source.target_node_id);
                    assert_eq!(edge.ownership, dependency.ownership);
                    assert_eq!(
                        edge.guard_definition.as_ref(),
                        Some(&dependency.guard_definition)
                    );
                    assert_eq!(
                        lowered.definition.as_ref(),
                        Some(&dependency.guard_definition)
                    );
                }
                let selected = lowered
                    .argument_indices
                    .iter()
                    .map(|&i| s.arguments[i].clone())
                    .collect::<Vec<_>>();
                assert_eq!(selected, source.bindings);
                let mut symbols = BTreeSet::new();
                names(&source.term, &mut symbols);
                assert!(lowered
                    .pending_constant_names
                    .iter()
                    .all(|n| symbols.contains(n)));
                if lowered.definition.is_none() {
                    assert!(!lowered.pending_constant_names.is_empty());
                    if integrated {
                        assert!(lowered
                            .pending_constant_names
                            .iter()
                            .all(|n| n.starts_with("Mpk.CSharp.Control.PatternStep.")));
                    }
                    pending += 1;
                    continue;
                }
                assert!(lowered.pending_constant_names.is_empty());
                compiled += 1;
                if id == "total_variable" && symbols.iter().any(|n| n.contains("Integer.MathLess"))
                {
                    // The exact original formula contains the header and after-edge
                    // value of the same source slot. Reverse the state values too.
                    for (old, new) in [
                        (3, 2),
                        (2, 3),
                        (2, 2),
                        (0, -1),
                        (i32::MIN, i32::MAX),
                        (i32::MAX, i32::MIN),
                    ] {
                        let values = source
                            .bindings
                            .iter()
                            .map(|b| if b.edge_id.is_some() { new } else { old })
                            .collect::<Vec<_>>();
                        let expected = mathematical(&source.term, &values)
                            .expect("original variable decrease formula");
                        let args = values
                            .iter()
                            .zip(&source.bindings)
                            .map(|(&n, b)| {
                                let depth = depths[b.type_id.as_str()];
                                if depth == 0 {
                                    V::Bit(n != 0)
                                } else {
                                    sparse_cube(
                                        depth,
                                        (0..32).filter(|i| n as u32 & (1 << i) != 0).collect(),
                                    )
                                }
                            })
                            .collect();
                        assert_eq!(
                            bit(run(&cert, lowered.definition.as_ref().unwrap(), args)),
                            expected,
                            "{id}: {old} -> {new}"
                        );
                        measure_observations += 1;
                    }
                    for a in &source.bindings {
                        for d in &source.bindings {
                            if a.kind == d.kind
                                && a.node_id == d.node_id
                                && a.value_id == d.value_id
                                && a.edge_id != d.edge_id
                            {
                                assert_ne!(
                                    lowered.argument_indices
                                        [source.bindings.iter().position(|b| b == a).unwrap()],
                                    lowered.argument_indices
                                        [source.bindings.iter().position(|b| b == d).unwrap()]
                                );
                                distinct_snapshots += 1;
                            }
                        }
                    }
                }
            }
            if let Some(definition) = &s.logical_implication_definition {
                assert!(s.pending_implication_reasons.is_empty());
                for seed in 0..4 {
                    let args = s
                        .arguments
                        .iter()
                        .enumerate()
                        .map(|(i, a)| {
                            let depth = depths[a.type_id.as_str()];
                            if depth == 0 {
                                V::Bit((i + seed) % 2 != 0)
                            } else {
                                sparse_cube(
                                    depth,
                                    BTreeSet::from([((i + seed) * 7) % (1usize << depth)]),
                                )
                            }
                        })
                        .collect::<Vec<_>>();
                    let eval = |p: &OrdinaryControlPredicateDefinition| {
                        bit(run(
                            &cert,
                            p.definition.as_ref().unwrap(),
                            p.argument_indices
                                .iter()
                                .map(|&i| args[i].clone())
                                .collect(),
                        ))
                    };
                    let expected = !s.assumptions.iter().all(eval) || s.goals.iter().all(eval);
                    assert_eq!(
                        bit(run(&cert, definition, args)),
                        expected,
                        "{id} {}",
                        s.source.id
                    );
                    implication_observations += 1;
                }
            } else {
                assert!(!s.pending_implication_reasons.is_empty());
            }
        }
        let mut altered = metadata.clone();
        if let Some(row) = altered["sequents"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|s| !s["arguments"].as_array().unwrap().is_empty())
        {
            row["arguments"][0]["edge_id"] = json!("wrong-source-edge");
            assert!(import(
                &serde_json::to_vec(&altered).unwrap(),
                p.certificate_bytes(),
                vir,
            )
            .is_err());
        }
        if integrated && id == "count_fill" {
            for key in [
                "native_source_program_sha256",
                "native_source_certificate_sha256",
            ] {
                let mut altered = metadata.clone();
                altered[key] = json!("0".repeat(64));
                assert!(import(
                    &serde_json::to_vec(&altered).unwrap(),
                    p.certificate_bytes(),
                    vir,
                )
                .is_err());
            }
            let mut altered = metadata.clone();
            let dependency = altered["sequents"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .flat_map(|s| s["assumptions"].as_array_mut().unwrap())
                .filter_map(|p| p.get_mut("native_guard_dependency"))
                .find(|d| d.get("ownership").is_some())
                .unwrap();
            dependency["edge_id"] = json!("wrong-native-edge");
            assert!(import(
                &serde_json::to_vec(&altered).unwrap(),
                p.certificate_bytes(),
                vir,
            )
            .is_err());
        }
        output(id, "json", &p.canonical_bytes(), integrated);
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        output(id, "hex", hex.as_bytes(), integrated);
        eprintln!(
            "control predicates {id}: {} original sequents",
            p.sequents().len()
        );
    }
    assert!(compiled > 0 && pending > 0 && distinct_snapshots > 0);
    assert!(measure_observations >= 6 && implication_observations > 0);
    assert_eq!(total_sequents, 215);
    if integrated {
        assert_eq!((compiled, pending), (398, 102));
        assert_eq!((native_guards, ownership_guards), (100, 4));
        assert_eq!(implication_observations, 452);
    } else {
        assert_eq!((compiled, pending), (383, 117));
        assert_eq!((native_guards, ownership_guards), (0, 0));
        assert_eq!(implication_observations, 392);
    }
    eprintln!("control predicates total: {total_sequents} sequents, {compiled} defined predicates, {pending} pending predicates, {measure_observations} original decrease observations, {implication_observations} implication observations, {distinct_snapshots} distinct snapshot pairs");
}
