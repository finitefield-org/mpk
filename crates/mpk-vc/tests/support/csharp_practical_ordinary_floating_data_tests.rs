//! W03 SSA relations reuse scalar bodies and retain all original use-point links.
use super::*;
fn requests() -> Value {
    json!([
        ("single-add", "float a=lower;float b=upper;float c=a+b;return checked((int)c);"),
        ("single-negate", "float a=lower;float c=-a;return c<a?lower:upper;"),
        ("double-compare", "long a=lower;long b=upper;double x=a;double y=b;return x<y?lower:upper;")
    ].into_iter().flat_map(|(id,body)| {
        let source=format!("namespace Quantifiers;public static class Entry{{public static int Run(int lower,int upper){{{body}}}}}\n");
        integer_codec_clause_tests::requests_for_source(vec![(id.into(),vec![truth()])],Some(&source)).as_array().unwrap().clone()
    }).collect::<Vec<_>>())
}
fn output(name: &str, bytes: &[u8]) {
    if let Some(path) = std::env::var_os("MPK_W09_FLOATING_DATA_OUT") {
        let root = Path::new(&path);
        fs::create_dir_all(root).unwrap();
        fs::write(root.join(name), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/floating-data");
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
    }
}
#[test]
fn csharp_03_t06_w09_floating_data_requests() {
    let requests = requests();
    if let Some(path) = std::env::var_os("MPK_W09_FLOATING_DATA_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&requests).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/floating-data/requests.json"),
            requests
        );
    }
}
#[test]
fn csharp_03_t06_w09_floating_data_original_source() {
    verify(true);
}
#[test]
fn csharp_03_t06_w09_floating_data_candidates() {
    verify(false);
}
fn verify(runtime: bool) {
    let bundle = b();
    let requests = requests();
    let responses: Value = if let Some(path) = std::env::var_os("MPK_W09_FLOATING_DATA_RESPONSES") {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/floating-data/responses.json")
    };
    let mut operation_count = 0;
    let mut observations = 0;
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
        let vir = emitted.vir();
        let program = generate_csharp_practical_ordinary_floating_data(vir).unwrap();
        let vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        let data = vc.data_vcs();
        let metadata: Value = serde_json::from_slice(&program.canonical_bytes()).unwrap();
        assert_eq!(metadata["data_vc_sha256"], data.hash());
        let included = program
            .definitions()
            .iter()
            .map(|d| d.source.id.as_str())
            .collect::<BTreeSet<_>>();
        let expected_pending = data
            .definitions()
            .iter()
            .filter(|d| !included.contains(d.id.as_str()))
            .map(|d| d.id.clone())
            .collect::<Vec<_>>();
        assert_eq!(program.pending_definition_ids(), expected_pending);
        for d in data
            .definitions()
            .iter()
            .filter(|d| expected_pending.contains(&d.id))
        {
            assert_ne!(format!("{:?}", d.family), "FloatingDecimal");
        }
        let cert = mpk_cert::decode_canonical_certificate(program.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let legacy = generate_csharp_practical_ordinary_floating(vir).unwrap();
        let old = mpk_cert::decode_canonical_certificate(legacy.certificate_bytes()).unwrap();
        let mut roots = BTreeSet::new();
        for d in program.definitions() {
            assert_eq!(
                &d.source,
                data.definitions()
                    .iter()
                    .find(|o| o.id == d.source.id)
                    .unwrap()
            );
            assert_eq!(
                &d.scalar,
                legacy
                    .definitions()
                    .iter()
                    .find(|s| s.operation.id == d.source.signature.id)
                    .unwrap()
            );
            roots.insert(d.scalar.result_definition.clone());
            roots.insert(d.scalar.success_definition.clone());
            roots.extend(d.scalar.ordered_failure_definitions.iter().cloned());
        }
        structural_equivalence_tests::same_definition_closure(&old, &cert, &roots).unwrap();
        for o in program.operations() {
            assert_eq!(
                &o.source,
                data.operations()
                    .iter()
                    .find(|source| source.id == o.source.id)
                    .unwrap()
            );
            let d = program
                .definitions()
                .iter()
                .find(|d| d.source.id == o.source.definition_id)
                .unwrap();
            let op = &d.source.signature.id;
            if runtime {
                for (inputs, normal, overflow) in cases(op) {
                    let result_width = physical_width(&d.source.signature.normal_result_type_id);
                    // A wrong high result bit catches truncated comparisons as well as
                    // wrong SSA positions. NaNs/zeros retain their actual bit patterns.
                    for wrong in [false, true] {
                        let actual = normal ^ if wrong { 1 << (result_width - 1) } else { 0 };
                        let values = inputs.iter().copied().chain([actual]).collect::<Vec<_>>();
                        assert_eq!(values.len(), o.source.subjects.len());
                        let args = o
                            .source
                            .subjects
                            .iter()
                            .zip(values)
                            .map(|(s, n)| encode(n, &s.type_id))
                            .collect::<Vec<_>>();
                        assert_eq!(
                            bit(run(&cert, &o.predicates["success_guard"], args.clone())),
                            !overflow,
                            "{op}"
                        );
                        assert_eq!(
                            bit(run(&cert, &o.predicates["success_goal"], args.clone())),
                            overflow || !wrong,
                            "{op}"
                        );
                        if !overflow {
                            assert_eq!(
                                bit(run(&cert, &o.predicates["success_relation"], args.clone())),
                                !wrong,
                                "{op}"
                            );
                        }
                        for (i, check) in o.source.checks.iter().enumerate() {
                            assert_eq!(check.check.id, "exception.overflow");
                            assert!(bit(run(
                                &cert,
                                &o.predicates[&format!("check.{i}.prefix")],
                                args.clone()
                            )));
                            assert_eq!(
                                bit(run(
                                    &cert,
                                    &o.predicates[&format!("check.{i}.failed")],
                                    args.clone()
                                )),
                                overflow
                            );
                            assert_eq!(
                                bit(run(
                                    &cert,
                                    &o.predicates[&format!("check.{i}.guard")],
                                    args.clone()
                                )),
                                overflow
                            );
                        }
                        observations += 1;
                    }
                }
            }
            operation_count += 1;
        }
        assert_eq!(
            program.operations().len(),
            data.operations()
                .iter()
                .filter(|o| included.contains(o.definition_id.as_str()))
                .count()
        );
        assert_eq!(
            import_csharp_practical_ordinary_floating_data(
                &program.canonical_bytes(),
                program.certificate_bytes(),
                vir
            )
            .unwrap(),
            program
        );
        let mut corrupt = metadata.clone();
        corrupt["operations"][0]["source"]["normal_successor_id"] = json!("changed");
        assert!(import_csharp_practical_ordinary_floating_data(
            &serde_json::to_vec(&corrupt).unwrap(),
            program.certificate_bytes(),
            vir
        )
        .is_err());
        let mut bytes = program.certificate_bytes().to_vec();
        let end = bytes.len() - 1;
        bytes[end] ^= 1;
        assert!(import_csharp_practical_ordinary_floating_data(
            &program.canonical_bytes(),
            &bytes,
            vir
        )
        .is_err());
        output(&format!("{id}.json"), &program.canonical_bytes());
        output(
            &format!("{id}.hex"),
            program
                .certificate_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
                .as_bytes(),
        );
        eprintln!(
            "floating data source {id}: {} definitions, {} use points, {} pending",
            program.definitions().len(),
            program.operations().len(),
            program.pending_definition_ids().len()
        );
    }
    output(
        "requests.json",
        &serde_json::to_vec_pretty(&requests).unwrap(),
    );
    output(
        "responses.json",
        &serde_json::to_vec_pretty(&responses).unwrap(),
    );
    eprintln!(
        "floating data source: {operation_count} use points, {observations} result/guard cases"
    );
    assert!(operation_count >= 9);
    assert_eq!(observations > 0, runtime);
}

