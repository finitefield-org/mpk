//! Combined source parsing and active typed-depth checks for compound inputs.
use super::*;
pub(super) fn height(v: &PracticalJsonValue) -> u32 {
    if let Some(xs) = v.as_array() {
        xs.iter().map(|v| height(v) + 1).max().unwrap_or(0)
    } else if let Some(xs) = v.utf16_members() {
        xs.iter().map(|(_, v)| height(v) + 1).max().unwrap_or(0)
    } else {
        0
    }
}
#[test]
fn csharp_03_t06_w09_json_depth_guard_compound_sources() {
    let cases = [
        ("product", "nested", r#"{"Item":{"Number":-2}}"#),
        ("product", "empty", r#"{}"#),
        (
            "sequence",
            "source-arrays",
            r#"{"Flags":[true,false],"Numbers":[-2,7],"Nested":[{"Values":[3]}]}"#,
        ),
        (
            "semantic-product",
            "money-entry",
            r#"{"EnumValue":{"Amount":"1.25","Currency":"7"},"TextValue":{"Amount":"-2","Currency":"JPY"},"Entry":{"Key":-2,"Value":true}}"#,
        ),
        (
            "transition",
            "scalar-map",
            r#"{"Outcome":{"State":2,"Events":[3],"Response":true},"Map":{"Items":[{"Key":4,"Value":true}]}}"#,
        ),
        (
            "transition",
            "compound",
            r#"{"Outcome":{"State":{"Name":"A","Version":1},"Events":[{"Code":2}],"Response":{"tag":"some","payload":true}}}"#,
        ),
        (
            "ordered",
            "integer",
            r#"{"Map":{"Items":[{"Key":1,"Value":true}]},"Set":{"Items":[2]}}"#,
        ),
        (
            "ordered",
            "decimal",
            r#"{"Map":{"Items":[{"Key":"1.25","Value":true}]},"Set":{"Items":["2"]}}"#,
        ),
        (
            "ordered",
            "string",
            r#"{"Map":{"Items":[{"Key":"A","Value":true}]},"Set":{"Items":["B"]}}"#,
        ),
        (
            "ordered",
            "compound",
            r#"{"Map":{"Items":[{"Key":{"Name":"A","Code":1},"Value":true}]},"Set":{"Items":[{"Name":"B","Code":2}]}}"#,
        ),
        (
            "sum",
            "all-sums",
            r#"{"Option":{"Tag":"1","Value":2},"Lookup":{"Tag":"0","Value":0},"Result":{"Tag":"1","Value":0,"Error":true},"Validation":{"Tag":"1","Value":0,"Errors":[3]},"Field":{"Tag":"2","Value":4},"Nested":{"Tag":"0","Value":{"tag":"some","payload":5},"Error":false}}"#,
        ),
    ];
    let selected = std::env::var("MPK_W09_JSON_DEPTH_COMPOUND_SELECT")
        .ok()
        .map(|v| v.split(',').map(str::to_owned).collect::<BTreeSet<_>>());
    let out = std::env::var_os("MPK_W09_JSON_DEPTH_COMPOUND_OUT").map(std::path::PathBuf::from);
    if let Some(out) = &out {
        fs::create_dir_all(out).unwrap();
    }
    let bundle = b();
    let mut rows = vec![];
    for (family, id, payload) in cases {
        let case = format!("{family}-{id}");
        if selected.as_ref().is_some_and(|s| !s.contains(&case)) {
            continue;
        }
        eprintln!("Compound depth envelope:{case}");
        let requests = read(&format!(
            "ordinary-foundation/json-{family}-sources/requests.json"
        ));
        let responses = read(&format!(
            "ordinary-foundation/json-{family}-sources/responses.json"
        ));
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
        let (context, captures) = support::replay_context(&bundle, request);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&response["facts"]).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_json_depth_guarded_envelopes(&emitted)
            .unwrap_or_else(|e| panic!("{case}: {e:?}"));
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let boundary = &emitted.boundaries()[0];
        assert_eq!(boundary.input_fields().len(), 1);
        assert_eq!(
            boundary.input_fields()[0].json_name(),
            "field0".encode_utf16().collect::<Vec<_>>()
        );
        let boundary_id = boundary
            .artifact()
            .value()
            .get("boundary_id")
            .unwrap()
            .as_str()
            .unwrap();
        let text = format!("{{\"field0\":{payload}}}");
        let captured = emitted
            .capture_boundary_input(
                &bundle,
                &context,
                &captures,
                BoundaryInputBytes {
                    boundary_id,
                    provenance_id: "test.compound.depth",
                    raw_bytes: text.as_bytes(),
                    canonical_document: text.as_bytes(),
                },
            )
            .unwrap_or_else(|e| panic!("{case} capture:{e:?}"));
        let d = &p.definitions()[0];
        let packet = run(
            &cert,
            &d.parse_definition,
            vec![document(
                text.len() as u32,
                &text.bytes().enumerate().collect::<Vec<_>>(),
            )],
        );
        let header = run(&cert, &d.header_definition, vec![packet.clone()]);
        let count = 1 + captured
            .arguments()
            .iter()
            .map(|a| super::boundary_field_tests::cells(a.value()))
            .sum::<u32>();
        for bit in 0..128 {
            let want = match bit {
                0 | 1 => true,
                2..=33 => text.len() & (1 << (bit - 2)) != 0,
                34..=65 => count & (1 << (bit - 34)) != 0,
                _ => false,
            };
            assert_eq!(
                leaf(&cert, header.clone(), 7, bit),
                want,
                "{case} header bit{bit}"
            );
        }
        let arguments = run(&cert, &d.value_definition, vec![packet]);
        let argument = run(
            &cert,
            &d.fields[0].argument_projection_definition,
            vec![arguments],
        );
        let tree = captured
            .capture()
            .artifact()
            .value()
            .get("canonical_value")
            .unwrap()
            .get("field0")
            .unwrap();
        let h = height(tree);
        let depth = p
            .typed_depth()
            .iter()
            .find(|v| v.carrier.type_id == captured.arguments()[0].value().type_id())
            .unwrap();
        for root in BTreeSet::from([0, 1, 32 - h, 33 - h, 32, 33, u32::MAX]) {
            let result = run(&cert, &depth.definition, vec![argument.clone(), word(root)]);
            assert_eq!(
                leaf(&cert, result, 0, 0),
                root.checked_add(h).is_some_and(|v| v <= 32),
                "{case} root{root},height{h}"
            );
        }
        let row = json!({"id":case,"source_request_id":id,"source_family":family,"document_utf8":text,"canonical_field_height":h,"program":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),"terms":cert.term_table.len(),"declarations":cert.declarations.len(),"certificate_sha256":mpk_cert::hash_hex(&mpk_cert::certificate_hash(p.certificate_bytes()))});
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
        if out.is_none() {
            assert_eq!(
                row,
                read(&format!(
                    "ordinary-foundation/json-depth-compound/{case}.json"
                )),
                "{case}: pinned metadata"
            );
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../develop/migrations/csharp-03/ordinary-foundation/json-depth-compound")
                .join(format!("{case}.hex"));
            let expected = p
                .certificate_bytes()
                .iter()
                .map(|v| format!("{v:02x}"))
                .collect::<String>();
            assert_eq!(
                fs::read_to_string(path).unwrap().trim(),
                expected,
                "{case}: pinned certificate"
            );
        }
        rows.push(row);
    }
    assert_eq!(
        rows.len(),
        selected.as_ref().map_or(cases.len(), BTreeSet::len)
    );
    eprintln!("Compound depth envelopes:{} actual source contexts;complete headers and canonical-tree depth boundaries",rows.len());
    if let Some(out) = out {
        fs::write(
            out.join("certificates.json"),
            serde_json::to_vec_pretty(&rows).unwrap(),
        )
        .unwrap();
    }
}
