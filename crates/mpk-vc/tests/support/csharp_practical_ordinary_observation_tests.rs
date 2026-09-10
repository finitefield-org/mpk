//! Source observation is distinct from IEEE value equality. These are ordinary
//! function checks; reconstruction/public-invariant theorems remain required.
use super::*;
use core_eval::{bit as observed_bit, run, sparse_cube, V};

fn type_id(name: &str) -> String {
    csharp_practical_declaration_id(&json!({"kind":"type","namespace":"ObservationCases","owner":"","name":name,"parameter_type_ids":[],"result_type_id":""})).unwrap()
}

#[test]
fn csharp_03_t06_w09_observation_source_requests() {
    let bundle = b();
    let own = type_id("Snapshot");
    let root = csharp_practical_declaration_id(&json!({"kind":"method","namespace":"ObservationCases","owner":type_id("Entry"),"name":"Run","parameter_type_ids":[own.clone()],"result_type_id":own})).unwrap();
    let code = "namespace ObservationCases;public readonly struct Snapshot{public readonly float Single;public readonly double Double;public readonly decimal Amount;public readonly float? Maybe;public readonly float[] Items;public readonly bool Tag;public readonly int Other;}public static class Entry{public static Snapshot Run(Snapshot value){return value;}}\n";
    let (context, captures) = support::context(&bundle, &root, code.as_bytes());
    let requests = json!([{"id":"complete-snapshot","compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.kind()==OriginalInputKind::Source{"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()}]);
    let bytes = serde_json::to_vec_pretty(&requests).unwrap();
    if let Some(out) = std::env::var_os("MPK_W09_OBSERVATION_REQUESTS_OUT") {
        fs::write(out, bytes).unwrap();
    } else {
        assert_eq!(fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/csharp-03/ordinary-foundation/observation-sources/requests.json")).unwrap(), bytes);
    }
}

pub(super) fn sources() -> Vec<(String, Value, Value)> {
    let mut sources = collection_tests::sources()
        .into_iter()
        .filter(|(id, _, _)| {
            matches!(
                id.as_str(),
                "binding-vc-money" | "exception-vc-allowed" | "extra-bool-float-map"
            )
        })
        .collect::<Vec<_>>();
    let requests = read("ordinary-foundation/observation-sources/requests.json");
    let responses = read("ordinary-foundation/observation-sources/responses.json");
    assert_eq!(requests.as_array().unwrap().len(), 1);
    assert_eq!(responses.as_array().unwrap().len(), 1);
    assert_eq!(requests[0]["id"], responses[0]["id"]);
    assert!(responses[0].get("reject").is_none());
    sources.push((
        "complete-snapshot".into(),
        requests[0].clone(),
        responses[0]["facts"].clone(),
    ));
    sources
}

#[test]
fn csharp_03_t06_w09_observations_original_source_certificates() {
    let bundle = b();
    let mut rows = vec![];
    let output = std::env::var_os("MPK_W09_OBSERVATIONS_OUT").map(std::path::PathBuf::from);
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/source-observations");
    for (id, row, facts) in sources() {
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_observations(emitted.vir()).unwrap();
        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&c).unwrap();
        let data: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        let layouts = generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap();
        let covered = p
            .definitions()
            .iter()
            .map(|d| d.carrier.type_id.as_str())
            .chain(
                data["erased_construction_type_ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap()),
            )
            .collect::<BTreeSet<_>>();
        assert_eq!(
            covered,
            layouts
                .carriers()
                .iter()
                .map(|c| c.type_id.as_str())
                .collect()
        );
        assert_eq!(
            import_csharp_practical_ordinary_observations(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                emitted.vir()
            )
            .unwrap(),
            p
        );
        for field in [
            "schema",
            "source_ir_sha256",
            "foundation_sha256",
            "definitions",
            "erased_construction_type_ids",
            "static_transformers",
            "certificate_sha256",
        ] {
            let mut changed = data.clone();
            changed[field] = json!("forged");
            assert!(
                import_csharp_practical_ordinary_observations(
                    &serde_json::to_vec(&changed).unwrap(),
                    p.certificate_bytes(),
                    emitted.vir()
                )
                .is_err(),
                "{field}"
            );
        }
        let mut changed = p.certificate_bytes().to_vec();
        *changed.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_observations(
            &p.canonical_bytes(),
            &changed,
            emitted.vir()
        )
        .is_err());
        let file = format!("{id}.hex");
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(output) = &output {
            fs::create_dir_all(output).unwrap();
            fs::write(output.join(&file), hex).unwrap();
        } else {
            assert_eq!(fs::read_to_string(fixture.join(&file)).unwrap(), hex);
        }
        eprintln!(
            "source observation {id}: {} concrete types, {} terms",
            p.definitions().len(),
            c.term_table.len()
        );
        rows.push(json!({"id":id,"file":file,"terms":c.term_table.len(),"declarations":c.declarations.len(),"metadata":data}));
    }
    assert_eq!(rows.len(), 4);
    if let Some(output) = output {
        fs::write(
            output.join("certificates.json"),
            serde_json::to_vec_pretty(&rows).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/source-observations/certificates.json"),
            json!(rows)
        );
    }
}

