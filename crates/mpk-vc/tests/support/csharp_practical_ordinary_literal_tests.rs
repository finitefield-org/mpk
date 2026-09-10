//! Original-source literal definitions, exact linkage and independent storage observations.
use super::*;
use core_eval::{bit as observed_bit, run, V};

#[test]
fn csharp_03_t06_w09_literals_original_source_certificates_and_semantics() {
    let bundle = b();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/literal-definitions");
    let output = std::env::var_os("MPK_W09_LITERALS_OUT").map(std::path::PathBuf::from);
    if let Some(dir) = &output {
        fs::create_dir_all(dir).unwrap();
    }
    let mut rows = vec![];
    let mut total_values = 0;
    let mut total_bindings = 0;
    let mut observations = 0;
    let mut kinds = BTreeSet::new();
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
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
        let vir = emitted.vir();
        let p = generate_csharp_practical_ordinary_literals(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let certificate = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&certificate).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_literals(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
        let types: BTreeMap<_, _> = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect();
        let mut expected = vec![];
        for function in vir.functions() {
            for block in &function.blocks {
                for v in &block.literal_values {
                    expected.push((
                        OrdinaryLiteralOrigin::BlockValue {
                            function_id: function.id.clone(),
                            block_id: block.node.id.clone(),
                            value_id: v.result.id.clone(),
                        },
                        v.value.clone(),
                    ));
                }
                for v in &block.exception_values {
                    expected.push((
                        OrdinaryLiteralOrigin::FailedCheck {
                            function_id: function.id.clone(),
                            block_id: block.node.id.clone(),
                            check_id: v.check_id.clone(),
                        },
                        v.value.clone(),
                    ));
                }
            }
        }
        assert_eq!(expected.len(), p.bindings().len());
        for ((origin, value), binding) in expected.iter().zip(p.bindings()) {
            assert_eq!(origin, &binding.origin);
            let def = p
                .definitions()
                .iter()
                .find(|d| d.name == binding.definition)
                .unwrap();
            assert_eq!(value, &def.value);
            assert_eq!(value.type_id(), def.carrier.type_id);
        }
        let mut source_observations = 0;
        for def in p.definitions() {
            kinds.insert(
                serde_json::to_value(&def.value).unwrap()["kind"]
                    .as_str()
                    .unwrap()
                    .to_owned(),
            );
            // The reference encoder independently lays out values as flat bits.
            // Very deep constant shapes need sparse probes; this corpus stays bounded.
            assert!(
                def.carrier.depth < 24,
                "{id}: add sparse reference observations"
            );
            let expected = relation_tests::storage(&def.value, &types);
            for (index, &value) in expected.iter().enumerate() {
                if expected.len() > 4096
                    && !value
                    && index >= 128
                    && index + 128 < expected.len()
                    && index % 4093 != 0
                {
                    continue;
                }
                let args = (0..def.carrier.depth)
                    .map(|bit| V::Bit(index & (1 << bit) != 0))
                    .collect();
                assert_eq!(
                    observed_bit(run(&certificate, &def.name, args)),
                    value,
                    "{id} {} bit {index}",
                    def.name
                );
                source_observations += 1;
            }
        }
        let metadata: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        for field in [
            "schema",
            "source_ir_sha256",
            "foundation_sha256",
            "definitions",
            "bindings",
            "certificate_sha256",
        ] {
            let mut changed = metadata.clone();
            changed[field] = json!("forged");
            assert!(
                import_csharp_practical_ordinary_literals(
                    &serde_json::to_vec(&changed).unwrap(),
                    p.certificate_bytes(),
                    vir
                )
                .is_err(),
                "{id} {field}"
            );
        }
        let mut bytes = p.certificate_bytes().to_vec();
        *bytes.last_mut().unwrap() ^= 1;
        assert!(
            import_csharp_practical_ordinary_literals(&p.canonical_bytes(), &bytes, vir).is_err()
        );
        if let Some((m, c)) = &previous {
            assert!(import_csharp_practical_ordinary_literals(m, c, vir).is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(dir) = &output {
            fs::write(dir.join(format!("{id}.hex")), &hex).unwrap();
        } else {
            assert_eq!(
                fs::read_to_string(fixture.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        rows.push(json!({"id":id,"program":metadata,"terms":certificate.term_table.len(),"declarations":certificate.declarations.len(),"observations":source_observations}));
        total_values += p.definitions().len();
        total_bindings += p.bindings().len();
        observations += source_observations;
        eprintln!(
            "literal source {id}: {} definitions, {} bindings, {source_observations} observations",
            p.definitions().len(),
            p.bindings().len()
        );
    }
    assert_eq!(rows.len(), 64);
    assert_eq!(
        (total_values, total_bindings, observations),
        (88, 105, 6550)
    );
    assert_eq!(kinds.len(), 12);
    let inventory = json!({"sources":rows,"definitions":total_values,"bindings":total_bindings,"observations":observations,"kinds":kinds});
    if let Some(dir) = &output {
        fs::write(
            dir.join("certificates.json"),
            serde_json::to_vec_pretty(&inventory).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/literal-definitions/certificates.json"),
            inventory
        );
    }
    eprintln!("literal totals: {total_values} definitions, {total_bindings} bindings, {observations} observations");
}

#[test]
fn csharp_03_t06_w09_literal_source_requests() {
    let bundle = b();
    let cases = [
        ("bool", "bool", "true"),
        ("i8", "sbyte", "-128"),
        ("u8", "byte", "255"),
        ("i16", "short", "-32768"),
        ("u16", "ushort", "65535"),
        ("i32", "int", "-2147483648"),
        ("u32", "uint", "4294967295u"),
        ("i64", "long", "-9223372036854775808L"),
        ("u64", "ulong", "18446744073709551615UL"),
        ("char", "char", "'\\uD800'"),
        ("string", "string", "\"a\\0\\uD800\\uDC00\\uffff\""),
        ("f32", "float", "-0.0f"),
        ("f64", "double", "1.5d"),
        ("decimal", "decimal", "1.2300m"),
        ("nullable-none", "int?", "null"),
        ("nullable-some", "int?", "42"),
        ("array", "int[]", "new int[]{-1,0,2147483647}"),
        ("enum", "Currency", "Currency.Jpy"),
        ("product", "Snapshot", "default(Snapshot)"),
        ("f32-nan", "float", "(0.0f / 0.0f)"),
        ("f64-infinity", "double", "(1.0d / 0.0d)"),
    ];
    let mut requests = vec![];
    for (id, ty, expression) in cases {
        let declarations = match ty {
            "Currency" => "public enum Currency:short{Usd=1,Jpy=7}",
            "Snapshot" => "public readonly struct Snapshot{public readonly bool Tag;public readonly decimal Amount;public readonly int? Maybe;}",
            _ => "",
        };
        let code = format!("namespace LiteralCases;{declarations}public static class Entry{{public static {ty} Run(){{return {expression};}}}}\n");
        let owner = csharp_practical_declaration_id(&json!({"kind":"type","namespace":"LiteralCases","owner":"","name":"Entry","parameter_type_ids":[],"result_type_id":""})).unwrap();
        let result_type = match ty {
            "Currency" | "Snapshot" => csharp_practical_declaration_id(&json!({"kind":"type","namespace":"LiteralCases","owner":"","name":ty,"parameter_type_ids":[],"result_type_id":""})).unwrap(),
            "int?" => csharp_practical_closed_instance_id(&bundle, &instance("option", vec![primitive("i32")])).unwrap(),
            "int[]" => csharp_practical_closed_instance_id(&bundle, &instance("bounded_sequence", vec![primitive("i32")])).unwrap(),
            _ => format!("mpk.csharp.value.{}.v1", match ty {
                "sbyte" => "i8", "byte" => "u8", "short" => "i16", "ushort" => "u16",
                "int" => "i32", "uint" => "u32", "long" => "i64", "ulong" => "u64", "float" => "f32", "double" => "f64", _ => ty,
            }),
        };
        // Bind the root to the concrete result identity reconstructed by capture.
        let root = csharp_practical_declaration_id(&json!({"kind":"method","namespace":"LiteralCases","owner":owner,"name":"Run","parameter_type_ids":[],"result_type_id":result_type})).unwrap();
        let (context, captures) = support::context(&bundle, &root, code.as_bytes());
        requests.push(json!({"id":format!("literal-{id}"),"compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.kind()==OriginalInputKind::Source{"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()}));
    }
    let bytes = serde_json::to_vec_pretty(&requests).unwrap();
    if let Some(path) = std::env::var_os("MPK_W09_LITERAL_REQUESTS_OUT") {
        fs::write(path, bytes).unwrap();
    } else {
        assert_eq!(fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/csharp-03/ordinary-foundation/literal-sources/requests.json")).unwrap(),bytes);
    }
}
fn sources() -> Vec<(String, Value, Value)> {
    let mut sources = structural_foundation_tests::sources();
    let requests = read("ordinary-foundation/literal-sources/requests.json");
    let responses = read("ordinary-foundation/literal-sources/responses.json");
    assert_eq!(requests.as_array().unwrap().len(), 21);
    assert_eq!(responses.as_array().unwrap().len(), 21);
    for (request, response) in requests
        .as_array()
        .unwrap()
        .iter()
        .zip(responses.as_array().unwrap())
    {
        assert_eq!(request["id"], response["id"]);
        assert!(
            response.get("reject").is_none(),
            "{}: {response}",
            request["id"]
        );
        sources.push((
            request["id"].as_str().unwrap().into(),
            request.clone(),
            response["facts"].clone(),
        ));
    }
    sources
}

#[path = "csharp_practical_ordinary_hex_codec_tests.rs"]
pub(super) mod hex_codec_tests;

#[path = "csharp_practical_ordinary_integer_format_tests.rs"]
pub(super) mod integer_format_tests;

#[path = "csharp_practical_ordinary_integer_parse_tests.rs"]
pub(super) mod integer_parse_tests;

#[path = "csharp_practical_ordinary_decimal_format_tests.rs"]
mod decimal_format_tests;

#[path = "csharp_practical_ordinary_decimal_fixed_format_tests.rs"]
mod decimal_fixed_format_tests;

#[path = "csharp_practical_ordinary_decimal_parse_tests.rs"]
mod decimal_parse_tests;

#[path = "csharp_practical_ordinary_calendar_codec_tests.rs"]
pub(super) mod calendar_codec_tests;

#[path = "csharp_practical_ordinary_boundary_document_tests.rs"]
mod boundary_document_tests;

#[path = "csharp_practical_ordinary_boundary_utf8_tests.rs"]
mod boundary_utf8_tests;

#[path = "csharp_practical_ordinary_json_string_tests.rs"]
mod json_string_tests;

#[path = "csharp_practical_ordinary_json_string_parse_tests.rs"]
mod json_string_parse_tests;

#[path = "csharp_practical_ordinary_boundary_fragment_tests.rs"]
mod boundary_fragment_tests;

#[path = "csharp_practical_ordinary_json_keyword_tests.rs"]
mod json_keyword_tests;

#[path = "csharp_practical_ordinary_json_token_tests.rs"]
mod json_token_tests;

#[path = "csharp_practical_ordinary_json_raw_limit_tests.rs"]
mod json_raw_limit_tests;
