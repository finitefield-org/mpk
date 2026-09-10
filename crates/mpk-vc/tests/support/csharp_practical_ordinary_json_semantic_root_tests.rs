//! Original source requests whose input root is projected by a semantic binding.
use super::*;
fn source_id(name: &str) -> String {
    csharp_practical_declaration_id(&json!({"kind":"type","namespace":"Boundary","owner":"","name":name,"parameter_type_ids":[],"result_type_id":""})).unwrap()
}
#[test]
fn csharp_03_t06_w09_json_semantic_root_requests() {
    let bundle = b();
    let mut rows = vec![];
    for role in [
        "money",
        "ordered_entry",
        "ordered_set",
        "ordered_map",
        "transition",
    ] {
        let extra = if role == "ordered_map" {
            "public readonly struct Pair{public readonly int Key;public readonly bool Value;}"
        } else {
            ""
        };
        let members=match role {
            "money"=>"public readonly decimal Amount;public readonly string Currency;",
            "ordered_entry"=>"public readonly int Key;public readonly bool Value;",
            "ordered_set"=>"public readonly int[] Items;",
            "ordered_map"=>"public readonly Pair[] Items;",
            "transition"=>"public readonly int State;public readonly int[] Events;public readonly bool Response;",
            _=>unreachable!(),
        };
        let code=format!("namespace Boundary;{extra}public readonly struct Payload{{{members}}}public static class Entry{{public static Payload Run(Payload p){{return p;}}}}\n");
        let mut request = super::source_tests::requests_for(&[(role, &code)])[0].clone();
        let (context, captures) = support::replay_context(&bundle, &request);
        let source_hash = captures
            .entries()
            .iter()
            .find(|e| e.kind() == OriginalInputKind::Source)
            .unwrap()
            .raw_sha256();
        let (members, args, bounds) = match role {
            "money" => (
                vec![
                    ("amount", "Amount", primitive("decimal")),
                    ("currency", "Currency", primitive("string")),
                ],
                vec![ty("string")],
                vec![],
            ),
            "ordered_entry" => (
                vec![
                    ("key", "Key", primitive("i32")),
                    ("value", "Value", primitive("bool")),
                ],
                vec![ty("i32"), ty("bool")],
                vec![],
            ),
            "ordered_set" => (
                vec![(
                    "elements",
                    "Items",
                    instance("bounded_sequence", vec![primitive("i32")]),
                )],
                vec![ty("i32")],
                vec![SemanticBound {
                    id: "length".into(),
                    maximum: 4096,
                }],
            ),
            "ordered_map" => (
                vec![(
                    "entries",
                    "Items",
                    instance(
                        "bounded_sequence",
                        vec![json!({"kind":"source","id":source_id("Pair")})],
                    ),
                )],
                vec![ty("i32"), ty("bool")],
                vec![SemanticBound {
                    id: "length".into(),
                    maximum: 4096,
                }],
            ),
            "transition" => (
                vec![
                    ("state", "State", primitive("i32")),
                    (
                        "events",
                        "Events",
                        instance("bounded_sequence", vec![primitive("i32")]),
                    ),
                    ("response", "Response", primitive("bool")),
                ],
                vec![ty("i32"), ty("i32"), ty("bool")],
                vec![SemanticBound {
                    id: "events".into(),
                    maximum: 4096,
                }],
            ),
            _ => unreachable!(),
        };
        let mut specs = vec![("Payload", role, members, args, bounds)];
        if role == "ordered_map" {
            specs.push((
                "Pair",
                "ordered_entry",
                vec![
                    ("key", "Key", primitive("i32")),
                    ("value", "Value", primitive("bool")),
                ],
                vec![ty("i32"), ty("bool")],
                vec![],
            ));
        }
        let bindings = specs
            .into_iter()
            .map(|(name, role, members, args, bounds)| {
                let id = source_id(name);
                SemanticBindingInput {
                    source_type_id: id.clone(),
                    source_content_sha256: source_hash.into(),
                    role: role.into(),
                    member_map: members
                        .into_iter()
                        .map(|(role, name, ty)| SemanticBindingMember {
                            role: role.into(),
                            member_id: csharp_practical_stored_member_id(
                                &id,
                                name,
                                &ty,
                                "readonly_field",
                            )
                            .unwrap(),
                        })
                        .collect(),
                    inferred_argument_ids: args,
                    tag_arms: vec![],
                    default_arm: "ineligible".into(),
                    bounds,
                    operation_map: vec![],
                    enum_arms: BTreeMap::new(),
                }
            })
            .collect();
        let sidecar = build_semantic_bindings(&context, &captures, bindings).unwrap();
        let inputs = request["inputs"].as_array_mut().unwrap();
        inputs.push(json!({"kind":"sidecar","path":"contracts/bindings.json","utf8":std::str::from_utf8(sidecar.canonical_bytes()).unwrap()}));
        inputs.sort_by_key(|v| v["path"].as_str().unwrap().to_owned());
        rows.push(request);
    }
    let bytes = serde_json::to_vec_pretty(&rows).unwrap();
    if let Some(out) = std::env::var_os("MPK_W09_JSON_SEMANTIC_ROOT_REQUESTS_OUT") {
        fs::write(out, bytes).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/json-semantic-root-sources/requests.json"),
            json!(rows)
        );
    }
}

