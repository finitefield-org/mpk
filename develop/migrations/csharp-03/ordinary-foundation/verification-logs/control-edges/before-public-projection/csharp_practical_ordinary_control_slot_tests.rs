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
