use super::*;
use core_eval::{apply, bit as observed_bit, run, V};
use mpk_cert::encode::Certificate;
fn addr(n: usize) -> usize {
    if n <= 1 {
        0
    } else {
        (usize::BITS - (n - 1).leading_zeros()) as usize
    }
}
fn raw(n: u128, width: usize) -> Vec<bool> {
    (0..(1usize << addr(width)))
        .map(|i| i < width && n & (1u128 << i) != 0)
        .collect()
}
fn product(children: Vec<Vec<bool>>) -> Vec<bool> {
    let roles = addr(children.len());
    let max = children.iter().map(|v| addr(v.len())).max().unwrap_or(0);
    let mut out = vec![false; 1 << (roles + max)];
    for (i, v) in children.iter().enumerate() {
        let padding = max - addr(v.len());
        for (j, &b) in v.iter().enumerate() {
            out[i | (j << (roles + padding))] = b;
        }
    }
    out
}
fn reference(shape: &OrdinaryShape) -> &str {
    match shape {
        OrdinaryShape::Reference { type_id } => type_id,
        OrdinaryShape::RoleBound { value, .. } => reference(value),
        _ => panic!("not a typed child"),
    }
}
fn shape_depth(shape: &OrdinaryShape, types: &BTreeMap<String, OrdinaryCarrier>) -> usize {
    match shape {
        OrdinaryShape::Bits { width } => addr(*width as usize),
        OrdinaryShape::Reference { type_id } => types[type_id].depth as usize,
        OrdinaryShape::RoleBound { value, .. } => shape_depth(value, types),
        OrdinaryShape::Product { fields } => {
            addr(fields.len())
                + fields
                    .iter()
                    .map(|f| shape_depth(&f.shape, types))
                    .max()
                    .unwrap_or(0)
        }
        OrdinaryShape::Sum { arms } => {
            1 + 5.max(
                arms.iter()
                    .map(|a| {
                        addr(a.fields.len())
                            + a.fields
                                .iter()
                                .map(|f| shape_depth(&f.shape, types))
                                .max()
                                .unwrap_or(0)
                    })
                    .max()
                    .unwrap_or(0),
            )
        }
        OrdinaryShape::Array { capacity, element } => {
            addr(*capacity as usize) + shape_depth(element, types)
        }
        OrdinaryShape::Sequence { capacity, element } => {
            1 + 5.max(addr(*capacity as usize) + shape_depth(element, types))
        }
    }
}
fn sequence(
    capacity: u32,
    element: &OrdinaryShape,
    values: Vec<Vec<bool>>,
    types: &BTreeMap<String, OrdinaryCarrier>,
) -> Vec<bool> {
    let bits = addr(capacity as usize);
    let d = shape_depth(element, types);
    let mut data = vec![false; 1 << (bits + d)];
    for (i, v) in values.iter().enumerate() {
        assert_eq!(v.len(), 1 << d);
        for (j, &b) in v.iter().enumerate() {
            data[i | (j << bits)] = b;
        }
    }
    product(vec![raw(values.len() as u128, 32), data])
}
pub(super) fn storage(
    value: &MonomorphicValue,
    types: &BTreeMap<String, OrdinaryCarrier>,
) -> Vec<bool> {
    let shape = &types[value.type_id()].shape;
    let scalar = |n: u128| match shape {
        OrdinaryShape::Bits { width } => raw(n, *width as usize),
        _ => panic!(),
    };
    let packed = match value {
        MonomorphicValue::Unit { .. } => vec![false],
        MonomorphicValue::Bool { value, .. } => vec![*value],
        MonomorphicValue::Signed { value, .. }
        | MonomorphicValue::Unsigned { value, .. }
        | MonomorphicValue::Enum { carrier: value, .. } => {
            scalar(value.parse::<i128>().unwrap() as u128)
        }
        MonomorphicValue::Char { utf16, .. } => scalar(*utf16 as u128),
        MonomorphicValue::F32Bits { bits, .. } | MonomorphicValue::F64Bits { bits, .. } => {
            scalar(u128::from_str_radix(bits, 16).unwrap())
        }
        MonomorphicValue::Guid { n, .. } => scalar(u128::from_str_radix(n, 16).unwrap()),
        MonomorphicValue::Date { day_number, .. } => scalar(*day_number as u128),
        MonomorphicValue::Time { ticks, .. }
        | MonomorphicValue::Duration { ticks, .. }
        | MonomorphicValue::Instant {
            milliseconds: ticks,
            ..
        } => scalar(ticks.parse::<i128>().unwrap() as u128),
        MonomorphicValue::DecimalBits {
            negative,
            scale,
            coefficient,
            ..
        } => product(vec![
            vec![*negative],
            raw(*scale as u128, 8),
            raw(coefficient.parse().unwrap(), 96),
        ]),
        MonomorphicValue::Product { fields, .. } => {
            product(fields.iter().map(|f| storage(&f.value, types)).collect())
        }
        MonomorphicValue::OrderedEntry { key, value, .. } => {
            product(vec![storage(key, types), storage(value, types)])
        }
        MonomorphicValue::Money {
            amount, currency, ..
        } => product(vec![storage(amount, types), storage(currency, types)]),
        MonomorphicValue::Transition {
            state,
            events,
            response,
            ..
        } => {
            let OrdinaryShape::Product { fields } = shape else {
                panic!()
            };
            let values = MonomorphicValue::Sequence {
                type_id: reference(&fields[1].shape).into(),
                elements: events.clone(),
            };
            product(vec![
                storage(state, types),
                storage(&values, types),
                storage(response, types),
            ])
        }
        MonomorphicValue::Sequence { elements, .. }
        | MonomorphicValue::Array { elements, .. }
        | MonomorphicValue::OrderedSet { elements, .. } => {
            let OrdinaryShape::Sequence { capacity, element } = shape else {
                panic!()
            };
            sequence(
                *capacity,
                element,
                elements.iter().map(|v| storage(v, types)).collect(),
                types,
            )
        }
        MonomorphicValue::String { utf16, .. } => {
            let OrdinaryShape::Sequence { capacity, element } = shape else {
                panic!()
            };
            sequence(
                *capacity,
                element,
                utf16.iter().map(|v| raw(*v as u128, 16)).collect(),
                types,
            )
        }
        MonomorphicValue::OrderedMap { entries, .. } => {
            let OrdinaryShape::Sequence { capacity, element } = shape else {
                panic!()
            };
            sequence(
                *capacity,
                element,
                entries
                    .iter()
                    .map(|e| product(vec![storage(&e.key, types), storage(&e.value, types)]))
                    .collect(),
                types,
            )
        }
        MonomorphicValue::ParseError { arm, .. } => scalar(match arm {
            ParseErrorArm::InputBound => 0,
            ParseErrorArm::Syntax => 1,
            ParseErrorArm::Noncanonical => 2,
            ParseErrorArm::ScalePrecision => 3,
            ParseErrorArm::Range => 4,
        }),
        _ => {
            let OrdinaryShape::Sum { arms } = shape else {
                panic!()
            };
            let (tag, children) = match value {
                MonomorphicValue::Option { arm, value, .. } => (
                    if *arm == OptionArm::None { 0 } else { 1 },
                    value.iter().map(|v| storage(v, types)).collect(),
                ),
                MonomorphicValue::BoundaryPresence { arm, value, .. } => (
                    match arm {
                        BoundaryArm::Missing => 0,
                        BoundaryArm::Null => 1,
                        BoundaryArm::Value => 2,
                    },
                    value.iter().map(|v| storage(v, types)).collect(),
                ),
                MonomorphicValue::TaggedSum { arm, payload, .. } => (
                    arms.iter().find(|a| &a.id == arm).unwrap().tag,
                    payload.iter().map(|v| storage(v, types)).collect(),
                ),
                MonomorphicValue::ClosedException { tag, payload, .. } => {
                    let children = if let Some(v) = payload {
                        let MonomorphicValue::Product { fields, .. } = v.as_ref() else {
                            panic!()
                        };
                        fields.iter().map(|f| storage(&f.value, types)).collect()
                    } else {
                        vec![]
                    };
                    (*tag, children)
                }
                _ => panic!("unsupported sample"),
            };
            let payload_depth = shape_depth(shape, types) - 1;
            let child = product(children);
            let child_depth = addr(child.len());
            let mut data = vec![false; 1 << payload_depth];
            for (i, &b) in child.iter().enumerate() {
                data[i << (payload_depth - child_depth)] = b;
            }
            product(vec![raw(tag as u128, 32), data])
        }
    };
    assert_eq!(packed.len(), 1 << types[value.type_id()].depth);
    packed
}
pub(super) fn sample(
    id: &str,
    seed: usize,
    types: &BTreeMap<String, OrdinaryCarrier>,
    facts: &Value,
    closed: &ClosedInstanceSet,
) -> MonomorphicValue {
    let own = id.to_owned();
    let shape = &types[id].shape;
    let s = |id: &str, seed| sample(id, seed, types, facts, closed);
    if let Some(source) = facts["types"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["id"] == id)
    {
        if source["kind"] == "enum" {
            let mut values = source["enum_values"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect::<Vec<_>>();
            values.sort_by_key(|v| v.parse::<i128>().unwrap());
            let underlying = source["enum_underlying"]
                .as_str()
                .unwrap()
                .trim_start_matches("mpk.csharp.value.")
                .trim_end_matches(".v1")
                .to_owned();
            return MonomorphicValue::Enum {
                type_id: own,
                underlying,
                carrier: values[seed % values.len()].into(),
            };
        }
        let OrdinaryShape::Product { fields } = shape else {
            panic!()
        };
        let members = source["members"].as_array().unwrap();
        assert_eq!(fields.len(), members.len());
        return MonomorphicValue::Product {
            type_id: own,
            fields: fields
                .iter()
                .zip(members)
                .enumerate()
                .map(|(i, (f, member))| NamedMonomorphicValue {
                    name: member["name"].as_str().unwrap().into(),
                    value: Box::new(s(
                        reference(&f.shape),
                        if seed < 3 {
                            seed
                        } else if i + 1 == fields.len() {
                            seed - 2
                        } else {
                            1
                        },
                    )),
                })
                .collect(),
        };
    }
    if let Some(token) = id
        .strip_prefix("mpk.csharp.value.")
        .and_then(|s| s.strip_suffix(".v1"))
    {
        return match token {
            "unit" => MonomorphicValue::Unit { type_id: own },
            "bool" => MonomorphicValue::Bool {
                type_id: own,
                value: !seed.is_multiple_of(2),
            },
            "i8" | "i16" | "i32" | "i64" => MonomorphicValue::Signed {
                type_id: own,
                value: (seed as i32 - 1).to_string(),
            },
            "u8" | "u16" | "u32" | "u64" => MonomorphicValue::Unsigned {
                type_id: own,
                value: seed.to_string(),
            },
            "char" => MonomorphicValue::Char {
                type_id: own,
                utf16: seed as u16,
            },
            "string" => MonomorphicValue::String {
                type_id: own,
                utf16: match seed % 4 {
                    0 => vec![],
                    1 => vec![65],
                    2 => vec![65, 66],
                    _ => vec![66],
                },
            },
            "f32" => MonomorphicValue::F32Bits {
                type_id: own,
                bits: ["7fc00000", "80000000", "00000000", "3f800000"][seed % 4].into(),
            },
            "f64" => MonomorphicValue::F64Bits {
                type_id: own,
                bits: [
                    "7ff8000000000000",
                    "8000000000000000",
                    "0000000000000000",
                    "3ff0000000000000",
                ][seed % 4]
                    .into(),
            },
            "decimal" => MonomorphicValue::DecimalBits {
                type_id: own,
                negative: seed == 2 || seed == 4,
                scale: if seed == 0 {
                    2
                } else if seed == 4 {
                    28
                } else {
                    0
                },
                coefficient: if seed == 0 {
                    "100"
                } else if seed == 2 {
                    "2"
                } else if seed >= 4 {
                    "0"
                } else {
                    "1"
                }
                .into(),
            },
            "date" => MonomorphicValue::Date {
                type_id: own,
                day_number: seed as u32,
            },
            "time" => MonomorphicValue::Time {
                type_id: own,
                ticks: seed.to_string(),
            },
            "duration" => MonomorphicValue::Duration {
                type_id: own,
                ticks: (seed as i32 - 1).to_string(),
            },
            "instant" => MonomorphicValue::Instant {
                type_id: own,
                milliseconds: (seed as i32 - 1).to_string(),
            },
            "guid" => MonomorphicValue::Guid {
                type_id: own,
                n: format!("{seed:032x}"),
            },
            "day_of_week" => MonomorphicValue::Enum {
                type_id: own,
                underlying: "i32".into(),
                carrier: (seed % 7).to_string(),
            },
            "parse_error" => MonomorphicValue::ParseError {
                type_id: own,
                arm: [
                    ParseErrorArm::InputBound,
                    ParseErrorArm::Syntax,
                    ParseErrorArm::Noncanonical,
                    ParseErrorArm::ScalePrecision,
                    ParseErrorArm::Range,
                ][seed % 5],
            },
            "exception" => {
                let OrdinaryShape::Sum { arms } = shape else {
                    panic!()
                };
                if seed >= 3 {
                    if let Some(arm) = arms.iter().find(|a| types.contains_key(&a.id)) {
                        return MonomorphicValue::ClosedException {
                            type_id: own,
                            tag: arm.tag,
                            source_type_id: Some(arm.id.clone()),
                            payload: Some(Box::new(s(&arm.id, seed - 3))),
                        };
                    }
                }
                let builtin = arms
                    .iter()
                    .filter(|a| a.fields.is_empty())
                    .collect::<Vec<_>>();
                let arm = builtin[seed % builtin.len()];
                MonomorphicValue::ClosedException {
                    type_id: own,
                    tag: arm.tag,
                    source_type_id: None,
                    payload: None,
                }
            }
            _ => panic!("unknown scalar {token}"),
        };
    }
    let entry = closed
        .entries()
        .iter()
        .find(|e| e["instance_id"] == id)
        .unwrap();
    let template = entry["template_id"]
        .as_str()
        .unwrap()
        .trim_start_matches("mpk.csharp.semantic.")
        .trim_end_matches(".v1");
    match (template, shape) {
        ("bounded_sequence", OrdinaryShape::Sequence { element, .. }) => {
            MonomorphicValue::Sequence {
                type_id: own,
                elements: (0..seed % 3)
                    .map(|i| s(reference(element), i + seed / 3))
                    .collect(),
            }
        }
        ("ordered_set", OrdinaryShape::Sequence { element, .. }) => MonomorphicValue::OrderedSet {
            type_id: own,
            elements: (0..seed % 3).map(|i| s(reference(element), i)).collect(),
        },
        ("ordered_map", OrdinaryShape::Sequence { element, .. }) => {
            let OrdinaryShape::Product { fields } = element.as_ref() else {
                panic!()
            };
            MonomorphicValue::OrderedMap {
                type_id: own,
                entries: (0..seed % 3)
                    .map(|i| MonomorphicMapEntry {
                        key: Box::new(s(reference(&fields[0].shape), i)),
                        value: Box::new(s(reference(&fields[1].shape), seed + i)),
                    })
                    .collect(),
            }
        }
        ("ordered_entry", OrdinaryShape::Product { fields }) => MonomorphicValue::OrderedEntry {
            type_id: own,
            key: Box::new(s(reference(&fields[0].shape), seed)),
            value: Box::new(s(reference(&fields[1].shape), seed)),
        },
        ("money", OrdinaryShape::Product { fields }) => MonomorphicValue::Money {
            type_id: own,
            // Currency order must win even when amount order points the other way.
            amount: Box::new(s(
                reference(&fields[0].shape),
                if seed == 1 { 2 } else { seed },
            )),
            currency: Box::new(s(reference(&fields[1].shape), seed)),
        },
        ("transition", OrdinaryShape::Product { fields }) => {
            let MonomorphicValue::Sequence { elements, .. } = s(reference(&fields[1].shape), seed)
            else {
                panic!()
            };
            MonomorphicValue::Transition {
                type_id: own,
                state: Box::new(s(reference(&fields[0].shape), seed)),
                events: elements,
                response: Box::new(s(reference(&fields[2].shape), seed)),
            }
        }
        ("option", OrdinaryShape::Sum { arms }) => MonomorphicValue::Option {
            type_id: own,
            arm: if seed.is_multiple_of(2) {
                OptionArm::None
            } else {
                OptionArm::Some
            },
            value: if seed.is_multiple_of(2) {
                None
            } else {
                Some(Box::new(s(reference(&arms[1].fields[0].shape), seed / 2)))
            },
        },
        ("boundary_field", OrdinaryShape::Sum { arms }) => MonomorphicValue::BoundaryPresence {
            type_id: own,
            arm: [BoundaryArm::Missing, BoundaryArm::Null, BoundaryArm::Value][seed % 3],
            value: if seed % 3 == 2 {
                Some(Box::new(s(reference(&arms[2].fields[0].shape), seed)))
            } else {
                None
            },
        },
        (_, OrdinaryShape::Sum { arms }) => {
            let arm = &arms[seed % arms.len()];
            let payload = arm
                .fields
                .iter()
                .map(|f| {
                    s(
                        reference(&f.shape),
                        if template == "validation" && arm.tag == 1 {
                            1
                        } else {
                            seed
                        },
                    )
                })
                .collect();
            MonomorphicValue::TaggedSum {
                type_id: own,
                arm: arm.id.clone(),
                payload,
            }
        }
        _ => panic!("unexpected template {template}"),
    }
}
fn value(bits: Vec<bool>) -> V {
    if bits.len() == 1 {
        V::Bit(bits[0])
    } else {
        V::Cube(bits)
    }
}
fn order(c: &Certificate, v: V) -> i32 {
    (0..32).fold(0u32, |n, i| {
        let mut v = v.clone();
        for b in 0..5 {
            v = apply(c, v, V::Bit(i & (1 << b) != 0));
        }
        n | ((observed_bit(v) as u32) << i)
    }) as i32
}
fn check_sequence_index_words(c: &Certificate, d: &OrdinaryRelationDefinition) {
    use mpk_cert::encode::{DeclarationKind, TermNode};
    let OrdinaryShape::Sequence { capacity, .. } = d.carrier.shape else {
        return;
    };
    let prefix = d.equality_definition.strip_suffix(".Equal").unwrap();
    let read_name = format!("{prefix}.ReadAt");
    let declaration = c
        .declarations
        .iter()
        .find(|decl| c.name_table[decl.name as usize] == d.equality_definition)
        .unwrap();
    let DeclarationKind::Def { value, .. } = declaration.kind else {
        panic!()
    };
    let mut todo = vec![value];
    let mut seen = BTreeSet::new();
    let mut indices = BTreeSet::new();
    while let Some(term) = todo.pop() {
        if !seen.insert(term) {
            continue;
        }
        match &c.term_table[term as usize] {
            TermNode::App {
                function,
                arguments,
            } => {
                if let TermNode::Const { global, .. } = c.term_table[*function as usize] {
                    if c.name_table[c.declarations[global as usize].name as usize] == read_name {
                        assert_eq!(arguments.len(), 2);
                        indices.insert(arguments[1]);
                    }
                }
                todo.push(*function);
                todo.extend(arguments);
            }
            TermNode::Lam { body, .. } | TermNode::Pi { body, .. } => todo.push(*body),
            TermNode::Let { value, body, .. } => {
                todo.push(*value);
                todo.push(*body);
            }
            _ => {}
        }
    }
    assert_eq!(
        indices.len(),
        1,
        "one shared index expression for both operands"
    );
    let index = *indices.first().unwrap();
    let width = addr(capacity as usize);
    let mut cases = BTreeSet::from([0u32, capacity - 1]);
    for bit in 0..width {
        cases.extend([(1u32 << bit) - 1, 1u32 << bit]);
    }
    for i in cases {
        // This is the actual ReadAt argument extracted from the generated
        // equality predicate, under its low-first selector binders and operands.
        let mut env = (0..width)
            .rev()
            .map(|b| V::Bit(i & (1 << b) != 0))
            .collect::<Vec<_>>();
        env.extend([V::Bit(false), V::Bit(false)]);
        assert_eq!(
            order(c, core_eval::eval(c, index, &env)) as u32,
            i,
            "{} index {i}",
            d.carrier.type_id
        );
    }
}
pub(super) fn sources() -> Vec<(String, Value, Value)> {
    let mut cases = vec![];
    for family in ["binding-vc", "construction-vc", "exception-vc"] {
        let reqs = read(&format!("{family}/requests.json"));
        let responses = read(&format!("{family}/responses.json"));
        for row in reqs.as_array().unwrap() {
            let response = responses
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["id"] == row["id"])
                .unwrap();
            if response.get("reject").is_some() {
                continue;
            }
            if family == "construction-vc"
                && !["positive_constructor", "enum_zero"].contains(&row["id"].as_str().unwrap())
            {
                continue;
            }
            if family == "exception-vc" && row["id"] != "allowed" {
                continue;
            }
            cases.push((
                format!("{family}-{}", row["id"].as_str().unwrap()),
                row.clone(),
                response["facts"].clone(),
            ));
        }
    }
    let rows = read("data-phase/data-stage-replay.json");
    for (label, id) in [
        (
            "nullable-float",
            "07aeba9ade3eef2735bcdebaee8ed8346aea1c3cfa6bfd0351f2afae2aadad12",
        ),
        (
            "floats",
            "0f374d42b0b7af270138710ba14df3757ee3c519d7dc69b252630177e90d5caf",
        ),
        (
            "nested-box",
            "04140f88b477a4a825b97a2d761a3f18b9dd1658fc84127b69a754b6f0138fad",
        ),
        (
            "string",
            "5c9185c954edd922bee4892b0ad4a612ee6efe729b9d0a907ea0b083dc32effc",
        ),
        (
            "date",
            "05e148b20eb9017bdab9a113ab247301fa596ef171961443f1c68727025fe98f",
        ),
        (
            "guid",
            "cbb74344e9ab18afe5cc41c88bc1b311b2bfb4714ecbc2d6896ff7d152af6efd",
        ),
    ] {
        let row = rows
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        cases.push((label.into(), row.clone(), row["outcome"]["facts"].clone()));
    }
    cases
}
#[test]
fn csharp_03_t06_w09_relations_original_sources_and_semantics() {
    // Use the same stack budget as the existing decimal core-observation suite.
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| relations_original_sources_and_semantics(true))
        .unwrap()
        .join()
        .unwrap();
}
#[test]
fn csharp_03_t06_w09_relations_original_source_certificates() {
    relations_original_sources_and_semantics(false);
}
fn relations_original_sources_and_semantics(observe: bool) {
    let bundle = b();
    let cases = sources();
    assert_eq!(cases.len(), 21);
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/relations");
    let output = std::env::var_os("MPK_W09_RELATIONS_OUT").map(std::path::PathBuf::from);
    if let Some(p) = &output {
        fs::create_dir_all(p).unwrap();
    }
    let mut metrics = vec![];
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    for (id, row, facts) in cases {
        eprintln!("relation source {id}");
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
        let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
        let types = layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect::<BTreeMap<_, _>>();
        let p = generate_csharp_practical_ordinary_relations(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let closed = emitted.closure().closed();
        let roots = emitted.closure().roots();
        let internal = closed
            .entries()
            .iter()
            .filter(|e| e["template_id"] == "mpk.csharp.semantic.sequence_construction.v1")
            .map(|e| e["instance_id"].as_str().unwrap())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            p.definitions()
                .iter()
                .map(|d| d.carrier.type_id.as_str())
                .collect::<Vec<_>>(),
            types
                .keys()
                .map(String::as_str)
                .filter(|id| !internal.contains(id))
                .collect::<Vec<_>>()
        );
        let bytes = p.certificate_bytes();
        let c = mpk_cert::decode_canonical_certificate(bytes).unwrap();
        eprintln!(
            "relation source {id}: generated {} terms",
            c.term_table.len()
        );
        let mut observations = 0;
        for d in p.definitions() {
            check_sequence_index_words(&c, d);
            let oracle =
                generate_structural_program(&bundle, roots, closed, &d.carrier.type_id).unwrap();
            assert_eq!(d.compare_definition.is_some(), oracle.is_total());
            if !oracle.is_total() {
                let root_name = d
                    .carrier
                    .type_id
                    .as_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>();
                assert!(!c.name_table.contains(&format!(
                    "Mpk.CSharp.Ordinary.Relation.T{root_name}.Compare"
                )));
            }
            let values = (0..6)
                .map(|seed| sample(&d.carrier.type_id, seed, &types, &facts, closed))
                .collect::<Vec<_>>();
            // Validate all source-derived samples even in the fast certificate
            // pass, before spending time observing deep decimal circuits.
            for value in &values {
                oracle.structural_equal(value, value).unwrap_or_else(|e| {
                    panic!("{id} {} sample validity: {e:?}", d.carrier.type_id)
                });
            }
            if !observe {
                continue;
            }
            eprintln!("relation source {id}: observe {}", d.carrier.type_id);
            let mut observed_pairs = BTreeSet::new();
            for (a, v) in [
                (0usize, 0usize),
                (0, 1),
                (1, 2),
                (2, 0),
                (1, 1),
                (3, 3),
                (3, 4),
                (1, 4),
                (2, 5),
                (3, 5),
                (5, 3),
                (4, 5),
            ] {
                let left = &values[a];
                let right = &values[v];
                if !observed_pairs.insert(serde_json::to_vec(&(left, right)).unwrap()) {
                    continue;
                }
                let expected = oracle
                    .structural_equal(left, right)
                    .unwrap_or_else(|e| panic!("{id} {} sample: {e:?}", d.carrier.type_id));
                let args = vec![value(storage(left, &types)), value(storage(right, &types))];
                assert_eq!(
                    observed_bit(run(&c, &d.equality_definition, args.clone())),
                    expected,
                    "{id} {} equality {a} {v}",
                    d.carrier.type_id
                );
                if let Some(cmp) = &d.compare_definition {
                    let expected = oracle.canonical_compare(left, right).unwrap();
                    assert_eq!(
                        order(&c, run(&c, cmp, args)).cmp(&0),
                        expected,
                        "{id} {} order {a} {v}",
                        d.carrier.type_id
                    );
                } else {
                    assert!(oracle.canonical_compare(left, right).is_err());
                }
                observations += 1;
            }
            // Empty sequence tails and empty active sum payloads are not
            // semantic fields. Their zero-padding domain is checked separately.
            let ignores_payload = match &d.carrier.shape {
                OrdinaryShape::Sequence { .. } => true,
                OrdinaryShape::Sum { arms } => arms.first().is_some_and(|a| a.fields.is_empty()),
                _ => false,
            };
            if ignores_payload {
                let bits = storage(&values[0], &types);
                let mut changed = bits.clone();
                *changed.last_mut().unwrap() ^= true;
                assert_eq!(
                    observed_bit(run(
                        &c,
                        &d.equality_definition,
                        vec![value(bits), value(changed)]
                    )),
                    oracle.structural_equal(&values[0], &values[0]).unwrap(),
                    "{id} {} ignored payload",
                    d.carrier.type_id
                );
                observations += 1;
            }
        }
        let metadata = p.canonical_bytes();
        assert_eq!(
            import_csharp_practical_ordinary_relations(&metadata, bytes, vir).unwrap(),
            p
        );
        let original: Value = serde_json::from_slice(&metadata).unwrap();
        for key in [
            "source_ir_sha256",
            "foundation_sha256",
            "certificate_sha256",
            "schema",
            "static_transformers",
        ] {
            let mut changed = original.clone();
            changed[key] = json!("forged");
            assert!(import_csharp_practical_ordinary_relations(
                &serde_json::to_vec(&changed).unwrap(),
                bytes,
                vir
            )
            .is_err());
        }
        let mut changed = original.clone();
        changed["definitions"] = json!([]);
        assert!(import_csharp_practical_ordinary_relations(
            &serde_json::to_vec(&changed).unwrap(),
            bytes,
            vir
        )
        .is_err());
        let mut bad = bytes.to_vec();
        *bad.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_relations(&metadata, &bad, vir).is_err());
        if let Some((m, c)) = &previous {
            if m != &metadata {
                assert!(import_csharp_practical_ordinary_relations(m, c, vir).is_err());
            }
        }
        previous = Some((metadata, bytes.to_vec()));
        let file = format!("{id}.hex");
        let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
        if let Some(out) = &output {
            fs::write(out.join(&file), hex).unwrap();
        } else {
            assert_eq!(fs::read_to_string(fixture.join(&file)).unwrap(), hex);
        }
        metrics.push(json!({"id":id,"file":file,"observations":observations,"terms":c.term_table.len(),"declarations":c.declarations.len(),"metadata":original}));
    }
    let metrics_file = if observe {
        "metrics.json"
    } else {
        "certificates.json"
    };
    if let Some(out) = output {
        fs::write(
            out.join(metrics_file),
            serde_json::to_vec_pretty(&metrics).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            serde_json::from_slice::<Value>(&fs::read(fixture.join(metrics_file)).unwrap())
                .unwrap(),
            json!(metrics)
        );
    }
}

