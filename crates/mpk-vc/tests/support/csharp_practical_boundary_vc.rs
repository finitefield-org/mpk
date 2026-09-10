//! W07: original boundary contracts, typed round trips and hostile evidence.
use super::*;
fn nodes(p: &BoundaryVcProgram) -> usize {
    p.sequents()
        .iter()
        .flat_map(|s| s.assumptions.iter().chain(&s.goals))
        .map(ContractTerm::nodes)
        .sum()
}
fn check(sequents: &[BoundarySequent], names: &[String]) {
    let mut defs = BTreeMap::new();
    fn visit(t: &ContractTerm, defs: &mut BTreeMap<String, String>) {
        match t {
            ContractTerm::Const { name, type_id } => {
                if let Some(old) = defs.insert(name.clone(), type_id.clone()) {
                    assert_eq!(old, *type_id, "{name}");
                }
            }
            ContractTerm::App {
                function, argument, ..
            } => {
                visit(function, defs);
                visit(argument, defs);
            }
            _ => (),
        }
    }
    for s in sequents {
        for t in s.assumptions.iter().chain(&s.goals) {
            super::construction::assert_typed(
                t,
                &s.subjects
                    .iter()
                    .map(|s| s.type_id.clone())
                    .collect::<Vec<_>>(),
            );
            visit(t, &mut defs);
        }
    }
    assert_eq!(
        defs.keys().collect::<Vec<_>>(),
        names.iter().collect::<Vec<_>>()
    );
    assert_eq!(
        sequents
            .iter()
            .map(|s| &s.id)
            .collect::<BTreeSet<_>>()
            .len(),
        sequents.len()
    );
}
fn decode(
    b: &ValidatedFoundationBundle,
    req: &Value,
    response: &Value,
) -> Option<(PracticalArtifactContext, CapturedInputSet, EmittedDataPhase)> {
    let (c, s) = support::replay_context(b, req);
    let facts = response.get("facts")?;
    let source =
        ValidatedDataSource::import_captured_facts(b, &c, &s, &serde_json::to_vec(facts).unwrap())
            .ok()?;
    let e = emit_data_phase(b, &c, &s, &source).ok()?;
    Some((c, s, e))
}
// Independent Boolean evaluation of generated required/null/payload rules.
fn legal(t: &ContractTerm, present: bool, null: bool, payload: bool) -> bool {
    let mut head = t;
    let mut args = vec![];
    while let ContractTerm::App {
        function, argument, ..
    } = head
    {
        args.push(argument.as_ref());
        head = function;
    }
    args.reverse();
    let ContractTerm::Const { name, .. } = head else {
        panic!("unexpected term")
    };
    if name.contains(".Present.") {
        return present;
    }
    if name.contains(".Null.") {
        return null;
    }
    if name.contains(".CanonicalTypedPayload.") {
        return payload;
    }
    let a = args
        .iter()
        .map(|a| legal(a, present, null, payload))
        .collect::<Vec<_>>();
    match name.as_str() {
        "Mpk.CSharp.Bool.true" => true,
        "Mpk.CSharp.Bool.false" => false,
        "Mpk.CSharp.Bool.Not" => !a[0],
        "Mpk.CSharp.Bool.And" => a[0] && a[1],
        "Mpk.CSharp.Bool.Or" => a[0] || a[1],
        _ => panic!("{name}"),
    }
}
// Finite countermodel for the universally quantified encode/reparse equation.
fn roundtrip(t: &ContractTerm, value: i64, corrupt_encoder: bool) -> i64 {
    if matches!(t, ContractTerm::Var { index: 0, .. }) {
        return value;
    }
    let mut head = t;
    let mut args = vec![];
    while let ContractTerm::App {
        function, argument, ..
    } = head
    {
        args.push(argument.as_ref());
        head = function;
    }
    args.reverse();
    let ContractTerm::Const { name, .. } = head else {
        panic!()
    };
    let values = args
        .iter()
        .map(|a| roundtrip(a, value, corrupt_encoder))
        .collect::<Vec<_>>();
    if name.contains(".EncodeOutput.") {
        return values[0] + i64::from(corrupt_encoder);
    }
    if name.contains(".ReparseOutput.") {
        return values[0];
    }
    if name.contains(".SourceObserveEqual.") {
        return i64::from(values[0] == values[1]);
    }
    panic!("unexpected formula {name}")
}
#[test]
fn csharp_03_t06_w07_original_contracts_and_three_state_obligations() {
    let b = b();
    let requests = read("boundary-attachment/requests.json");
    let responses = read("boundary-attachment/responses.json");
    let mut goldens = vec![];
    for r in requests.as_array().unwrap() {
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["id"] == r["id"])
            .unwrap();
        let Some((context, captures, e)) = decode(&b, r, response) else {
            continue;
        };
        let src = PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir: e.vir(),
        };
        let vc = generate_csharp_practical_vc(src).unwrap_or_else(|e| panic!("{} {e:?}", r["id"]));
        let p = vc.boundary_vcs();
        assert_eq!(p.contracts().len(), 1);
        check(p.sequents(), p.definition_names());
        // W09: the transport is a private 1 MiB byte cube, not the bounded
        // application string carrier. This assertion fails on the old route.
        for sequent in p
            .sequents()
            .iter()
            .filter(|s| s.kind == "input_acceptance" || s.kind == "input_field")
        {
            assert_eq!(
                sequent.subjects[0].type_id,
                "Mpk.CSharp.Ordinary.BoundaryDocument"
            );
        }

        assert!(nodes(p) > 0);
        let output = p
            .sequents()
            .iter()
            .find(|s| s.kind == "output_round_trip")
            .unwrap();
        assert!(serde_json::to_string(&output.assumptions)
            .unwrap()
            .contains(".BoundaryReachableReturn."));
        let equation = p
            .sequents()
            .iter()
            .find(|s| s.kind == "output_round_trip")
            .unwrap()
            .goals
            .last()
            .unwrap();
        for value in [-1, 0, 1] {
            assert_eq!(roundtrip(equation, value, false), 1);
            assert_eq!(roundtrip(equation, value, true), 0);
        }
        assert_eq!(
            import_csharp_practical_boundary_vcs(&p.canonical_bytes(), src).unwrap(),
            *p
        );
        let doc = parse_canonical_practical_json(
            PracticalArtifactKind::BoundaryContract,
            p.contracts()[0].as_bytes(),
        )
        .unwrap();
        for field in doc.get("input_fields").unwrap().as_array().unwrap() {
            let fid = field.get("field_id").unwrap().as_str().unwrap();
            let seq = p
                .sequents()
                .iter()
                .find(|s| s.kind == "input_field" && s.id.ends_with(&format!(".{fid}")))
                .unwrap();
            let required = field.get("required") == Some(&PracticalJsonValue::Bool(true));
            let nullable = field.get("nullable") == Some(&PracticalJsonValue::Bool(true));
            let reject = field
                .get("missing_rule")
                .unwrap()
                .get("mode")
                .unwrap()
                .as_str()
                == Some("reject");
            for (present, null) in [(false, false), (true, true), (true, false), (false, true)] {
                for payload in [false, true] {
                    let expected = if !present {
                        !null && !required && !reject
                    } else if null {
                        nullable
                    } else {
                        payload
                    };
                    assert_eq!(
                        legal(&seq.goals[0], present, null, payload),
                        expected,
                        "{fid}"
                    );
                }
            }
        }
        let j: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        for key in ["contracts", "sequents", "definition_names"] {
            let mut m = j.clone();
            m[key].as_array_mut().unwrap().pop();
            assert!(import_csharp_practical_boundary_vcs(
                &super::construction::edited_bytes(&p.canonical_bytes(), &m),
                src
            )
            .is_err());
        }
        goldens
            .push(json!({"id":r["id"],"boundary_sha256":p.hash(),"sequents":p.sequents().len()}));
    }
    assert_eq!(goldens.len(), 15);
    if let Ok(path) = std::env::var("MPK_T06_W07_GOLDEN_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&goldens).unwrap()).unwrap();
    } else {
        assert_eq!(json!(goldens), read("boundary-vc/goldens.json"));
    }
}
#[test]
fn csharp_03_t06_w07_complete_run_relations_and_hostile_serializers() {
    let b = b();
    let requests = read("boundary-output/source-requests.json");
    let responses = read("boundary-output/source-responses.json");
    let mut count = 0;
    let mut goldens = vec![];
    for r in
        requests.as_array().unwrap().iter().filter(|r| {
            r["id"] != "decb5507362a7672c711a5befd63efc1038c2c0a86ee20137a1c4c472e0d1684"
        })
    {
        let response = responses
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["id"] == r["id"])
            .unwrap();
        let (context, captures, e) = decode(&b, r, response).unwrap();
        let unit = e.boundaries()[0].input_fields().is_empty();
        let doc = if unit {
            "{}"
        } else {
            r#"{"field0":{"Number":7,"Text":"😀\ud800","Amount":"1.25","Maybe":{"tag":"some","payload":9}}}"#
        };
        let id = e.boundaries()[0]
            .artifact()
            .value()
            .get("boundary_id")
            .unwrap()
            .as_str()
            .unwrap();
        let input = e
            .capture_boundary_input(
                &b,
                &context,
                &captures,
                BoundaryInputBytes {
                    boundary_id: id,
                    provenance_id: "test.canonical",
                    raw_bytes: doc.as_bytes(),
                    canonical_document: doc.as_bytes(),
                },
            )
            .unwrap();
        let returned = if unit {
            MonomorphicValue::Unit {
                type_id: ty("unit"),
            }
        } else {
            input.arguments()[0].value().clone()
        };
        let run = e
            .capture_boundary_output(&b, &context, &captures, &input, &returned)
            .unwrap();
        let p = e
            .generate_boundary_run_vcs(&b, &context, &captures, &run)
            .unwrap();
        check(p.sequents(), p.definition_names());
        goldens.push(json!({"id":r["id"],"run_sha256":p.hash(),"values":p.values().len(),"sequents":p.sequents().len()}));
        assert_eq!(
            e.import_boundary_run_vcs(&b, &context, &captures, &run, &p.canonical_bytes())
                .unwrap(),
            p
        );
        let goal = p
            .sequents()
            .iter()
            .find(|s| s.kind == "captured_output_round_trip")
            .unwrap()
            .goals
            .last()
            .unwrap();
        let ContractTerm::App {
            function, argument, ..
        } = goal
        else {
            panic!()
        };
        let ContractTerm::App { argument: left, .. } = function.as_ref() else {
            panic!()
        };
        // The complete returned/reparsed literal bodies are compared, never a
        // serializer-provided boolean. Independently changed storage is unequal.
        let get = |t: &ContractTerm| {
            let ContractTerm::Const { name, .. } = t else {
                panic!()
            };
            &p.values().iter().find(|v| &v.name == name).unwrap().value
        };
        assert_eq!(get(left), get(argument));
        let mut bad = serde_json::to_value(get(argument)).unwrap();
        fn change_payload(v: &mut Value) -> bool {
            if v.get("kind").and_then(Value::as_str) == Some("signed") {
                v["value"] = json!("8");
                return true;
            }
            match v {
                Value::Object(m) => m.values_mut().any(change_payload),
                Value::Array(xs) => xs.iter_mut().any(change_payload),
                _ => false,
            }
        }
        if !unit {
            assert!(change_payload(&mut bad));
            assert_ne!(serde_json::to_value(get(left)).unwrap(), bad);
            let mut m: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
            m["values"][0]["value"] = bad;
            assert!(e
                .import_boundary_run_vcs(
                    &b,
                    &context,
                    &captures,
                    &run,
                    &super::construction::edited_bytes(&p.canonical_bytes(), &m)
                )
                .is_err());
        }
        let j: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        for key in [
            "input_capture",
            "output_capture",
            "manifest",
            "artifacts",
            "canonical_input",
            "canonical_output",
            "boundary_contract",
            "source_ir_sha256",
            "boundary_program_sha256",
        ] {
            let mut m = j.clone();
            m[key] = json!("tampered");
            assert!(
                e.import_boundary_run_vcs(
                    &b,
                    &context,
                    &captures,
                    &run,
                    &super::construction::edited_bytes(&p.canonical_bytes(), &m)
                )
                .is_err(),
                "{key}"
            );
        }
        let mut m = j.clone();
        m["literal_value_cells"] = json!(0);
        assert!(e
            .import_boundary_run_vcs(
                &b,
                &context,
                &captures,
                &run,
                &super::construction::edited_bytes(&p.canonical_bytes(), &m)
            )
            .is_err());
        for key in ["values", "relations"] {
            let mut m = j.clone();
            if key == "values" {
                m[key].as_array_mut().unwrap().pop();
            } else {
                m[key]["sequents"].as_array_mut().unwrap().pop();
            }
            assert!(e
                .import_boundary_run_vcs(
                    &b,
                    &context,
                    &captures,
                    &run,
                    &super::construction::edited_bytes(&p.canonical_bytes(), &m)
                )
                .is_err());
        }
        for bytes in [
            b"{ }".as_slice(),
            b"{\"extra\":0}".as_slice(),
            b"{\"x\":0,\"x\":1}".as_slice(),
        ] {
            assert!(e
                .import_boundary_output_run(
                    &b,
                    &context,
                    &captures,
                    &input,
                    &returned,
                    BoundaryOutputEvidence {
                        canonical_document: bytes,
                        capture: run.capture().artifact().canonical_bytes(),
                        manifest: run.manifest().canonical_bytes(),
                        artifacts: run.artifacts().canonical_bytes()
                    }
                )
                .is_err());
        }
        // A run from a different original-source capture cannot be used here.
        let other = &requests
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["id"] != r["id"])
            .unwrap();
        let (other_context, other_captures) = support::replay_context(&b, other);
        assert!(e
            .generate_boundary_run_vcs(&b, &other_context, &other_captures, &run)
            .is_err());
        count += 1;
    }
    assert_eq!(count, 2);
    if let Ok(path) = std::env::var("MPK_T06_W07_RUN_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&goldens).unwrap()).unwrap();
    } else {
        assert_eq!(json!(goldens), read("boundary-vc/run-goldens.json"));
    }
}

