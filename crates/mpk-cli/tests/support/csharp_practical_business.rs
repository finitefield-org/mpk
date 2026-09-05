//! CSHARP-03-T03-W13 executable relations, independently checked against the
//! frozen framework observations. Finite tests do not discharge binding VCs.
use super::*;
fn value(token: &str, s: &str) -> MonomorphicValue {
    match token {
        "date" => MonomorphicValue::Date {
            type_id: ty(token),
            day_number: s.parse().unwrap(),
        },
        "time" => MonomorphicValue::Time {
            type_id: ty(token),
            ticks: s.into(),
        },
        "duration" => MonomorphicValue::Duration {
            type_id: ty(token),
            ticks: s.into(),
        },
        "instant" => MonomorphicValue::Instant {
            type_id: ty(token),
            milliseconds: s.into(),
        },
        "guid" => MonomorphicValue::Guid {
            type_id: ty(token),
            n: s.replace('-', "").to_ascii_lowercase(),
        },
        "string" => MonomorphicValue::String {
            type_id: ty(token),
            utf16: s.encode_utf16().collect(),
        },
        "decimal" => {
            let text = s.trim_start_matches('-');
            let scale = text.split_once('.').map_or(0, |(_, d)| d.len() as u8);
            MonomorphicValue::DecimalBits {
                type_id: ty(token),
                negative: s.starts_with('-'),
                scale,
                coefficient: text.replace('.', "").parse::<u128>().unwrap().to_string(),
            }
        }
        _ => raw(token, s),
    }
}
fn show(v: &MonomorphicValue) -> String {
    match v {
        MonomorphicValue::DecimalBits {
            negative,
            scale,
            coefficient,
            ..
        } => {
            let mut n = coefficient.parse::<u128>().unwrap();
            let mut scale = *scale;
            while scale > 0 && n % 10 == 0 {
                n /= 10;
                scale -= 1;
            }
            let mut text = n.to_string();
            if scale > 0 {
                while text.len() <= scale as usize {
                    text.insert(0, '0');
                }
                text.insert(text.len() - scale as usize, '.');
            }
            if *negative && n != 0 {
                text.insert(0, '-');
            }
            text
        }
        MonomorphicValue::Date { day_number, .. } => day_number.to_string(),
        MonomorphicValue::Time { ticks, .. } | MonomorphicValue::Duration { ticks, .. } => {
            ticks.clone()
        }
        MonomorphicValue::Instant { milliseconds, .. } => milliseconds.clone(),
        MonomorphicValue::Guid { n, .. } => n.clone(),
        MonomorphicValue::Enum { carrier, .. } => carrier.clone(),
        MonomorphicValue::String { utf16, .. } => String::from_utf16(utf16).unwrap(),
        _ => encoded(v),
    }
}
fn operation(
    b: &ValidatedFoundationBundle,
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
    id: &str,
    out: &str,
    args: &[MonomorphicValue],
) -> Result<MonomorphicValue, BusinessError> {
    BusinessOperation::new(
        id,
        &args
            .iter()
            .map(|v| v.type_id().to_owned())
            .collect::<Vec<_>>(),
        &ty(out),
    )
    .unwrap()
    .evaluate(b, r, c, args)
}
fn observe(
    b: &ValidatedFoundationBundle,
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
    m: &MoneyModel<'_>,
    predicate: &CurrencyDomain,
    id: &str,
    a: &[String],
) -> Result<Vec<String>, BusinessError> {
    let (token, op) = id.split_once('.').unwrap();
    let parse = |i: usize, t: &str| value(t, &a[i]);
    if token == "money" {
        let make = |i| {
            m.create(parse(i, "decimal"), parse(i + 1, "string"), 28, predicate)
                .unwrap()
        };
        let sign = |o: std::cmp::Ordering| {
            match o {
                std::cmp::Ordering::Less => "-1",
                std::cmp::Ordering::Equal => "0",
                std::cmp::Ordering::Greater => "1",
            }
            .to_owned()
        };
        let result = match op {
            "create" => m.create(
                parse(0, "decimal"),
                parse(1, "string"),
                a[2].parse().unwrap(),
                predicate,
            ),
            "add" | "subtract" => m.add_or_subtract(&make(0), &make(2), op == "subtract"),
            "multiply" | "divide" => m.scale(
                &make(0),
                parse(2, "decimal"),
                a[3].parse().unwrap(),
                a[4].parse().unwrap(),
                op == "divide",
            ),
            "amount_compare" => {
                return m
                    .amount_compare(&make(0), &make(2))
                    .map(|v| vec!["ok".into(), sign(v)])
            }
            "equal" => return Ok(vec![m.structural_equal(&make(0), &make(2))?.to_string()]),
            "compare" => return Ok(vec![sign(m.storage_compare(&make(0), &make(2))?)]),
            _ => panic!("{id}"),
        }?;
        let text = String::from_utf16(&m.encode_amount(&result)?).unwrap();
        assert_eq!(text, show(m.amount(&result)?));
        return Ok(vec!["ok".into(), text, show(m.currency(&result)?)]);
    }
    let args = match (token, op) {
        ("date", "construct") => a.iter().map(|s| value("i32", s)).collect::<Vec<_>>(),
        ("time" | "duration", "construct") => vec![parse(0, "i64")],
        ("date", "add_days" | "add_months" | "add_years") => vec![parse(0, token), parse(1, "i32")],
        ("time" | "instant", "add_duration" | "subtract_duration") => {
            vec![parse(0, token), parse(1, "duration")]
        }
        _ => a.iter().map(|s| value(token, s)).collect(),
    };
    if op == "compare" {
        let mut out = vec![];
        for name in [
            "compare",
            "equal",
            "not_equal",
            "less",
            "less_equal",
            "greater",
            "greater_equal",
        ] {
            if token == "guid" && out.len() == 3 {
                break;
            }
            out.push(show(&operation(
                b,
                r,
                c,
                &format!("{token}.{name}"),
                if name == "compare" { "i32" } else { "bool" },
                &args,
            )?));
        }
        return Ok(out);
    }
    let output = if id == "time.subtract" || id == "instant.difference" {
        "duration"
    } else {
        token
    };
    let result = operation(b, r, c, id, output, &args)?;
    if token == "instant" {
        return Ok(vec!["ok".into(), show(&result)]);
    }
    let properties: &[&str] = if token == "date" {
        &["year", "month", "day", "day_number", "day_of_week"]
    } else if token == "time" && op != "subtract" {
        &["ticks", "hour", "minute", "second", "millisecond"]
    } else if id == "duration.construct" {
        &[
            "ticks",
            "days",
            "hours",
            "minutes",
            "seconds",
            "milliseconds",
        ]
    } else {
        &[]
    };
    if properties.is_empty() {
        return Ok(vec![show(&result)]);
    }
    properties
        .iter()
        .map(|property| {
            operation(
                b,
                r,
                c,
                &format!("{token}.{property}"),
                if *property == "ticks" {
                    "i64"
                } else if *property == "day_of_week" {
                    "day_of_week"
                } else {
                    "i32"
                },
                std::slice::from_ref(&result),
            )
            .map(|v| show(&v))
        })
        .collect()
}
#[test]
fn csharp_03_t03_w13_all_1007_frozen_business_vectors() {
    let b = bundle();
    let (r, c, ids) = fixture(
        &b,
        &[instance("money", vec![primitive("string")])],
        json!({}),
    );
    let m = MoneyModel::new(&b, &r, &c, &ids[0]).unwrap();
    let predicate = CurrencyDomain::new(
        &b,
        &r,
        &c,
        &ty("string"),
        vec![value("string", "AAA"), value("string", "BBB")],
    )
    .unwrap();
    let vectors = file("develop/specs/vectors/csharp-practical-foundation-v1.json");
    let mut runtime = vec![];
    let mut count = 0;
    for row in vectors["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["implementation_owner"] == "CSHARP-03-T03-W13")
    {
        count += 1;
        let mathematical = row["inputs"].is_array();
        let (id, args) = if mathematical {
            let arr = row["inputs"].as_array().unwrap();
            let instant = row["id"].as_str().unwrap().starts_with("business.instant_");
            (
                if instant {
                    format!("instant.{}", arr[0].as_str().unwrap())
                } else {
                    "money.create".into()
                },
                arr[usize::from(instant)..]
                    .iter()
                    .map(|v| {
                        v.as_str()
                            .map(str::to_owned)
                            .unwrap_or_else(|| v.to_string())
                    })
                    .collect::<Vec<_>>(),
            )
        } else {
            runtime.push(json!({"id":row["id"],"operation":row["inputs"]["operation"],"inputs":row["inputs"]["inputs"],"expected":row["expected"]}));
            (
                row["inputs"]["operation"].as_str().unwrap().into(),
                row["inputs"]["inputs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_owned())
                    .collect(),
            )
        };
        let actual = observe(&b, &r, &c, &m, &predicate, &id, &args);
        if mathematical {
            let expected = &row["expected"]["value"];
            if let Some(error) = expected["error"].as_str() {
                assert_eq!(actual.unwrap_err().error_id(), Some(error), "{}", row["id"]);
            } else {
                let actual = actual.unwrap();
                if id == "money.create" {
                    assert_eq!(
                        actual,
                        vec![
                            "ok".into(),
                            show(&value("decimal", expected["amount"].as_str().unwrap())),
                            expected["currency"].as_str().unwrap().into()
                        ]
                    );
                } else {
                    assert_eq!(
                        actual[1],
                        expected["value"]
                            .as_str()
                            .map(str::to_owned)
                            .unwrap_or_else(|| expected["value"].to_string()),
                        "{}",
                        row["id"]
                    );
                }
            }
        } else {
            let (kind, values) = match actual {
                Ok(v) => ("value", v),
                Err(e) => {
                    if let Some(t) = e.exception_type() {
                        ("exception", vec![t.into()])
                    } else {
                        ("value", vec!["error".into(), e.error_id().unwrap().into()])
                    }
                }
            };
            assert_eq!(
                json!({"kind":kind,"value":values}),
                row["expected"],
                "{}: {id} {args:?}",
                row["id"]
            );
        }
    }
    assert_eq!(count, 1007);
    assert_eq!(runtime.len(), 994);
    assert_eq!(
        json!(runtime),
        file("develop/migrations/csharp-03/business/business-runtime.json")
    );
}
#[test]
fn csharp_03_t03_w13_shared_codecs_and_exact_signatures() {
    let b = bundle();
    let (r, c, _) = fixture(&b, &[], json!({}));
    for (token, variant, s) in [
        ("date", None, "3652058"),
        ("time", None, "863999999999"),
        ("duration", None, "-9223372036854775808"),
        ("instant", None, "9223372036854775807"),
        ("guid", Some("n"), "fedcba98876543218000ffffffffffff"),
        ("guid", Some("d"), "fedcba98876543218000ffffffffffff"),
    ] {
        let codec = BusinessOperation::boundary_codec(token, variant).unwrap();
        let v = value(token, s);
        assert_eq!(
            codec.parse(&codec.format(&b, &r, &c, &v).unwrap()).unwrap(),
            v
        );
    }
    assert!(BusinessOperation::boundary_codec("money", None).is_err());
    assert!(BusinessOperation::new("guid.new_guid", &[], &ty("guid")).is_err());
    assert!(BusinessOperation::new(
        "instant.add_duration",
        &[ty("i64"), ty("duration")],
        &ty("instant")
    )
    .is_err());
    let op = BusinessOperation::new(
        "instant.add_duration",
        &[ty("instant"), ty("duration")],
        &ty("instant"),
    )
    .unwrap();
    assert_eq!(
        op.evaluate(&b, &r, &c, &[value("i64", "0"), value("duration", "0")]),
        Err(BusinessError::OperandType)
    );
    // Every day in a Gregorian 400-year cycle, including the century exception.
    for n in 0..146097 {
        let v = value("date", &n.to_string());
        let parts = ["year", "month", "day"].map(|p| {
            operation(
                &b,
                &r,
                &c,
                &format!("date.{p}"),
                "i32",
                std::slice::from_ref(&v),
            )
            .unwrap()
        });
        assert_eq!(
            operation(&b, &r, &c, "date.construct", "date", &parts).unwrap(),
            v
        );
    }
}

fn binding_input(
    source: &Value,
    role: &str,
    roles: &[(&str, &str)],
    args: Vec<String>,
) -> SemanticBindingInput {
    SemanticBindingInput {
        source_type_id: source["id"].as_str().unwrap().into(),
        source_content_sha256: source["source_sha256"].as_str().unwrap().into(),
        role: role.into(),
        member_map: roles
            .iter()
            .map(|(role, name)| SemanticBindingMember {
                role: (*role).into(),
                member_id: source["members"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|m| m["name"] == *name)
                    .unwrap()["id"]
                    .as_str()
                    .unwrap()
                    .into(),
            })
            .collect(),
        tag_arms: vec![],
        inferred_argument_ids: args,
        default_arm: "ineligible".into(),
        bounds: vec![],
        operation_map: vec![],
    }
}
#[test]
fn csharp_03_t03_w13_wrapper_projection_is_content_bound_and_field_complete() {
    let b = bundle();
    for role in ["instant", "money"] {
        let members = if role == "instant" {
            vec![
                ("Milliseconds", primitive("i64")),
                ("Extra", primitive("i32")),
            ]
        } else {
            vec![
                ("Amount", primitive("decimal")),
                ("Currency", primitive("string")),
                ("Extra", primitive("i32")),
            ]
        };
        let source = source_fixture("BusinessValue", "readonly_struct", &members, &[]);
        let sourceid = source["id"].as_str().unwrap();
        let types = vec![
            json!({"kind":"source","id":sourceid}),
            if role == "instant" {
                primitive("instant")
            } else {
                instance("money", vec![primitive("string")])
            },
        ];
        let (r, c, ids) = fixture(&b, &types, json!({sourceid:source.clone()}));
        let roles = if role == "instant" {
            vec![("milliseconds", "Milliseconds")]
        } else {
            vec![("amount", "Amount"), ("currency", "Currency")]
        };
        let input = binding_input(
            &source,
            role,
            &roles,
            if role == "money" {
                vec![ty("string")]
            } else {
                vec![]
            },
        );
        let build = |input: &SemanticBindingInput| {
            BusinessBindingPlan::new(&b, &r, &c, input, &BTreeMap::new(), &[], &BTreeMap::new())
        };
        let plan = build(&input).unwrap();
        assert_eq!(plan.semantic_type_id(), ids[1]);
        assert!(!plan.default_eligible());
        assert!(plan.obligations().iter().all(|o| !o.discharged));
        assert_eq!(
            plan.obligations()
                .iter()
                .filter(|o| o.kind == "field_complete_reconstruction")
                .count(),
            members.len()
        );
        let original = product(
            sourceid,
            if role == "instant" {
                vec![
                    ("Milliseconds", value("i64", "-9223372036854775808")),
                    ("Extra", value("i32", "7")),
                ]
            } else {
                vec![
                    ("Amount", value("decimal", "1.00")),
                    ("Currency", value("string", "AAA")),
                    ("Extra", value("i32", "7")),
                ]
            },
        );
        let mut copy = original.clone();
        let projection = plan.project(&b, &r, &c, &original).unwrap();
        if let MonomorphicValue::Product { fields, .. } = &mut copy {
            let last = fields.len() - 1;
            *fields[last].value = value("i32", "8");
        }
        assert_eq!(plan.project(&b, &r, &c, &copy).unwrap(), projection);
        assert_eq!(
            plan.check_source_round_trip(&b, &r, &c, &original, &copy),
            Err(BusinessError::ObservationLoss)
        );
        if role == "money" {
            let mut copy = original.clone();
            if let MonomorphicValue::Product { fields, .. } = &mut copy {
                *fields[0].value = value("decimal", "1");
            }
            plan.check_source_round_trip(&b, &r, &c, &original, &copy)
                .unwrap();
        }
        for mutation in 0..7 {
            let mut bad = input.clone();
            match mutation {
                0 => bad.source_content_sha256 = "0".repeat(64),
                1 => {
                    bad.member_map[0].member_id =
                        source["members"].as_array().unwrap().last().unwrap()["id"]
                            .as_str()
                            .unwrap()
                            .into()
                }
                2 => bad.default_arm = "zero".into(),
                3 => bad.inferred_argument_ids.push(ty("i64")),
                4 => bad.tag_arms.push(SemanticArmMapping {
                    source_tag: "0".into(),
                    semantic_arm: "ok".into(),
                }),
                5 => bad.bounds.push(SemanticBound {
                    id: "amount".into(),
                    maximum: 28,
                }),
                _ => bad.member_map.push(bad.member_map[0].clone()),
            };
            assert!(build(&bad).is_err(), "{role} mutation {mutation}");
        }
    }
}
#[test]
fn csharp_03_t03_w13_fallible_helper_needs_separate_result_and_exhaustive_errors() {
    let b = bundle();
    let wrapper = source_fixture(
        "Instant",
        "readonly_struct",
        &[("Milliseconds", primitive("i64"))],
        &[],
    );
    let wid = wrapper["id"].as_str().unwrap();
    let tags = source_fixture("ResultTag", "enum", &[], &[0, 1]);
    let tid = tags["id"].as_str().unwrap();
    let errors = source_fixture("InstantError", "enum", &[], &[4, 9]);
    let eid = errors["id"].as_str().unwrap();
    let result = source_fixture(
        "InstantResult",
        "readonly_struct",
        &[
            ("Tag", json!({"kind":"source","id":tid})),
            ("Value", json!({"kind":"source","id":wid})),
            ("Error", json!({"kind":"source","id":eid})),
        ],
        &[],
    );
    let rid = result["id"].as_str().unwrap();
    let (r, c, _) = fixture(
        &b,
        &[
            json!({"kind":"source","id":rid}),
            instance(
                "result",
                vec![
                    json!({"kind":"source","id":wid}),
                    json!({"kind":"source","id":eid}),
                ],
            ),
        ],
        json!({wid:wrapper,tid:tags,eid:errors,rid:result}),
    );
    let mut outcome = binding_input(
        &result,
        "result",
        &[("tag", "Tag"), ("value", "Value"), ("error", "Error")],
        vec![wid.into(), eid.into()],
    );
    outcome.tag_arms = vec![
        SemanticArmMapping {
            semantic_arm: "ok".into(),
            source_tag: "0".into(),
        },
        SemanticArmMapping {
            semantic_arm: "error".into(),
            source_tag: "1".into(),
        },
    ];
    let outcome = OutcomeBindingPlan::new(&b, &r, &c, &outcome, &BTreeMap::new()).unwrap();
    let mut input = binding_input(
        &wrapper,
        "instant",
        &[("milliseconds", "Milliseconds")],
        vec![],
    );
    let method=csharp_practical_declaration_id(&json!({"kind":"method","namespace":"Example","owner":wid,"name":"Add","parameter_type_ids":[ty("duration")],"result_type_id":rid})).unwrap();
    input.operation_map = vec![SemanticOperationMapping {
        operation: "add_duration".into(),
        member_id: method.clone(),
    }];
    let calls = BTreeMap::from([(
        method.clone(),
        ClosedOperationSignature {
            id: method.clone(),
            tag: ClosedOperationTag::SourceCall,
            argument_type_ids: vec![wid.into(), ty("duration")],
            normal_result_type_id: rid.into(),
            ordered_checks: vec![],
        },
    )]);
    let maps = BTreeMap::from([(
        eid.into(),
        BTreeMap::from([
            ("precision".into(), "4".into()),
            ("range".into(), "9".into()),
        ]),
    )]);
    let outcomes = vec![outcome];
    let plan = BusinessBindingPlan::new(&b, &r, &c, &input, &calls, &outcomes, &maps).unwrap();
    assert!(plan.obligations().iter().all(|o| !o.discharged));
    assert_eq!(
        plan.obligations()
            .iter()
            .filter(|o| o.kind.starts_with("operation_"))
            .count(),
        3
    );
    assert!(BusinessBindingPlan::new(&b, &r, &c, &input, &calls, &[], &maps).is_err());
    for mutation in 0..4 {
        let mut calls = calls.clone();
        let mut maps = maps.clone();
        match mutation {
            0 => calls.get_mut(&method).unwrap().argument_type_ids[0] = ty("i64"),
            1 => calls.get_mut(&method).unwrap().normal_result_type_id = wid.into(),
            2 => {
                maps.get_mut(eid).unwrap().remove("precision");
            }
            _ => {
                maps.get_mut(eid)
                    .unwrap()
                    .insert("precision".into(), "9".into());
            }
        };
        assert!(
            BusinessBindingPlan::new(&b, &r, &c, &input, &calls, &outcomes, &maps).is_err(),
            "mutation {mutation}"
        );
    }
}
#[test]
fn csharp_03_t03_w13_private_manifest_and_frozen_runtime_binding() {
    let manifest = file("develop/migrations/csharp-03/business/business-inputs.json");
    assert_eq!(
        manifest["schema"],
        "mpk.csharp_practical.t03_w13.business_inputs.v1"
    );
    assert_eq!(manifest["work_item"], "CSHARP-03-T03-W13");
    let mut canonical = serde_json::to_vec(&manifest).unwrap();
    canonical.push(b'\n');
    assert_eq!(
        canonical,
        read("develop/migrations/csharp-03/business/business-inputs.json")
    );
    let files = manifest["files"].as_array().unwrap();
    assert_eq!(files.len(), 13);
    let mut previous = "";
    for f in files {
        let path = f["path"].as_str().unwrap();
        assert!(path > previous);
        previous = path;
        let bytes = read(path);
        assert_eq!(f["size_bytes"], bytes.len());
        assert_eq!(f["sha256"], format!("{:x}", Sha256::digest(&bytes)));
    }
    let record = file("develop/migrations/csharp-03/probes/runtime-foundation-data.json");
    let rows = file("develop/migrations/csharp-03/business/business-runtime.json");
    for row in rows.as_array().unwrap() {
        let id = row["id"]
            .as_str()
            .unwrap()
            .strip_prefix("business.runtime_")
            .unwrap();
        for observation in record["observations"].as_array().unwrap() {
            let actual = observation["vectors"]
                .as_array()
                .unwrap()
                .iter()
                .find(|v| v["id"] == id)
                .unwrap();
            assert_eq!(actual["operation"], row["operation"]);
            assert_eq!(actual["inputs"], row["inputs"]);
            assert_eq!(actual["observed"], row["expected"]);
        }
    }
}

fn product(id: &str, fields: Vec<(&str, MonomorphicValue)>) -> MonomorphicValue {
    MonomorphicValue::Product {
        type_id: id.into(),
        fields: fields
            .into_iter()
            .map(|(name, value)| NamedMonomorphicValue {
                name: name.into(),
                value: Box::new(value),
            })
            .collect(),
    }
}

#[test]
fn csharp_03_t03_w13_money_exact_enum_currency_and_public_default() {
    let b = bundle();
    let en = source_fixture("Currency", "enum", &[], &[0, 1]);
    let eid = en["id"].as_str().unwrap();
    let (r, c, ids) = fixture(
        &b,
        &[instance("money", vec![json!({"kind":"source","id":eid})])],
        json!({eid:en}),
    );
    let model = MoneyModel::new(&b, &r, &c, &ids[0]).unwrap();
    let currency = |n: &str| MonomorphicValue::Enum {
        type_id: eid.into(),
        underlying: "i32".into(),
        carrier: n.into(),
    };
    let predicate = CurrencyDomain::new(&b, &r, &c, eid, vec![currency("1")]).unwrap();
    assert_eq!(
        model.create(value("decimal", "1.001"), currency("0"), 29, &predicate),
        Err(BusinessError::InvalidCurrency)
    );
    assert_eq!(
        model.create(value("decimal", "1.001"), currency("1"), 29, &predicate),
        Err(BusinessError::InvalidScale)
    );
    assert_eq!(
        model.create(value("decimal", "1.001"), currency("1"), 2, &predicate),
        Err(BusinessError::InvalidPrecision)
    );
    assert!(model
        .create(value("decimal", "-1.00"), currency("1"), 2, &predicate)
        .is_ok());
    assert!(domain_default(&b, &r, &c, &ids[0]).is_err());
    assert!(model
        .create(value("decimal", "1"), value("string", "1"), 0, &predicate)
        .is_err());
}

#[test]
fn csharp_03_t03_w13_pinned_source_harness_when_available() {
    let package = file("develop/migrations/csharp-03/build-inputs/build-inputs.json");
    let archives = package["toolchain_inputs"]["archives"].as_array().unwrap();
    if !cfg!(target_os = "linux") {
        return;
    }
    let cache=root().join("release/build-input-cache/csharp/d4af1170b2813a5581bb0f60b65fd4e7509576093045557b88689bf7e0876b4f/archives");
    let count = archives
        .iter()
        .filter(|archive| {
            cache
                .join(format!(
                    "{}.{}",
                    archive["id"].as_str().unwrap(),
                    archive["kind"].as_str().unwrap()
                ))
                .is_file()
        })
        .count();
    assert!(
        count == 0 || count == archives.len(),
        "partial pinned cache"
    );
    if count == 0 {
        return;
    }
    let output =
        std::process::Command::new(root().join("scripts/build-csharp-practical-frontend.sh"))
            .arg("--test-business")
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .output()
            .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