#[test]
fn csharp_03_t06_w09_money_relation_ties_and_numeric_equality() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let bundle = b();
            let (_, row, facts) = sources()
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
            let layouts = generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap();
            let types = layouts
                .carriers()
                .iter()
                .map(|c| (c.type_id.clone(), c.clone()))
                .collect::<BTreeMap<_, _>>();
            let p = generate_csharp_practical_ordinary_relations(emitted.vir()).unwrap();
            let closed = emitted.closure().closed();
            let roots = emitted.closure().roots();
            let id = closed
                .entries()
                .iter()
                .find(|e| e["template_id"] == "mpk.csharp.semantic.money.v1")
                .unwrap()["instance_id"]
                .as_str()
                .unwrap();
            let d = p
                .definitions()
                .iter()
                .find(|d| d.carrier.type_id == id)
                .unwrap();
            let original = sample(id, 0, &types, &facts, closed);
            let mut equivalent = original.clone();
            let MonomorphicValue::Money { amount, .. } = &mut equivalent else {
                panic!()
            };
            **amount = sample("mpk.csharp.value.decimal.v1", 1, &types, &facts, closed);
            let mut smaller = original.clone();
            let MonomorphicValue::Money { amount, .. } = &mut smaller else {
                panic!()
            };
            **amount = sample("mpk.csharp.value.decimal.v1", 2, &types, &facts, closed);
            let oracle = generate_structural_program(&bundle, roots, closed, id).unwrap();
            assert!(oracle.structural_equal(&original, &equivalent).unwrap());
            assert_eq!(
                oracle.canonical_compare(&original, &equivalent).unwrap(),
                std::cmp::Ordering::Equal
            );
            assert_eq!(
                oracle.canonical_compare(&original, &smaller).unwrap(),
                std::cmp::Ordering::Greater
            );
            let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
            for (label, right, equal, sign) in [
                (
                    "same currency, equivalent decimal representations",
                    &equivalent,
                    true,
                    0,
                ),
                ("same currency, different amounts", &smaller, false, 1),
            ] {
                let args = vec![
                    value(storage(&original, &types)),
                    value(storage(right, &types)),
                ];
                assert_eq!(
                    observed_bit(run(&c, &d.equality_definition, args.clone())),
                    equal,
                    "{label}"
                );
                assert_eq!(
                    order(&c, run(&c, d.compare_definition.as_ref().unwrap(), args)).signum(),
                    sign,
                    "{label}"
                );
            }
        })
        .unwrap()
        .join()
        .unwrap();
}
