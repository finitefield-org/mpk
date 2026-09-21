//! Published-array reads preserve original SSA subjects and index-range edges.
use super::super::relation_tests::{sample, storage};
use super::*;

fn input(bits: Vec<bool>) -> V {
    if bits.len() == 1 {
        V::Bit(bits[0])
    } else {
        V::Cube(bits)
    }
}
fn output(name: &str, bytes: &[u8]) {
    if let Some(root) = std::env::var_os("MPK_W09_SEQUENCE_DATA_OUT") {
        fs::create_dir_all(&root).unwrap();
        fs::write(Path::new(&root).join(name), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/sequence-data");
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
    }
}
fn sources() -> Vec<(String, Value, Value)> {
    let replay = read("data-phase/data-stage-replay.json");
    let ids = [
        (
            "i32-length",
            "aa0414dd24a419452c20f15454fef4c8f2b94c33964b01bc99fec3e5d5b3756d",
        ),
        (
            "i32-read",
            "434aa6f50a997c294f597282d3ccf50262ba3a25d8c9f18f27da39a2acc3ac75",
        ),
        (
            "nullable-length",
            "608585340ebac38993d126df590cd934daf102d405b8c6bad042223c6fd25fe8",
        ),
        (
            "product-length",
            "04140f88b477a4a825b97a2d761a3f18b9dd1658fc84127b69a754b6f0138fad",
        ),
        (
            "string-length",
            "00a2c148f1f2b35970ec19b457bbaabcb9658f58172dcc564ae94546b4db9416",
        ),
        (
            "field-read",
            "8f1d93ca0c2d78a6631a15770ba865f2aa52385744a035bac6fdf675d09c030a",
        ),
        (
            "property-read",
            "60f0c407bb7c3a92b5b534cc0e1fd6501ccf56a85e3f0984333f0ce3a75cef6f",
        ),
    ];
    ids.into_iter()
        .map(|(label, id)| {
            let row = replay
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["id"] == id)
                .unwrap();
            assert!(row["outcome"].get("reject").is_none());
            (label.into(), row.clone(), row["outcome"]["facts"].clone())
        })
        .collect()
}
#[test]
fn csharp_03_t06_w09_sequence_data_candidates() {
    verify(false);
}
#[test]
fn csharp_03_t06_w09_sequence_data_original_source() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| verify(true))
        .unwrap()
        .join()
        .unwrap();
}
// Independent low-bit-first sequence encoding. Avoid materializing the full
// physical cube for sequences of wide elements such as strings.
fn encoded_sequence(
    d: &OrdinarySequenceOperations,
    child_depth: u32,
    length: u32,
    cells: &BTreeMap<usize, Vec<bool>>,
) -> V {
    let depth = d.carrier.depth;
    let padding = depth - 1 - 12 - child_depth;
    let mut ones = BTreeSet::new();
    for bit in 0..32 {
        if length & (1 << bit) != 0 {
            ones.insert(bit << (depth - 5));
        }
    }
    for (&index, cell) in cells {
        assert!(index < 4096);
        for (leaf, &on) in cell.iter().enumerate() {
            if on {
                ones.insert(1 | (index << (1 + padding)) | (leaf << (13 + padding)));
            }
        }
    }
    sparse_cube(depth, ones)
}
fn verify(runtime: bool) {
    let bundle = b();
    let filter = std::env::var("MPK_W09_SEQUENCE_DATA_CONTEXT").ok();
    let op_filter = std::env::var("MPK_W09_SEQUENCE_DATA_OPERATION").ok();
    let mut observations = 0usize;
    let mut manifest = vec![];
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    let mut kinds = BTreeSet::new();
    for (id, request, facts) in sources() {
        let (context, captures) = support::replay_context(&bundle, &request);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let vir = emitted.vir();
        let p = generate_csharp_practical_ordinary_sequence_data(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        let data = vc.data_vcs();
        let foundations = generate_csharp_practical_ordinary_sequences(vir).unwrap();
        let available = foundations
            .definitions()
            .iter()
            .map(|d| (d.carrier.type_id.as_str(), d))
            .collect::<BTreeMap<_, _>>();
        assert!(!p.definitions().is_empty(), "{id}");
        assert_eq!(
            p.definitions()
                .iter()
                .map(|d| &d.source)
                .collect::<Vec<_>>(),
            data.definitions()
                .iter()
                .filter(|d| d.family == DataDefinitionFamily::Foundation
                    && available.contains_key(d.signature.id.rsplit_once('.').unwrap().0))
                .collect::<Vec<_>>()
        );
        for d in p.definitions() {
            assert_eq!(&d.sequence, available[d.sequence.carrier.type_id.as_str()]);
        }
        let included = p
            .definitions()
            .iter()
            .map(|d| d.source.id.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            p.pending_definition_ids(),
            data.definitions()
                .iter()
                .filter(|d| !included.contains(d.id.as_str()))
                .map(|d| d.id.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            p.operations().iter().map(|o| &o.source).collect::<Vec<_>>(),
            data.operations()
                .iter()
                .filter(|o| included.contains(o.definition_id.as_str()))
                .collect::<Vec<_>>()
        );
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_sequence_data(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        if let Some((meta, bytes)) = &previous {
            assert!(import_csharp_practical_ordinary_sequence_data(meta, bytes, vir).is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        let types = generate_csharp_practical_ordinary_carriers(vir)
            .unwrap()
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        if runtime && filter.as_ref().is_none_or(|s| s == &id) {
            for d in p.definitions() {
                let sig = &d.source.signature;
                let suffix = sig.id.rsplit('.').next().unwrap();
                if op_filter
                    .as_ref()
                    .is_some_and(|s| s != suffix && s != &sig.id)
                {
                    continue;
                }
                eprintln!("sequence data start: {id} {}", sig.id);
                let started = std::time::Instant::now();
                kinds.insert(suffix.to_owned());
                assert!(matches!(suffix, "length" | "read"), "native probe only");
                let child_depth = types[&d.sequence.element_type_id].depth;
                for length in [0u32, 1, 2, 4095, 4096] {
                    let mut cells = BTreeMap::new();
                    if length > 0 {
                        for (index, seed) in [(0, 1), (length as usize - 1, 2)] {
                            cells.insert(
                                index,
                                storage(
                                    &sample(
                                        &d.sequence.element_type_id,
                                        seed,
                                        &types,
                                        &facts,
                                        emitted.closure().closed(),
                                    ),
                                    &types,
                                ),
                            );
                        }
                    }
                    let sequence = encoded_sequence(&d.sequence, child_depth, length, &cells);
                    let indices = if suffix == "length" {
                        vec![0]
                    } else {
                        BTreeSet::from([
                            i32::MIN,
                            -1,
                            0,
                            1,
                            length as i32 - 1,
                            length as i32,
                            4096,
                            1 << 30,
                            i32::MAX,
                        ])
                        .into_iter()
                        .collect()
                    };
                    for index in indices {
                        let failed = suffix == "read" && (index < 0 || index as u32 >= length);
                        let raw = if suffix == "length" {
                            (0..32).map(|i| length & (1 << i) != 0).collect::<Vec<_>>()
                        } else {
                            cells
                                .get(&(index as usize))
                                .cloned()
                                .unwrap_or_else(|| vec![false; 1 << child_depth])
                        };
                        let mut inputs = vec![sequence.clone()];
                        if suffix == "read" {
                            inputs.push(input(
                                (0..32).map(|i| (index as u32) & (1 << i) != 0).collect(),
                            ));
                        }
                        for failure in &d.failure_definitions {
                            assert_eq!(bit(run(&cert, failure, inputs.clone())), failed);
                        }
                        for wrong in [None, Some(0), Some(raw.len() - 1)] {
                            let mut bits = raw.clone();
                            if let Some(i) = wrong {
                                bits[i] = !bits[i];
                            }
                            let mut args = inputs.clone();
                            args.push(input(bits));
                            if !failed {
                                assert_eq!(
                                    bit(run(&cert, &d.relation_definition, args.clone())),
                                    wrong.is_none(),
                                    "{id} {suffix} length {length} index {index} definition"
                                );
                            }
                            for o in p
                                .operations()
                                .iter()
                                .filter(|o| o.source.definition_id == d.source.id)
                            {
                                assert_eq!(
                                    o.source
                                        .subjects
                                        .iter()
                                        .map(|s| &s.type_id)
                                        .collect::<Vec<_>>(),
                                    sig.argument_type_ids
                                        .iter()
                                        .chain(std::iter::once(&sig.normal_result_type_id))
                                        .collect::<Vec<_>>()
                                );
                                assert_eq!(
                                    bit(run(&cert, &o.predicates["success_guard"], args.clone())),
                                    !failed
                                );
                                assert_eq!(
                                    bit(run(&cert, &o.predicates["success_goal"], args.clone())),
                                    failed || wrong.is_none()
                                );
                                if !failed {
                                    assert_eq!(
                                        bit(run(
                                            &cert,
                                            &o.predicates["success_relation"],
                                            args.clone()
                                        )),
                                        wrong.is_none()
                                    );
                                }
                                for (index, check) in o.source.checks.iter().enumerate() {
                                    assert_eq!(check.check.id, "index_range");
                                    assert_eq!(index, 0);
                                    assert!(bit(run(
                                        &cert,
                                        &o.predicates["check.0.prefix"],
                                        args.clone()
                                    )));
                                    assert_eq!(
                                        bit(run(
                                            &cert,
                                            &o.predicates["check.0.failed"],
                                            args.clone()
                                        )),
                                        failed
                                    );
                                    assert_eq!(
                                        bit(run(
                                            &cert,
                                            &o.predicates["check.0.guard"],
                                            args.clone()
                                        )),
                                        failed
                                    );
                                }
                            }
                            observations += 1;
                        }
                    }
                }
                eprintln!(
                    "sequence data end: {id} {} {:.3}s",
                    sig.id,
                    started.elapsed().as_secs_f64()
                );
            }
        }
        let meta: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        for key in [
            "source_ir_sha256",
            "foundation_sha256",
            "data_vc_sha256",
            "certificate_sha256",
        ] {
            let mut changed = meta.clone();
            changed[key] = json!("changed");
            assert!(import_csharp_practical_ordinary_sequence_data(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut changed = meta.clone();
        changed["definitions"][0]["relation_definition"] = json!("changed");
        assert!(import_csharp_practical_ordinary_sequence_data(
            &serde_json::to_vec(&changed).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
        if let Some(index) = p
            .definitions()
            .iter()
            .position(|d| !d.failure_definitions.is_empty())
        {
            let mut changed = meta.clone();
            changed["definitions"][index]["failure_definitions"][0] = json!("changed");
            assert!(import_csharp_practical_ordinary_sequence_data(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        for field in ["function_id", "node_id", "normal_successor_id"] {
            if !p.operations().is_empty() {
                let mut changed = meta.clone();
                changed["operations"][0]["source"][field] = json!("changed");
                assert!(import_csharp_practical_ordinary_sequence_data(
                    &serde_json::to_vec(&changed).unwrap(),
                    p.certificate_bytes(),
                    vir
                )
                .is_err());
            }
        }
        let mut damaged = p.certificate_bytes().to_vec();
        *damaged.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_sequence_data(
            &p.canonical_bytes(),
            &damaged,
            vir
        )
        .is_err());
        output(&format!("{id}.json"), &p.canonical_bytes());
        output(
            &format!("{id}.hex"),
            format!(
                "{}\n",
                p.certificate_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            )
            .as_bytes(),
        );
        manifest.push(json!({"id":id,"definitions":p.definitions().len(),"use_points":p.operations().len(),"pending_definition_ids":p.pending_definition_ids(),"certificate_sha256":meta["certificate_sha256"],"bytes":p.certificate_bytes().len()}));
        eprintln!(
            "sequence data context {id}: {} definitions, {} SSA points",
            p.definitions().len(),
            p.operations().len()
        );
    }
    assert_eq!(manifest.len(), 7);
    if runtime {
        assert!(observations > 0, "selector matched no sequence operation");
        if filter.is_none() && op_filter.is_none() {
            assert_eq!(
                kinds,
                ["length", "read"].map(str::to_owned).into_iter().collect()
            );
        }
    }
    output(
        "certificates.json",
        &serde_json::to_vec_pretty(&manifest).unwrap(),
    );
    eprintln!(
        "sequence data: {} contexts, {observations} result observations",
        manifest.len()
    );
}
