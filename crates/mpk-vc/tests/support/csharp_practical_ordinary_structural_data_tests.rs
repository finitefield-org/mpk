//! Structural data use-point semantics and explicit remaining operation families.
use super::*;
fn requests() -> Value {
    json!([
        ("i32", "", "return lower==upper?lower:upper;"),
        ("not-equal", "", "return lower!=upper?lower:upper;"),
        ("i64", "", "long a=lower;long b=upper;return a==b?lower:upper;"),
        ("boolean", "", "bool a=lower<upper;bool b=lower>upper;return a==b?lower:upper;"),
        ("enum", "public enum Choice{Zero=0,One=1,High=7}", "Choice a=lower==0?Choice.Zero:Choice.High;Choice b=upper==0?Choice.Zero:Choice.One;return a==b?lower:upper;")
    ].into_iter().flat_map(|(id,extra,body)| {
        let source=format!("namespace Quantifiers;{extra}public static class Entry{{public static int Run(int lower,int upper){{{body}}}}}\n");
        integer_codec_clause_tests::requests_for_source(vec![(id.into(),vec![truth()])],Some(&source)).as_array().unwrap().clone()
    }).collect::<Vec<_>>())
}
fn output(name: &str, bytes: &[u8]) {
    if let Some(path) = std::env::var_os("MPK_W09_STRUCTURAL_DATA_OUT") {
        let root = Path::new(&path);
        fs::create_dir_all(root).unwrap();
        fs::write(root.join(name), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/structural-data");
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
    }
}
#[test]
fn csharp_03_t06_w09_structural_data_requests() {
    let r = requests();
    if let Some(path) = std::env::var_os("MPK_W09_STRUCTURAL_DATA_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&r).unwrap()).unwrap();
    } else {
        assert_eq!(read("ordinary-foundation/structural-data/requests.json"), r);
    }
}
#[test]
fn csharp_03_t06_w09_structural_data_candidates() {
    verify(false);
}
#[test]
fn csharp_03_t06_w09_structural_data_original_source() {
    verify(true);
}
fn verify(runtime: bool) {
    let bundle = b();
    let requests = requests();
    let responses: Value = if let Some(path) = std::env::var_os("MPK_W09_STRUCTURAL_DATA_RESPONSES")
    {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/structural-data/responses.json")
    };
    let mut observations = 0;
    let mut use_points = 0;
    let mut types = BTreeSet::new();
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
        let p = generate_csharp_practical_ordinary_structural_data(vir).unwrap();
        assert!(
            !p.definitions().is_empty(),
            "{id} needs an actual structural source operation"
        );
        let vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        let data = vc.data_vcs();
        let meta: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        assert_eq!(meta["data_vc_sha256"], data.hash());
        let included = p
            .definitions()
            .iter()
            .map(|d| d.source.id.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            p.pending_definition_ids(),
            data.definitions()
                .iter()
                .filter(|d| !included.contains(d.id.as_str()))
                .map(|d| d.id.clone())
                .collect::<Vec<_>>()
        );
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let legacy = generate_csharp_practical_ordinary_relations(vir).unwrap();
        let old = mpk_cert::decode_canonical_certificate(legacy.certificate_bytes()).unwrap();
        let roots = p
            .definitions()
            .iter()
            .map(|d| d.value_definition.clone())
            .collect::<BTreeSet<_>>();
        structural_equivalence_tests::same_definition_closure(&old, &cert, &roots).unwrap();
        let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
        for d in p.definitions() {
            assert_eq!(
                &d.source,
                data.definitions()
                    .iter()
                    .find(|x| x.id == d.source.id)
                    .unwrap()
            );
            types.insert(d.source.signature.argument_type_ids[0].clone());
        }
        for o in p.operations() {
            assert_eq!(
                &o.source,
                data.operations()
                    .iter()
                    .find(|x| x.id == o.source.id)
                    .unwrap()
            );
            let d = p
                .definitions()
                .iter()
                .find(|d| d.source.id == o.source.definition_id)
                .unwrap();
            let ty = &d.source.signature.argument_type_ids[0];
            assert_eq!(d.source.signature.tag, ClosedOperationTag::StructuralEqual);
            if runtime {
                let depth = layouts
                    .carriers()
                    .iter()
                    .find(|c| &c.type_id == ty)
                    .unwrap()
                    .depth;
                let mut samples = if ty.ends_with(".bool.v1") {
                    vec![(0, 0), (0, 1), (1, 0), (1, 1)]
                } else {
                    vec![(0, 0), (1, 1), (7, 7), (0, 1), (1, 0), (1, 7), (7, 1)]
                };
                if ty.ends_with(".i64.v1") {
                    samples.extend([
                        (1u64 << 40, 1u64 << 40),
                        (1 << 40, (1 << 40) + 1),
                        (1 << 63, 0),
                    ]);
                }
                let value = |n: u64| {
                    if depth == 0 {
                        V::Bit(n != 0)
                    } else {
                        V::Cube((0..(1 << depth)).map(|i| n & (1 << i) != 0).collect())
                    }
                };
                for (left, right) in samples {
                    for result in [false, true] {
                        let args = vec![value(left), value(right), V::Bit(result)];
                        assert!(bit(run(
                            &cert,
                            &o.predicates["success_guard"],
                            args.clone()
                        )));
                        assert_eq!(
                            bit(run(&cert, &o.predicates["success_relation"], args.clone())),
                            result == (left == right),
                            "{id}: {ty} {left}/{right} result {result}"
                        );
                        assert_eq!(
                            bit(run(&cert, &o.predicates["success_goal"], args)),
                            result == (left == right)
                        );
                        observations += 1;
                    }
                }
            }
            use_points += 1;
        }
        assert_eq!(
            p.operations().len(),
            data.operations()
                .iter()
                .filter(|o| included.contains(o.definition_id.as_str()))
                .count()
        );
        if id == "not-equal" {
            assert!(data
                .definitions()
                .iter()
                .any(|d| d.signature.id == "boolean.not"
                    && p.pending_definition_ids().contains(&d.id)));
        }
        assert_eq!(
            import_csharp_practical_ordinary_structural_data(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        for field in ["normal_successor_id", "function_id"] {
            let mut broken = meta.clone();
            broken["operations"][0]["source"][field] = json!("changed");
            assert!(import_csharp_practical_ordinary_structural_data(
                &serde_json::to_vec(&broken).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut corrupt = p.certificate_bytes().to_vec();
        let last = corrupt.len() - 1;
        corrupt[last] ^= 1;
        assert!(import_csharp_practical_ordinary_structural_data(
            &p.canonical_bytes(),
            &corrupt,
            vir
        )
        .is_err());
        output(&format!("{id}.json"), &p.canonical_bytes());
        output(
            &format!("{id}.hex"),
            p.certificate_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
                .as_bytes(),
        );
        eprintln!(
            "structural data {id}: {} definitions, {} use points, runtime={runtime}",
            p.definitions().len(),
            p.operations().len()
        );
    }
    assert_eq!(types.len(), 4);
    assert!(use_points >= 5);
    if runtime {
        assert!(observations >= 50);
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
        "structural data: {use_points} use points, {} operand types, {observations} result cases",
        types.len()
    );
}
