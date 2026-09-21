//! Source-bound slot relations; observations do not discharge control sequents.
use super::*;
use std::path::PathBuf;

fn value(depth: u32, on: bool) -> V {
    if depth == 0 {
        V::Bit(on)
    } else {
        sparse_cube(
            depth,
            if on {
                BTreeSet::from([0])
            } else {
                BTreeSet::new()
            },
        )
    }
}
fn write_or_compare(id: &str, ext: &str, bytes: &[u8]) {
    let root = std::env::var_os("MPK_W09_CONTROL_SLOTS_OUT").map(PathBuf::from);
    let file = format!("{id}.{ext}");
    if let Some(root) = root {
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(file), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/control-slots");
        assert_eq!(fs::read(root.join(&file)).unwrap(), bytes, "{file}");
    }
}
fn array_snapshot(depth: u32, length: u32, private: bool, complete: bool) -> V {
    let mut ones = (0..32)
        .filter(|b| length & (1 << b) != 0)
        .map(|b| b << (depth - 5))
        .collect::<BTreeSet<_>>();
    if private && complete {
        for i in 0..length.min(16384) as usize {
            ones.insert(2 | (i << (depth - 14)));
        }
    }
    if length > 0 {
        for (i, value) in [(0, 7u32), (length.min(4096) - 1, 11)] {
            for b in 0..32 {
                if value & (1 << b) != 0 {
                    ones.insert(if private {
                        1 | ((i as usize) << 2) | (b << 16)
                    } else {
                        1 | ((i as usize) << 1) | (b << 13)
                    });
                }
            }
        }
    }
    sparse_cube(depth, ones)
}
#[test]
fn csharp_03_t06_w09_control_slot_array_snapshots() {
    let bundle = b();
    let requests = read("control-vc/loop-requests.json");
    let request = requests
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "index_update")
        .unwrap();
    let responses = read("control-emission/loop-responses.json");
    let facts = &responses
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "index_update")
        .unwrap()["facts"];
    let (context, captures) = support::replay_context(&bundle, request);
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(facts).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    let p = generate_csharp_practical_ordinary_control_slots(emitted.vir()).unwrap();
    assert_eq!(
        import_csharp_practical_ordinary_control_slots(
            &p.canonical_bytes(),
            p.certificate_bytes(),
            emitted.vir()
        )
        .unwrap(),
        p
    );
    let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    validate_csharp_practical_certificate_structure(&c).unwrap();
    let mut count = 0;
    let mut observations = 0;
    for f in p.functions() {
        assert_eq!(f.relations.len(), 1 + f.source.transfers.len());
        for r in &f.relations {
            let Some(memory) = &r.memory_binding else {
                continue;
            };
            count += 1;
            let transfer = r.transfer.as_ref().unwrap();
            assert_eq!(transfer, &memory.transfer);
            assert!(!memory.public_slot_projection_pending);
            let ssa = r.arguments.last().unwrap();
            assert_eq!(ssa.value_id.as_ref(), Some(&memory.arguments[1].value_id));
            assert_eq!(ssa.type_id, transfer.value.type_id);
            assert_eq!(ssa.node_id, memory.arguments[1].node_id);
            if transfer.kind == "load" {
                assert_ne!(ssa.value_id.as_ref(), Some(&transfer.value.id));
            }
            let public_depth = r
                .arguments
                .iter()
                .find(|a| a.role == "after_value" && a.slot_id == transfer.slot)
                .unwrap()
                .depth;
            for (length, complete, expected) in [
                (0, true, true),
                (1, true, true),
                (4095, true, true),
                (4096, true, true),
                (4097, true, false),
                (16384, true, false),
                (16385, true, false),
                (u32::MAX, true, false),
                (1, false, false),
                (4096, false, false),
            ] {
                eprintln!(
                    "snapshot {} length {length} complete {complete}",
                    transfer.source_node_id
                );
                let native = array_snapshot(ssa.depth, length, true, complete);
                let published = array_snapshot(public_depth, length, false, complete);
                let mut args = r
                    .arguments
                    .iter()
                    .map(|a| {
                        if a.role == "ssa" {
                            native.clone()
                        } else if a.role.ends_with("assigned") {
                            V::Bit(true)
                        } else if a.slot_id == transfer.slot {
                            published.clone()
                        } else {
                            value(a.depth, false)
                        }
                    })
                    .collect::<Vec<_>>();
                assert_eq!(
                    bit(run(
                        &c,
                        memory.public_projection_definedness.as_ref().unwrap(),
                        vec![native.clone()]
                    )),
                    expected
                );
                assert_eq!(
                    bit(run(
                        &c,
                        memory.public_projection_definition.as_ref().unwrap(),
                        vec![native, published]
                    )),
                    expected
                );
                assert_eq!(
                    bit(run(&c, &r.definition, args.clone())),
                    length <= 4096,
                    "source array snapshot: {} length {length} complete {complete}",
                    transfer.source_node_id
                );
                observations += 3;
                if length <= 4096 {
                    let after = r
                        .arguments
                        .iter()
                        .position(|a| a.role == "after_value" && a.slot_id == transfer.slot)
                        .unwrap();
                    args[after] =
                        sparse_cube(public_depth, BTreeSet::from([(1usize << public_depth) - 1]));
                    assert!(!bit(run(&c, &r.definition, args.clone())));
                    observations += 1;
                }
            }
        }
    }
    assert_eq!(count, 4);
    let original: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
    for (fi, f) in original["functions"].as_array().unwrap().iter().enumerate() {
        if let Some(ri) = f["relations"]
            .as_array()
            .unwrap()
            .iter()
            .position(|r| r["memory_binding"].is_object())
        {
            for field in ["memory_binding", "arguments"] {
                let mut altered = original.clone();
                altered["functions"][fi]["relations"][ri]
                    .as_object_mut()
                    .unwrap()
                    .remove(field);
                assert!(import_csharp_practical_ordinary_control_slots(
                    &serde_json::to_vec(&altered).unwrap(),
                    p.certificate_bytes(),
                    emitted.vir()
                )
                .is_err());
            }
            for field in [
                "public_projection_definition",
                "source_slot_snapshot_definition",
            ] {
                let mut changed = original.clone();
                changed["functions"][fi]["relations"][ri]["memory_binding"][field] =
                    json!("wrong-projection");
                assert!(import_csharp_practical_ordinary_control_slots(
                    &serde_json::to_vec(&changed).unwrap(),
                    p.certificate_bytes(),
                    emitted.vir()
                )
                .is_err());
            }
        }
    }
    write_or_compare("index_update", "json", &p.canonical_bytes());
    write_or_compare(
        "index_update",
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
    eprintln!("source array snapshots: {count} transfers, {observations} observations");
}
#[test]
fn csharp_03_t06_w09_control_slot_source_relations() {
    let bundle = b();
    let requests = read("control-vc/loop-requests.json");
    let responses = read("control-emission/loop-responses.json");
    let pattern_cases = read("control-emission/source-cases.json");
    let mut observations = 0;
    let mut transfers = BTreeSet::new();
    for id in [
        "while",
        "for",
        "short_circuit",
        "switch",
        "is_binding",
        "guard_order",
        "type",
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
                let row = pattern_cases
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
        let p = generate_csharp_practical_ordinary_control_slots(vir).unwrap();
        assert!(!p.functions().is_empty(), "{id}");
        assert_eq!(
            import_csharp_practical_ordinary_control_slots(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir,
            )
            .unwrap(),
            p
        );
        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&c).unwrap();
        eprintln!("control slots {id}: runtime start");
        for f in p.functions() {
            assert_eq!(f.relations.len(), 1 + f.source.transfers.len());
            assert_eq!(
                f.relations
                    .iter()
                    .filter_map(|r| r.transfer.clone())
                    .collect::<Vec<_>>(),
                f.source.transfers
            );
            for r in &f.relations {
                assert_eq!(
                    r.execution_scope,
                    if r.transfer.is_some() {
                        "successful_transfer"
                    } else {
                        "function_entry"
                    }
                );
                let writing = r.transfer.as_ref().is_some_and(|t| t.kind != "load");
                if let Some(t) = &r.transfer {
                    transfers.insert(t.kind.clone());
                }
                let args = r
                    .arguments
                    .iter()
                    .map(|a| {
                        let on = if a.role == "entry_assigned" {
                            f.source.entry_values.iter().any(|(s, _)| s == &a.slot_id)
                        } else if a.role.ends_with("assigned") {
                            true
                        } else {
                            writing
                                && (a.role == "ssa"
                                    || (a.role == "after_value"
                                        && r.transfer.as_ref().unwrap().slot == a.slot_id))
                        };
                        value(a.depth, on)
                    })
                    .collect::<Vec<_>>();
                assert!(
                    bit(run(&c, &r.definition, args.clone())),
                    "{id}: {}",
                    r.definition
                );
                observations += 1;
                if id == "type" && r.transfer.as_ref().is_some_and(|t| t.slot == "local:0") {
                    let mut present = args.clone();
                    for (i, a) in r.arguments.iter().enumerate() {
                        if a.slot_id == "local:0" && (a.role.ends_with("value") || a.role == "ssa")
                        {
                            assert_eq!(a.depth, 6);
                            // Option<Box(17)>: tag word 1 and nonzero stored payload.
                            present[i] = sparse_cube(6, BTreeSet::from([0, 1, 9]));
                        }
                    }
                    assert!(bit(run(&c, &r.definition, present.clone())));
                    let after = r
                        .arguments
                        .iter()
                        .position(|a| a.slot_id == "local:0" && a.role == "after_value")
                        .unwrap();
                    present[after] = value(6, false);
                    assert!(!bit(run(&c, &r.definition, present)));
                    observations += 2;
                }
                for (i, a) in r.arguments.iter().enumerate() {
                    let mut changed = args.clone();
                    let should_accept = match a.role.as_str() {
                        "entry_assigned" => {
                            let assigned =
                                f.source.entry_values.iter().any(|(s, _)| s == &a.slot_id);
                            changed[i] = V::Bit(!assigned);
                            false
                        }
                        "entry_value" => {
                            changed[i] = value(a.depth, true);
                            !f.source.entry_values.iter().any(|(s, _)| s == &a.slot_id)
                        }
                        "after_assigned" => {
                            changed[i] = V::Bit(false);
                            false
                        }
                        "after_value" => {
                            let written = writing && r.transfer.as_ref().unwrap().slot == a.slot_id;
                            changed[i] = value(a.depth, !written);
                            false
                        }
                        "before_assigned" => {
                            changed[i] = V::Bit(false);
                            writing && r.transfer.as_ref().unwrap().slot == a.slot_id
                        }
                        "before_value" => {
                            changed[i] = value(a.depth, true);
                            writing && r.transfer.as_ref().unwrap().slot == a.slot_id
                        }
                        "ssa" => {
                            changed[i] = value(a.depth, !writing);
                            false
                        }
                        _ => panic!("unknown slot argument role"),
                    };
                    assert_eq!(
                        bit(run(&c, &r.definition, changed)),
                        should_accept,
                        "{id}: {} {a:?}",
                        r.definition
                    );
                    observations += 1;
                }
                if let Some((i, a)) = r
                    .arguments
                    .iter()
                    .enumerate()
                    .find(|(_, a)| a.role == "after_value" && a.depth >= 5)
                {
                    let mut changed = args.clone();
                    let mut ones = BTreeSet::from([(1usize << a.depth) - 1]);
                    if writing && r.transfer.as_ref().unwrap().slot == a.slot_id {
                        ones.insert(0);
                    }
                    changed[i] = sparse_cube(a.depth, ones);
                    assert!(
                        !bit(run(&c, &r.definition, changed)),
                        "{id}: high physical bit"
                    );
                    observations += 1;
                }
                // Inactive locals carry arbitrary storage. A frame preserves
                // assignedness without inventing a default value for that storage.
                if let Some(t) = &r.transfer {
                    if let Some(slot) = f.source.slots.iter().find(|(s, _)| s != &t.slot) {
                        let mut changed = args.clone();
                        for (i, a) in r
                            .arguments
                            .iter()
                            .enumerate()
                            .filter(|(_, a)| a.slot_id == slot.0)
                        {
                            if a.role.ends_with("assigned") {
                                changed[i] = V::Bit(false);
                            }
                            if a.role == "after_value" {
                                changed[i] = value(a.depth, true);
                            }
                        }
                        assert!(bit(run(&c, &r.definition, changed)));
                        observations += 1;
                    }
                }
            }
        }
        if id == "type" {
            let overrides = &p.functions()[0].slot_type_overrides;
            assert_eq!(overrides.len(), 1);
            let option = overrides.get("local:0").unwrap();
            let local = p.functions()[0]
                .source
                .slots
                .iter()
                .find(|(slot, _)| slot == "local:0")
                .unwrap();
            assert_ne!(option, &local.1);
            for r in &p.functions()[0].relations {
                for a in r
                    .arguments
                    .iter()
                    .filter(|a| a.slot_id == "local:0" && a.role.ends_with("value"))
                {
                    assert_eq!(&a.type_id, option);
                }
            }
            let mut changed: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
            changed["functions"][0]["slot_type_overrides"]["local:0"] = json!(local.1);
            assert!(import_csharp_practical_ordinary_control_slots(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let original: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        for field in [
            "source_ir_sha256",
            "foundation_sha256",
            "control_vc_sha256",
            "application_scope_pending",
        ] {
            let mut altered = original.clone();
            altered[field] = if field == "application_scope_pending" {
                json!(false)
            } else {
                json!("wrong")
            };
            assert!(import_csharp_practical_ordinary_control_slots(
                &serde_json::to_vec(&altered).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        for field in ["transfer", "execution_scope", "arguments", "definition"] {
            let mut altered = original.clone();
            altered["functions"][0]["relations"][1]
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(import_csharp_practical_ordinary_control_slots(
                &serde_json::to_vec(&altered).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut altered = original.clone();
        altered["functions"][0]["source"]["transfers"] = json!([]);
        assert!(import_csharp_practical_ordinary_control_slots(
            &serde_json::to_vec(&altered).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
        let mut damaged = p.certificate_bytes().to_vec();
        *damaged.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_control_slots(
            &p.canonical_bytes(),
            &damaged,
            vir
        )
        .is_err());
        write_or_compare(id, "json", &p.canonical_bytes());
        write_or_compare(
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
            "control slots {id}: {} functions, {} cumulative observations",
            p.functions().len(),
            observations
        );
    }
    assert!(
        transfers.contains("load")
            && transfers.contains("store")
            && transfers.contains("pattern_bind")
    );
    assert!(observations > 100);
}

#[test]
fn csharp_03_t06_w09_control_slot_count_fill_candidate() {
    let bundle = b();
    let requests = read("control-vc/loop-requests.json");
    let request = requests
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "count_fill")
        .unwrap();
    let responses = read("control-emission/loop-responses.json");
    let facts = &responses
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "count_fill")
        .unwrap()["facts"];
    let (context, captures) = support::replay_context(&bundle, request);
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(facts).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    let p = generate_csharp_practical_ordinary_control_slots(emitted.vir()).unwrap();
    assert_eq!(
        import_csharp_practical_ordinary_control_slots(
            &p.canonical_bytes(),
            p.certificate_bytes(),
            emitted.vir()
        )
        .unwrap(),
        p
    );
    let certificate = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    validate_csharp_practical_certificate_structure(&certificate).unwrap();
    let bindings = p
        .functions()
        .iter()
        .flat_map(|f| &f.relations)
        .filter_map(|r| r.memory_binding.as_ref())
        .collect::<Vec<_>>();
    assert_eq!(bindings.len(), 4);
    let native_only = bindings
        .iter()
        .filter(|m| m.source_slot_type_id == m.arguments[1].type_id)
        .count();
    assert_eq!(native_only, 1);
    let mut observations = 0;
    for r in p.functions().iter().flat_map(|f| &f.relations) {
        let Some(memory) = &r.memory_binding else {
            continue;
        };
        let transfer = r.transfer.as_ref().unwrap();
        let native_depth = r.arguments.last().unwrap().depth;
        for complete in [true, false] {
            let native = array_snapshot(native_depth, 2, true, complete);
            let native_slot = memory.source_slot_type_id == memory.arguments[1].type_id;
            let mut arguments = r
                .arguments
                .iter()
                .map(|a| {
                    if a.role == "ssa" {
                        native.clone()
                    } else if a.role.ends_with("assigned") {
                        V::Bit(true)
                    } else if a.slot_id == transfer.slot {
                        array_snapshot(a.depth, 2, native_slot, complete)
                    } else {
                        value(a.depth, false)
                    }
                })
                .collect::<Vec<_>>();
            assert!(bit(run(&certificate, &r.definition, arguments.clone())));
            let after = r
                .arguments
                .iter()
                .position(|a| a.role == "after_value" && a.slot_id == transfer.slot)
                .unwrap();
            arguments[after] = value(r.arguments[after].depth, false);
            assert!(!bit(run(&certificate, &r.definition, arguments)));
            observations += 2;
        }
    }
    assert_eq!(observations, 16);
    eprintln!("count/fill source slots: {} relations, {} memory bindings, {native_only} native temporary bindings", p.functions().iter().map(|f| f.relations.len()).sum::<usize>(), bindings.len());
    write_or_compare("count_fill", "json", &p.canonical_bytes());
    write_or_compare(
        "count_fill",
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
}
