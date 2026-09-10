//! Decimal contract configuration selection and targeted actual observations.
use super::fixed_codec_clause_tests::alias;
use super::*;
const MODES: [&str; 5] = [
    "ToEven",
    "AwayFromZero",
    "ToZero",
    "ToNegativeInfinity",
    "ToPositiveInfinity",
];
const MAX: &str = "79228162514264337593543950335";
const OVER: &str = "79228162514264337593543950336";
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Config {
    scale: Option<u8>,
    rounding: Option<String>,
}
impl Config {
    fn normalized() -> Self {
        Self {
            scale: None,
            rounding: None,
        }
    }
    fn fixed(scale: u8, mode: &str) -> Self {
        Self {
            scale: Some(scale),
            rounding: Some(mode.into()),
        }
    }
    fn id(&self) -> &str {
        if self.scale.is_none() {
            "decimal.normalized"
        } else {
            "decimal.fixed"
        }
    }
    fn parameters(&self) -> J {
        J::object(vec![
            ("scale", self.scale.map_or(J::Null, |s| J::U64(s.into()))),
            (
                "rounding",
                self.rounding.as_ref().map_or(J::Null, J::string),
            ),
        ])
    }
    fn codec(&self) -> BoundaryCodec {
        BoundaryCodec::new(
            self.id(),
            &ty("decimal"),
            self.scale,
            self.rounding.as_deref(),
        )
        .unwrap()
    }
}
fn literal(token: &str, value: &str) -> J {
    J::object(vec![
        ("tag", J::string("literal")),
        ("type_id", J::string(ty(token))),
        ("value", J::string(value)),
    ])
}
fn result_id() -> String {
    csharp_practical_closed_instance_id(
        &b(),
        &instance(
            "result",
            vec![primitive("decimal"), primitive("parse_error")],
        ),
    )
    .unwrap()
}
fn formatted(config: &Config, value: &str) -> J {
    J::object(vec![
        ("tag", J::string("codec_format")),
        ("type_id", J::string(ty("string"))),
        ("codec_id", J::string(config.id())),
        ("codec_parameters", config.parameters()),
        ("value", literal("decimal", value)),
        ("mode", J::string("canonical")),
    ])
}
fn parsed(config: &Config, text: J, arm: &str) -> J {
    let value = J::object(vec![
        ("tag", J::string("codec_parse")),
        ("type_id", J::string(result_id())),
        ("codec_id", J::string(config.id())),
        ("codec_parameters", config.parameters()),
        ("text", text),
    ]);
    J::object(vec![
        ("tag", J::string("tagged_is")),
        ("type_id", J::string(ty("bool"))),
        ("value", value),
        ("arm", J::string(arm)),
    ])
}
fn requests() -> Value {
    let normalized = Config::normalized();
    let mut rows = vec![];
    let mut clauses = ["0", "1.25", MAX]
        .into_iter()
        .map(|v| parsed(&normalized, formatted(&normalized, v), "ok"))
        .collect::<Vec<_>>();
    for text in ["0.0", "+1", "x", "0.00000000000000000000000000001"] {
        assert!(normalized
            .codec()
            .parse(&text.encode_utf16().collect::<Vec<_>>())
            .is_err());
        clauses.push(parsed(&normalized, literal("string", text), "error"));
    }
    rows.push(("normalized".into(), clauses));
    for mode in MODES {
        let normal = parsed(&normalized, formatted(&normalized, "1.25"), "ok");
        let mut clauses = vec![];
        if mode == "ToEven" {
            clauses.push(normal.clone());
        }
        for scale in 0..=28 {
            let config = Config::fixed(scale, mode);
            clauses.push(parsed(&config, formatted(&config, "1.25"), "ok"));
        }
        if mode != "ToEven" {
            clauses.push(normal);
        }
        clauses.push(parsed(
            &Config::fixed(0, mode),
            literal("string", "x"),
            "error",
        ));
        clauses.push(parsed(
            &Config::fixed(1, mode),
            literal("string", "0.000"),
            "error",
        ));
        // The reference parser trims excess fractional zeros to fit the
        // coefficient. Fixed formatting MAX at scale 28 still parses as MAX.
        let maximum_text = format!("{MAX}.{}", "0".repeat(28));
        let maximum = Config::fixed(28, mode)
            .codec()
            .parse(&maximum_text.encode_utf16().collect::<Vec<_>>())
            .unwrap();
        assert!(
            matches!(maximum, MonomorphicValue::DecimalBits { negative: false, scale: 0, coefficient, .. } if coefficient == MAX)
        );
        clauses.push(parsed(
            &Config::fixed(28, mode),
            formatted(&Config::fixed(28, mode), MAX),
            "ok",
        ));
        assert_eq!(
            Config::fixed(0, mode)
                .codec()
                .parse(&OVER.encode_utf16().collect::<Vec<_>>()),
            Err(ParseErrorArm::Range)
        );
        clauses.push(parsed(
            &Config::fixed(0, mode),
            literal("string", OVER),
            "error",
        ));
        assert_eq!(clauses.len(), 34);
        rows.push((mode.into(), clauses));
    }
    integer_codec_clause_tests::requests_for(rows)
}
#[test]
fn csharp_03_t06_w09_decimal_codec_clause_requests() {
    let requests = requests();
    if let Some(path) = std::env::var_os("MPK_W09_DECIMAL_CODEC_REQUESTS_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&requests).unwrap()).unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/decimal-codec-clauses/requests.json"),
            requests
        );
    }
}
fn output(name: &str, bytes: &[u8]) {
    if let Some(root) = std::env::var_os("MPK_W09_DECIMAL_CODEC_CLAUSES_OUT") {
        let root = std::path::PathBuf::from(root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(name), bytes).unwrap();
    } else {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/decimal-codec-clauses");
        assert_eq!(fs::read(root.join(name)).unwrap(), bytes, "{name}");
    }
}
fn config(d: &ContractDefinition) -> Config {
    let p: Value = serde_json::from_str(&d.parameters).unwrap();
    Config {
        scale: p["codec_parameters"]["scale"]
            .as_u64()
            .map(|n| n.try_into().unwrap()),
        rounding: p["codec_parameters"]["rounding"].as_str().map(Into::into),
    }
}
#[test]
fn csharp_03_t06_w09_decimal_codec_clauses_original_source() {
    let bundle = b();
    let requests = requests();
    let responses: Value = if let Some(path) = std::env::var_os("MPK_W09_DECIMAL_CODEC_RESPONSES") {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    } else {
        read("ordinary-foundation/decimal-codec-clauses/responses.json")
    };
    let runtime_contexts = std::env::var("MPK_W09_DECIMAL_CODEC_RUNTIME_CONTEXTS")
        .ok()
        .map(|v| v.split(',').map(str::to_owned).collect::<BTreeSet<_>>());
    if let Some(contexts) = &runtime_contexts {
        assert!(!contexts.is_empty());
        assert!(contexts
            .iter()
            .all(|id| id == "normalized" || MODES.contains(&id.as_str())));
    }
    let mut coverage = BTreeSet::new();
    let mut orders = BTreeSet::new();
    let mut selected = 0;
    let mut attachments = 0;
    let mut observed_cases = BTreeSet::new();
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
        let p = generate_csharp_practical_ordinary_contract_expressions(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        metadata(&p, &vc);
        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&c).unwrap();
        let normalized = generate_csharp_practical_ordinary_decimal_formats(vir).unwrap();
        let fixed = generate_csharp_practical_ordinary_decimal_fixed_formats(vir).unwrap();
        let parsers = generate_csharp_practical_ordinary_decimal_parsers(vir).unwrap();
        let mut roots = [BTreeSet::new(), BTreeSet::new(), BTreeSet::new()];
        let recipes = vir
            .contract_expressions()
            .iter()
            .flat_map(|e| e.definitions())
            .filter(|d| matches!(d.tag.as_str(), "codec_parse" | "codec_format"))
            .map(|d| (d.name.clone(), d.clone()))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(recipes.len(), if id == "normalized" { 2 } else { 60 });
        for d in recipes.values() {
            let cfg = config(d);
            coverage.insert((d.tag.clone(), cfg.clone()));
            let (family, target) = if d.tag == "codec_parse" {
                let p = parsers
                    .definitions()
                    .iter()
                    .find(|p| {
                        p.codec_id == cfg.id() && p.scale == cfg.scale && p.rounding == cfg.rounding
                    })
                    .unwrap();
                (2, &p.parse_definition)
            } else if cfg.scale.is_none() {
                (0, &normalized.definitions()[0].format_definition)
            } else {
                let f = fixed
                    .definitions()
                    .iter()
                    .find(|f| {
                        Some(f.scale) == cfg.scale && Some(&f.rounding) == cfg.rounding.as_ref()
                    })
                    .unwrap();
                (1, &f.format_definition)
            };
            roots[family].insert(target.clone());
            let decl = c
                .declarations
                .iter()
                .find(|x| c.name_table[x.name as usize] == alias(&d.name))
                .unwrap();
            let mpk_cert::encode::DeclarationKind::Def { value, .. } = decl.kind else {
                panic!()
            };
            let mpk_cert::encode::TermNode::Const { global, .. } = c.term_table[value as usize]
            else {
                panic!()
            };
            assert_eq!(
                &c.name_table[c.declarations[global as usize].name as usize],
                target
            );
        }
        for (bytes, roots) in [
            (normalized.certificate_bytes(), &roots[0]),
            (fixed.certificate_bytes(), &roots[1]),
            (parsers.certificate_bytes(), &roots[2]),
        ] {
            if !roots.is_empty() {
                structural_equivalence_tests::same_definition_closure(
                    &mpk_cert::decode_canonical_certificate(bytes).unwrap(),
                    &c,
                    roots,
                )
                .unwrap();
            }
        }
        if id != "normalized" {
            let position = |name: &str| {
                c.declarations
                    .iter()
                    .position(|d| c.name_table[d.name as usize] == name)
                    .unwrap()
            };
            orders.insert(
                position(&normalized.definitions()[0].format_definition)
                    < position(&fixed.definitions()[0].format_definition),
            );
        }
        let types = generate_csharp_practical_ordinary_carriers(vir)
            .unwrap()
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        // The unchanged codec bodies are covered by exact dependency closures
        // above and their existing semantic suites. Execute only representative
        // new source composition/definedness paths, maximum-value trimming and
        // genuine out-of-range input rejection.
        for d in p.definitions() {
            attachments += 1;
            if runtime_contexts
                .as_ref()
                .is_some_and(|contexts| !contexts.contains(id))
            {
                continue;
            }
            let e = vir
                .contract_expressions()
                .iter()
                .find(|e| e.attachment_sha256() == d.attachment_sha256)
                .unwrap();
            let encoded: Value = serde_json::from_slice(&e.canonical_bytes()).unwrap();
            let expression: Value =
                serde_json::from_str(encoded["canonical_expression"].as_str().unwrap()).unwrap();
            let text = &expression["value"]["text"];
            let case = if text["tag"] == "codec_format" {
                let scale = text["codec_parameters"]["scale"].as_u64();
                let token = text["value"]["value"].as_str().unwrap();
                if id == "normalized" && scale.is_none() && token == "1.25" {
                    "normalized"
                } else if scale == Some(1) && token == "1.25" {
                    "fixed1"
                } else if id == "ToEven" && scale == Some(28) && token == MAX {
                    "maximum"
                } else {
                    continue;
                }
            } else if id == "ToEven" && text["value"] == OVER {
                "range"
            } else if id == "normalized" && text["value"] == "x" {
                "syntax"
            } else {
                continue;
            };
            let case = format!("{id}:{case}");
            assert!(observed_cases.insert(case.clone()));
            eprintln!("decimal source condition {case}");
            let args = d
                .subjects
                .iter()
                .map(|(_, ty)| sparse_cube(types[ty].depth, BTreeSet::new()))
                .collect::<Vec<_>>();
            assert!(
                bit(run(&c, &d.definedness_definition, args.clone())),
                "{id}: definedness"
            );
            assert!(
                bit(run(&c, &d.value_definition, args)),
                "{id}: source value"
            );
            selected += 1;
        }
        assert_eq!(
            import_csharp_practical_ordinary_contract_expressions(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        let mut changed: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        changed["definitions"][0]["result_type"] = json!("changed");
        assert!(import_csharp_practical_ordinary_contract_expressions(
            &serde_json::to_vec(&changed).unwrap(),
            p.certificate_bytes(),
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
        eprintln!("decimal codec clause {id} passed");
    }
    let mut expected = BTreeSet::new();
    for tag in ["codec_parse", "codec_format"] {
        expected.insert((tag.to_owned(), Config::normalized()));
        for mode in MODES {
            for scale in 0..=28 {
                expected.insert((tag.to_owned(), Config::fixed(scale, mode)));
            }
        }
    }
    assert_eq!(coverage, expected);
    assert_eq!(orders, BTreeSet::from([false, true]));
    let mut expected_cases = MODES
        .into_iter()
        .map(|m| format!("{m}:fixed1"))
        .collect::<BTreeSet<_>>();
    expected_cases.extend([
        "normalized:normalized".into(),
        "normalized:syntax".into(),
        "ToEven:maximum".into(),
        "ToEven:range".into(),
    ]);
    if let Some(contexts) = &runtime_contexts {
        expected_cases.retain(|case| contexts.contains(case.split(':').next().unwrap()));
    }
    assert_eq!((attachments, selected), (177, expected_cases.len()));
    assert_eq!(observed_cases, expected_cases);
    eprintln!("decimal codec clauses: {} aliases, {attachments} attachments, {selected} selected source conditions", coverage.len());
    output(
        "requests.json",
        &serde_json::to_vec_pretty(&requests).unwrap(),
    );
    output(
        "responses.json",
        &serde_json::to_vec_pretty(&responses).unwrap(),
    );
}

// Permit same-byte checking independently of the slower semantic observations.
// The source test above compares its fully observed program with these bytes.
#[test]
fn csharp_03_t06_w09_decimal_codec_clause_candidates() {
    let bundle = b();
    let requests = requests();
    assert_eq!(
        read("ordinary-foundation/decimal-codec-clauses/requests.json"),
        requests
    );
    let responses = read("ordinary-foundation/decimal-codec-clauses/responses.json");
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
        let p = generate_csharp_practical_ordinary_contract_expressions(emitted.vir()).unwrap();
        validate_csharp_practical_certificate_structure(
            &mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap(),
        )
        .unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_contract_expressions(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                emitted.vir(),
            )
            .unwrap(),
            p
        );
        output(&format!("{id}.json"), &p.canonical_bytes());
        output(
            &format!("{id}.hex"),
            p.certificate_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
                .as_bytes(),
        );
    }
    assert_eq!(requests.as_array().unwrap().len(), 6);
}
