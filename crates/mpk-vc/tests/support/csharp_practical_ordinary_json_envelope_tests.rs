//! Original complete boundary documents; semantic arguments and full packet mask.
use super::super::super::super::{relation_tests, structural_equivalence_tests};
use super::boundary_field_tests::cells;
use super::*;

fn verify_packet(
    cert: &mpk_cert::encode::Certificate,
    d: &OrdinaryJsonEnvelopeDefinition,
    text: &[u8],
    expected: Option<&[DecodedBoundaryArgument]>,
    types: &BTreeMap<String, OrdinaryCarrier>,
) {
    let result = run(
        cert,
        &d.parse_definition,
        vec![document(
            text.len() as u32,
            &text.iter().copied().enumerate().collect::<Vec<_>>(),
        )],
    );
    let mut wanted = vec![false; 1 << d.packet_depth];
    if let Some(arguments) = expected {
        let count = 1 + arguments.iter().map(|a| cells(a.value())).sum::<u32>();
        wanted[0] = true;
        wanted[2] = true;
        for i in 0..32 {
            wanted[(2 + i) << 1] = text.len() & (1 << i) != 0;
            wanted[(34 + i) << 1] = count & (1 << i) != 0;
        }
        // Independent product layout from captured argument order and existing
        // semantic carriers, rather than accepting the generated offsets.
        let roles = if arguments.len() <= 1 {
            0
        } else {
            usize::BITS - (arguments.len() - 1).leading_zeros()
        };
        let max_depth = arguments
            .iter()
            .map(|a| types[a.value().type_id()].depth)
            .max()
            .unwrap_or(0);
        assert_eq!(d.arguments_depth, roles + max_depth);
        for (index, arg) in arguments.iter().enumerate() {
            let field_depth = types[arg.value().type_id()].depth;
            for (bit, value) in relation_tests::storage(arg.value(), types)
                .into_iter()
                .enumerate()
            {
                let address = index | (bit << (d.arguments_depth - field_depth));
                wanted[1 | (address << 1)] = value;
            }
        }
    }
    for (i, want) in wanted.into_iter().enumerate() {
        assert_eq!(
            leaf(cert, result.clone(), d.packet_depth as usize, i),
            want,
            "{} bit{i} document{}",
            d.contract_sha256,
            String::from_utf8_lossy(text)
        );
    }
}
#[test]
fn csharp_03_t06_w09_json_envelopes_original_documents() {
    source_documents(0);
}
#[test]
fn csharp_03_t06_w09_json_depth_guarded_envelopes_original_documents() {
    source_documents(1);
}
#[test]
fn csharp_03_t06_w09_json_typed_guarded_envelopes_original_documents() {
    source_documents(2);
}
#[test]
fn csharp_03_t06_w09_json_limits_guarded_envelopes_original_documents() {
    source_documents(3);
}
fn check_raw_guard_closures(emitted: &EmittedDataPhase, p: &OrdinaryJsonEnvelopeProgram) {
    let previous =
        generate_csharp_practical_ordinary_json_typed_guarded_envelopes(emitted).unwrap();
    let raw = generate_csharp_practical_ordinary_json_raw_limits(emitted.vir()).unwrap();
    let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    for bytes in [previous.certificate_bytes(), raw.certificate_bytes()] {
        let old = mpk_cert::decode_canonical_certificate(bytes).unwrap();
        let names = old
            .declarations
            .iter()
            .map(|d| old.name_table[d.name as usize].clone())
            .collect();
        structural_equivalence_tests::same_definition_closure(&old, &cert, &names).unwrap();
    }
    assert_eq!(p.typed_depth(), previous.typed_depth());
    assert_eq!(p.typed_nodes(), previous.typed_nodes());
    let mut standalone = raw.definition().unwrap().clone();
    let integrated = p.raw_limits().unwrap();
    // This metric records the owning builder's cumulative transformer count.
    assert!(integrated.static_transformers >= standalone.static_transformers);
    assert!(integrated.static_transformers <= 16_384);
    standalone.static_transformers = integrated.static_transformers;
    assert_eq!(integrated, &standalone);
    for (d, old) in p.definitions().iter().zip(previous.definitions()) {
        assert_eq!(
            d.parse_definition,
            format!("{}.RawLimits", old.parse_definition)
        );
        assert_eq!(d.unguarded_parse_definition, old.unguarded_parse_definition);
    }
}
fn source_documents(mode: u8) {
    let bundle = b();
    let requests = read("boundary-attachment/requests.json");
    let responses = read("boundary-attachment/responses.json");
    let accepted = read("boundary-vc/goldens.json");
    let conformance = read("boundary-input/conformance.json");
    let mut sources = accepted
        .as_array()
        .unwrap()
        .iter()
        .map(|a| {
            let request = requests
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["id"] == a["id"])
                .unwrap()
                .clone();
            let response = responses
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["id"] == a["id"])
                .unwrap()
                .clone();
            (
                request["case"].as_str().unwrap().to_owned(),
                request,
                response,
            )
        })
        .collect::<Vec<_>>();
    let extra = read("boundary-output/source-requests.json");
    let extra_responses = read("boundary-output/source-responses.json");
    let request = extra
        .as_array()
        .unwrap()
        .iter()
        .find(|r| {
            r["inputs"].as_array().unwrap().iter().any(|i| {
                i["path"] == "contracts/boundary.json"
                    && parse_canonical_practical_json(
                        PracticalArtifactKind::BoundaryContract,
                        i["utf8"].as_str().unwrap().as_bytes(),
                    )
                    .unwrap()
                    .get("input_fields")
                    .and_then(PracticalJsonValue::as_array)
                    .unwrap()
                    .is_empty()
            })
        })
        .unwrap()
        .clone();
    let response = extra_responses
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == request["id"])
        .unwrap()
        .clone();
    sources.push(("empty-input".into(), request, response));
    let selected = std::env::var("MPK_W09_JSON_ENVELOPES_SELECT")
        .ok()
        .map(|s| s.split(',').map(str::to_owned).collect::<BTreeSet<_>>())
        .or_else(|| {
            // The raw adapter shares all previous parser definitions. Exercise
            // required/empty/default paths here; retain broader parser runtime
            // coverage in their existing suites and verify compound closures.
            (mode == 3).then(|| {
                [
                    "required",
                    "empty-input",
                    "nullable_default",
                    "presence_missing_default",
                ]
                .into_iter()
                .map(str::to_owned)
                .collect()
            })
        });
    let output = std::env::var_os(match mode {
        0 => "MPK_W09_JSON_ENVELOPES_OUT",
        1 => "MPK_W09_JSON_DEPTH_GUARDED_ENVELOPES_OUT",
        2 => "MPK_W09_JSON_TYPED_GUARDED_ENVELOPES_OUT",
        3 => "MPK_W09_JSON_LIMITS_GUARDED_ENVELOPES_OUT",
        _ => unreachable!(),
    })
    .map(std::path::PathBuf::from);
    let family = match mode {
        0 => "json-envelopes",
        1 => "json-depth-guarded-envelopes",
        2 => "json-typed-guarded-envelopes",
        3 => "json-limits-guarded-envelopes",
        _ => unreachable!(),
    };
    let generate = match mode {
        0 => generate_csharp_practical_ordinary_json_envelopes,
        1 => generate_csharp_practical_ordinary_json_depth_guarded_envelopes,
        2 => generate_csharp_practical_ordinary_json_typed_guarded_envelopes,
        3 => generate_csharp_practical_ordinary_json_limits_guarded_envelopes,
        _ => unreachable!(),
    };
    let import = match mode {
        0 => import_csharp_practical_ordinary_json_envelopes,
        1 => import_csharp_practical_ordinary_json_depth_guarded_envelopes,
        2 => import_csharp_practical_ordinary_json_typed_guarded_envelopes,
        3 => import_csharp_practical_ordinary_json_limits_guarded_envelopes,
        _ => unreachable!(),
    };
    if let Some(out) = &output {
        fs::create_dir_all(out).unwrap();
    }
    let mut rows = vec![];
    let mut good_count = 0;
    let mut bad_count = 0;
    for (case, request, response) in sources {
        if selected.as_ref().is_some_and(|s| !s.contains(&case)) {
            continue;
        }
        eprintln!("Envelope source: {case}");
        let (context, captures) = support::replay_context(&bundle, &request);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&response["facts"]).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate(&emitted).unwrap_or_else(|e| panic!("{case}: {e:?}"));
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let prior = generate_csharp_practical_ordinary_json_boundary_fields(&emitted).unwrap();
        assert_eq!(p.field_decoders(), prior.fields());
        let old = mpk_cert::decode_canonical_certificate(prior.certificate_bytes()).unwrap();
        let names = old
            .declarations
            .iter()
            .map(|d| old.name_table[d.name as usize].clone())
            .collect();
        structural_equivalence_tests::same_definition_closure(&old, &cert, &names).unwrap();
        let layouts = generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap();
        let types = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        if mode > 0 {
            let base = generate_csharp_practical_ordinary_json_envelopes(&emitted).unwrap();
            let old = mpk_cert::decode_canonical_certificate(base.certificate_bytes()).unwrap();
            let names = old
                .declarations
                .iter()
                .map(|d| old.name_table[d.name as usize].clone())
                .collect();
            structural_equivalence_tests::same_definition_closure(&old, &cert, &names).unwrap();
            assert_eq!(
                p.definitions()[0].unguarded_parse_definition.as_deref(),
                Some(base.definitions()[0].parse_definition.as_str())
            );
        }
        if mode == 2 {
            let previous =
                generate_csharp_practical_ordinary_json_depth_guarded_envelopes(&emitted).unwrap();
            let old = mpk_cert::decode_canonical_certificate(previous.certificate_bytes()).unwrap();
            let names = old
                .declarations
                .iter()
                .map(|d| old.name_table[d.name as usize].clone())
                .collect();
            structural_equivalence_tests::same_definition_closure(&old, &cert, &names).unwrap();
            assert_eq!(p.typed_depth(), previous.typed_depth());
            let counts = generate_csharp_practical_ordinary_json_typed_nodes(&emitted).unwrap();
            assert_eq!(p.typed_nodes(), counts.definitions());
            let old = mpk_cert::decode_canonical_certificate(counts.certificate_bytes()).unwrap();
            let names = old
                .declarations
                .iter()
                .map(|d| old.name_table[d.name as usize].clone())
                .collect();
            structural_equivalence_tests::same_definition_closure(&old, &cert, &names).unwrap();
        }
        if mode == 3 {
            check_raw_guard_closures(&emitted, &p);
        }
        assert_eq!(p.definitions().len(), 1);
        let d = &p.definitions()[0];
        let boundary = &emitted.boundaries()[0];
        let boundary_id = boundary
            .artifact()
            .value()
            .get("boundary_id")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(
            d.fields
                .iter()
                .map(|f| f.field_id.as_str())
                .collect::<Vec<_>>(),
            boundary
                .input_fields()
                .iter()
                .map(|f| f.id())
                .collect::<Vec<_>>()
        );
        let mut documents = conformance["runs"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["source_snapshot_sha256"] == request["id"])
            .map(|r| r["document_utf8"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>();
        if case == "empty-input" {
            documents.push("{}".into());
        }
        assert!(
            !documents.is_empty(),
            "source requires original input evidence"
        );
        for (index, text) in documents.iter().enumerate() {
            let capture = emitted
                .capture_boundary_input(
                    &bundle,
                    &context,
                    &captures,
                    BoundaryInputBytes {
                        boundary_id,
                        provenance_id: "test.ordinary.envelope",
                        raw_bytes: text.as_bytes(),
                        canonical_document: text.as_bytes(),
                    },
                )
                .unwrap();
            verify_packet(&cert, d, text.as_bytes(), Some(capture.arguments()), &types);
            good_count += 1;
            if index == 0
                && [
                    "required",
                    "optional_default",
                    "surrogate_name",
                    "empty-input",
                ]
                .contains(&case.as_str())
            {
                let value = parse_canonical_practical_json(
                    PracticalArtifactKind::BoundaryInput,
                    text.as_bytes(),
                )
                .unwrap();
                let members = value
                    .utf16_members()
                    .unwrap()
                    .into_iter()
                    .map(|(n, v)| (n, v.clone()))
                    .collect::<Vec<_>>();
                let mut bad = vec![
                    "null".to_owned(),
                    "[]".into(),
                    format!(" {text}"),
                    format!("{text}x"),
                    format!("{},}}", &text[..text.len() - 1]),
                ];
                let sep = if members.is_empty() { "" } else { "," };
                bad.push(format!("{}{sep}\"__unknown\":0}}", &text[..text.len() - 1]));
                if let Some((name, value)) = members.first() {
                    let token =
                        canonical_practical_json_bytes(&PracticalJsonValue::from_utf16_members(
                            vec![(name.clone(), value.clone())],
                        ))
                        .unwrap();
                    bad.push(format!(
                        "{},{}",
                        &text[..text.len() - 1],
                        std::str::from_utf8(&token[1..]).unwrap()
                    ));
                }
                if members.len() > 1 {
                    let reverse = PracticalJsonValue::from_utf16_members(
                        members.iter().cloned().rev().collect(),
                    );
                    bad.push(
                        String::from_utf8(canonical_practical_json_bytes(&reverse).unwrap())
                            .unwrap(),
                    );
                }
                if let Some(required) = boundary.input_fields().iter().find(|f| f.required()) {
                    let missing = PracticalJsonValue::from_utf16_members(
                        members
                            .iter()
                            .filter(|(n, _)| n.as_slice() != required.json_name())
                            .cloned()
                            .collect(),
                    );
                    bad.push(
                        String::from_utf8(canonical_practical_json_bytes(&missing).unwrap())
                            .unwrap(),
                    );
                }
                for bad in bad {
                    assert!(
                        emitted
                            .capture_boundary_input(
                                &bundle,
                                &context,
                                &captures,
                                BoundaryInputBytes {
                                    boundary_id,
                                    provenance_id: "test.ordinary.invalid",
                                    raw_bytes: bad.as_bytes(),
                                    canonical_document: bad.as_bytes()
                                }
                            )
                            .is_err(),
                        "invalid fixture accepted: {bad}"
                    );
                    verify_packet(&cert, d, bad.as_bytes(), None, &types);
                    bad_count += 1;
                }
            }
        }
        if case == "empty-input" {
            assert_eq!(
                import(&p.canonical_bytes(), p.certificate_bytes(), &emitted).unwrap(),
                p
            );
            let mut bad: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
            bad["definitions"] = json!([]);
            assert!(import(
                &serde_json::to_vec(&bad).unwrap(),
                p.certificate_bytes(),
                &emitted
            )
            .is_err());
            assert!(import(&p.canonical_bytes(), prior.certificate_bytes(), &emitted).is_err());
        }
        let row = json!({"id":case,"source_snapshot_sha256":request["id"],"program":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),"terms":cert.term_table.len(),"declarations":cert.declarations.len(),"certificate_sha256":mpk_cert::hash_hex(&mpk_cert::certificate_hash(p.certificate_bytes()))});
        if output.is_none() {
            let pinned = read(&format!("ordinary-foundation/{family}/{case}.json"));
            assert_eq!(row, pinned, "{case}: pinned envelope metadata");
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../develop/migrations/csharp-03/ordinary-foundation")
                .join(family)
                .join(format!("{case}.hex"));
            let expected = p
                .certificate_bytes()
                .iter()
                .map(|v| format!("{v:02x}"))
                .collect::<String>();
            assert_eq!(fs::read_to_string(path).unwrap().trim(), expected);
        }
        if let Some(out) = &output {
            fs::write(
                out.join(format!("{case}.hex")),
                p.certificate_bytes()
                    .iter()
                    .map(|v| format!("{v:02x}"))
                    .collect::<String>()
                    + "\n",
            )
            .unwrap();
            fs::write(
                out.join(format!("{case}.json")),
                serde_json::to_vec_pretty(&row).unwrap(),
            )
            .unwrap();
        }
        rows.push(row);
    }
    assert_eq!(rows.len(), selected.as_ref().map_or(16, BTreeSet::len));
    if selected.is_none() {
        assert_eq!(good_count, 30);
    }
    eprintln!("Envelopes:{}source contexts,{good_count} complete original documents,{bad_count} rejected mutated documents;full output packets",rows.len());
    if let Some(out) = output {
        fs::write(
            out.join("certificates.json"),
            serde_json::to_vec_pretty(&rows).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn csharp_03_t06_w09_unguarded_envelope_metadata_preserved() {
    let bundle = b();
    let requests = read("boundary-attachment/requests.json");
    let responses = read("boundary-attachment/responses.json");
    let request = requests
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["case"] == "required")
        .unwrap();
    let response = responses
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == request["id"])
        .unwrap();
    let (context, captures) = support::replay_context(&bundle, request);
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(&response["facts"]).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    let p = generate_csharp_practical_ordinary_json_envelopes(&emitted).unwrap();
    let pinned = read("ordinary-foundation/json-envelopes/required.json");
    assert_eq!(
        serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),
        pinned["program"]
    );
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/json-envelopes/required.hex");
    assert_eq!(
        fs::read_to_string(path).unwrap().trim(),
        p.certificate_bytes()
            .iter()
            .map(|v| format!("{v:02x}"))
            .collect::<String>()
    );
    assert!(p.typed_nodes().is_empty());
    let depth = generate_csharp_practical_ordinary_json_depth_guarded_envelopes(&emitted).unwrap();
    assert!(depth.typed_nodes().is_empty());
    assert_eq!(
        serde_json::from_slice::<Value>(&depth.canonical_bytes()).unwrap(),
        read("ordinary-foundation/json-depth-guarded-envelopes/required.json")["program"]
    );
    let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/csharp-03/ordinary-foundation/json-depth-guarded-envelopes/required.hex");
    assert_eq!(
        fs::read_to_string(path).unwrap().trim(),
        depth
            .certificate_bytes()
            .iter()
            .map(|v| format!("{v:02x}"))
            .collect::<String>()
    );
    assert!(p.typed_depth().is_empty());
    assert!(p
        .definitions()
        .iter()
        .all(|d| d.unguarded_parse_definition.is_none()));
}