fn input(value: &MonomorphicValue, types: &BTreeMap<String, OrdinaryCarrier>) -> V {
    let bits = relation_tests::storage(value, types);
    sparse_cube(
        bits.len().trailing_zeros(),
        bits.into_iter()
            .enumerate()
            .filter_map(|(i, b)| b.then_some(i))
            .collect(),
    )
}

fn field_mut<'a>(value: &'a mut MonomorphicValue, name: &str) -> &'a mut MonomorphicValue {
    let MonomorphicValue::Product { fields, .. } = value else {
        panic!("source product")
    };
    fields
        .iter_mut()
        .find(|f| f.name == name)
        .unwrap()
        .value
        .as_mut()
}
fn decimal(coefficient: u128, scale: u8, negative: bool) -> MonomorphicValue {
    MonomorphicValue::DecimalBits {
        type_id: ty("decimal"),
        coefficient: coefficient.to_string(),
        scale,
        negative,
    }
}
fn float(bits: &str) -> MonomorphicValue {
    MonomorphicValue::F32Bits {
        type_id: ty("f32"),
        bits: bits.into(),
    }
}
fn double(bits: &str) -> MonomorphicValue {
    MonomorphicValue::F64Bits {
        type_id: ty("f64"),
        bits: bits.into(),
    }
}

