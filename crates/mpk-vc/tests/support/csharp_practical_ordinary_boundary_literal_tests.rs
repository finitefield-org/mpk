//! Actual-source boundary captures; returned values remain pending native results.
use super::*;
use core_eval::{bit as observed_bit, run, V};

fn wide_document() -> String {
    let fields = (0..32)
        .map(|i| {
            format!(
                "\"F{i}\":{}",
                if i < 16 {
                    r#"["-1","9223372036854775807"]"#
                } else {
                    "\"x\""
                }
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{{\"field0\":{{{fields}}}}}")
}
#[test]
fn csharp_03_t06_w09_boundary_literals_source_runs_and_mutations() {
    let bundle = b();
    let requests = read("boundary-output/source-requests.json");
    let responses = read("boundary-output/source-responses.json");
    let output = std::env::var_os("MPK_W09_BOUNDARY_LITERALS_OUT").map(std::path::PathBuf::from);
    if let Some(dir) = &output {
        fs::create_dir_all(dir).unwrap();
    }
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/boundary-literals");
    let mut rows = vec![];
    let mut previous_source_run = None;
    let mut total_observations = 0;
    for request in requests.as_array().unwrap() {
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == request["id"])
            .unwrap();
        assert!(response.get("reject").is_none());
        let (context, captures) = support::replay_context(&bundle, request);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&response["facts"]).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let vir = emitted.vir();
        let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
        let types = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        let boundary = &emitted.boundaries()[0];
        let boundary_id = boundary
            .artifact()
            .value()
            .get("boundary_id")
            .unwrap()
            .as_str()
            .unwrap();
        let unit = boundary.input_fields().is_empty();
        let wide =
            request["id"] == "decb5507362a7672c711a5befd63efc1038c2c0a86ee20137a1c4c472e0d1684";
        let source_name = if unit {
            "unit"
        } else if wide {
            "wide"
        } else {
            "record"
        };
        let variants: &[&str] = if unit {
            &["base", "provenance"]
        } else if wide {
            &["base", "provenance", "last-field"]
        } else {
            &["base", "provenance", "cohort", "none"]
        };
        let mut base: Option<(BoundaryRunVcProgram, Vec<u8>, Vec<u8>)> = None;
        for variant in variants {
            let document = if unit {
                "{}".into()
            } else if wide {
                wide_document()
            } else if *variant == "none" {
                r#"{"field0":{"Number":-7,"Text":"","Amount":"0","Maybe":{"tag":"none"}}}"#.into()
            } else {
                r#"{"field0":{"Number":7,"Text":"😀\ud800","Amount":"1.25","Maybe":{"tag":"some","payload":9}}}"#.into()
            };
            if wide && *variant == "base" {
                // Frozen i64 JSON uses canonical strings, never JSON numbers.
                for invalid in [
                    document.replace(
                        r#"["-1","9223372036854775807"]"#,
                        "[-1,9223372036854775807]",
                    ),
                    document.replace("9223372036854775807", "9223372036854775808"),
                    document.replace(r#""-1""#, r#""-0""#),
                ] {
                    assert!(emitted
                        .capture_boundary_input(
                            &bundle,
                            &context,
                            &captures,
                            BoundaryInputBytes {
                                boundary_id,
                                provenance_id: "test.ordinary.invalid",
                                raw_bytes: invalid.as_bytes(),
                                canonical_document: invalid.as_bytes()
                            }
                        )
                        .is_err());
                }
            }
            let input = emitted
                .capture_boundary_input(
                    &bundle,
                    &context,
                    &captures,
                    BoundaryInputBytes {
                        boundary_id,
                        provenance_id: if *variant == "provenance" {
                            "test.ordinary.changed"
                        } else {
                            "test.ordinary.base"
                        },
                        raw_bytes: document.as_bytes(),
                        canonical_document: document.as_bytes(),
                    },
                )
                .unwrap();
            let mut returned = if unit {
                MonomorphicValue::Unit {
                    type_id: ty("unit"),
                }
            } else {
                input.arguments()[0].value().clone()
            };
            if *variant == "cohort" {
                let MonomorphicValue::Product { fields, .. } = &mut returned else {
                    panic!()
                };
                *fields
                    .iter_mut()
                    .find(|f| f.name == "Amount")
                    .unwrap()
                    .value = MonomorphicValue::DecimalBits {
                    type_id: ty("decimal"),
                    negative: false,
                    scale: 4,
                    coefficient: "12500".into(),
                };
            }
            if *variant == "last-field" {
                let MonomorphicValue::Product { fields, .. } = &mut returned else {
                    panic!()
                };
                *fields.iter_mut().find(|f| f.name == "F31").unwrap().value =
                    MonomorphicValue::String {
                        type_id: ty("string"),
                        utf16: vec![0xffff],
                    };
            }
            let captured = emitted
                .capture_boundary_output(&bundle, &context, &captures, &input, &returned)
                .unwrap();
            let run_vcs = emitted
                .generate_boundary_run_vcs(&bundle, &context, &captures, &captured)
                .unwrap();
            assert_eq!(run_vcs.source_ir_sha256(), vir.hash());
            let p = generate_csharp_practical_ordinary_boundary_literals(vir, &run_vcs).unwrap();
            assert_eq!(
                import_csharp_practical_ordinary_boundary_literals(
                    &p.canonical_bytes(),
                    p.certificate_bytes(),
                    vir,
                    &run_vcs
                )
                .unwrap(),
                p
            );
            let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
            validate_csharp_practical_certificate_structure(&cert).unwrap();
            assert_eq!(p.bindings().len(), run_vcs.values().len());
            assert!(!p.bindings().is_empty());
            for (binding, original) in p.bindings().iter().zip(run_vcs.values()) {
                assert_eq!(binding.boundary_symbol, original.name);
                assert_eq!(binding.type_id, original.value.type_id());
                let def = p
                    .definitions()
                    .iter()
                    .find(|d| d.name == binding.definition)
                    .unwrap();
                assert_eq!(def.value, original.value);
                assert_eq!(def.carrier.type_id, original.value.type_id());
            }
            let mut observed = 0;
            for def in p.definitions() {
                assert!(def.carrier.depth <= 24);
                let expected = relation_tests::storage(&def.value, &types);
                for (index, &bit) in expected.iter().enumerate() {
                    if expected.len() > 4096
                        && !bit
                        && index >= 128
                        && index + 128 < expected.len()
                        && index % 8191 != 0
                    {
                        continue;
                    }
                    let args = (0..def.carrier.depth)
                        .map(|s| V::Bit(index & (1 << s) != 0))
                        .collect();
                    assert_eq!(
                        observed_bit(run(&cert, &def.name, args)),
                        bit,
                        "{source_name}-{variant} {} bit {index}",
                        def.name
                    );
                    observed += 1;
                }
            }
            if *variant == "cohort" {
                assert_ne!(captured.returned_value(), captured.reparsed_value());
                assert_eq!(p.definitions().len(), 2);
                assert_ne!(
                    relation_tests::storage(captured.returned_value(), &types),
                    relation_tests::storage(captured.reparsed_value(), &types)
                );
            }
            if *variant == "provenance" {
                let (before, metadata, bytes) = base.as_ref().unwrap();
                assert_ne!(before.hash(), run_vcs.hash());
                assert_eq!(bytes, p.certificate_bytes());
                assert_ne!(metadata, &p.canonical_bytes());
                assert!(import_csharp_practical_ordinary_boundary_literals(
                    metadata, bytes, vir, &run_vcs
                )
                .is_err());
                assert!(import_csharp_practical_ordinary_boundary_literals(
                    &p.canonical_bytes(),
                    p.certificate_bytes(),
                    vir,
                    before
                )
                .is_err());
            }
            if let Some(other) = &previous_source_run {
                assert!(generate_csharp_practical_ordinary_boundary_literals(vir, other).is_err());
            }
            let metadata: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
            for field in [
                "schema",
                "source_ir_sha256",
                "foundation_sha256",
                "boundary_program_sha256",
                "boundary_run_sha256",
                "definitions",
                "bindings",
                "certificate_sha256",
            ] {
                let mut changed = metadata.clone();
                changed[field] = json!("forged");
                assert!(
                    import_csharp_practical_ordinary_boundary_literals(
                        &serde_json::to_vec(&changed).unwrap(),
                        p.certificate_bytes(),
                        vir,
                        &run_vcs
                    )
                    .is_err(),
                    "{field}"
                );
            }
            let mut changed = metadata.clone();
            changed["bindings"][0]["type_id"] = json!("forged");
            assert!(import_csharp_practical_ordinary_boundary_literals(
                &serde_json::to_vec(&changed).unwrap(),
                p.certificate_bytes(),
                vir,
                &run_vcs
            )
            .is_err());
            let mut corrupt = p.certificate_bytes().to_vec();
            *corrupt.last_mut().unwrap() ^= 1;
            assert!(import_csharp_practical_ordinary_boundary_literals(
                &p.canonical_bytes(),
                &corrupt,
                vir,
                &run_vcs
            )
            .is_err());
            let id = format!("{source_name}-{variant}");
            let hex = p
                .certificate_bytes()
                .iter()
                .map(|v| format!("{v:02x}"))
                .collect::<String>()
                + "\n";
            if let Some(dir) = &output {
                fs::write(dir.join(format!("{id}.hex")), hex).unwrap();
            } else {
                assert_eq!(
                    fs::read_to_string(fixture.join(format!("{id}.hex"))).unwrap(),
                    hex
                );
            }
            rows.push(json!({"id":id,"source_request_id":request["id"],"program":metadata,"run_vcs_sha256":run_vcs.hash(),"terms":cert.term_table.len(),"declarations":cert.declarations.len(),"observations":observed}));
            eprintln!(
                "boundary literal {id}: {} definitions, {observed} observations",
                p.definitions().len()
            );
            total_observations += observed;
            if *variant == "base" {
                base = Some((run_vcs, p.canonical_bytes(), p.certificate_bytes().to_vec()));
            }
        }
        previous_source_run = Some(base.unwrap().0);
    }
    assert_eq!(rows.len(), 9);
    let result = json!({"sources":3,"runs":rows,"observations":total_observations});
    if let Some(dir) = &output {
        fs::write(
            dir.join("certificates.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/boundary-literals/certificates.json"),
            result
        );
    }
    eprintln!("boundary literal total: {total_observations} observations");
}