#[test]
fn csharp_03_t06_w09_json_typed_guarded_compound_definition_closures() {
    compound_definition_closures(false);
}
#[test]
fn csharp_03_t06_w09_json_limits_guarded_compound_definition_closures() {
    compound_definition_closures(true);
}
fn compound_definition_closures(raw_guard: bool) {
    let bundle = b();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation");
    let decode = |path: std::path::PathBuf| {
        let h = fs::read_to_string(path).unwrap();
        let h = h.trim();
        let raw = (0..h.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        mpk_cert::decode_canonical_certificate(&raw).unwrap()
    };
    let out = std::env::var_os(if raw_guard {
        "MPK_W09_JSON_LIMITS_GUARDED_COMPOUND_OUT"
    } else {
        "MPK_W09_JSON_TYPED_GUARDED_COMPOUND_OUT"
    })
    .map(std::path::PathBuf::from);
    let family_out = if raw_guard {
        "json-limits-guarded-envelopes"
    } else {
        "json-typed-guarded-envelopes"
    };
    if let Some(out) = &out {
        fs::create_dir_all(out).unwrap();
    }
    let mut rows = vec![];
    for family in ["json-depth-compound", "json-semantic-root"] {
        for input in read(&format!("ordinary-foundation/{family}/certificates.json"))
            .as_array()
            .unwrap()
        {
            let id = input["id"].as_str().unwrap();
            let case = format!("{family}-{id}");
            let source_family = if family == "json-semantic-root" {
                "json-semantic-root-sources".into()
            } else {
                format!("json-{}-sources", input["source_family"].as_str().unwrap())
            };
            let request_id = if family == "json-semantic-root" {
                &input["id"]
            } else {
                &input["source_request_id"]
            };
            let requests = read(&format!(
                "ordinary-foundation/{source_family}/requests.json"
            ));
            let responses = read(&format!(
                "ordinary-foundation/{source_family}/responses.json"
            ));
            let request = requests
                .as_array()
                .unwrap()
                .iter()
                .find(|r| &r["id"] == request_id)
                .unwrap();
            let response = responses
                .as_array()
                .unwrap()
                .iter()
                .find(|r| &r["id"] == request_id)
                .unwrap();
            let (context, captures) = support::replay_context(&bundle, request);
            let source = ValidatedDataSource::import_captured_facts(
                &bundle,
                &context,
                &captures,
                &serde_json::to_vec(&response["facts"]).unwrap(),
            )
            .unwrap();
            let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
            let p = if raw_guard {
                generate_csharp_practical_ordinary_json_limits_guarded_envelopes(&emitted).unwrap()
            } else {
                generate_csharp_practical_ordinary_json_typed_guarded_envelopes(&emitted).unwrap()
            };
            if raw_guard {
                check_raw_guard_closures(&emitted, &p);
            }
            let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
            validate_csharp_practical_certificate_structure(&cert).unwrap();
            for old in [
                decode(root.join(family).join(format!("{id}.hex"))),
                decode(root.join("json-typed-nodes").join(format!("{case}.hex"))),
            ] {
                let names = old
                    .declarations
                    .iter()
                    .map(|d| old.name_table[d.name as usize].clone())
                    .collect();
                structural_equivalence_tests::same_definition_closure(&old, &cert, &names).unwrap();
            }
            let nodes = read(&format!("ordinary-foundation/json-typed-nodes/{case}.json"));
            assert_eq!(
                serde_json::to_value(p.typed_nodes()).unwrap(),
                nodes["program"]["definitions"]
            );
            assert_eq!(
                serde_json::to_value(p.typed_depth()).unwrap(),
                input["program"]["typed_depth"]
            );
            assert_eq!(p.definitions().len(), 1);
            assert_eq!(
                p.definitions()[0].parse_definition,
                format!(
                    "{}.TypedNodes{}",
                    input["program"]["definitions"][0]["parse_definition"]
                        .as_str()
                        .unwrap(),
                    if raw_guard { ".RawLimits" } else { "" }
                )
            );
            let row = json!({"id":case,"source_family":source_family,"source_request_id":request_id,"prior_runtime_metadata":format!("{family}/{id}.json"),"prior_node_count_metadata":format!("json-typed-nodes/{case}.json"),"runtime_repeated":false,"program":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),"terms":cert.term_table.len(),"declarations":cert.declarations.len(),"certificate_sha256":mpk_cert::hash_hex(&mpk_cert::certificate_hash(p.certificate_bytes()))});
            if let Some(out) = &out {
                fs::write(
                    out.join(format!("{case}.hex")),
                    p.certificate_bytes()
                        .iter()
                        .map(|v| format!("{v:02x}"))
                        .collect::<String>()
                        + "\n",
                )
                .unwrap();
                fs::write(
                    out.join(format!("{case}.json")),
                    serde_json::to_vec_pretty(&row).unwrap(),
                )
                .unwrap();
            } else {
                assert_eq!(
                    row,
                    read(&format!("ordinary-foundation/{family_out}/{case}.json"))
                );
                assert_eq!(
                    fs::read_to_string(root.join(family_out).join(format!("{case}.hex")))
                        .unwrap()
                        .trim(),
                    p.certificate_bytes()
                        .iter()
                        .map(|v| format!("{v:02x}"))
                        .collect::<String>()
                );
            }
            rows.push(row);
        }
    }
    assert_eq!(rows.len(), 16);
    if let Some(out) = &out {
        fs::write(
            out.join("certificates.json"),
            serde_json::to_vec_pretty(&rows).unwrap(),
        )
        .unwrap();
    }
    eprintln!("Guarded compound definitions:16 complete predecessor parser/count closures preserved;raw guard={raw_guard};no parser runtime repeated");
}

