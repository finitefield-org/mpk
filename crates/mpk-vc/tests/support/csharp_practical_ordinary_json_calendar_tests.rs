//! Actual-source inputs for reachable calendar JSON codecs and cumulative limits.
use super::*;
use mpk_vc::csharp_practical_vir_validation::ValidatedPracticalVir;
use sha2::{Digest, Sha256};

fn fixture_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/json-calendar-sources")
}

fn source_requests() -> Value {
    use PracticalJsonValue as J;
    let bundle = b();
    let template = read("boundary-output/source-requests.json")[0].clone();
    let root = template["roots"][0].as_str().unwrap();
    let mut requests = vec![];
    for (id, pairs) in [
        ("date", vec![("System.DateOnly", "Date")]),
        ("time", vec![("System.TimeOnly", "Time")]),
        (
            "date-time",
            vec![("System.DateOnly", "Date"), ("System.TimeOnly", "Time")],
        ),
        (
            "date-time-decimal",
            vec![
                ("System.DateOnly", "Date"),
                ("System.TimeOnly", "Time"),
                ("decimal", "Amount"),
            ],
        ),
    ] {
        let fields = pairs
            .iter()
            .map(|(ty, name)| format!("public readonly {ty} {name};"))
            .collect::<String>();
        let parameters = pairs
            .iter()
            .enumerate()
            .map(|(i, (ty, _))| format!("{ty} p{i}"))
            .collect::<Vec<_>>()
            .join(",");
        let body = pairs
            .iter()
            .enumerate()
            .map(|(i, (_, name))| format!("{name}=p{i};"))
            .collect::<String>();
        let args = pairs
            .iter()
            .map(|(_, name)| format!("p.{name}"))
            .collect::<Vec<_>>()
            .join(",");
        let source = format!("namespace Boundary;public readonly struct Payload{{{fields}public Payload({parameters}){{{body}}}}}public static class Entry{{public static Payload Run(Payload p){{return new Payload({args});}}}}\n");
        let (context, captures) = support::context_with_sidecars(
            &bundle,
            root,
            source.as_bytes(),
            vec![
                "contracts/boundary.json".into(),
                "contracts/method.json".into(),
            ],
            |context| {
                [
                    (
                        "contracts/boundary.json",
                        PracticalArtifactKind::BoundaryContract,
                        "MPK-CSHARP-BOUNDARY-CONTRACT-1.0",
                    ),
                    (
                        "contracts/method.json",
                        PracticalArtifactKind::MethodContract,
                        "MPK-CSHARP-METHOD-CONTRACT-1.0",
                    ),
                ]
                .into_iter()
                .map(|(path, kind, domain)| {
                    let input = template["inputs"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|v| v["path"] == path)
                        .unwrap();
                    let mut value = parse_canonical_practical_json(
                        kind,
                        input["utf8"].as_str().unwrap().as_bytes(),
                    )
                    .unwrap();
                    let J::Object(fields) = &mut value else {
                        panic!()
                    };
                    fields.retain(|(key, _)| key != "contract_sha256");
                    for (key, value) in fields.iter_mut() {
                        match key.as_str() {
                            "semantic_context" => *value = context.semantic_context().clone(),
                            "source_content_sha256" => {
                                *value =
                                    J::string(format!("{:x}", Sha256::digest(source.as_bytes())))
                            }
                            _ => {}
                        }
                    }
                    let bytes = canonical_practical_json_bytes(&value).unwrap();
                    let mut hash = Sha256::new();
                    hash.update(domain);
                    hash.update([0]);
                    hash.update(bytes);
                    let J::Object(fields) = &mut value else {
                        unreachable!()
                    };
                    fields.push((
                        "contract_sha256".into(),
                        J::string(format!("{:x}", hash.finalize())),
                    ));
                    canonical_practical_json_bytes(&value).unwrap()
                })
                .collect()
            },
        );
        requests.push(json!({"id":id,"compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),
            "inputs":captures.entries().iter().map(|entry|json!({"kind":if entry.kind()==OriginalInputKind::Source{"source"}else{"sidecar"},"path":entry.path(),"utf8":std::str::from_utf8(entry.bytes()).unwrap()})).collect::<Vec<_>>()}));
    }
    json!(requests)
}

