use super::*;
use core_eval::{bit, run, sparse_cube};

#[test]
fn csharp_03_t06_w09_source_clauses_original_construction_invariants() {
    let bundle = b();
    let requests = read("construction-vc/requests.json");
    let responses = read("construction-vc/responses.json");
    let expected = read("construction-vc/goldens.json");
    let output = std::env::var_os("MPK_W09_SOURCE_CLAUSES_OUT").map(std::path::PathBuf::from);
    if let Some(p) = &output {
        fs::create_dir_all(p).unwrap();
    }
    let mut rows = vec![];
    let mut observations = 0;
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    for input in expected.as_array().unwrap() {
        let id = input["id"].as_str().unwrap();
        let request = requests
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == input["id"])
            .unwrap();
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == input["id"])
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
        let p = generate_csharp_practical_ordinary_source_clauses(emitted.vir())
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_source_clauses(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                emitted.vir()
            )
            .unwrap(),
            p
        );
        if let Some((metadata, bytes)) = &previous {
            assert!(import_csharp_practical_ordinary_source_clauses(
                metadata,
                bytes,
                emitted.vir()
            )
            .is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        assert_eq!(p.definitions().len(), if id == "enum_zero" { 0 } else { 1 });
        for d in p.definitions() {
            let invariant = &input["construction"]["types"][0];
            assert_eq!(d.source_type_id, invariant["type_id"].as_str().unwrap());
            assert_eq!(
                d.expression_sha256,
                invariant["public_clauses"][0]["expression_sha256"]
                    .as_str()
                    .unwrap()
            );
            // Original initializer sources also retain a string member before
            // the tested integer. Derive its address from captured member order
            // and the carrier,not from the generated projection definition.
            let members = invariant["members"].as_array().unwrap();
            let integer_members = members
                .iter()
                .enumerate()
                .filter(|(_, m)| m["type_id"] == "mpk.csharp.value.i32.v1")
                .map(|(i, _)| i)
                .collect::<Vec<_>>();
            assert_eq!(integer_members.len(), 1);
            let member_index = integer_members[0];
            let layouts = generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap();
            let depth = layouts
                .carriers()
                .iter()
                .find(|c| c.type_id == d.source_type_id)
                .unwrap()
                .depth;
            for n in [i32::MIN, -1, 0, 1, i32::MAX] {
                let bits = (0..32)
                    .filter(|i| (n as u32) & (1 << i) != 0)
                    .map(|i| member_index | (i << (depth - 5)))
                    .collect();
                let actual = bit(run(&cert, &d.definition, vec![sparse_cube(depth, bits)]));
                assert_eq!(
                    actual,
                    if id == "positive_default" {
                        n >= 0
                    } else {
                        n > 0
                    },
                    "{id}: {n}"
                );
                observations += 1;
            }
        }
        let mut meta: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        meta["construction_sha256"] = json!("forged");
        assert!(import_csharp_practical_ordinary_source_clauses(
            &serde_json::to_vec(&meta).unwrap(),
            p.certificate_bytes(),
            emitted.vir()
        )
        .is_err());
        let mut bytes = p.certificate_bytes().to_vec();
        *bytes.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_source_clauses(
            &p.canonical_bytes(),
            &bytes,
            emitted.vir()
        )
        .is_err());
        let row = json!({"id":id,"program":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),"terms":cert.term_table.len(),"declarations":cert.declarations.len()});
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|v| format!("{v:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(out) = &output {
            fs::write(out.join(format!("{id}.hex")), &hex).unwrap();
        } else {
            let root = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../develop/migrations/csharp-03/ordinary-foundation/source-clauses");
            assert_eq!(
                fs::read_to_string(root.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        rows.push(row);
    }
    assert_eq!(observations, 30);
    assert_eq!(rows.len(), 7);
    if let Some(out) = &output {
        fs::write(
            out.join("certificates.json"),
            serde_json::to_vec_pretty(&rows).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/source-clauses/certificates.json"),
            json!(rows)
        );
    }
    eprintln!("Source clauses:7 original construction sources,30 signed boundary observations;not construction/publication proofs");
}