#[test]
fn csharp_03_t06_w09_observations_complete_snapshot_semantics() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(check_snapshot)
        .unwrap()
        .join()
        .unwrap();
}
fn check_snapshot() {
    let bundle = b();
    let (_, row, facts) = sources()
        .into_iter()
        .find(|(id, _, _)| id == "complete-snapshot")
        .unwrap();
    let (context, captures) = support::replay_context(&bundle, &row);
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(&facts).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    let p = generate_csharp_practical_ordinary_observations(emitted.vir()).unwrap();
    let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    let types = p
        .definitions()
        .iter()
        .map(|d| (d.carrier.type_id.clone(), d.carrier.clone()))
        .collect::<BTreeMap<_, _>>();
    let semantic = generate_csharp_practical_ordinary_relations(emitted.vir()).unwrap();
    let semantic_c = mpk_cert::decode_canonical_certificate(semantic.certificate_bytes()).unwrap();
    let check = |label: &str, left: &MonomorphicValue, right: &MonomorphicValue, expected: bool| {
        validate_monomorphic_value(
            &bundle,
            emitted.closure().roots(),
            emitted.closure().closed(),
            left,
        )
        .unwrap();
        validate_monomorphic_value(
            &bundle,
            emitted.closure().roots(),
            emitted.closure().closed(),
            right,
        )
        .unwrap();
        assert_eq!(left.type_id(), right.type_id());
        let definition = &p
            .definitions()
            .iter()
            .find(|d| d.carrier.type_id == left.type_id())
            .unwrap()
            .equality_definition;
        eprintln!("source observation: {label}");
        assert_eq!(
            observed_bit(run(
                &c,
                definition,
                vec![input(left, &types), input(right, &types)]
            )),
            expected,
            "{label}"
        );
    };
    let mut observations = 0;
    for (left, right, equal, semantic_equal) in [
        (float("7fc00000"), float("7fc00000"), true, false),
        (float("7fc00000"), float("7fc00001"), false, false),
        (float("7f800001"), float("7fc00001"), false, false),
        (float("00000000"), float("80000000"), false, true),
        (float("7f800000"), float("7f800000"), true, true),
        (
            double("fff8000000000001"),
            double("fff8000000000001"),
            true,
            false,
        ),
        (
            double("7ff8000000000001"),
            double("fff8000000000001"),
            false,
            false,
        ),
        (
            double("7ff0000000000001"),
            double("7ff8000000000001"),
            false,
            false,
        ),
        (
            double("0000000000000000"),
            double("8000000000000000"),
            false,
            true,
        ),
        (
            double("fff0000000000000"),
            double("7ff0000000000000"),
            false,
            false,
        ),
    ] {
        check("IEEE bits versus value equality", &left, &right, equal);
        let definition = &semantic
            .definitions()
            .iter()
            .find(|d| d.carrier.type_id == left.type_id())
            .unwrap()
            .equality_definition;
        assert_eq!(
            observed_bit(run(
                &semantic_c,
                definition,
                vec![input(&left, &types), input(&right, &types)]
            )),
            semantic_equal
        );
        observations += 1;
    }
    let snapshot = type_id("Snapshot");
    let mut baseline =
        relation_tests::sample(&snapshot, 0, &types, &facts, emitted.closure().closed());
    *field_mut(&mut baseline, "Single") = float("7fc00000");
    *field_mut(&mut baseline, "Double") = double("fff8000000000001");
    *field_mut(&mut baseline, "Amount") = decimal(100, 2, false);
    let nullable_type = field_mut(&mut baseline, "Maybe").type_id().to_owned();
    let OrdinaryShape::Sum { arms } = &types[&nullable_type].shape else {
        panic!("nullable sum")
    };
    let some = arms
        .iter()
        .find(|a| a.fields.len() == 1)
        .unwrap()
        .id
        .clone();
    let none = arms
        .iter()
        .find(|a| a.fields.is_empty())
        .unwrap()
        .id
        .clone();
    assert_eq!((some.as_str(), none.as_str()), ("some", "none"));
    *field_mut(&mut baseline, "Maybe") = MonomorphicValue::Option {
        type_id: nullable_type.clone(),
        arm: OptionArm::Some,
        value: Some(Box::new(float("7fc00001"))),
    };
    let array_type = field_mut(&mut baseline, "Items").type_id().to_owned();
    *field_mut(&mut baseline, "Items") = MonomorphicValue::Array {
        type_id: array_type.clone(),
        elements: vec![float("7fc00000"), float("80000000")],
    };
    *field_mut(&mut baseline, "Tag") = MonomorphicValue::Bool {
        type_id: ty("bool"),
        value: false,
    };
    *field_mut(&mut baseline, "Other") = MonomorphicValue::Signed {
        type_id: ty("i32"),
        value: "0".into(),
    };
    check(
        "complete NaN-containing source snapshot is reflexive",
        &baseline,
        &baseline,
        true,
    );
    observations += 1;
    for (label, field, value, equal) in [
        ("single NaN payload", "Single", float("7fc00001"), false),
        (
            "double NaN sign",
            "Double",
            double("7ff8000000000001"),
            false,
        ),
        (
            "equivalent decimal cohort",
            "Amount",
            decimal(1, 0, false),
            true,
        ),
        (
            "different decimal value",
            "Amount",
            decimal(101, 2, false),
            false,
        ),
        (
            "nullable active NaN payload",
            "Maybe",
            MonomorphicValue::Option {
                type_id: nullable_type.clone(),
                arm: OptionArm::Some,
                value: Some(Box::new(float("7fc00002"))),
            },
            false,
        ),
        (
            "nullable distinct arms",
            "Maybe",
            MonomorphicValue::Option {
                type_id: nullable_type,
                arm: OptionArm::None,
                value: None,
            },
            false,
        ),
        (
            "array NaN payload",
            "Items",
            MonomorphicValue::Array {
                type_id: array_type.clone(),
                elements: vec![float("7fc00001"), float("80000000")],
            },
            false,
        ),
        (
            "array signed zero",
            "Items",
            MonomorphicValue::Array {
                type_id: array_type.clone(),
                elements: vec![float("7fc00000"), float("00000000")],
            },
            false,
        ),
        (
            "array length",
            "Items",
            MonomorphicValue::Array {
                type_id: array_type,
                elements: vec![float("7fc00000")],
            },
            false,
        ),
        (
            "source tag",
            "Tag",
            MonomorphicValue::Bool {
                type_id: ty("bool"),
                value: true,
            },
            false,
        ),
        (
            "other field remains observable with false tag",
            "Other",
            MonomorphicValue::Signed {
                type_id: ty("i32"),
                value: "1".into(),
            },
            false,
        ),
    ] {
        let mut changed = baseline.clone();
        *field_mut(&mut changed, field) = value;
        check(label, &baseline, &changed, equal);
        observations += 1;
    }
    let mut negative_zero = baseline.clone();
    *field_mut(&mut negative_zero, "Amount") = decimal(0, 28, true);
    let mut positive_zero = baseline.clone();
    *field_mut(&mut positive_zero, "Amount") = decimal(0, 0, false);
    check(
        "decimal zero sign/cohort is unobservable",
        &negative_zero,
        &positive_zero,
        true,
    );
    observations += 1;
    assert_eq!(observations, 23);
    eprintln!("source observation: {observations} concrete counterexample/boundary pairs; 10 independent semantic-equality contrasts");
}