#[test]
fn csharp_03_t06_w09_json_calendar_source_requests() {
    let bytes = serde_json::to_vec_pretty(&source_requests()).unwrap();
    if let Some(out) = std::env::var_os("MPK_W09_JSON_CALENDAR_REQUESTS_OUT") {
        fs::write(out, bytes).unwrap();
    } else {
        assert_eq!(
            fs::read(fixture_root().join("requests.json")).unwrap(),
            bytes
        );
    }
}

#[test]
fn csharp_03_t06_w09_json_calendar_source_captures() {
    let bundle = b();
    let requests = source_requests();
    let responses: Value =
        serde_json::from_slice(&fs::read(fixture_root().join("responses.json")).unwrap()).unwrap();
    assert_eq!(responses.as_array().unwrap().len(), 4);
    for request in requests.as_array().unwrap() {
        let id = request["id"].as_str().unwrap();
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        assert!(response.get("reject").is_none(), "{id}: {response}");
        let (context, captures) = support::replay_context(&bundle, request);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&response["facts"]).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let program = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir: emitted.vir(),
        })
        .unwrap();
        assert_eq!(program.boundary_vcs().contracts().len(), 1);
        let layouts = generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap();
        for (token, expected) in [
            ("date", id.contains("date")),
            ("time", id.contains("time")),
            ("decimal", id.contains("decimal")),
        ] {
            assert_eq!(
                layouts.carriers().iter().any(|c| c.type_id == ty(token)),
                expected,
                "{id}: {token}"
            );
        }
    }
}

fn each_source(mut f: impl FnMut(&str, &ValidatedPracticalVir)) {
    let bundle = b();
    let requests = source_requests();
    let responses: Value =
        serde_json::from_slice(&fs::read(fixture_root().join("responses.json")).unwrap()).unwrap();
    for request in requests.as_array().unwrap() {
        let id = request["id"].as_str().unwrap();
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
        f(id, emitted.vir());
    }
}

