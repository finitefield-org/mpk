//! Decode original captured boundary field bytes and compare complete carriers.
use super::super::super::super::{relation_tests, structural_equivalence_tests};
use super::*;

pub(super) fn cells(value: &MonomorphicValue) -> u32 {
    1 + match value {
        MonomorphicValue::Option { value, .. }
        | MonomorphicValue::BoundaryPresence { value, .. } => value.as_deref().map_or(0, cells),
        MonomorphicValue::String { utf16, .. } => utf16.len() as u32,
        MonomorphicValue::Product { fields, .. } => fields.iter().map(|f| cells(&f.value)).sum(),
        MonomorphicValue::Array { elements, .. }
        | MonomorphicValue::Sequence { elements, .. }
        | MonomorphicValue::OrderedSet { elements, .. } => elements.iter().map(cells).sum(),
        MonomorphicValue::OrderedMap { entries, .. } => entries
            .iter()
            .map(|e| cells(&e.key) + cells(&e.value))
            .sum(),
        MonomorphicValue::OrderedEntry { key, value, .. } => cells(key) + cells(value),
        MonomorphicValue::Money {
            amount, currency, ..
        } => cells(amount) + cells(currency),
        MonomorphicValue::TaggedSum { payload, .. } => payload.iter().map(cells).sum(),
        MonomorphicValue::Transition {
            state,
            events,
            response,
            ..
        } => cells(state) + events.iter().map(cells).sum::<u32>() + cells(response),
        MonomorphicValue::Signed { .. }
        | MonomorphicValue::Unsigned { .. }
        | MonomorphicValue::DecimalBits { .. }
        | MonomorphicValue::Enum { .. }
        | MonomorphicValue::Instant { .. }
        | MonomorphicValue::Bool { .. } => 0,
        _ => panic!("add an independent cell oracle for {value:?}"),
    }
}
#[allow(clippy::too_many_arguments)]
fn check(
    cert: &mpk_cert::encode::Certificate,
    d: &OrdinaryJsonBoundaryFieldDefinition,
    text: &[u8],
    start: u32,
    ending: u8,
    depth: u32,
    expected: Option<&MonomorphicValue>,
    types: &BTreeMap<String, OrdinaryCarrier>,
) {
    let parsed = run(
        cert,
        &d.parse_definition,
        vec![
            document(
                text.len() as u32,
                &text.iter().copied().enumerate().collect::<Vec<_>>(),
            ),
            word(start),
            word_tag(ending),
            word(depth),
        ],
    );
    let mut wanted = vec![false; 1 << d.packet_depth];
    if let Some(value) = expected {
        wanted[0] = true;
        wanted[2] = ending == 0;
        let end = text.len() - usize::from(ending != 0);
        let count = cells(value);
        for i in 0..32 {
            wanted[(2 + i) << 1] = end & (1 << i) != 0;
            wanted[(34 + i) << 1] = count & (1 << i) != 0;
        }
        for (i, bit) in relation_tests::storage(value, types)
            .into_iter()
            .enumerate()
        {
            wanted[1 | (i << 1)] = bit;
        }
    }
    for (i, want) in wanted.into_iter().enumerate() {
        assert_eq!(
            leaf(cert, parsed.clone(), d.packet_depth as usize, i),
            want,
            "{}:{} bit{i} input{:?} depth{depth}",
            d.contract_sha256,
            d.field_id,
            String::from_utf8_lossy(text)
        );
    }
}
#[test]
fn csharp_03_t06_w09_json_boundary_fields_original_sources() {
    let bundle = b();
    let requests = read("boundary-attachment/requests.json");
    let responses = read("boundary-attachment/responses.json");
    let accepted = read("boundary-vc/goldens.json");
    let conformance = read("boundary-input/conformance.json");
    let output = std::env::var_os("MPK_W09_JSON_BOUNDARY_FIELDS_OUT").map(std::path::PathBuf::from);
    if let Some(out) = &output {
        fs::create_dir_all(out).unwrap();
    }
    let selected = std::env::var("MPK_W09_JSON_BOUNDARY_FIELDS_SELECT").ok();
    let selected = selected.map(|s| s.split(',').map(str::to_owned).collect::<BTreeSet<_>>());
    let reuse = std::env::var_os("MPK_W09_JSON_BOUNDARY_FIELDS_REUSE_VERIFIED_CORE")
        .map(std::path::PathBuf::from);
    assert!(
        reuse.is_none() || selected.is_some(),
        "prior-core reuse requires an explicit source subset"
    );
    let mut reused_contexts = 0;
    let mut rows = vec![];
    let mut documents = 0;
    let mut cases = 0;
    for accepted in accepted.as_array().unwrap() {
        let id = accepted["id"].as_str().unwrap();
        let request = requests
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        let case = request["case"].as_str().unwrap();
        if selected
            .as_ref()
            .is_some_and(|wanted| !wanted.contains(case))
        {
            continue;
        }
        eprintln!("Boundary field source: {case}");
        let (context, captures) = support::replay_context(&bundle, request);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&response["facts"]).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_json_boundary_fields(&emitted)
            .unwrap_or_else(|e| panic!("{case}: {e:?}"));
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let old = generate_csharp_practical_ordinary_json_products(emitted.vir()).unwrap();
        let old_cert = mpk_cert::decode_canonical_certificate(old.certificate_bytes()).unwrap();
        let names = old_cert
            .declarations
            .iter()
            .map(|d| old_cert.name_table[d.name as usize].clone())
            .collect();
        structural_equivalence_tests::same_definition_closure(&old_cert, &cert, &names).unwrap();
        let layouts = generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap();
        let types = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        let boundary = &emitted.boundaries()[0];
        assert_eq!(p.fields().len(), boundary.input_fields().len());
        let boundary_id = boundary
            .artifact()
            .value()
            .get("boundary_id")
            .unwrap()
            .as_str()
            .unwrap();
        if let Some(prior) = &reuse {
            let previous = fs::read_to_string(prior.join(format!("{case}.hex"))).unwrap();
            let bytes = (0..previous.trim().len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&previous[i..i + 2], 16).unwrap())
                .collect::<Vec<_>>();
            assert_eq!(
                bytes,
                p.certificate_bytes(),
                "{case}: cannot reuse observations from different certificate bytes"
            );
            eprintln!("Boundary field source {case}: exact prior certificate bytes; core observations reused, not rerun");
            reused_contexts += 1;
        } else {
            for fixture in conformance["runs"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|r| r["source_snapshot_sha256"] == id)
            {
                let text = fixture["document_utf8"].as_str().unwrap().as_bytes();
                let run = emitted
                    .capture_boundary_input(
                        &bundle,
                        &context,
                        &captures,
                        BoundaryInputBytes {
                            boundary_id,
                            provenance_id: "test.ordinary.fields",
                            raw_bytes: text,
                            canonical_document: text,
                        },
                    )
                    .unwrap();
                let document =
                    parse_canonical_practical_json(PracticalArtifactKind::BoundaryInput, text)
                        .unwrap();
                let members = document.utf16_members().unwrap();
                for ((f, d), argument) in boundary
                    .input_fields()
                    .iter()
                    .zip(p.fields())
                    .zip(run.arguments())
                {
                    let expected = argument.value();
                    if let Some((_, raw)) = members
                        .iter()
                        .find(|(name, _)| name.as_slice() == f.json_name())
                    {
                        let token = canonical_practical_json_bytes(raw).unwrap();
                        check(&cert, d, &token, 0, 0, 1, Some(expected), &types);
                        let mut wrapped = vec![b'@'];
                        wrapped.extend(&token);
                        wrapped.push(b',');
                        check(&cert, d, &wrapped, 1, 1, 32, Some(expected), &types);
                        check(&cert, d, &token, 0, 0, 33, None, &types);
                        cases += 3;
                    } else {
                        let name = d
                            .missing_value_definition
                            .as_ref()
                            .expect("admitted omission must have a value");
                        assert_eq!(d.missing_cells, Some(u64::from(cells(expected))));
                        let actual = run_core(&cert, name);
                        for (i, want) in relation_tests::storage(expected, &types)
                            .into_iter()
                            .enumerate()
                        {
                            assert_eq!(
                                leaf(&cert, actual.clone(), d.semantic_carrier.depth as usize, i),
                                want,
                                "{case}: missing bit{i}"
                            );
                        }
                        cases += 1;
                    }
                }
                documents += 1;
            }
            for d in p.fields() {
                check(&cert, d, b"?", 0, 0, 1, None, &types);
                cases += 1;
                if !d.nullable {
                    check(&cert, d, b"null", 0, 0, 1, None, &types);
                    cases += 1;
                }
            }
        }
        // One exact import and linked metadata/certificate mutation set; all
        // contexts above independently reconstruct their source and definitions.
        if case == "nullable_default" {
            assert_eq!(
                import_csharp_practical_ordinary_json_boundary_fields(
                    &p.canonical_bytes(),
                    p.certificate_bytes(),
                    &emitted
                )
                .unwrap(),
                p
            );
            for key in ["fields", "values", "schema"] {
                let mut bad: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
                bad[key] = json!("changed");
                assert!(import_csharp_practical_ordinary_json_boundary_fields(
                    &serde_json::to_vec(&bad).unwrap(),
                    p.certificate_bytes(),
                    &emitted
                )
                .is_err());
            }
            assert!(import_csharp_practical_ordinary_json_boundary_fields(
                &p.canonical_bytes(),
                old.certificate_bytes(),
                &emitted
            )
            .is_err());
        }
        let row = json!({"id":case,"source_snapshot_sha256":id,"program":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),"certificate_sha256":mpk_cert::hash_hex(&mpk_cert::certificate_hash(p.certificate_bytes())),"terms":cert.term_table.len(),"declarations":cert.declarations.len()});
        if let Some(out) = &output {
            fs::write(
                out.join(format!("{case}.hex")),
                p.certificate_bytes()
                    .iter()
                    .map(|v| format!("{v:02x}"))
                    .collect::<String>(),
            )
            .unwrap();
        }
        if let Some(out) = &output {
            fs::write(
                out.join(format!("{case}.json")),
                serde_json::to_vec_pretty(&row).unwrap(),
            )
            .unwrap();
        } else {
            let pins = Path::new(env!("CARGO_MANIFEST_DIR")).join(
                "../../develop/migrations/csharp-03/ordinary-foundation/json-boundary-fields",
            );
            let hex = fs::read_to_string(pins.join(format!("{case}.hex"))).unwrap();
            assert_eq!(
                hex.trim(),
                p.certificate_bytes()
                    .iter()
                    .map(|v| format!("{v:02x}"))
                    .collect::<String>()
            );
            let expected: Value =
                serde_json::from_slice(&fs::read(pins.join(format!("{case}.json"))).unwrap())
                    .unwrap();
            assert_eq!(row, expected);
        }
        rows.push(row);
    }
    if let Some(selected) = selected {
        assert_eq!(rows.len(), selected.len());
        assert!(documents > 0 || reused_contexts == rows.len());
    } else {
        assert_eq!(rows.len(), 15);
        assert_eq!(documents, 29);
        assert_eq!(reused_contexts, 0);
    }
    eprintln!("Boundary fields: {} source contexts,{documents} original documents,{cases} complete carrier/packet cases,{reused_contexts} contexts reused by exact bytes", rows.len());
    if let Some(out) = output {
        fs::write(
            out.join("certificates.json"),
            serde_json::to_vec_pretty(&rows).unwrap(),
        )
        .unwrap();
    }
}
fn run_core(cert: &mpk_cert::encode::Certificate, name: &str) -> V {
    run(cert, name, vec![])
}