#[test]
fn csharp_03_t06_w09_json_limits_guarded_import_rejects_linkage_mutations() {
    let bundle = b();
    let requests = read("boundary-attachment/requests.json");
    let responses = read("boundary-attachment/responses.json");
    let request = requests
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["case"] == "required")
        .unwrap();
    let response = responses
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == request["id"])
        .unwrap();
    let (context, captures) = support::replay_context(&bundle, request);
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(&response["facts"]).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    let p = generate_csharp_practical_ordinary_json_limits_guarded_envelopes(&emitted).unwrap();
    let metadata: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
    assert_eq!(
        import_csharp_practical_ordinary_json_limits_guarded_envelopes(
            &p.canonical_bytes(),
            p.certificate_bytes(),
            &emitted,
        )
        .unwrap(),
        p
    );
    for pointer in [
        "/schema",
        "/raw_limits/valid_definition",
        "/raw_limits/document/length_definition",
        "/raw_limits/scan_steps",
        "/definitions/0/parse_definition",
        "/parsers/source_ir_sha256",
        "/parsers/foundation_sha256",
    ] {
        let mut changed = metadata.clone();
        *changed.pointer_mut(pointer).unwrap() = json!("forged");
        assert!(
            import_csharp_practical_ordinary_json_limits_guarded_envelopes(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                &emitted,
            )
            .is_err(),
            "metadata mutation {pointer}"
        );
    }
    let mut changed = metadata;
    changed.as_object_mut().unwrap().remove("raw_limits");
    assert!(
        import_csharp_practical_ordinary_json_limits_guarded_envelopes(
            &serde_json::to_vec(&changed).unwrap(),
            p.certificate_bytes(),
            &emitted,
        )
        .is_err()
    );
    let prior = generate_csharp_practical_ordinary_json_typed_guarded_envelopes(&emitted).unwrap();
    assert!(prior.raw_limits().is_none());
    assert!(
        import_csharp_practical_ordinary_json_limits_guarded_envelopes(
            &p.canonical_bytes(),
            prior.certificate_bytes(),
            &emitted,
        )
        .is_err()
    );
    let mut bytes = p.certificate_bytes().to_vec();
    *bytes.last_mut().unwrap() ^= 1;
    assert!(
        import_csharp_practical_ordinary_json_limits_guarded_envelopes(
            &p.canonical_bytes(),
            &bytes,
            &emitted,
        )
        .is_err()
    );
}
