//! Generated Money functions are ordinary definitions, not source-body proofs.
use super::*;

pub(super) fn sources() -> Vec<(String, Value, Value)> {
    let mut sources = relation_tests::sources();
    let requests = read("ordinary-foundation/money-sources/requests.json");
    let responses = read("ordinary-foundation/money-sources/responses.json");
    assert_eq!(requests.as_array().unwrap().len(), 1);
    assert_eq!(responses.as_array().unwrap().len(), 1);
    assert_eq!(requests[0]["id"], responses[0]["id"]);
    assert!(responses[0].get("reject").is_none());
    sources.push((
        requests[0]["id"].as_str().unwrap().into(),
        requests[0].clone(),
        responses[0]["facts"].clone(),
    ));
    sources
}

#[test]
fn csharp_03_t06_w09_money_original_source_certificates() {
    let bundle = b();
    let mut metrics = vec![];
    let output = std::env::var_os("MPK_W09_MONEY_OUT").map(std::path::PathBuf::from);
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/money-operations");
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
        let expected = emitted
            .closure()
            .closed()
            .entries()
            .iter()
            .filter(|e| e["template_id"] == "mpk.csharp.semantic.money.v1")
            .collect::<Vec<_>>();
        if expected.is_empty() {
            continue;
        }
        eprintln!(
            "money generation {id}: {} concrete instances",
            expected.len()
        );
        let p = generate_csharp_practical_ordinary_money(emitted.vir()).unwrap();
        assert_eq!(p.definitions().len(), expected.len());
        for (d, entry) in p.definitions().iter().zip(expected) {
            assert_eq!(d.carrier.type_id, entry["instance_id"].as_str().unwrap());
            assert_eq!(d.operations.len(), 10);
            for (op, frozen) in d
                .operations
                .iter()
                .zip(entry["operation_definitions"].as_array().unwrap())
            {
                assert_eq!(op.operation_id, frozen["id"].as_str().unwrap());
                assert_eq!(json!(op.argument_type_ids), frozen["argument_type_ids"]);
                assert_eq!(
                    op.result_type_id,
                    frozen["normal_result_type_id"].as_str().unwrap()
                );
                assert_eq!(
                    json!(op.failures.iter().map(|f| &f.label).collect::<Vec<_>>()),
                    frozen["error_precedence"]
                );
                assert_eq!(
                    op.currency_predicate_argument_type_id.as_deref(),
                    op.operation_id
                        .ends_with(".create")
                        .then_some(d.currency_type_id.as_str())
                );
            }
        }
        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&c).unwrap();
        let data: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_money(
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
            "static_transformers",
            "certificate_sha256",
        ] {
            let mut changed = data.clone();
            changed[field] = json!("forged");
            assert!(
                import_csharp_practical_ordinary_money(
                    &serde_json::to_vec(&changed).unwrap(),
                    p.certificate_bytes(),
                    emitted.vir()
                )
                .is_err(),
                "{field}"
            );
        }
        for field in [
            "operation_id",
            "argument_type_ids",
            "result_type_id",
            "currency_predicate_argument_type_id",
            "normal_definition",
            "failures",
        ] {
            let mut changed = data.clone();
            changed["definitions"][0]["operations"][0][field] = json!(null);
            assert!(
                import_csharp_practical_ordinary_money(
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
        assert!(import_csharp_practical_ordinary_money(
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
        metrics.push(json!({"id":id,"file":file,"terms":c.term_table.len(),"declarations":c.declarations.len(),"metadata":data}));
    }
    assert_eq!(metrics.len(), 2);
    assert_eq!(
        metrics
            .iter()
            .map(|r| r["metadata"]["definitions"].as_array().unwrap().len())
            .sum::<usize>(),
        3
    );
    eprintln!(
        "Money ordinary certificates: {} original source contexts",
        metrics.len()
    );
    if let Some(output) = output {
        fs::write(
            output.join("certificates.json"),
            serde_json::to_vec_pretty(&metrics).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/money-operations/certificates.json"),
            json!(metrics)
        );
    }
}

use core_eval::{bit as observed_bit, run, sparse_cube, V};
use sequence_tests::{number, observe, word};

fn decimal(coefficient: u128, scale: u8, negative: bool) -> MonomorphicValue {
    MonomorphicValue::DecimalBits {
        type_id: ty("decimal"),
        coefficient: coefficient.to_string(),
        scale,
        negative,
    }
}
fn input(value: &MonomorphicValue, types: &BTreeMap<String, OrdinaryCarrier>) -> V {
    let bits = relation_tests::storage(value, types);
    sparse_cube(
        bits.len().trailing_zeros(),
        bits.into_iter()
            .enumerate()
            .filter_map(|(i, v)| v.then_some(i))
            .collect(),
    )
}
type ObserveResult = fn(
    &mpk_cert::Certificate,
    &OrdinaryMoneyDefinition,
    &BTreeMap<String, OrdinaryCarrier>,
    &str,
    Vec<V>,
    Result<MonomorphicValue, BusinessError>,
);

fn audit_oracle_result(
    _: &mpk_cert::Certificate,
    _: &OrdinaryMoneyDefinition,
    _: &BTreeMap<String, OrdinaryCarrier>,
    operation: &str,
    _: Vec<V>,
    expected: Result<MonomorphicValue, BusinessError>,
) {
    if let Err(error) = expected {
        assert!(
            error.error_id().is_some(),
            "{operation}: unnamed oracle error {error:?}"
        );
    }
}

fn check(
    c: &mpk_cert::Certificate,
    d: &OrdinaryMoneyDefinition,
    types: &BTreeMap<String, OrdinaryCarrier>,
    operation: &str,
    args: Vec<V>,
    expected: Result<MonomorphicValue, BusinessError>,
) {
    let op = d
        .operations
        .iter()
        .find(|op| op.operation_id.ends_with(&format!(".{operation}")))
        .unwrap();
    let failure = op
        .failures
        .iter()
        .find(|f| observed_bit(run(c, &f.definition, args.clone())));
    match expected {
        Err(error) => {
            let expected = error
                .error_id()
                .expect("Money oracle inputs must yield a semantic result or named error");
            assert_eq!(
                failure.map(|f| f.label.as_str()),
                Some(expected),
                "{operation}: {error:?}"
            );
        }
        Ok(value) => {
            assert!(
                failure.is_none(),
                "{operation}: unexpected {:?}",
                failure.map(|f| &f.label)
            );
            let actual = run(c, &op.normal_definition, args);
            match value {
                MonomorphicValue::Bool { value, .. } => assert_eq!(observed_bit(actual), value),
                MonomorphicValue::Signed { value, .. } => {
                    assert_eq!(number(c, actual) as i32, value.parse::<i32>().unwrap())
                }
                _ => observe(c, actual, &relation_tests::storage(&value, types)),
            }
        }
    }
}
fn signed(value: i32) -> MonomorphicValue {
    MonomorphicValue::Signed {
        type_id: ty("i32"),
        value: value.to_string(),
    }
}
fn boolean(value: bool) -> MonomorphicValue {
    MonomorphicValue::Bool {
        type_id: ty("bool"),
        value,
    }
}

#[test]
fn csharp_03_t06_w09_money_original_source_semantics() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| check_original_source_semantics(check))
        .unwrap()
        .join()
        .unwrap();
}

fn check_original_source_semantics(observe_result: ObserveResult) {
    let bundle = b();
    let (_, row, facts) = relation_tests::sources()
        .into_iter()
        .find(|(id, _, _)| id == "binding-vc-money")
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
    let p = generate_csharp_practical_ordinary_money(emitted.vir()).unwrap();
    let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    let layouts = generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap();
    let types = layouts
        .carriers()
        .iter()
        .map(|c| (c.type_id.clone(), c.clone()))
        .collect::<BTreeMap<_, _>>();
    let d = &p.definitions()[0];
    let model = MoneyModel::new(
        &bundle,
        emitted.closure().roots(),
        emitted.closure().closed(),
        &d.carrier.type_id,
    )
    .unwrap();
    let currency = MonomorphicValue::String {
        type_id: ty("string"),
        utf16: "USD".encode_utf16().collect(),
    };
    let other_currency = MonomorphicValue::String {
        type_id: ty("string"),
        utf16: "JPY".encode_utf16().collect(),
    };
    let domain = CurrencyDomain::new(
        &bundle,
        emitted.closure().roots(),
        emitted.closure().closed(),
        &d.currency_type_id,
        vec![currency.clone()],
    )
    .unwrap();
    let relations = generate_csharp_practical_ordinary_relations(emitted.vir()).unwrap();
    let currency_equal = &relations
        .definitions()
        .iter()
        .find(|r| r.carrier.type_id == d.currency_type_id)
        .unwrap()
        .equality_definition;
    // Supply a genuine ordinary partial application. No host callback is
    // allowed to answer the generated currency predicate.
    let predicate = run(&c, currency_equal, vec![input(&currency, &types)]);
    let money = |amount: MonomorphicValue, currency: &MonomorphicValue| MonomorphicValue::Money {
        type_id: d.carrier.type_id.clone(),
        amount: Box::new(amount),
        currency: Box::new(currency.clone()),
    };
    let mut observations = 0;
    for (amount, scale, allowed) in [
        (decimal(100, 2, false), 0, true),
        (decimal(101, 2, false), 1, true),
        (decimal(0, 28, true), 0, true),
        (decimal(1, 28, false), 28, true),
        (decimal((1u128 << 96) - 1, 0, false), 28, true),
        (decimal(101, 2, false), -1, true),
        (decimal(101, 2, false), 29, true),
        (decimal(101, 2, false), i32::MIN, true),
        (decimal(101, 2, false), -1, false),
        (decimal(101, 2, false), 1, false),
    ] {
        let cur = if allowed { &currency } else { &other_currency };
        eprintln!("Money create: scale {scale}, currency allowed {allowed}");
        observe_result(
            &c,
            d,
            &types,
            "create",
            vec![
                predicate.clone(),
                input(&amount, &types),
                input(cur, &types),
                word(scale as u32),
            ],
            model.create(amount, cur.clone(), scale, &domain),
        );
        observations += 1;
    }
    let one = money(decimal(100, 2, false), &currency);
    for operation in ["amount", "currency"] {
        let expected = if operation == "amount" {
            model.amount(&one).unwrap()
        } else {
            model.currency(&one).unwrap()
        };
        observe_result(
            &c,
            d,
            &types,
            operation,
            vec![input(&one, &types)],
            Ok(expected.clone()),
        );
        observations += 1;
    }
    for (a, b, same_currency) in [
        (decimal(100, 2, false), decimal(1, 0, false), true),
        (decimal(25, 1, true), decimal(3, 0, false), true),
        (
            decimal((1u128 << 96) - 1, 0, false),
            decimal(1, 0, false),
            true,
        ),
        (
            decimal((1u128 << 96) - 1, 0, true),
            decimal(1, 0, false),
            true,
        ),
        (
            decimal((1u128 << 96) - 1, 0, false),
            decimal(1, 0, false),
            false,
        ),
        (decimal(1, 0, false), decimal(99, 0, false), false),
    ] {
        let a = money(a, &currency);
        let b = money(
            b,
            if same_currency {
                &currency
            } else {
                &other_currency
            },
        );
        for operation in ["add", "subtract", "amount_compare", "equal", "compare"] {
            eprintln!("Money {operation}: same currency {same_currency}");
            let expected = match operation {
                "add" | "subtract" => model.add_or_subtract(&a, &b, operation == "subtract"),
                "amount_compare" => model.amount_compare(&a, &b).map(|v| signed(v as i32)),
                "equal" => model.structural_equal(&a, &b).map(boolean),
                "compare" => model.storage_compare(&a, &b).map(|v| signed(v as i32)),
                _ => unreachable!(),
            };
            observe_result(
                &c,
                d,
                &types,
                operation,
                vec![input(&a, &types), input(&b, &types)],
                expected,
            );
            observations += 1;
        }
    }
    let mut cases = vec![];
    for mode in 0..5 {
        for negative in [false, true] {
            cases.push((decimal(125, 2, negative), decimal(1, 0, false), 1, mode));
        }
    }
    cases.extend([
        (decimal(1, 0, false), decimal(0, 0, true), -1, 5),
        (decimal(1, 0, false), decimal(0, 0, true), 29, 0),
        (decimal(1, 0, false), decimal(0, 0, true), 1, 5),
        (decimal(1, 0, false), decimal(0, 0, true), 1, -1),
        (decimal(1, 0, false), decimal(0, 0, true), 1, 0),
        (
            decimal((1u128 << 96) - 1, 0, false),
            decimal(2, 0, false),
            0,
            0,
        ),
        (
            decimal((1u128 << 96) - 1, 0, false),
            decimal(1, 1, false),
            0,
            0,
        ),
        (decimal(1, 0, false), decimal(3, 0, false), 28, 0),
    ]);
    for (amount, quantity, scale, mode) in cases {
        let value = money(amount, &currency);
        for operation in ["multiply", "divide"] {
            eprintln!("Money {operation}: scale {scale}, mode {mode}, quantity {quantity:?}");
            let expected =
                model.scale(&value, quantity.clone(), scale, mode, operation == "divide");
            observe_result(
                &c,
                d,
                &types,
                operation,
                vec![
                    input(&value, &types),
                    input(&quantity, &types),
                    word(scale as u32),
                    word(mode as u32),
                ],
                expected,
            );
            observations += 1;
        }
    }
    assert_eq!(observations, 78);
    eprintln!("Money: {observations} cases across all 10 operations; currency predicate, exact cohorts, signed rounding, overflow and error precedence");
}

#[test]
fn csharp_03_t06_w09_money_source_requests() {
    let bundle = b();
    let id = |name: &str| {
        csharp_practical_declaration_id(&json!({"kind":"type","namespace":"MoneyCases","owner":"","name":name,"parameter_type_ids":[],"result_type_id":""})).unwrap()
    };
    let envelope = id("Envelope");
    let root = csharp_practical_declaration_id(&json!({"kind":"method","namespace":"MoneyCases","owner":id("Entry"),"name":"Run","parameter_type_ids":[envelope.clone()],"result_type_id":envelope})).unwrap();
    let code = "namespace MoneyCases;public enum Currency{Usd=1,Jpy=7}public readonly struct StringMoney{public readonly decimal Amount;public readonly string Currency;}public readonly struct EnumMoney{public readonly decimal Amount;public readonly Currency Currency;}public readonly struct Envelope{public readonly StringMoney StringValue;public readonly EnumMoney EnumValue;}public static class Entry{public static Envelope Run(Envelope value){return value;}}\n";
    let (plain, captures) = support::context(&bundle, &root, code.as_bytes());
    let mut bindings = vec![];
    for (name, currency, currency_id) in [
        ("StringMoney", primitive("string"), ty("string")),
        (
            "EnumMoney",
            json!({"kind":"source","id":id("Currency")}),
            id("Currency"),
        ),
    ] {
        let source = id(name);
        bindings.push(SemanticBindingInput {
            source_type_id: source.clone(),
            source_content_sha256: captures.entries()[0].raw_sha256().into(),
            role: "money".into(),
            member_map: [
                ("amount", "Amount", primitive("decimal")),
                ("currency", "Currency", currency),
            ]
            .into_iter()
            .map(|(role, name, t)| SemanticBindingMember {
                role: role.into(),
                member_id: csharp_practical_stored_member_id(&source, name, &t, "readonly_field")
                    .unwrap(),
            })
            .collect(),
            inferred_argument_ids: vec![currency_id],
            tag_arms: vec![],
            default_arm: "ineligible".into(),
            bounds: vec![],
            operation_map: vec![],
            enum_arms: BTreeMap::new(),
        });
    }
    let sidecar = build_semantic_bindings(&plain, &captures, bindings)
        .unwrap()
        .canonical_bytes()
        .to_vec();
    let (context, captures) =
        support::context_with_sidecar(&bundle, &root, code.as_bytes(), |_| sidecar);
    let requests = json!([{"id":"shared-string-enum-money","compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.kind()==OriginalInputKind::Source{"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()}]);
    let bytes = serde_json::to_vec_pretty(&requests).unwrap();
    if let Some(out) = std::env::var_os("MPK_W09_MONEY_REQUESTS_OUT") {
        fs::write(out, bytes).unwrap();
    } else {
        assert_eq!(
            fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(
                "../../develop/migrations/csharp-03/ordinary-foundation/money-sources/requests.json"
            ))
            .unwrap(),
            bytes
        );
    }
}