#[test]
fn csharp_03_t06_w07_deterministic_byte_mutations_keep_exact_evidence() {
    let b = b();
    let requests = read("boundary-attachment/requests.json");
    let responses = read("boundary-attachment/responses.json");
    let r = &requests[0];
    let response = responses
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["id"] == r["id"])
        .unwrap();
    let (ctx, captures, e) = decode(&b, r, response).unwrap();
    let doc = br#"{"field0":7,"field1":"0"}"#;
    let id = e.boundaries()[0]
        .artifact()
        .value()
        .get("boundary_id")
        .unwrap()
        .as_str()
        .unwrap();
    fn bytes<'a>(id: &'a str, d: &'a [u8]) -> BoundaryInputBytes<'a> {
        BoundaryInputBytes {
            boundary_id: id,
            provenance_id: "test.canonical",
            raw_bytes: d,
            canonical_document: d,
        }
    }
    let original = e
        .capture_boundary_input(&b, &ctx, &captures, bytes(id, doc))
        .unwrap();
    let mut accepted = 0;
    let mut rejected = 0;
    for at in 0..doc.len() {
        for byte in [
            0, 255, b' ', b'0', b'1', b'"', b'\\', b'{', b'}', b':', b',',
        ] {
            if byte == doc[at] {
                continue;
            }
            let mut m = doc.to_vec();
            m[at] = byte;
            match e.capture_boundary_input(&b, &ctx, &captures, bytes(id, &m)) {
                Ok(run) => {
                    accepted += 1;
                    let parsed: Value = serde_json::from_slice(&m).unwrap();
                    assert_eq!(parsed.as_object().unwrap().len(), 2);
                    assert!(parsed["field0"].is_i64());
                    assert!(parsed["field1"].is_string());
                    assert_ne!(
                        run.capture().artifact().hash(),
                        original.capture().artifact().hash()
                    );
                }
                Err(_) => rejected += 1,
            }
            assert!(e
                .import_boundary_input_run(
                    &b,
                    &ctx,
                    &captures,
                    bytes(id, &m),
                    BoundaryInputEvidence {
                        capture: original.capture().artifact().canonical_bytes(),
                        manifest: original.manifest().canonical_bytes(),
                        artifacts: original.artifacts().canonical_bytes()
                    }
                )
                .is_err());
        }
    }
    assert!(accepted > 0 && rejected > 200);
}