#[test]
fn csharp_03_t06_w09_json_semantic_root_depth_envelopes() {
    let requests = read("ordinary-foundation/json-semantic-root-sources/requests.json");
    let responses = read("ordinary-foundation/json-semantic-root-sources/responses.json");
    let bundle = b();
    let mut rows = vec![];
    let out = std::env::var_os("MPK_W09_JSON_SEMANTIC_ROOT_OUT").map(std::path::PathBuf::from);
    if let Some(out) = &out {
        fs::create_dir_all(out).unwrap();
    }
    for (role, payload, source_payload) in [
        (
            "money",
            r#"{"amount":"1.25","currency":"JPY"}"#,
            r#"{"Amount":"1.25","Currency":"JPY"}"#,
        ),
        (
            "ordered_entry",
            r#"{"key":1,"value":true}"#,
            r#"{"Key":1,"Value":true}"#,
        ),
        ("ordered_set", r#"[1,3]"#, r#"{"Items":[1,3]}"#),
        (
            "ordered_map",
            r#"[{"key":1,"value":true},{"key":3,"value":false}]"#,
            r#"{"Items":[{"Key":1,"Value":true},{"Key":3,"Value":false}]}"#,
        ),
        (
            "transition",
            r#"{"state":2,"events":[3],"response":true}"#,
            r#"{"State":2,"Events":[3],"Response":true}"#,
        ),
    ] {
        eprintln!("Semantic root envelope:{role}");
        let request = requests
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == role)
            .unwrap();
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == role)
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
        let p = generate_csharp_practical_ordinary_json_depth_guarded_envelopes(&emitted).unwrap();
        let field = &p.field_decoders()[0];
        assert_eq!(field.source_type_id, source_id("Payload"));
        assert_ne!(field.source_type_id, field.semantic_carrier.type_id);
        let entry = emitted
            .closure()
            .closed()
            .entries()
            .iter()
            .find(|e| e["instance_id"] == field.semantic_carrier.type_id)
            .unwrap();
        assert_eq!(
            entry["template_id"],
            format!("mpk.csharp.semantic.{role}.v1")
        );
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let boundary = &emitted.boundaries()[0];
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
                    provenance_id: "test.semantic.root",
                    raw_bytes: text.as_bytes(),
                    canonical_document: text.as_bytes(),
                },
            )
            .unwrap();
        assert_eq!(captured.arguments().len(), 1);
        assert_eq!(
            captured.arguments()[0].value().type_id(),
            field.semantic_carrier.type_id
        );
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
        let count = 1 + super::boundary_field_tests::cells(captured.arguments()[0].value());
        for index in 0..128 {
            let expected = match index {
                0 | 1 => true,
                2..=33 => text.len() & (1 << (index - 2)) != 0,
                34..=65 => count & (1 << (index - 34)) != 0,
                _ => false,
            };
            assert_eq!(
                leaf(&cert, header.clone(), 7, index),
                expected,
                "{role} header bit{index}"
            );
        }
        let arguments = run(&cert, &d.value_definition, vec![packet]);
        let value = run(
            &cert,
            &d.fields[0].argument_projection_definition,
            vec![arguments],
        );
        let layouts = generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap();
        let types = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        assert!(field.semantic_carrier.depth <= 22);
        let expected = super::super::super::super::relation_tests::storage(
            captured.arguments()[0].value(),
            &types,
        );
        let mut probes = (0..expected.len().min(128)).collect::<BTreeSet<_>>();
        for (i, bit) in expected.iter().enumerate().filter(|(_, v)| **v) {
            assert!(*bit);
            probes.insert(i);
            if i > 0 {
                probes.insert(i - 1);
            }
            if i + 1 < expected.len() {
                probes.insert(i + 1);
            }
        }
        probes.extend((0..field.semantic_carrier.depth).map(|i| 1 << i));
        for i in &probes {
            assert_eq!(
                leaf(
                    &cert,
                    value.clone(),
                    field.semantic_carrier.depth as usize,
                    *i
                ),
                expected[*i],
                "{role} argument bit{i}"
            );
        }
        let tree = captured
            .capture()
            .artifact()
            .value()
            .get("canonical_value")
            .unwrap()
            .get("field0")
            .unwrap();
        let height = super::depth_compound_tests::height(tree);
        let depth = p
            .typed_depth()
            .iter()
            .find(|x| x.carrier.type_id == field.semantic_carrier.type_id)
            .unwrap();
        for root in BTreeSet::from([0, 1, 32 - height, 33 - height, 32, 33, u32::MAX]) {
            let valid = run(&cert, &depth.definition, vec![value.clone(), word(root)]);
            assert_eq!(
                leaf(&cert, valid, 0, 0),
                root.checked_add(height).is_some_and(|v| v <= 32)
            );
        }
        let bad = format!("{{\"field0\":{source_payload}}}");
        assert!(emitted
            .capture_boundary_input(
                &bundle,
                &context,
                &captures,
                BoundaryInputBytes {
                    boundary_id,
                    provenance_id: "test.wrong.root.layer",
                    raw_bytes: bad.as_bytes(),
                    canonical_document: bad.as_bytes()
                }
            )
            .is_err());
        let invalid = run(
            &cert,
            &d.parse_definition,
            vec![document(
                bad.len() as u32,
                &bad.bytes().enumerate().collect::<Vec<_>>(),
            )],
        );
        let header = run(&cert, &d.header_definition, vec![invalid]);
        for i in 0..128 {
            assert!(
                !leaf(&cert, header.clone(), 7, i),
                "{role} wrong source representation bit{i}"
            );
        }
        let row = json!({"id":role,"document_utf8":text,"rejected_source_document":bad,"argument_bit_probes":probes.len(),"canonical_field_height":height,"program":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),"terms":cert.term_table.len(),"declarations":cert.declarations.len(),"certificate_sha256":mpk_cert::hash_hex(&mpk_cert::certificate_hash(p.certificate_bytes()))});
        if let Some(out) = &out {
            fs::write(
                out.join(format!("{role}.hex")),
                p.certificate_bytes()
                    .iter()
                    .map(|v| format!("{v:02x}"))
                    .collect::<String>()
                    + "\n",
            )
            .unwrap();
            fs::write(
                out.join(format!("{role}.json")),
                serde_json::to_vec_pretty(&row).unwrap(),
            )
            .unwrap();
        }
        if out.is_none() {
            assert_eq!(
                row,
                read(&format!(
                    "ordinary-foundation/json-semantic-root/{role}.json"
                )),
                "{role}: pinned metadata"
            );
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../develop/migrations/csharp-03/ordinary-foundation/json-semantic-root")
                .join(format!("{role}.hex"));
            let expected = p
                .certificate_bytes()
                .iter()
                .map(|v| format!("{v:02x}"))
                .collect::<String>();
            assert_eq!(
                fs::read_to_string(path).unwrap().trim(),
                expected,
                "{role}: pinned certificate"
            );
        }
        rows.push(row);
    }
    assert_eq!(rows.len(), 5);
    if let Some(out) = out {
        fs::write(
            out.join("certificates.json"),
            serde_json::to_vec_pretty(&rows).unwrap(),
        )
        .unwrap();
    }
}
