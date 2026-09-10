//! Canonical typed trees from the independent input capture are the depth oracle.
use super::super::super::super::relation_tests;
use super::*;
fn height(v: &PracticalJsonValue) -> u32 {
    if let Some(xs) = v.as_array() {
        xs.iter().map(|x| height(x) + 1).max().unwrap_or(0)
    } else if let Some(xs) = v.utf16_members() {
        xs.iter().map(|(_, x)| height(x) + 1).max().unwrap_or(0)
    } else {
        0
    }
}
#[test]
fn csharp_03_t06_w09_json_typed_depth_original_inputs() {
    let bundle = b();
    let requests = read("boundary-attachment/requests.json");
    let responses = read("boundary-attachment/responses.json");
    let accepted = read("boundary-vc/goldens.json");
    let conformance = read("boundary-input/conformance.json");
    let out = std::env::var_os("MPK_W09_JSON_TYPED_DEPTH_OUT").map(std::path::PathBuf::from);
    if let Some(out) = &out {
        fs::create_dir_all(out).unwrap();
    }
    let mut rows = vec![];
    let mut observed = 0;
    for entry in accepted.as_array().unwrap() {
        let request = requests
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == entry["id"])
            .unwrap();
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == entry["id"])
            .unwrap();
        let case = request["case"].as_str().unwrap();
        eprintln!("Typed JSON depth source:{case}");
        let (context, captures) = support::replay_context(&bundle, request);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&response["facts"]).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_json_typed_depth(&emitted).unwrap();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let layouts = generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap();
        let types = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        let boundary = &emitted.boundaries()[0];
        let id = boundary
            .artifact()
            .value()
            .get("boundary_id")
            .unwrap()
            .as_str()
            .unwrap();
        for doc in conformance["runs"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["source_snapshot_sha256"] == request["id"])
        {
            let text = doc["document_utf8"].as_str().unwrap();
            let run_capture = emitted
                .capture_boundary_input(
                    &bundle,
                    &context,
                    &captures,
                    BoundaryInputBytes {
                        boundary_id: id,
                        provenance_id: "test.typed.depth",
                        raw_bytes: text.as_bytes(),
                        canonical_document: text.as_bytes(),
                    },
                )
                .unwrap();
            let canonical = run_capture
                .capture()
                .artifact()
                .value()
                .get("canonical_value")
                .unwrap()
                .utf16_members()
                .unwrap();
            assert_eq!(canonical.len(), run_capture.arguments().len());
            for ((_, tree), arg) in canonical.iter().zip(run_capture.arguments()) {
                let d = p
                    .definitions()
                    .iter()
                    .find(|d| d.carrier.type_id == arg.value().type_id())
                    .unwrap();
                let h = height(tree);
                let stored = relation_tests::storage(arg.value(), &types);
                let value = if d.carrier.depth == 0 {
                    V::Bit(stored[0])
                } else {
                    V::Cube(stored)
                };
                for root_depth in BTreeSet::from([0, 32 - h, 33 - h, 32, 33, u32::MAX]) {
                    let root = V::Cube((0..32).map(|i| root_depth & (1 << i) != 0).collect());
                    let result = run(&cert, &d.definition, vec![value.clone(), root]);
                    assert_eq!(
                        leaf(&cert, result, 0, 0),
                        root_depth.checked_add(h).is_some_and(|n| n <= 32),
                        "{case}: {} at{root_depth},height{h}",
                        arg.field_id()
                    );
                    observed += 1;
                }
            }
        }
        if case == "nullable_default" {
            assert_eq!(
                import_csharp_practical_ordinary_json_typed_depth(
                    &p.canonical_bytes(),
                    p.certificate_bytes(),
                    &emitted
                )
                .unwrap(),
                p
            );
            let mut bad: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
            bad["definitions"] = json!([]);
            assert!(import_csharp_practical_ordinary_json_typed_depth(
                &serde_json::to_vec(&bad).unwrap(),
                p.certificate_bytes(),
                &emitted
            )
            .is_err());
            let mut bad = p.certificate_bytes().to_vec();
            let n = bad.len();
            bad[n - 1] ^= 1;
            assert!(import_csharp_practical_ordinary_json_typed_depth(
                &p.canonical_bytes(),
                &bad,
                &emitted
            )
            .is_err());
        }
        let row = json!({"id":case,"source_snapshot_sha256":request["id"],"program":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),"terms":cert.term_table.len(),"declarations":cert.declarations.len(),"certificate_sha256":mpk_cert::hash_hex(&mpk_cert::certificate_hash(p.certificate_bytes()))});
        if out.is_none() {
            assert_eq!(
                row,
                read(&format!("ordinary-foundation/json-typed-depth/{case}.json"))
            );
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../develop/migrations/csharp-03/ordinary-foundation/json-typed-depth")
                .join(format!("{case}.hex"));
            assert_eq!(
                fs::read_to_string(path).unwrap().trim(),
                p.certificate_bytes()
                    .iter()
                    .map(|v| format!("{v:02x}"))
                    .collect::<String>()
            );
        }
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
        }
        rows.push(row);
    }
    assert_eq!(rows.len(), 15);
    eprintln!("Typed JSON depth:15 original source contexts,{observed} core depth observations");
    if let Some(out) = out {
        fs::write(
            out.join("certificates.json"),
            serde_json::to_vec_pretty(&rows).unwrap(),
        )
        .unwrap();
    }
}