#[test]
fn csharp_03_t06_w09_money_shared_string_enum_semantics() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| check_shared_string_enum_semantics(check))
        .unwrap()
        .join()
        .unwrap();
}

fn check_shared_string_enum_semantics(observe_result: ObserveResult) {
    let bundle = b();
    let (_, row, facts) = sources()
        .into_iter()
        .find(|(id, _, _)| id == "shared-string-enum-money")
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
    let p = generate_csharp_practical_ordinary_money(emitted.vir()).unwrap();
    let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    let layouts = generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap();
    let types = layouts
        .carriers()
        .iter()
        .map(|c| (c.type_id.clone(), c.clone()))
        .collect::<BTreeMap<_, _>>();
    let relations = generate_csharp_practical_ordinary_relations(emitted.vir()).unwrap();
    let mut observations = 0;
    let mut currency_types = BTreeSet::new();
    assert_eq!(p.definitions().len(), 2);
    for d in p.definitions() {
        currency_types.insert(d.currency_type_id.clone());
        let model = MoneyModel::new(
            &bundle,
            emitted.closure().roots(),
            emitted.closure().closed(),
            &d.carrier.type_id,
        )
        .unwrap();
        let (currency, other) = if d.currency_type_id == ty("string") {
            (
                MonomorphicValue::String {
                    type_id: ty("string"),
                    utf16: "USD".encode_utf16().collect(),
                },
                MonomorphicValue::String {
                    type_id: ty("string"),
                    utf16: "JPY".encode_utf16().collect(),
                },
            )
        } else {
            (
                MonomorphicValue::Enum {
                    type_id: d.currency_type_id.clone(),
                    underlying: "i32".into(),
                    carrier: "1".into(),
                },
                MonomorphicValue::Enum {
                    type_id: d.currency_type_id.clone(),
                    underlying: "i32".into(),
                    carrier: "7".into(),
                },
            )
        };
        let domain = CurrencyDomain::new(
            &bundle,
            emitted.closure().roots(),
            emitted.closure().closed(),
            &d.currency_type_id,
            vec![currency.clone()],
        )
        .unwrap();
        let currency_equal = &relations
            .definitions()
            .iter()
            .find(|r| r.carrier.type_id == d.currency_type_id)
            .unwrap()
            .equality_definition;
        let predicate = run(&c, currency_equal, vec![input(&currency, &types)]);
        let amount = decimal(125, 2, true);
        let a = model
            .create(amount.clone(), currency.clone(), 2, &domain)
            .unwrap();
        let b = model
            .create(decimal(1, 0, false), currency.clone(), 2, &domain)
            .unwrap();
        for cur in [&currency, &other] {
            eprintln!("Shared Money create: {} / {cur:?}", d.currency_type_id);
            observe_result(
                &c,
                d,
                &types,
                "create",
                vec![
                    predicate.clone(),
                    input(&amount, &types),
                    input(cur, &types),
                    word(2),
                ],
                model.create(amount.clone(), cur.clone(), 2, &domain),
            );
            observations += 1;
        }
        for operation in ["amount", "currency"] {
            let expected = if operation == "amount" {
                model.amount(&a).unwrap()
            } else {
                model.currency(&a).unwrap()
            };
            observe_result(
                &c,
                d,
                &types,
                operation,
                vec![input(&a, &types)],
                Ok(expected.clone()),
            );
            observations += 1;
        }
        for operation in ["add", "subtract", "amount_compare", "equal", "compare"] {
            eprintln!("Shared Money {operation}: {}", d.currency_type_id);
            let expected = match operation {
                "add" | "subtract" => model.add_or_subtract(&a, &b, operation == "subtract"),
                "amount_compare" => model.amount_compare(&a, &b).map(|v| signed(v as i32)),
                "equal" => model.structural_equal(&a, &b).map(boolean),
                "compare" => model.storage_compare(&a, &b).map(|v| signed(v as i32)),
                _ => unreachable!(),
            };
            observe_result(
                &c,
                d,
                &types,
                operation,
                vec![input(&a, &types), input(&b, &types)],
                expected,
            );
            observations += 1;
        }
        for operation in ["multiply", "divide"] {
            let quantity = decimal(1, 0, false);
            eprintln!("Shared Money {operation}: {}", d.currency_type_id);
            observe_result(
                &c,
                d,
                &types,
                operation,
                vec![
                    input(&a, &types),
                    input(&quantity, &types),
                    word(1),
                    word(3),
                ],
                model.scale(&a, quantity, 1, 3, operation == "divide"),
            );
            observations += 1;
        }
    }
    assert_eq!(currency_types.len(), 2);
    assert_eq!(observations, 22);
    eprintln!("Shared Money: two currency types, all 20 operations, {observations} cases");
}

/// This fast check audits the identical case matrices used by the ordinary
/// observers. It does not evaluate generated operations and cannot be counted
/// as their semantic acceptance. In particular, no unnamed OperandType or
/// Signature oracle failure may silently match a missing generated failure.
#[test]
fn csharp_03_t06_w09_money_oracle_case_domains() {
    check_original_source_semantics(audit_oracle_result);
    check_shared_string_enum_semantics(audit_oracle_result);
}
