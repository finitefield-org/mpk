//! Original source conditions, state mutations and compatibility reconstruction.
use super::bit as observed;
use super::*;

#[path = "csharp_practical_ordinary_control_pattern_proof_tests.rs"]
mod proof_types;
#[path = "csharp_practical_ordinary_control_pattern_route_tests.rs"]
mod routes;

fn historical_condition_bodies(
    mut certificate: mpk_cert::encode::Certificate,
) -> BTreeMap<String, Value> {
    use sha2::{Digest, Sha256};
    let names: BTreeMap<String, String> = serde_json::from_value(read(
        "probes/boolean-proof-elimination-refreeze/pattern-current-identity-names.json",
    ))
    .unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(serde_json::to_vec(&names).unwrap())),
        "6fae7588d5f2fe3fc96dad32c3e069ff0f5068d8a5c12b513fb50b36d45776a6"
    );
    // The archived 61-condition checkpoint retains its original bytes. Rebind
    // only its independently replayed context names before comparing every
    // original declaration's complete type, body, proof and argument order.
    for name in &mut certificate.name_table {
        for (previous, current) in &names {
            if name.contains(previous) {
                *name = name.replace(previous, current);
            }
        }
    }
    declaration_bodies(&certificate)
}

fn each_context(mut check: impl FnMut(&str, &ValidatedPracticalVir)) {
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
        check(id, emitted.vir());
    }
}

fn physical(depth: u32, value: bool) -> V {
    if depth == 0 {
        V::Bit(value)
    } else {
        V::Cube(vec![value; 1usize << depth])
    }
}

fn flip(value: &mut V) {
    match value {
        V::Bit(value) => *value = !*value,
        V::Cube(bits) => bits[0] = !bits[0],
        _ => panic!("physical source observation"),
    }
}

fn word(value: i32) -> V {
    V::Cube((0..32).map(|i| value as u32 & (1 << i) != 0).collect())
}

fn prepare_primitive(
    entry: &OrdinaryControlPatternSourceDefinition,
    args: &mut [V],
    depths: &BTreeMap<&str, u32>,
) {
    let result = entry.source.original_binding_count - 1;
    let input = entry.source.operands.first().map(|o| o.binding_index);
    match entry.source_step.operation.as_str() {
        "unary_update" => {
            args[result] = word(
                if entry.source_step.source_kind.as_deref() == Some("Increment") {
                    0i32.checked_add(1).unwrap()
                } else {
                    0i32.checked_sub(1).unwrap()
                },
            );
        }
        "pattern_equal" => args[result] = V::Bit(true),
        "binary"
            if entry
                .source_step
                .source_traits
                .as_deref()
                .unwrap()
                .starts_with("Equals|") =>
        {
            args[result] = V::Bit(true)
        }
        "binary"
            if entry
                .source_step
                .source_traits
                .as_deref()
                .unwrap()
                .starts_with("Divide|") =>
        {
            args[input.unwrap()] = word(7);
            args[entry.source.operands[1].binding_index] = word(2);
            args[result] = word(7i32.checked_div(2).unwrap());
        }
        "pattern_type" if entry.semantic_rule == "source_nonnullable_type_presence" => {
            args[result] = V::Bit(true)
        }
        "pattern_bind"
            if entry.semantic_rule
                == "successful_source_payload_extraction_and_assigned_slot_store" =>
        {
            // Independent canonical Some tag; the payload is the zero Box.
            flip(&mut args[input.unwrap()]);
        }
        "element" => {
            let operand = &entry.source.operands[0];
            args[operand.binding_index] = super::super::native_array(
                depths[operand.native_value.type_id.as_str()],
                false,
                &[0, 19],
                true,
            );
        }
        _ => {}
    }
}

#[test]
fn csharp_03_t06_w09_pattern_sources_original_conditions() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(original_conditions)
        .unwrap()
        .join()
        .unwrap();
}

