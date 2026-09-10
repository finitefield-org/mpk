//! Original typed Transition JSON, including shared implicit-container joins.
use super::*;
#[path = "csharp_practical_ordinary_json_transition_runtime.rs"]
mod runtime;
fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/json-transition-sources")
}
fn pins() -> std::path::PathBuf {
    std::env::var_os("MPK_W09_JSON_TRANSITION_OUT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root().with_file_name("json-transition"))
}
fn source_id(name: &str) -> String {
    csharp_practical_declaration_id(&json!({"kind":"type","namespace":"Boundary","owner":"","name":name,"parameter_type_ids":[],"result_type_id":""})).unwrap()
}
fn source_ty(name: &str) -> Value {
    json!({"kind":"source","id":source_id(name)})
}
fn requests() -> Value {
    let bundle = b();
    let mut all = vec![];
    for compound in [false, true] {
        let id = if compound { "compound" } else { "scalar-map" };
        let extra = if compound {
            "public readonly struct State{public readonly string Name;public readonly int Version;}public readonly struct Event{public readonly int Code;}"
        } else {
            "public readonly struct Pair{public readonly int Key;public readonly bool Value;}public readonly struct MapRep{public readonly Pair[] Items;}"
        };
        let state = if compound { "State" } else { "int" };
        let event = if compound { "Event" } else { "int" };
        let response = if compound { "bool?" } else { "bool" };
        let map_field = if compound {
            ""
        } else {
            "public readonly MapRep Map;"
        };
        let code=format!("namespace Boundary;{extra}public readonly struct Outcome{{public readonly {state} State;public readonly {event}[] Events;public readonly {response} Response;}}public readonly struct Payload{{public readonly Outcome Outcome;{map_field}}}public static class Entry{{public static Payload Run(Payload p){{return p;}}}}\n");
        let mut request = source_tests::requests_for(&[(id, &code)])[0].clone();
        let (context, captures) = support::replay_context(&bundle, &request);
        let source_hash = captures
            .entries()
            .iter()
            .find(|e| e.kind() == OriginalInputKind::Source)
            .unwrap()
            .raw_sha256();
        let st = if compound {
            source_ty("State")
        } else {
            primitive("i32")
        };
        let ev = if compound {
            source_ty("Event")
        } else {
            primitive("i32")
        };
        let rt = if compound {
            instance("option", vec![primitive("bool")])
        } else {
            primitive("bool")
        };
        let response_id = if compound {
            csharp_practical_closed_instance_id(&bundle, &rt).unwrap()
        } else {
            ty("bool")
        };
        let mut specs = vec![(
            "Outcome",
            "transition",
            vec![
                ("state", "State", st),
                ("events", "Events", instance("bounded_sequence", vec![ev])),
                ("response", "Response", rt),
            ],
            vec![
                if compound {
                    source_id("State")
                } else {
                    ty("i32")
                },
                if compound {
                    source_id("Event")
                } else {
                    ty("i32")
                },
                response_id,
            ],
            vec![SemanticBound {
                id: "events".into(),
                maximum: 4096,
            }],
        )];
        if !compound {
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
            specs.push((
                "MapRep",
                "ordered_map",
                vec![(
                    "entries",
                    "Items",
                    instance("bounded_sequence", vec![source_ty("Pair")]),
                )],
                vec![ty("i32"), ty("bool")],
                vec![SemanticBound {
                    id: "length".into(),
                    maximum: 4096,
                }],
            ));
        }
        let bindings = specs
            .into_iter()
            .map(|(name, role, members, arguments, bounds)| {
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
                    inferred_argument_ids: arguments,
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
        inputs.push(json!({"kind":"sidecar","path":"contracts/data.json","utf8":std::str::from_utf8(sidecar.canonical_bytes()).unwrap()}));
        inputs.sort_by_key(|i| i["path"].as_str().unwrap().to_owned());
        all.push(request);
    }
    json!(all)
}
#[test]
fn csharp_03_t06_w09_json_transition_requests() {
    let bytes = serde_json::to_vec_pretty(&requests()).unwrap();
    if let Some(path) = std::env::var_os("MPK_W09_JSON_TRANSITION_REQUESTS_OUT") {
        fs::write(path, bytes).unwrap();
    } else {
        assert_eq!(fs::read(root().join("requests.json")).unwrap(), bytes);
    }
}
#[test]
fn csharp_03_t06_w09_json_transition_sources() {
    let requests: Value =
        serde_json::from_slice(&fs::read(root().join("requests.json")).unwrap()).unwrap();
    let responses: Value =
        serde_json::from_slice(&fs::read(root().join("responses.json")).unwrap()).unwrap();
    assert_eq!(requests.as_array().unwrap().len(), 2);
    assert_eq!(responses.as_array().unwrap().len(), 2);
    let bundle = b();
    let mut rows = vec![];
    for request in requests.as_array().unwrap() {
        let id = request["id"].as_str().unwrap();
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        assert!(response.get("reject").is_none(), "{id}:{response}");
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
        let p = generate_csharp_practical_ordinary_json_products(vir).unwrap();
        let transitions = p
            .products()
            .iter()
            .enumerate()
            .filter(|(_, p)| p.template_id.as_deref() == Some("mpk.csharp.semantic.transition.v1"))
            .collect::<Vec<_>>();
        assert_eq!(transitions.len(), 1);
        let (index, t) = transitions[0];
        assert_eq!(t.member_names, ["state", "events", "response"]);
        assert_eq!(t.container_payload_member_ids, ["events"]);
        let OrdinaryShape::Product { fields } = &t.carrier.shape else {
            panic!("Transition product");
        };
        let OrdinaryShape::Reference { type_id } = &fields[1].shape else {
            panic!("typed events");
        };
        assert_eq!(
            p.sequences()
                .iter()
                .find(|s| s.carrier.type_id == *type_id)
                .unwrap()
                .capacity,
            4096
        );
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        if id == "scalar-map" {
            assert_eq!(p.collections().len(), 1);
            let join = &p.collections()[0].join_definition;
            assert_eq!(
                cert.declarations
                    .iter()
                    .filter(|d| &cert.name_table[d.name as usize] == join)
                    .count(),
                1
            );
        }
        assert_eq!(
            import_csharp_practical_ordinary_json_products(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        for value in [json!([]), json!(["state"]), json!(["response"])] {
            let mut bad: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
            bad["products"][index]["container_payload_member_ids"] = value;
            assert!(import_csharp_practical_ordinary_json_products(
                &serde_json::to_vec(&bad).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let values = generate_csharp_practical_ordinary_json_values(vir).unwrap();
        let syntax = generate_csharp_practical_ordinary_json_syntax(vir).unwrap();
        assert_eq!(p.primitives(), values.definitions());
        for bytes in [values.certificate_bytes(), syntax.certificate_bytes()] {
            let old = mpk_cert::decode_canonical_certificate(bytes).unwrap();
            let names = old
                .declarations
                .iter()
                .map(|d| old.name_table[d.name as usize].clone())
                .collect();
            super::super::super::super::structural_equivalence_tests::same_definition_closure(
                &old, &cert, &names,
            )
            .unwrap();
        }
        let row = json!({"id":id,"program":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),"terms":cert.term_table.len(),"declarations":cert.declarations.len()});
        eprintln!(
            "Transition {id}: {} terms,{} declarations,{} static transformers",
            cert.term_table.len(),
            cert.declarations.len(),
            row["program"]["static_transformers"]
        );
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        let out = pins();
        if std::env::var_os("MPK_W09_JSON_TRANSITION_OUT").is_some() {
            fs::create_dir_all(&out).unwrap();
            fs::write(out.join(format!("{id}.hex")), hex).unwrap();
        } else {
            assert_eq!(
                fs::read_to_string(out.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        rows.push(row);
    }
    let bytes = serde_json::to_vec_pretty(&rows).unwrap();
    if std::env::var_os("MPK_W09_JSON_TRANSITION_OUT").is_some() {
        fs::write(pins().join("certificates.json"), bytes).unwrap();
    } else {
        assert_eq!(fs::read(pins().join("certificates.json")).unwrap(), bytes);
    }
}
