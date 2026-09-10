use super::*;
use core_eval::{apply, bit as observed_bit, run, V};

#[test]
fn csharp_03_t06_w09_defaults_source_eligibility_edges() {
    let bundle = b();
    let rows = read("data-phase/data-stage-replay.json");
    let cases = [
        (
            "required",
            "3f5a3d6025a985270ba58a3eb960782988dab3411d6825db2bd16d5cb69dbbf0",
        ),
        (
            "no-enum-zero",
            "b709aea1c17e601dae9cf1f96bd2fa15c7cb29c15c35656803b972b969156522",
        ),
        (
            "sealed-class",
            "077c7587981ea6724bfa60081d30e1fb10e941e30f85b8d72541f7361c6645cd",
        ),
        (
            "nested-sources",
            "f0219603fe7565fabe57876c5135941b287b1f66906ef2c463f7ce72c88cda19",
        ),
    ];
    for (label, id) in cases {
        let row = rows
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        let facts = &row["outcome"]["facts"];
        let (context, captures) = support::replay_context(&bundle, row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let program = generate_csharp_practical_ordinary_defaults(emitted.vir()).unwrap();
        let source_defaults = program
            .definitions()
            .iter()
            .filter(|d| {
                facts["types"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|t| t["id"] == d.carrier.type_id)
            })
            .collect::<Vec<_>>();
        assert!(!source_defaults.is_empty(), "{label}");
        if label != "nested-sources" {
            for definition in source_defaults {
                assert!(definition.structural_candidate.is_none(), "{label}");
                assert!(!definition.public_candidate_admitted, "{label}");
            }
            continue;
        }
        assert_eq!(source_defaults.len(), 17);
        let mut counts = vec![];
        for definition in source_defaults {
            let candidate = definition.structural_candidate.as_ref().unwrap();
            assert!(definition.public_candidate_admitted);
            // Each original source adds one product around the scalar leaf.
            assert_eq!(
                candidate.logical_cells,
                candidate.source_requirements.len() as u64 + 1
            );
            counts.push(candidate.logical_cells);
        }
        counts.sort();
        assert_eq!(counts, (2..=18).collect::<Vec<_>>());
        let metadata = program.canonical_bytes();
        let data: Value = serde_json::from_slice(&metadata).unwrap();
        let index = data["definitions"]
            .as_array()
            .unwrap()
            .iter()
            .position(|d| d["structural_candidate"]["logical_cells"] == 18)
            .unwrap();
        for field in ["source_requirements", "logical_cells", "definition"] {
            let mut changed = data.clone();
            let candidate = &mut changed["definitions"][index]["structural_candidate"];
            match field {
                "source_requirements" => {
                    candidate[field].as_array_mut().unwrap().remove(0);
                }
                "logical_cells" => candidate[field] = json!(17),
                _ => candidate[field] = json!("Std.Bool.false"),
            }
            assert!(
                import_csharp_practical_ordinary_defaults(
                    &serde_json::to_vec(&changed).unwrap(),
                    program.certificate_bytes(),
                    emitted.vir()
                )
                .is_err(),
                "accepted changed {field}"
            );
        }
        let oversized = vec![0; 16 * 1024 * 1024 + 1];
        assert!(matches!(
            import_csharp_practical_ordinary_defaults(
                &oversized,
                program.certificate_bytes(),
                emitted.vir()
            ),
            Err(OrdinaryCarrierError::Limit)
        ));
        assert!(matches!(
            import_csharp_practical_ordinary_defaults(&metadata, &oversized, emitted.vir()),
            Err(OrdinaryCarrierError::Limit)
        ));
    }
}

#[test]
fn csharp_03_t06_w09_defaults_original_sources_and_mutations() {
    let bundle = b();
    let output = std::env::var_os("MPK_W09_DEFAULTS_OUT").map(std::path::PathBuf::from);
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/defaults");
    if let Some(p) = &output {
        fs::create_dir_all(p).unwrap();
    }
    let mut metrics = vec![];
    let mut available = 0;
    let mut unavailable = 0;
    let mut source_conditions = 0;
    let mut structural_only = 0;
    let mut source_graphs = 0;
    let mut absent_without_payload_default = 0;
    let mut sources = relation_tests::sources()
        .into_iter()
        .chain(domain_sources::sources())
        .collect::<Vec<_>>();
    let rows = read("data-phase/data-stage-replay.json");
    let nullable_string = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "57de098014bbd8059f221b24525f74c6cb70320bfdcac33794de4ae0af3e143e")
        .unwrap();
    sources.push((
        "nullable-string-default".into(),
        nullable_string.clone(),
        nullable_string["outcome"]["facts"].clone(),
    ));
    for (id, row, facts) in sources {
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let program = generate_csharp_practical_ordinary_defaults(emitted.vir()).unwrap();
        let certificate =
            mpk_cert::decode_canonical_certificate(program.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&certificate).unwrap();
        let types = program
            .definitions()
            .iter()
            .map(|d| (d.carrier.type_id.clone(), d.carrier.clone()))
            .collect::<BTreeMap<_, _>>();
        for definition in program.definitions() {
            let oracle = domain_default(
                &bundle,
                emitted.closure().roots(),
                emitted.closure().closed(),
                &definition.carrier.type_id,
            );
            assert_eq!(
                definition.public_candidate_admitted,
                oracle.is_ok(),
                "{id} {} admission",
                definition.carrier.type_id
            );
            assert_eq!(
                observed_bit(run(
                    &certificate,
                    &definition.public_admission_definition,
                    vec![]
                )),
                oracle.is_ok()
            );
            if let Some(t) = facts["types"]
                .as_array()
                .unwrap()
                .iter()
                .find(|t| t["id"] == definition.carrier.type_id)
            {
                let graph = &t["recursive_default"];
                assert_eq!(
                    definition.structural_candidate.is_some(),
                    !graph.is_null(),
                    "{id} {} captured default graph",
                    definition.carrier.type_id
                );
                if !graph.is_null() {
                    let mut counts = vec![];
                    let mut source_requirements = BTreeMap::new();
                    for (index, node) in graph["nodes"].as_array().unwrap().iter().enumerate() {
                        let mut cells = 1u64;
                        for child in node["members"].as_array().unwrap() {
                            let child = child.as_u64().unwrap() as usize;
                            assert!(child < index);
                            cells += counts[child];
                        }
                        counts.push(cells);
                        if let Some(source) = facts["types"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|source| source["id"] == node["type_id"])
                        {
                            let requirement = OrdinarySourceDefaultRequirement {
                                source_type_id: source["id"].as_str().unwrap().into(),
                                source_sha256: source["source_sha256"].as_str().unwrap().into(),
                                declared_public_default: source["public_default"]
                                    .as_bool()
                                    .unwrap(),
                            };
                            source_requirements
                                .insert(requirement.source_type_id.clone(), requirement);
                        }
                    }
                    assert_eq!(
                        definition
                            .structural_candidate
                            .as_ref()
                            .unwrap()
                            .logical_cells,
                        counts[graph["root"].as_u64().unwrap() as usize]
                    );
                    // Compare the entire independently captured set: checking
                    // only emitted entries would miss a dropped obligation.
                    assert_eq!(
                        definition
                            .structural_candidate
                            .as_ref()
                            .unwrap()
                            .source_requirements,
                        source_requirements.into_values().collect::<Vec<_>>(),
                        "{id} {} complete source conditions",
                        definition.carrier.type_id
                    );
                    source_graphs += 1;
                }
            }
            if let Some(candidate) = &definition.structural_candidate {
                available += 1;
                structural_only += usize::from(!definition.public_candidate_admitted);
                let value = run(&certificate, &candidate.definition, vec![]);
                let depth = definition.carrier.depth;
                let addresses: Vec<Vec<bool>> = if depth <= 10 {
                    (0..1usize << depth)
                        .map(|address| (0..depth).map(|i| address & (1 << i) != 0).collect())
                        .collect()
                } else {
                    [vec![false; depth as usize], vec![true; depth as usize]]
                        .into_iter()
                        .chain((0..depth).map(|bit| (0..depth).map(|i| i == bit).collect()))
                        .collect()
                };
                // Observe all small default leaves and high/mixed selectors
                // for larger cubes without allocating their inactive storage.
                for address in addresses {
                    let mut leaf = value.clone();
                    for &selector in &address {
                        leaf = apply(&certificate, leaf, V::Bit(selector));
                    }
                    assert!(!observed_bit(leaf), "{id} default address {address:?}");
                }
                if let OrdinaryShape::Sum { arms } = &definition.carrier.shape {
                    for arm in arms
                        .iter()
                        .filter(|a| matches!(a.id.as_str(), "some" | "found"))
                    {
                        let OrdinaryShape::Reference { type_id } = &arm.fields[0].shape else {
                            panic!()
                        };
                        if program
                            .definitions()
                            .iter()
                            .find(|d| d.carrier.type_id == *type_id)
                            .unwrap()
                            .structural_candidate
                            .is_none()
                        {
                            assert_eq!(candidate.logical_cells, 1);
                            assert!(candidate.source_requirements.is_empty());
                            assert!(definition.public_candidate_admitted);
                            absent_without_payload_default += 1;
                        }
                    }
                }
                for requirement in &candidate.source_requirements {
                    let t = facts["types"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|t| t["id"] == requirement.source_type_id)
                        .unwrap();
                    assert_eq!(t["source_sha256"], requirement.source_sha256);
                    assert_eq!(t["public_default"], requirement.declared_public_default);
                    source_conditions += 1;
                }
                if let Ok(value) = oracle {
                    assert_eq!(candidate.logical_cells, domain_tests::cells(&value) as u64);
                    let encoded = relation_tests::storage(&value, &types);
                    assert!(encoded.iter().all(|&b| !b));
                }
            } else {
                unavailable += 1;
            }
        }
        let metadata = program.canonical_bytes();
        assert_eq!(
            import_csharp_practical_ordinary_defaults(
                &metadata,
                program.certificate_bytes(),
                emitted.vir()
            )
            .unwrap(),
            program
        );
        let data: Value = serde_json::from_slice(&metadata).unwrap();
        for field in [
            "schema",
            "source_ir_sha256",
            "foundation_sha256",
            "certificate_sha256",
            "definitions",
        ] {
            let mut changed = data.clone();
            changed[field] = json!("forged");
            assert!(import_csharp_practical_ordinary_defaults(
                &serde_json::to_vec(&changed).unwrap(),
                program.certificate_bytes(),
                emitted.vir()
            )
            .is_err());
        }
        let mut changed = program.certificate_bytes().to_vec();
        *changed.last_mut().unwrap() ^= 1;
        assert!(
            import_csharp_practical_ordinary_defaults(&metadata, &changed, emitted.vir()).is_err()
        );
        let file = format!("{id}.hex");
        let hex = program
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(p) = &output {
            fs::write(p.join(&file), hex).unwrap();
        } else {
            assert_eq!(fs::read_to_string(fixtures.join(&file)).unwrap(), hex);
        }
        metrics.push(json!({"id":id,"file":file,"terms":certificate.term_table.len(),"declarations":certificate.declarations.len(),"metadata":data}));
    }
    assert_eq!(metrics.len(), 29);
    assert!(absent_without_payload_default > 0);
    assert!(
        available > 50
            && unavailable > 10
            && source_conditions > 5
            && structural_only > 0
            && source_graphs > 5,
        "coverage {available}/{unavailable}/{source_conditions}/{structural_only}/{source_graphs}"
    );
    eprintln!("defaults: {available} candidates, {unavailable} ineligible, {source_conditions} pending source conditions, {structural_only} structural-only, {source_graphs} source graphs");
    if let Some(p) = output {
        fs::write(
            p.join("certificates.json"),
            serde_json::to_vec_pretty(&metrics).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            serde_json::from_slice::<Value>(&fs::read(fixtures.join("certificates.json")).unwrap())
                .unwrap(),
            json!(metrics)
        );
    }
}