fn original_conditions() {
    let mut coverage = [0usize; 5];
    let mut previous: Option<Vec<u8>> = None;
    each_context(|id, vir| {
        let control = generate_csharp_practical_control_vcs_with_pattern_observations(vir).unwrap();
        let p =
            generate_csharp_practical_ordinary_control_predicates_with_pattern_observations(vir)
                .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let prior =
            generate_csharp_practical_ordinary_control_predicates_with_pattern_captures(vir)
                .unwrap();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/control-predicates/with-pattern-captures");
        assert_eq!(
            fs::read(root.join(format!("{id}.json"))).unwrap(),
            prior.canonical_bytes()
        );
        let old = mpk_cert::decode_canonical_certificate(prior.certificate_bytes()).unwrap();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        assert!(cert.proof_node_table.is_empty() && cert.theory_certificates.is_empty());
        assert!(!cert
            .declarations
            .iter()
            .any(|d| matches!(d.kind, mpk_cert::encode::DeclarationKind::Axiom { .. })));
        let bodies = declaration_bodies(&cert);
        // The new scopes quantify the added source operands and snapshots.
        // The historical API is pinned above; shared native bodies stay exact.
        let replaced_scopes = prior
            .pattern_scopes()
            .iter()
            .chain(prior.pattern_capture_scopes())
            .flat_map(|scope| &scope.executions)
            .filter_map(|execution| execution.definition.as_ref())
            .collect::<BTreeSet<_>>();
        for (name, body) in declaration_bodies(&old) {
            if !replaced_scopes.contains(&name) {
                assert_eq!(bodies.get(&name), Some(&body), "{id}: preserved {name}");
            }
        }
        let prior_conditions = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/control-predicates/with-pattern-observations");
        let hex = fs::read_to_string(prior_conditions.join(format!("{id}.hex"))).unwrap();
        let old_bytes = hex
            .trim()
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect::<Vec<_>>();
        let old_conditions = mpk_cert::decode_canonical_certificate(&old_bytes).unwrap();
        for (name, body) in historical_condition_bodies(old_conditions) {
            assert_eq!(
                bodies.get(&name),
                Some(&body),
                "{id}: preserved condition checkpoint {name}"
            );
        }
        let metadata: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        let old_metadata: Value = serde_json::from_slice(&prior.canonical_bytes()).unwrap();
        assert_eq!(
            metadata["native_source_program_sha256"],
            old_metadata["native_source_program_sha256"]
        );
        assert_eq!(
            metadata["native_source_certificate_sha256"],
            old_metadata["native_source_certificate_sha256"]
        );
        assert_eq!(metadata["control_vc_sha256"], control.hash());
        assert_eq!(
            p.pattern_sources().len(),
            control.pattern_observations().len()
        );
        let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
        let depths = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.as_str(), c.depth))
            .collect::<BTreeMap<_, _>>();
        for entry in p.pattern_sources() {
            coverage[0] += 1;
            assert!(entry.native_source_equivalence_proof_pending);
            let source = control
                .sequents()
                .iter()
                .find(|s| s.id == entry.source.sequent_id)
                .unwrap();
            let goal = &source.goals[0];
            let sequent = p
                .sequents()
                .iter()
                .find(|s| s.source.id == source.id)
                .unwrap();
            assert_eq!(&sequent.source, source);
            for scope in p
                .pattern_scopes()
                .iter()
                .chain(p.pattern_capture_scopes())
                .filter(|s| s.source_sequent_id == source.id)
            {
                assert_eq!(
                    scope.original_pattern_predicate_pending,
                    entry.definition.is_none()
                );
                for execution in &scope.executions {
                    assert_eq!(execution.observations.len(), goal.bindings.len());
                    for observation in &execution.observations {
                        assert_eq!(
                            &execution.arguments[observation.observation_argument_index],
                            &observation.source
                        );
                        if let Some(index) = observation.native_argument_index {
                            assert_eq!(
                                execution.source_execution.arguments[index],
                                observation.native_definition_point
                            );
                        }
                        if let Some(index) = observation.captured_argument_index {
                            assert_eq!(
                                execution.arguments[index],
                                observation.native_definition_point
                            );
                        }
                    }
                }
            }
            let Some(definition) = &entry.definition else {
                coverage[2] += 1;
                assert!(!entry.pending_definition_reasons.is_empty());
                assert_eq!(
                    sequent.goals[0].pending_constant_names,
                    vec![entry.source_name.clone()]
                );
                continue;
            };
            coverage[1] += 1;
            assert!(entry.pending_definition_reasons.is_empty());
            assert!(sequent.goals[0].definition.is_some());
            assert!(sequent.goals[0].pending_constant_names.is_empty());
            assert!(sequent.logical_implication_definition.is_some());
            let mut args = goal
                .bindings
                .iter()
                .map(|b| physical(depths[b.type_id.as_str()], false))
                .collect::<Vec<_>>();
            let result = entry.source.original_binding_count - 1;
            if entry.source_step.operation == "constant" {
                if let Some(number) = entry
                    .source_constant
                    .as_deref()
                    .unwrap()
                    .strip_prefix("System.Int32:")
                {
                    let number = number.parse::<i32>().unwrap() as u32;
                    args[result] = V::Cube((0..32).map(|i| number & (1 << i) != 0).collect());
                } else {
                    assert_eq!(entry.source_constant.as_deref(), Some("null"));
                }
            }
            if let Some(slot) = &entry.source.slot {
                args[slot.after_assigned_index] = V::Bit(true);
                if entry.source_step.operation == "load" {
                    args[slot.before_assigned_index] = V::Bit(true);
                }
            }
            prepare_primitive(entry, &mut args, &depths);
            assert!(
                observed(run(&cert, definition, args.clone())),
                "{id}: {}",
                entry.source_step.operation
            );
            coverage[3] += 1;
            let mut wrong = args.clone();
            flip(&mut wrong[result]);
            assert!(
                !observed(run(&cert, definition, wrong)),
                "{id}: wrong result"
            );
            coverage[4] += 1;
            let inputs = &entry.source.operands;
            let mut reject = |values| {
                assert!(
                    !observed(run(&cert, definition, values)),
                    "{id}: {} rejected input",
                    entry.source_step.operation
                );
                coverage[4] += 1;
            };
            match entry.source_step.operation.as_str() {
                "unary_update" => {
                    let mut wrong = args.clone();
                    wrong[inputs[0].binding_index] = word(
                        if entry.source_step.source_kind.as_deref() == Some("Increment") {
                            i32::MAX
                        } else {
                            i32::MIN
                        },
                    );
                    reject(wrong);
                    let mut changed = args.clone();
                    changed[inputs[0].binding_index] = word(4);
                    changed[result] = word(
                        if entry.source_step.source_kind.as_deref() == Some("Increment") {
                            5
                        } else {
                            3
                        },
                    );
                    assert!(
                        observed(run(&cert, definition, changed)),
                        "{id}: changed update input"
                    );
                    coverage[3] += 1;
                }
                "binary"
                    if entry
                        .source_step
                        .source_traits
                        .as_deref()
                        .unwrap()
                        .starts_with("Divide|") =>
                {
                    let mut wrong = args.clone();
                    wrong[inputs[1].binding_index] = word(0);
                    reject(wrong);
                    let mut wrong = args.clone();
                    wrong[inputs[0].binding_index] = word(i32::MIN);
                    wrong[inputs[1].binding_index] = word(-1);
                    reject(wrong);
                }
                "pattern_bind" if !entry.native_primitive_definitions.is_empty() => {
                    let mut none = args.clone();
                    flip(&mut none[inputs[0].binding_index]);
                    reject(none);
                    let mut wrong = args.clone();
                    if let V::Cube(bits) = &mut wrong[inputs[0].binding_index] {
                        // Option payload starts in role one, after low padding.
                        let payload = entry.source_step.result.as_ref().unwrap();
                        let input_depth = depths[inputs[0].native_value.type_id.as_str()];
                        let payload_depth = depths[payload.type_id.as_str()];
                        bits[1 | (1 << (input_depth - payload_depth))] = true;
                    }
                    reject(wrong);
                }
                "element" => {
                    for index in [-1, 2, 4096] {
                        let mut wrong = args.clone();
                        wrong[inputs[1].binding_index] = word(index);
                        reject(wrong);
                    }
                    let mut changed = args.clone();
                    changed[inputs[1].binding_index] = word(1);
                    changed[result] = word(19);
                    assert!(
                        observed(run(&cert, definition, changed)),
                        "{id}: second source element"
                    );
                    coverage[3] += 1;
                }
                "convert" => {
                    let mut wrong = args.clone();
                    flip(&mut wrong[inputs[0].binding_index]);
                    reject(wrong);
                }
                _ => {}
            }
            if let Some(slot) = &entry.source.slot {
                let mut wrong = args.clone();
                wrong[slot.after_assigned_index] = V::Bit(false);
                assert!(
                    !observed(run(&cert, definition, wrong)),
                    "{id}: after unassigned"
                );
                coverage[4] += 1;
                let mut wrong = args.clone();
                flip(&mut wrong[slot.after_value_index]);
                assert!(
                    !observed(run(&cert, definition, wrong)),
                    "{id}: wrong stored/frame value"
                );
                coverage[4] += 1;
                if entry.source_step.operation == "load" {
                    let mut wrong = args.clone();
                    wrong[slot.before_assigned_index] = V::Bit(false);
                    assert!(
                        !observed(run(&cert, definition, wrong)),
                        "{id}: load unassigned"
                    );
                    coverage[4] += 1;
                } else {
                    args[slot.before_assigned_index] = V::Bit(true);
                    flip(&mut args[slot.before_value_index]);
                    assert!(
                        observed(run(&cert, definition, args)),
                        "{id}: overwritten old value"
                    );
                    coverage[3] += 1;
                }
            }
        }
        let bytes = p.canonical_bytes();
        assert_eq!(
            import_csharp_practical_ordinary_control_predicates_with_pattern_observations(
                &bytes,
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        if let Some(previous) = &previous {
            assert!(
                import_csharp_practical_ordinary_control_predicates_with_pattern_observations(
                    previous,
                    p.certificate_bytes(),
                    vir
                )
                .is_err()
            );
        }
        previous = Some(bytes.clone());
        if !p.pattern_sources().is_empty() {
            let mut changed: Value = serde_json::from_slice(&bytes).unwrap();
            changed["pattern_sources"][0]["native_source_equivalence_proof_pending"] = json!(false);
            assert!(
                import_csharp_practical_ordinary_control_predicates_with_pattern_observations(
                    &serde_json::to_vec(&changed).unwrap(),
                    p.certificate_bytes(),
                    vir
                )
                .is_err()
            );
        }
        let output = std::env::var_os("MPK_W09_CONTROL_PATTERN_SOURCE_OUTPUT");
        let root = output.as_ref().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/control-predicates/with-pattern-primitives"));
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        if output.is_some() {
            fs::create_dir_all(&root).unwrap();
            fs::write(root.join(format!("{id}.json")), bytes).unwrap();
            fs::write(root.join(format!("{id}.hex")), hex).unwrap();
        } else {
            assert_eq!(fs::read(root.join(format!("{id}.json"))).unwrap(), bytes);
            assert_eq!(
                fs::read_to_string(root.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
    });
    assert_eq!(coverage[0], 102);
    assert_eq!(coverage[1..3], [81, 21]);
    eprintln!("original pattern source conditions: {} goals, {} definitions, {} pending; {} positive and {} changed-value/state observations", coverage[0], coverage[1], coverage[2], coverage[3], coverage[4]);
}