fn physical_width(ty: &str) -> u32 {
    if ty.ends_with(".bool.v1") {
        1
    } else if ty.ends_with(".f64.v1") || ty.ends_with(".i64.v1") {
        64
    } else {
        assert!(ty.ends_with(".f32.v1") || ty.ends_with(".i32.v1"));
        32
    }
}
fn encode(value: u64, ty: &str) -> V {
    let width = physical_width(ty);
    if width == 1 {
        V::Bit(value != 0)
    } else {
        V::Cube((0..width).map(|i| value & (1u64 << i) != 0).collect())
    }
}
fn cases(op: &str) -> Vec<(Vec<u64>, u64, bool)> {
    match op {
        "numeric.conversion.int32_to_single" => [0, i32::MIN, i32::MAX, 16_777_217]
            .into_iter()
            .map(|v| (vec![v as u32 as u64], (v as f32).to_bits() as u64, false))
            .collect(),
        "numeric.conversion.int64_to_double" => [0, i64::MIN, i64::MAX, 9_007_199_254_740_993]
            .into_iter()
            .map(|v| (vec![v as u64], (v as f64).to_bits(), false))
            .collect(),
        "numeric.conversion.single_to_int32.checked" => [
            7.75f32,
            -0.0,
            -2147483648.0,
            2147483648.0,
            f32::INFINITY,
            f32::NAN,
        ]
        .into_iter()
        .map(|v| {
            (
                vec![v.to_bits() as u64],
                v as i32 as u32 as u64,
                !v.is_finite() || !(-2147483648.0..2147483648.0).contains(&v),
            )
        })
        .collect(),
        "floating.single.add" => [
            (1.5f32, 2.25f32),
            (-0.0, -0.0),
            (f32::MAX, f32::MAX),
            (f32::from_bits(1), f32::from_bits(1)),
        ]
        .into_iter()
        .map(|(a, b)| {
            (
                vec![a.to_bits() as u64, b.to_bits() as u64],
                (a + b).to_bits() as u64,
                false,
            )
        })
        .collect(),
        "floating.single.negate" => [0.0f32, -0.0, f32::INFINITY, f32::from_bits(0x7fc01234)]
            .into_iter()
            .map(|v| {
                (
                    vec![v.to_bits() as u64],
                    (v.to_bits() ^ (1 << 31)) as u64,
                    false,
                )
            })
            .collect(),
        "floating.single.less" => [
            (1.0f32, 2.0f32),
            (-0.0, 0.0),
            (f32::NAN, 1.0),
            (f32::NEG_INFINITY, f32::INFINITY),
        ]
        .into_iter()
        .map(|(a, b)| {
            (
                vec![a.to_bits() as u64, b.to_bits() as u64],
                u64::from(a < b),
                false,
            )
        })
        .collect(),
        "floating.double.less" => [
            (1.0f64, 2.0f64),
            (-0.0, 0.0),
            (f64::NAN, 1.0),
            (f64::NEG_INFINITY, f64::INFINITY),
        ]
        .into_iter()
        .map(|(a, b)| (vec![a.to_bits(), b.to_bits()], u64::from(a < b), false))
        .collect(),
        other => panic!("unexpected native floating operation {other}"),
    }
}
