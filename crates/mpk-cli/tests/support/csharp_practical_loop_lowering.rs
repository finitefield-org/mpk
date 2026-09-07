//! CSHARP-03-T04-W02 actual-source goldens and independent CFG interpretation.
use super::*;
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};
fn lowering_cases() -> Vec<Value> {
    serde_json::from_slice(&read(
        "develop/migrations/csharp-03/loop-lowering/source-cases.json",
    ))
    .unwrap()
}
fn prepare(
    case: &Value,
    mutation: impl FnOnce(&mut Value),
) -> Result<LoweredLoopControl, LoopLoweringError> {
    prepare_with(case, "total", false, mutation)
}
fn prepare_with(
    case: &Value,
    termination: &str,
    empty_decreases: bool,
    mutation: impl FnOnce(&mut Value),
) -> Result<LoweredLoopControl, LoopLoweringError> {
    let b = bundle();
    let (r, c) = roots(&b);
    let mut source = case.clone();
    source["facts"] = case["lowering"]["facts"].clone();
    let (context, captures) = context_support::context_with_sidecar(
        &b,
        case["root"].as_str().unwrap(),
        case["source"].as_str().unwrap().as_bytes(),
        |ctx| {
            let mut rows = rows(&source);
            if empty_decreases {
                for row in &mut rows {
                    set(row, "decreases", J::Array(vec![]));
                }
            }
            hashed(document(ctx, &source, termination, rows))
        },
    );
    let facts = serde_json::to_vec(&source["facts"]).unwrap();
    let mut wire = case["lowering"].clone();
    mutation(&mut wire);
    prepare_loop_lowering(
        &b,
        &r,
        &c,
        &context,
        &captures,
        &facts,
        &serde_json::to_vec(&wire).unwrap(),
        &DataContractEnvironment::default(),
    )
}
#[derive(Clone, Debug)]
pub(super) enum R {
    Number(i64),
    Bool(bool),
    Array(Rc<RefCell<Vec<R>>>),
    Text(Vec<u16>),
    Null,
    // Test fixture immutable value; T03 construction is independently tested.
    BoxValue(i64),
    Float(f64),
    Decimal(i128, u32),
}
impl R {
    pub(super) fn number(&self) -> Result<i64, String> {
        if let Self::Number(n) = self {
            Ok(*n)
        } else {
            Err("interpreter_number".into())
        }
    }
    pub(super) fn boolean(&self) -> Result<bool, String> {
        if let Self::Bool(n) = self {
            Ok(*n)
        } else {
            Err("interpreter_bool".into())
        }
    }
    fn length(&self) -> Result<usize, String> {
        match self {
            Self::Array(a) => Ok(a.borrow().len()),
            Self::Text(s) => Ok(s.len()),
            Self::Null => Err("NullReferenceException".into()),
            _ => Err("interpreter_length".into()),
        }
    }
}
fn int(n: i64) -> Result<R, String> {
    i32::try_from(n)
        .map(|v| R::Number(v.into()))
        .map_err(|_| "OverflowException".into())
}
pub(super) fn evaluate(
    n: &LoopControlNode,
    source: Option<&LoopSourceOperation>,
    v: &[R],
    slots: &mut BTreeMap<String, R>,
) -> Result<R, String> {
    let fail = || "interpreter_operation".to_owned();
    let op = source
        .map(|o| o.traits.split('|').next().unwrap())
        .unwrap_or("");
    match n.operation.as_str() {
        "load" => slots
            .get(&n.slot)
            .cloned()
            .ok_or_else(|| format!("missing_slot:{}", n.slot)),
        "store" | "pattern_bind" => {
            slots.insert(n.slot.clone(), v[0].clone());
            Ok(v[0].clone())
        }
        "zero" => Ok(R::Number(0)),
        "true" | "pattern_true" => Ok(R::Bool(true)),
        "pattern_false" => Ok(R::Bool(false)),
        "initializer_index" => Ok(R::Number(n.slot.parse().map_err(|_| fail())?)),
        "constant" | "pattern_constant" => {
            let value = source
                .and_then(|s| s.constant.as_deref())
                .ok_or_else(fail)?;
            if value == "null" {
                return Ok(R::Null);
            }
            let (tag, value) = value.split_once(':').ok_or_else(fail)?;
            match tag {
                "bool" => Ok(R::Bool(value == "true")),
                "f64" => Ok(R::Float(f64::from_bits(u64::from_str_radix(value,16).map_err(|_|fail())?))),
                "f32" => Ok(R::Float(f32::from_bits(u32::from_str_radix(value,16).map_err(|_|fail())?) as f64)),
                "decimal" => {
                    let bits=value.split(',').map(|s|u32::from_str_radix(s,16).unwrap()).collect::<Vec<_>>();
                    let magnitude=i128::from(bits[0])+(i128::from(bits[1])<<32)+(i128::from(bits[2])<<64);
                    Ok(R::Decimal(if bits[3]>>31==1 {-magnitude}else{magnitude},(bits[3]>>16)&255))
                },
                "string_utf16" => Ok(R::Text(
                    (0..value.len())
                        .step_by(4)
                        .map(|i| u16::from_str_radix(&value[i..i + 4], 16).unwrap())
                        .collect(),
                )),
                "System.Int32" | "char" => Ok(R::Number(value.parse().map_err(|_| fail())?)),
                _ => Err(fail()),
            }
        }
        "less" => Ok(R::Bool(v[0].number()? < v[1].number()?)),
        "increment" => int(v[0].number()? + 1),
        "unary_update" => int(v[0].number()?
            + if source.ok_or_else(fail)?.kind == "Increment" {
                1
            } else {
                -1
            }),
        "convert" | "iteration_convert" => Ok(v[0].clone()),
        "unary" => match op {
            "Minus" => int(-v[0].number()?),
            "Plus" => Ok(v[0].clone()),
            "Not" => Ok(R::Bool(!v[0].boolean()?)),
            _ => Err(fail()),
        },
        "binary" => {
            if let (R::Float(a),R::Float(b))=(&v[0],&v[1]) {
                return match op {"Divide"=>Ok(R::Float(a/b)),_=>Err(fail())};
            }
            let a = v[0].number()?;
            let b = v[1].number()?;
            match op {
                "Add" => int(a + b),
                "Subtract" => int(a - b),
                "Multiply" => int(a * b),
                "Divide" | "Remainder" => {
                    if b == 0 {
                        Err("DivideByZeroException".into())
                    } else if op == "Divide" {
                        int(a / b)
                    } else {
                        int(a % b)
                    }
                }
                "LessThan" => Ok(R::Bool(a < b)),
                "LessThanOrEqual" => Ok(R::Bool(a <= b)),
                "GreaterThan" => Ok(R::Bool(a > b)),
                "GreaterThanOrEqual" => Ok(R::Bool(a >= b)),
                "Equals" => Ok(R::Bool(a == b)),
                "NotEquals" => Ok(R::Bool(a != b)),
                _ => Err(format!("binary:{op}")),
            }
        }
        "pattern_not_null" | "pattern_type" => Ok(R::Bool(!matches!(v[0], R::Null))),
        "pattern_equal" => Ok(R::Bool(match (&v[0], &v[1]) {
            (R::Null, R::Null) => true,
            (R::Float(a),R::Float(b))=>a==b || a.is_nan()&&b.is_nan(),
            (R::Decimal(a,sa),R::Decimal(b,sb))=>a*10i128.pow(*sb)==b*10i128.pow(*sa),
            (R::Number(a), R::Number(b)) => a == b,
            (R::Bool(a), R::Bool(b)) => a == b,
            (R::Text(a), R::Text(b)) => a == b,
            _ => false,
        })),
        "pattern_relational" => {
            if let (R::Float(a),R::Float(b))=(&v[0],&v[1]) {return Ok(R::Bool(match op {"GreaterThan"=>a>b,"LessThan"=>a<b,"GreaterThanOrEqual"=>a>=b,"LessThanOrEqual"=>a<=b,_=>return Err(fail())}));}
            if let (R::Decimal(a,sa),R::Decimal(b,sb))=(&v[0],&v[1]) {let a=a*10i128.pow(*sb);let b=b*10i128.pow(*sa);return Ok(R::Bool(match op {"GreaterThan"=>a>b,"LessThan"=>a<b,"GreaterThanOrEqual"=>a>=b,"LessThanOrEqual"=>a<=b,_=>return Err(fail())}));}
            if matches!(v[0], R::Null) {
                return Ok(R::Bool(false));
            }
            let a = v[0].number()?;
            let b = v[1].number()?;
            Ok(R::Bool(match op {
                "LessThan" => a < b,
                "LessThanOrEqual" => a <= b,
                "GreaterThan" => a > b,
                "GreaterThanOrEqual" => a >= b,
                _ => return Err(fail()),
            }))
        }
        "pattern_length" => Ok(R::Bool(
            v[0].length()? == n.slot.parse::<usize>().map_err(|_| fail())?,
        )),
        "pattern_element" => {
            let index = n.slot.parse::<usize>().map_err(|_| fail())?;
            if let R::Array(a) = &v[0] {
                a.borrow().get(index).cloned().ok_or_else(fail)
            } else {
                Err(fail())
            }
        }
        "construct" if source.is_some_and(|s| s.symbol == csharp_practical_declaration_id(&json!({"kind":"constructor","namespace":"Business","owner":csharp_practical_declaration_id(&json!({"kind":"type","namespace":"Business","owner":"","name":"Box","parameter_type_ids":[],"result_type_id":""})).unwrap(),"name":"Box","parameter_type_ids":[ty("i32")],"result_type_id":csharp_practical_declaration_id(&json!({"kind":"type","namespace":"Business","owner":"","name":"Box","parameter_type_ids":[],"result_type_id":""})).unwrap()})).unwrap()) => {
            Ok(R::BoxValue(v[0].number()?))
        }
        "length" | "member" | "pattern_member" => {
            if let R::BoxValue(value) = v[0] {
                if !source
                    .is_some_and(|s| s.symbol.ends_with(".Stored") || s.symbol.ends_with(".Value"))
                {
                    return Err(fail());
                }
                int(value)
            } else {
                int(v[0].length()? as i64)
            }
        }
        "allocate" => {
            let count = v[0].number()?;
            if count < 0 {
                return Err("OverflowException".into());
            }
            if count > 4096 {
                return Err("profile_bound".into());
            }
            Ok(R::Array(Rc::new(RefCell::new(vec![
                R::Number(0);
                count as usize
            ]))))
        }
        "element" | "update" => {
            let length = v[0].length()?;
            let index = v[1].number()?;
            if index < 0 || index as usize >= length {
                return Err("IndexOutOfRangeException".into());
            }
            match &v[0] {
                R::Array(a) => {
                    if n.operation == "update" {
                        a.borrow_mut()[index as usize] = v[2].clone();
                        Ok(v[2].clone())
                    } else {
                        Ok(a.borrow()[index as usize].clone())
                    }
                }
                R::Text(t) if n.operation == "element" => Ok(R::Number(t[index as usize].into())),
                _ => Err(fail()),
            }
        }
        _ => Err(format!("interpreter:{}", n.operation)),
    }
}
pub(super) fn interpret(f: &LoopControlFunction, run: &Value) -> Result<i64, String> {
    let mut slots = BTreeMap::from([
        ("parameter:0".into(), R::Number(run["n"].as_i64().unwrap())),
        (
            "parameter:1".into(),
            R::Array(Rc::new(RefCell::new(
                run["a"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| R::Number(v.as_i64().unwrap()))
                    .collect(),
            ))),
        ),
        (
            "parameter:2".into(),
            R::Text(run["s"].as_str().unwrap().encode_utf16().collect()),
        ),
    ]);
    let nodes = f
        .nodes
        .iter()
        .map(|n| (n.id.as_str(), n))
        .collect::<BTreeMap<_, _>>();
    let mut values = BTreeMap::<String, R>::new();
    let mut current = &f.nodes[0];
    let mut pending = None;
    for _ in 0..100_000 {
        let inputs = current
            .inputs
            .iter()
            .map(|id| {
                values
                    .get(id)
                    .cloned()
                    .ok_or_else(|| format!("missing_value:{id}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        match current.kind.as_str() {
            "return" => return inputs.first().ok_or("missing_return")?.number(),
            "throw" => {
                return Err(
                    if current.slot == "System.Runtime.CompilerServices.SwitchExpressionException" {
                        "SwitchExpressionException".into()
                    } else {
                        pending.take().ok_or("missing_exception")?
                    },
                )
            }
            "branch" | "loop_header" => {
                current = nodes[current.successors[usize::from(!inputs[0].boolean()?)].as_str()];
                continue;
            }
            "evaluate" => match evaluate(
                current,
                current.source_ordinal.map(|i| &f.operations[i]),
                &inputs,
                &mut slots,
            ) {
                Ok(value) => {
                    values.insert(current.result.clone(), value);
                }
                Err(error) => {
                    if current.exceptional_successors.len() != 1 {
                        return Err(format!("missing_exception_edge:{error}"));
                    }
                    pending = Some(error);
                    current = nodes[current.exceptional_successors[0].as_str()];
                    continue;
                }
            },
            "entry" | "jump" | "break" | "continue" | "pattern_decision" => (),
            kind => return Err(format!("interpreter_node:{kind}")),
        }
        current = nodes[current.successors.first().ok_or("missing_edge")?.as_str()];
    }
    Err("step_limit".into())
}
#[test]
fn csharp_03_t04_w02_original_clr_and_independent_loop_interpreter_agree() {
    let mut count = 0;
    for case in lowering_cases()
        .into_iter()
        .filter(|c| c["accepted"] == true && !c["runs"].as_array().unwrap().is_empty())
    {
        let prepared = prepare(&case, |_| {}).unwrap_or_else(|e| panic!("{}: {e:?}", case["id"]));
        assert_eq!(prepared.artifact_count(), 0);
        let f = prepared
            .functions()
            .iter()
            .find(|f| f.callable_id == case["root"])
            .unwrap();
        for run in case["runs"].as_array().unwrap() {
            let actual = interpret(f, run);
            let expected = if let Some(value) = run["value"].as_i64() {
                Ok(value)
            } else {
                Err(run["error"].as_str().unwrap().into())
            };
            assert_eq!(actual, expected, "{}: {run}", case["id"]);
            count += 1;
        }
    }
    assert!(count >= 400);
}
#[test]
fn csharp_03_t04_w02_graph_attachment_ids_operands_and_abrupt_mutations_reject() {
    let case = lowering_cases()
        .into_iter()
        .find(|c| c["id"] == "nested")
        .unwrap();
    prepare(&case, |_| {}).unwrap();
    assert!(prepare(&case, |v| v["functions"][0]["nodes"][1]["id"] =
        json!("wrong"))
    .is_err());
    assert!(prepare(&case, |v| v["functions"][0]["loops"][1]["parent"] =
        Value::Null)
    .is_err());
    assert!(
        prepare(&case, |v| v["functions"][0]["loops"][0]["backedges"] =
            json!([]))
        .is_err()
    );
    assert!(prepare(&case, |v| {
        let nodes = v["functions"][0]["nodes"].as_array_mut().unwrap();
        let n = nodes.iter_mut().find(|n| n["kind"] == "break").unwrap();
        n["successors"] = json!([n["id"].as_str().unwrap()]);
    })
    .is_err());
    assert!(prepare(&case, |v| v["normalized_syntax_sha256"] =
        json!("0".repeat(64)))
    .is_err());
    assert!(prepare(&case, |v| {
        let n = v["functions"][0]["nodes"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|n| n["operation"] == "binary")
            .unwrap();
        n["inputs"][0] = json!("missing_value");
    })
    .is_err());
}
#[test]
fn csharp_03_t04_w02_collection_obligations_and_foreach_write_rejections_remain_pending() {
    let cases = lowering_cases();
    for id in ["count_fill", "count_fill_disagreement", "sort", "dedup"] {
        let case = cases.iter().find(|c| c["id"] == id).unwrap();
        let prepared = prepare(case, |_| {}).unwrap();
        assert!(prepared
            .contracts()
            .loops()
            .iter()
            .any(|l| l.obligations.contains(&"publication_complete".into())));
    }
    for id in [
        "borrow_write",
        "borrow_alias",
        "readonly_write",
        "goto",
        "switch",
    ] {
        assert_eq!(
            cases.iter().find(|c| c["id"] == id).unwrap()["accepted"],
            false
        );
    }
}

#[test]
fn csharp_03_t04_w02_real_ordered_binding_clauses_survive_cfg_lowering() {
    for mut source in lowering_cases().into_iter().filter(|c| {
        c["id"]
            .as_str()
            .is_some_and(|id| id.starts_with("set_") || id.starts_with("map_"))
    }) {
        source["facts"] = source["lowering"]["facts"].clone();
        let name = source["id"].as_str().unwrap().to_owned();
        check_collection_projection_source(&name, source);
    }
}
#[test]
fn csharp_03_t04_w02_explicit_var_foreach_equivalence_and_termination_boundaries() {
    let cases = lowering_cases();
    for name in ["foreach_array", "foreach_string"] {
        let explicit = cases.iter().find(|c| c["id"] == name).unwrap();
        let inferred = cases
            .iter()
            .find(|c| c["id"] == format!("{name}_var"))
            .unwrap();
        assert_eq!(
            explicit["lowering"]["functions"],
            inferred["lowering"]["functions"]
        );
        let partial = prepare_with(explicit, "partial", true, |_| {}).unwrap();
        assert!(partial
            .contracts()
            .loops()
            .iter()
            .all(|l| l.termination == "partial" && l.decreases.is_empty()));
        assert_eq!(
            prepare_with(explicit, "total", true, |_| {}).unwrap_err(),
            LoopLoweringError::Contract(LoopContractError::MissingDecreases)
        );
        assert_eq!(
            prepare_with(explicit, "total", true, |v| v["schema"] = json!("bad")).unwrap_err(),
            LoopLoweringError::Contract(LoopContractError::MissingDecreases)
        );
    }
}
#[test]
fn csharp_03_t04_w02_live_inputs_and_private_route_are_exact() {
    use sha2::{Digest, Sha256};
    let manifest: Value = serde_json::from_slice(&read(
        "develop/migrations/csharp-03/loop-lowering/loop-lowering-inputs.json",
    ))
    .unwrap();
    assert_eq!(manifest["work_item"], "CSHARP-03-T04-W02");
    for row in manifest["files"].as_array().unwrap() {
        let bytes = read(row["path"].as_str().unwrap());
        assert_eq!(row["size_bytes"], bytes.len());
        assert_eq!(row["sha256"], format!("{:x}", Sha256::digest(bytes)));
    }
    assert!(
        !String::from_utf8(read("csharp-tools/csharp2vir/csharp2vir.csproj"))
            .unwrap()
            .contains("PracticalLoopLowering")
    );
}

#[test]
fn csharp_03_t04_w02_cfg_budget_is_inclusive_and_precedes_contract_typing() {
    let case = lowering_cases()
        .into_iter()
        .find(|c| c["id"] == "while")
        .unwrap();
    let pad = |v: &mut Value, count: usize| {
        let f = &mut v["functions"][0];
        let owner = f["callable_id"].as_str().unwrap().to_owned();
        let nodes = f["nodes"].as_array_mut().unwrap();
        let target = nodes.iter().find(|n| n["kind"] == "return").unwrap()["id"].clone();
        while nodes.len() < count {
            nodes.push(json!({"id":format!("{owner}.node.{:06}",nodes.len()),"kind":"jump","source_ordinal":null,"operation":"","inputs":[],"result":"","slot":"","successors":[target],"exceptional_successors":[]}));
        }
    };
    for count in [1023, 1024] {
        prepare(&case, |v| pad(v, count)).unwrap();
    }
    assert_eq!(
        prepare(&case, |v| pad(v, 1025)).unwrap_err(),
        LoopLoweringError::Limit("cfg_blocks_per_method")
    );
    assert_eq!(
        prepare_with(&case, "total", true, |v| pad(v, 1025)).unwrap_err(),
        LoopLoweringError::Limit("cfg_blocks_per_method")
    );
}

#[test]
fn csharp_03_t04_w02_source_operation_cannot_change_into_a_synthetic_opcode() {
    let case = lowering_cases()
        .into_iter()
        .find(|c| c["id"] == "while")
        .unwrap();
    assert_eq!(
        prepare(&case, |v| {
            let node = v["functions"][0]["nodes"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|n| n["operation"] == "binary")
                .unwrap();
            node["operation"] = json!("less");
        })
        .unwrap_err(),
        LoopLoweringError::Operand
    );
}