#[test]
fn csharp_03_t06_w09_json_calendar_programs() {
    use super::super::super::structural_equivalence_tests::same_definition_closure;
    let out = std::env::var_os("MPK_W09_JSON_CALENDAR_OUT").map(std::path::PathBuf::from);
    let root = fixture_root().parent().unwrap().join("json-calendar");
    let target = out.as_ref().unwrap_or(&root);
    if out.is_some() {
        fs::create_dir_all(target).unwrap();
    }
    let mut rows = vec![];
    each_source(|id, vir| {
        let token = generate_csharp_practical_ordinary_json_tokens(vir).unwrap();
        let calendar = generate_csharp_practical_ordinary_calendar_codecs(vir).unwrap();
        let structural = generate_csharp_practical_ordinary_structural_boundary(vir)
            .unwrap_or_else(|e| panic!("{id}: combined generation {e:?}"));
        assert_eq!(
            token
                .definition()
                .unwrap()
                .quoted_calendars
                .iter()
                .map(|d| &d.codec)
                .collect::<Vec<_>>(),
            calendar.definitions().iter().collect::<Vec<_>>()
        );
        let calendar_cert =
            mpk_cert::decode_canonical_certificate(calendar.certificate_bytes()).unwrap();
        let token_cert = mpk_cert::decode_canonical_certificate(token.certificate_bytes()).unwrap();
        let structural_cert =
            mpk_cert::decode_canonical_certificate(structural.certificate_bytes()).unwrap();
        for (old, new) in [
            (&calendar_cert, &token_cert),
            (&token_cert, &structural_cert),
        ] {
            let names = old
                .declarations
                .iter()
                .map(|d| old.name_table[d.name as usize].clone())
                .collect();
            same_definition_closure(old, new, &names).unwrap();
        }
        for (kind, bytes, metadata) in [
            ("tokens", token.certificate_bytes(), token.canonical_bytes()),
            (
                "structural",
                structural.certificate_bytes(),
                structural.canonical_bytes(),
            ),
        ] {
            let cert = mpk_cert::decode_canonical_certificate(bytes).unwrap();
            validate_csharp_practical_certificate_structure(&cert).unwrap();
            let file = format!("{id}-{kind}.hex");
            let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
            if out.is_some() {
                fs::write(target.join(&file), &hex).unwrap();
            } else {
                assert_eq!(fs::read_to_string(target.join(&file)).unwrap(), hex);
            }
            let metadata: Value = serde_json::from_slice(&metadata).unwrap();
            eprintln!(
                "calendar {id}/{kind}: {} terms, {} declarations",
                cert.term_table.len(),
                cert.declarations.len()
            );
            rows.push(json!({"id":id,"kind":kind,"file":file,"terms":cert.term_table.len(),"declarations":cert.declarations.len(),"program":metadata}));
        }
        import_csharp_practical_ordinary_json_tokens(
            &token.canonical_bytes(),
            token.certificate_bytes(),
            vir,
        )
        .unwrap();
        let mut corrupted = token.certificate_bytes().to_vec();
        corrupted[0] ^= 1;
        assert!(import_csharp_practical_ordinary_json_tokens(
            &token.canonical_bytes(),
            &corrupted,
            vir
        )
        .is_err());
        let mut meta: Value = serde_json::from_slice(&token.canonical_bytes()).unwrap();
        meta["definition"]["quoted_calendars"][0]["codec"]["value_depth"] = json!(0);
        assert!(import_csharp_practical_ordinary_json_tokens(
            &serde_json::to_vec(&meta).unwrap(),
            token.certificate_bytes(),
            vir
        )
        .is_err());
    });
    let bytes = serde_json::to_vec_pretty(&rows).unwrap();
    if out.is_some() {
        fs::write(target.join("certificates.json"), bytes).unwrap();
    } else {
        assert_eq!(fs::read(target.join("certificates.json")).unwrap(), bytes);
    }
}

#[test]
fn csharp_03_t06_w09_json_calendar_decimal_component_costs() {
    each_source(|id, vir| {
        if id != "date-time-decimal" {
            return;
        }
        let structural = generate_csharp_practical_ordinary_structural_foundations(vir).unwrap();
        let json = generate_csharp_practical_ordinary_json_tokens(vir).unwrap();
        for (kind, bytes, transformers) in [
            (
                "structural",
                structural.certificate_bytes(),
                structural.static_transformers(),
            ),
            (
                "json",
                json.certificate_bytes(),
                json.definition().unwrap().static_transformers,
            ),
        ] {
            let cert = mpk_cert::decode_canonical_certificate(bytes).unwrap();
            validate_csharp_practical_certificate_structure(&cert).unwrap();
            eprintln!(
                "{id} independent {kind}: {} terms, {} declarations, {transformers} transformers",
                cert.term_table.len(),
                cert.declarations.len()
            );
        }
        // These independently valid components are diagnostic evidence only:
        // json_calendar_programs must still pass their actual shared generation.
    });
}

#[test]
fn csharp_03_t06_w09_json_calendar_runtime() {
    each_source(|id, vir| {
        if id != "date-time" {
            return;
        }
        let program = generate_csharp_practical_ordinary_json_tokens(vir).unwrap();
        let cert = mpk_cert::decode_canonical_certificate(program.certificate_bytes()).unwrap();
        let definitions = &program.definition().unwrap().quoted_calendars;
        let mut count = 0;
        let mut check =
            |kind: &str, text: &str, start: u32, ending: u8, expected: Option<(u64, u32, bool)>| {
                let def = definitions
                    .iter()
                    .find(|d| d.codec.codec_id == kind)
                    .unwrap();
                let doc = document(
                    text.len() as u32,
                    &text.bytes().enumerate().collect::<Vec<_>>(),
                );
                let result = run(
                    &cert,
                    &def.parse_definition,
                    vec![doc, word(start), word_tag(ending)],
                );
                let mut wanted = [false; 128];
                if let Some((value, end, whole)) = expected {
                    wanted[0] = true;
                    wanted[1] = whole;
                    for i in 0..64 {
                        wanted[2 + i] = value & (1u64 << i) != 0;
                    }
                    for i in 0..32 {
                        wanted[66 + i] = end & (1u32 << i) != 0;
                    }
                }
                for (i, bit) in wanted.into_iter().enumerate() {
                    assert_eq!(
                        leaf(&cert, result.clone(), 7, i),
                        bit,
                        "{kind} {text:?} {start}/{ending}: bit {i}"
                    );
                }
                count += 1;
                eprintln!("calendar packet {count}: {kind} {text:?}");
            };
        for (kind, text, value) in [
            ("date", "0001-01-01", 0),
            ("date", "1970-01-01", 719162),
            ("date", "2000-02-29", 730178),
            ("date", "9999-12-31", 3652058),
            ("time", "00:00:00.0000000", 0),
            ("time", "12:34:56.1234567", 452961234567),
            ("time", "23:59:59.9999999", 863999999999),
        ] {
            let quoted = format!("\"{text}\"");
            check(
                kind,
                &quoted,
                0,
                0,
                Some((value, quoted.len() as u32, true)),
            );
        }
        for (ending, delimiter) in [(1, ','), (2, ']'), (3, '}')] {
            check(
                "date",
                &format!("[\"1970-01-01\"{delimiter}"),
                1,
                ending,
                Some((719162, 13, false)),
            );
        }
        for (kind, text) in [
            ("date", "1900-02-29"),
            ("date", "2001-02-29"),
            ("date", "0000-01-01"),
            ("date", "10000-01-01"),
            ("date", "2020-04-31"),
            ("date", "2020-13-01"),
            ("date", "2020-1-01"),
            ("time", "24:00:00.0000000"),
            ("time", "23:60:00.0000000"),
            ("time", "23:59:60.0000000"),
            ("time", "00:00:00.000000"),
            ("time", "00:00:00.00000000"),
        ] {
            check(kind, &format!("\"{text}\""), 0, 0, None);
        }
        for (text, start, ending) in [
            ("\"1970-01-01\":", 0, 4),
            ("\"1970-01-01\"", 0, 255),
            ("\"1970-01-01\" ", 0, 0),
            ("\"1970-01-01", 0, 0),
            ("\"1970-01-01\"", 1, 0),
            ("\"1970-01-01\"", u32::MAX, 0),
            ("\"1970-01-01\"]", 0, 1),
            ("\"1970-01-01\\u0030\"", 0, 0),
        ] {
            check("date", text, start, ending, None);
        }
        assert_eq!(count, 30);
    });
}

#[test]
fn csharp_03_t06_w09_json_calendar_previous_token_preservation() {
    use super::super::super::structural_equivalence_tests::same_definition_closure;
    let path = fixture_root()
        .parent()
        .unwrap()
        .join("json-calendar/previous-format/date-time-tokens.hex");
    let hex = fs::read_to_string(path).unwrap();
    let bytes = (0..hex.trim().len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect::<Vec<_>>();
    let old = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
    let roots: BTreeSet<String> = old
        .declarations
        .iter()
        .map(|d| old.name_table[d.name as usize].clone())
        .filter(|name| name.starts_with("Mpk.CSharp.Ordinary.JsonTokens."))
        .collect();
    assert!(roots.len() > 150);
    each_source(|id, vir| {
        if id != "date-time" {
            return;
        }
        let program = generate_csharp_practical_ordinary_json_tokens(vir).unwrap();
        let new = mpk_cert::decode_canonical_certificate(program.certificate_bytes()).unwrap();
        let checked = same_definition_closure(&old, &new, &roots).unwrap();
        eprintln!(
            "Calendar JSON token preservation: {} roots, {} dependency declarations",
            roots.len(),
            checked.len()
        );
    });
}
