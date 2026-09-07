//! Differential observer for ordinary VIR, independent of the source handoff.
use super::*;
use mpk_vc::csharp_practical_vir_validation as v;
use std::collections::BTreeMap;

fn literal(value: &MonomorphicValue) -> Value {
    match value {
        MonomorphicValue::Unit { .. } | MonomorphicValue::Option { value: None, .. } => Value::Null,
        MonomorphicValue::Signed { value, .. } | MonomorphicValue::Unsigned { value, .. } => {
            json!(value.parse::<i64>().unwrap())
        }
        MonomorphicValue::Bool { value, .. } => json!(i64::from(*value)),
        MonomorphicValue::Char { utf16, .. } => json!(*utf16),
        MonomorphicValue::String { utf16, .. } => json!(utf16),
        MonomorphicValue::Enum { carrier, .. } => json!(carrier.parse::<i64>().unwrap()),
        MonomorphicValue::Option {
            value: Some(value), ..
        } => json!({"some":literal(value)}),
        MonomorphicValue::F32Bits { .. }
        | MonomorphicValue::F64Bits { .. }
        | MonomorphicValue::DecimalBits { .. }
        | MonomorphicValue::ClosedException { .. } => serde_json::to_value(value).unwrap(),
        _ => panic!("unsupported observer literal {value:?}"),
    }
}
fn typed(value: &Value, ty: &str) -> MonomorphicValue {
    if value.is_object() {
        return serde_json::from_value(value.clone()).unwrap();
    }
    if ty == "mpk.csharp.value.bool.v1" {
        MonomorphicValue::Bool {
            type_id: ty.into(),
            value: value.as_i64().unwrap() != 0,
        }
    } else if ty == "mpk.csharp.value.char.v1" {
        MonomorphicValue::Char {
            type_id: ty.into(),
            utf16: value.as_u64().unwrap() as u16,
        }
    } else if ty.starts_with("mpk.csharp.value.u") {
        MonomorphicValue::Unsigned {
            type_id: ty.into(),
            value: value.as_i64().unwrap().to_string(),
        }
    } else {
        MonomorphicValue::Signed {
            type_id: ty.into(),
            value: value.as_i64().unwrap().to_string(),
        }
    }
}
fn present(value: &Value) -> Result<Value, String> {
    value
        .get("some")
        .cloned()
        .ok_or_else(|| "InvalidOperationException".into())
}
// Frames suspend after local search and before unwind, allowing callers to run
// their filters first. These are observer frames, never source-handoff nodes.
struct Frame {
    function: String,
    values: BTreeMap<String, Value>,
    current: String,
    previous: String,
    pending: Option<Value>,
    child: Option<(Box<Frame>, String)>,
    unwinding: bool,
}
enum Outcome {
    Returned(Value),
    Escaped(Value),
    Searching(Value, Box<Frame>),
}
struct Observer<'a> {
    bundle: &'a ValidatedFoundationBundle,
    emitted: &'a EmittedDataPhase,
    steps: usize,
    epoch: u64,
    trace: Vec<String>,
    last_exception_payload: Option<Value>,
}
impl Observer<'_> {
    fn frame(&self, function: &v::PracticalVirFunction, args: Vec<Value>) -> Frame {
        assert_eq!(function.parameter_values.len(), args.len());
        Frame {
            function: function.id.clone(),
            values: function
                .parameter_values
                .iter()
                .zip(args)
                .map(|(p, v)| (p.id.clone(), v))
                .collect(),
            current: function.blocks[0].node.id.clone(),
            previous: String::new(),
            pending: None,
            child: None,
            unwinding: false,
        }
    }
    fn stamp(&mut self, mut value: Value) -> Value {
        if value.get("__epoch").is_none() {
            self.epoch += 1;
            value["__epoch"] = json!(self.epoch);
        }
        value
    }
    fn exception_type(&self, value: &Value) -> String {
        if let Some(ty) = value["source_type_id"].as_str() {
            return ty.to_owned();
        }
        derive_closed_exception_universe(
            self.emitted.closure().roots(),
            self.emitted.closure().closed(),
            self.emitted.vir().source_exceptions(),
        )
        .unwrap()
        .arms()
        .iter()
        .find(|a| Some(u64::from(a.tag)) == value["tag"].as_u64())
        .expect("closed exception tag")
        .type_id
        .clone()
    }
    fn settle(&mut self, mut result: Outcome) -> Outcome {
        while let Outcome::Searching(_, mut frame) = result {
            frame.unwinding = true;
            result = self.resume(*frame);
        }
        result
    }
    fn call(
        &mut self,
        function: &v::PracticalVirFunction,
        args: Vec<Value>,
    ) -> Result<Value, String> {
        let frame = self.frame(function, args);
        let result = self.resume(frame);
        match self.settle(result) {
            Outcome::Returned(v) => Ok(v),
            Outcome::Escaped(v) => {
                self.last_exception_payload = v.get("payload").filter(|v| !v.is_null()).cloned();
                Err(self.exception_type(&v).rsplit('.').next().unwrap().into())
            }
            Outcome::Searching(..) => unreachable!(),
        }
    }
    fn route_failure(&mut self, f: &mut Frame, block: &v::PracticalVirBlock, value: Value) {
        let function = self
            .emitted
            .vir()
            .functions()
            .iter()
            .find(|fun| fun.id == f.function)
            .unwrap();
        let ty = self.exception_type(&value);
        let edge = block
            .node
            .exceptional_successors
            .iter()
            .find(|e| e.exception_type_id == ty)
            .expect("typed exceptional successor");
        if let Some((region, clause, filter)) = function
            .exception_regions
            .iter()
            .flat_map(|r| {
                r.catches
                    .iter()
                    .filter_map(move |c| Some((r, c, c.filter.as_ref()?.execution.as_ref()?)))
            })
            .find(|(_, _, filter)| filter.node_ids.contains(&block.node.id))
        {
            self.trace.push(format!(
                "filter:{}:{}:throw:{}",
                region
                    .id
                    .rsplit('.')
                    .next()
                    .unwrap()
                    .parse::<usize>()
                    .unwrap(),
                clause.ordinal,
                ty
            ));
            let entry = function
                .blocks
                .iter()
                .find(|b| b.node.id == filter.entry_node_id)
                .unwrap();
            f.pending = Some(f.values[&entry.handler_exception_value.as_ref().unwrap().id].clone());
        } else {
            f.pending = Some(value);
        }
        f.previous = block.node.id.clone();
        f.current = edge.target_id.clone();
    }
    fn resume(&mut self, mut f: Frame) -> Outcome {
        let function = self
            .emitted
            .vir()
            .functions()
            .iter()
            .find(|fun| fun.id == f.function)
            .unwrap();
        let blocks = function
            .blocks
            .iter()
            .map(|b| (b.node.id.as_str(), b))
            .collect::<BTreeMap<_, _>>();
        loop {
            self.steps += 1;
            assert!(self.steps <= 100_000, "observer step bound");
            let block = blocks[f.current.as_str()];
            let is_catch = function.exception_regions.iter().any(|r| {
                r.catches
                    .iter()
                    .any(|c| c.handler_entry_node_id == f.current)
            });
            // An escaping caller must itself finish phase one before unwinding
            // the suspended callee. A selected catch/finally starts phase two.
            if block.node.tag == ControlNodeTag::Exit && f.child.is_some() && !f.unwinding {
                return Outcome::Searching(f.pending.clone().unwrap(), Box::new(f));
            }
            if f.child.is_some()
                && (is_catch
                    || matches!(
                        block.node.tag,
                        ControlNodeTag::FinallyEntry | ControlNodeTag::Exit
                    ))
            {
                let (mut child, call_id) = f.child.take().unwrap();
                child.unwinding = true;
                let pending = f.pending.clone().unwrap();
                let outcome = self.resume(*child);
                match outcome {
                    Outcome::Returned(v) => {
                        let source = blocks[call_id.as_str()];
                        f.values
                            .insert(source.invocation.as_ref().unwrap().result.id.clone(), v);
                        f.previous = call_id;
                        f.current = source.node.normal_successor_ids[0].clone();
                        f.pending = None;
                        continue;
                    }
                    Outcome::Escaped(v) if v["__epoch"] == pending["__epoch"] => {}
                    Outcome::Escaped(v) => {
                        self.route_failure(&mut f, blocks[call_id.as_str()], v);
                        continue;
                    }
                    Outcome::Searching(v, child) => {
                        f.child = Some((child, call_id.clone()));
                        self.route_failure(&mut f, blocks[call_id.as_str()], v);
                        continue;
                    }
                }
            }
            let phis = block
                .phi_values
                .iter()
                .map(|p| {
                    let input = p
                        .incoming
                        .iter()
                        .find(|i| i.predecessor_node_id == f.previous)
                        .unwrap();
                    (p.value.id.clone(), f.values[&input.value_id].clone())
                })
                .collect::<Vec<_>>();
            f.values.extend(phis);
            for v in &block.literal_values {
                let value = literal(&v.value);
                let value = if matches!(v.value, MonomorphicValue::ClosedException { .. }) {
                    self.stamp(value)
                } else {
                    value
                };
                f.values.insert(v.result.id.clone(), value);
            }
            if let Some(bound) = &block.handler_exception_value {
                if let Some(id) = &block.handler_exception_source_id {
                    f.pending = Some(f.values[id].clone());
                }
                f.values.insert(
                    bound.id.clone(),
                    f.pending.clone().expect("caught exception"),
                );
            }
            for region in &function.exception_regions {
                let id = region
                    .id
                    .rsplit('.')
                    .next()
                    .unwrap()
                    .parse::<usize>()
                    .unwrap();
                if region.try_entry_node_id == f.current {
                    self.trace.push(format!("try:{id}"));
                }
                if region.finally_entry_node_id.as_ref() == Some(&f.current) {
                    self.trace.push(format!("finally:{id}"));
                }
                for clause in &region.catches {
                    if clause.handler_entry_node_id == f.current {
                        self.trace.push(format!("catch:{id}:{}", clause.ordinal));
                    }
                    if let Some(filter) = clause.filter.as_ref().and_then(|v| v.execution.as_ref())
                    {
                        if filter.entry_node_id == f.current {
                            self.trace
                                .push(format!("filter:{id}:{}:enter", clause.ordinal));
                        }
                        if filter.result_node_id == f.current {
                            self.trace.push(format!(
                                "filter:{id}:{}:{}",
                                clause.ordinal,
                                f.values[block.condition_value_id.as_ref().unwrap()]
                                    .as_i64()
                                    .unwrap()
                                    != 0
                            ));
                        }
                    }
                }
            }
            if function
                .control_protocol
                .as_ref()
                .is_some_and(|p| p.escape_search_node_ids.contains(&f.current))
                && !f.unwinding
            {
                f.previous = f.current;
                f.current = block.node.normal_successor_ids[0].clone();
                return Outcome::Searching(f.pending.clone().unwrap(), Box::new(f));
            }
            if let Some(call) = &block.invocation {
                let args = call
                    .operands
                    .iter()
                    .map(|p| f.values[&p.id].clone())
                    .collect();
                let result = if let Some(callee) = self
                    .emitted
                    .vir()
                    .functions()
                    .iter()
                    .find(|fun| fun.id == call.operation_id)
                {
                    let frame = self.frame(callee, args);
                    let outcome = self.resume(frame);
                    let is_filter = function
                        .exception_regions
                        .iter()
                        .flat_map(|r| &r.catches)
                        .filter_map(|c| c.filter.as_ref()?.execution.as_ref())
                        .any(|filter| filter.node_ids.contains(&f.current));
                    let outcome = if is_filter {
                        self.settle(outcome)
                    } else {
                        outcome
                    };
                    match outcome {
                        Outcome::Returned(v) => Ok(v),
                        Outcome::Escaped(v) => Err(v),
                        Outcome::Searching(v, child) => {
                            f.child = Some((child, f.current.clone()));
                            Err(v)
                        }
                    }
                } else {
                    match self.operation(call, args) {
                        Ok(v) => Ok(v),
                        Err(error) => {
                            let edge = block
                                .node
                                .exceptional_successors
                                .iter()
                                .find(|e| {
                                    e.exception_type_id.rsplit('.').next() == Some(error.as_str())
                                })
                                .expect("builtin failure edge");
                            let v = block
                                .exception_values
                                .iter()
                                .find(|v| v.check_id == edge.check_id)
                                .expect("builtin failure value");
                            Err(self.stamp(literal(&v.value)))
                        }
                    }
                };
                match result {
                    Ok(v) => {
                        f.values.insert(call.result.id.clone(), v);
                    }
                    Err(v) => {
                        self.route_failure(&mut f, block, v);
                        continue;
                    }
                }
            }
            let next = match block.node.tag {
                ControlNodeTag::Return => {
                    return Outcome::Returned(
                        block
                            .return_value_ids
                            .first()
                            .map(|id| f.values[id].clone())
                            .unwrap_or(Value::Null),
                    )
                }
                ControlNodeTag::Throw | ControlNodeTag::Rethrow => {
                    let value = f.values[block.abrupt_value_id.as_ref().unwrap()].clone();
                    self.route_failure(&mut f, block, value);
                    continue;
                }
                ControlNodeTag::Exit => {
                    return Outcome::Escaped(f.pending.expect("exceptional exit"))
                }
                ControlNodeTag::Branch | ControlNodeTag::LoopHeader => {
                    &block.node.normal_successor_ids[usize::from(
                        f.values[block.condition_value_id.as_ref().unwrap()]
                            .as_i64()
                            .unwrap()
                            == 0,
                    )]
                }
                ControlNodeTag::Break | ControlNodeTag::Continue => {
                    match block.node.abrupt.as_ref().unwrap() {
                        AbruptCompletion::Break { target_id, .. }
                        | AbruptCompletion::Continue { target_id, .. } => target_id,
                        _ => panic!("loop completion"),
                    }
                }
                _ => &block.node.normal_successor_ids[0],
            };
            f.previous = f.current;
            f.current = next.clone();
        }
    }
    fn operation(&mut self, call: &OperationInvocation, args: Vec<Value>) -> Result<Value, String> {
        let id = call.operation_id.as_str();
        let n = |i: usize| args[i].as_i64().unwrap();
        if let Some(ty) = id.strip_prefix("mpk.csharp.value.exception.v1.is_type.") {
            let exception = self
                .emitted
                .vir()
                .source_exceptions()
                .iter()
                .find(|e| e.type_id == ty);
            let universe = derive_closed_exception_universe(
                self.emitted.closure().roots(),
                self.emitted.closure().closed(),
                self.emitted.vir().source_exceptions(),
            )
            .unwrap();
            let tag = universe
                .arms()
                .iter()
                .find(|a| a.type_id == ty)
                .map(|a| a.tag);
            return Ok(json!(i64::from(if exception.is_some() {
                args[0]["source_type_id"] == ty
            } else {
                args[0]["tag"].as_u64() == tag.map(u64::from)
            })));
        }
        if let Some(member) = id.strip_prefix("mpk.csharp.value.exception.v1.payload.") {
            return Ok(args[0]["payload"][member].clone());
        }
        if id.starts_with("floating.") || id.starts_with("decimal.") {
            let types = call
                .operands
                .iter()
                .map(|p| p.type_id.clone())
                .collect::<Vec<_>>();
            let operation = NumericOperation::new(id, &types, &call.result.type_id, None).unwrap();
            let operands = args
                .iter()
                .zip(&types)
                .map(|(v, t)| typed(v, t))
                .collect::<Vec<_>>();
            let value = operation
                .evaluate(
                    self.bundle,
                    self.emitted.closure().roots(),
                    self.emitted.closure().closed(),
                    &operands,
                )
                .map_err(|e| {
                    e.exception_type()
                        .expect("numeric exception")
                        .rsplit('.')
                        .next()
                        .unwrap()
                        .to_owned()
                })?;
            return Ok(literal(&value));
        }
        if let Some(op) = id.strip_prefix("integer.convert.") {
            let parts = op.split('.').collect::<Vec<_>>();
            let number = n(0);
            let target = parts[1];
            let converted = match target {
                "i8" => number as i8 as i64,
                "u8" => number as u8 as i64,
                "i16" => number as i16 as i64,
                "u16" | "char" => number as u16 as i64,
                "i32" => number as i32 as i64,
                "u32" => number as u32 as i64,
                "i64" | "u64" => number,
                _ => panic!("integer target {target}"),
            };
            if parts[2] == "checked" && converted != number {
                return Err("OverflowException".into());
            }
            return Ok(json!(converted));
        }
        if let Some(op) = id.strip_prefix("integer.") {
            let parts = op.split('.').collect::<Vec<_>>();
            let operation = parts[1];
            let value = match operation {
                "add" => n(0) + n(1),
                "subtract" => n(0) - n(1),
                "multiply" => n(0) * n(1),
                "negate" => -n(0),
                "plus" => n(0),
                "divide" | "remainder" => {
                    if n(1) == 0 {
                        return Err("DivideByZeroException".into());
                    }
                    if n(0) == i64::from(i32::MIN) && n(1) == -1 {
                        return Err("OverflowException".into());
                    }
                    if operation == "divide" {
                        n(0) / n(1)
                    } else {
                        n(0) % n(1)
                    }
                }
                "equal" => i64::from(n(0) == n(1)),
                "not_equal" => i64::from(n(0) != n(1)),
                "less" => i64::from(n(0) < n(1)),
                "less_equal" => i64::from(n(0) <= n(1)),
                "greater" => i64::from(n(0) > n(1)),
                "greater_equal" => i64::from(n(0) >= n(1)),
                "and" => n(0) & n(1),
                "or" => n(0) | n(1),
                "xor" => n(0) ^ n(1),
                "not" => !n(0),
                _ => panic!("integer op {id}"),
            };
            if call.result.type_id == "mpk.csharp.value.i32.v1" {
                if parts[2] == "checked" && i32::try_from(value).is_err() {
                    return Err("OverflowException".into());
                }
                return Ok(json!(value as i32));
            }
            return Ok(json!(value));
        }
        if id == "boolean.not" {
            return Ok(json!(i64::from(n(0) == 0)));
        }
        if id.starts_with("structural.equal.") {
            return Ok(json!(i64::from(args[0] == args[1])));
        }
        if let Some(exception) = id.strip_prefix("mpk.csharp.value.exception.v1.construct.") {
            return Ok(self.stamp(json!({"source_type_id":exception,"payload":args[0]})));
        }
        if let Some(owner) = id.strip_prefix("value.construct.") {
            let roots: Value =
                serde_json::from_slice(self.emitted.closure().roots().canonical_json()).unwrap();
            let members = roots["source_types"][owner]["members"].as_array().unwrap();
            assert_eq!(members.len(), args.len());
            return Ok(Value::Object(
                members
                    .iter()
                    .zip(args)
                    .map(|(member, value)| (member["id"].as_str().unwrap().to_owned(), value))
                    .collect(),
            ));
        }
        if id.starts_with("object.begin.") {
            return Ok(json!({}));
        }
        if let Some(member) = id.strip_prefix("object.write.") {
            let mut object = args[0].clone();
            object[member] = args[1].clone();
            return Ok(object);
        }
        if id.starts_with("object.finalize.") {
            return Ok(args[0].clone());
        }
        if let Some(member) = id.strip_prefix("field.read.") {
            return Ok(args[0][member].clone());
        }
        if id.starts_with("reference.value.") {
            return present(&args[0]).map_err(|_| "NullReferenceException".into());
        }
        if id == "string.length" || id == "string.index" {
            let s = present(&args[0]).map_err(|_| "NullReferenceException".to_owned())?;
            if id == "string.length" {
                return Ok(json!(s.as_array().unwrap().len()));
            }
            return s
                .as_array()
                .unwrap()
                .get(n(1) as usize)
                .cloned()
                .ok_or_else(|| "IndexOutOfRangeException".into());
        }
        if id.starts_with("construction.complete.") {
            return Ok(json!(i64::from(
                args[0].as_array().unwrap().iter().all(|v| !v.is_null())
            )));
        }
        if id.starts_with("mpk.csharp.instance.") {
            return match id.rsplit('.').next().unwrap() {
                "has_value" => Ok(json!(i64::from(!args[0].is_null()))),
                "some" => Ok(json!({"some":args[0]})),
                "none" => Ok(Value::Null),
                "value" => present(&args[0]),
                "value_or" => Ok(args[0].get("some").unwrap_or(&args[1]).clone()),
                "length" => Ok(json!(args[0].as_array().unwrap().len())),
                "read" => args[0]
                    .as_array()
                    .unwrap()
                    .get(n(1) as usize)
                    .cloned()
                    .ok_or_else(|| "IndexOutOfRangeException".into()),
                "allocate" => {
                    if n(0) < 0 {
                        return Err("OverflowException".into());
                    }
                    assert!(n(0) <= 4096);
                    Ok(json!(vec![
                        if n(1) != 0 { json!(0) } else { Value::Null };
                        n(0) as usize
                    ]))
                }
                "fill" | "rewrite" => {
                    let mut array = args[0].clone();
                    if let Some(slot) = array.as_array_mut().unwrap().get_mut(n(1) as usize) {
                        *slot = args[2].clone();
                        Ok(array)
                    } else {
                        Err("IndexOutOfRangeException".into())
                    }
                }
                "freeze" => {
                    assert!(args[0].as_array().unwrap().iter().all(|v| !v.is_null()));
                    Ok(args[0].clone())
                }
                _ => panic!("foundation op {id}"),
            };
        }
        panic!("unobserved operation {id}")
    }
}
pub(super) fn execute(
    b: &ValidatedFoundationBundle,
    emitted: &EmittedDataPhase,
    root: &str,
    run: &Value,
) -> Result<Value, String> {
    let function = emitted
        .vir()
        .functions()
        .iter()
        .find(|f| f.id == root)
        .unwrap();
    let mut observer = Observer {
        bundle: b,
        emitted,
        steps: 0,
        epoch: 0,
        trace: vec![],
        last_exception_payload: None,
    };
    let args = vec![
        run["n"].clone(),
        run["a"].clone(),
        json!(run["s"]
            .as_str()
            .unwrap_or("")
            .encode_utf16()
            .collect::<Vec<_>>()),
    ];
    observer.call(
        function,
        args.into_iter()
            .take(function.parameter_values.len())
            .collect(),
    )
}

pub(super) fn execute_exception(
    b: &ValidatedFoundationBundle,
    emitted: &EmittedDataPhase,
    root: &str,
    run: &Value,
) -> (Result<Value, String>, Option<Value>) {
    let function = emitted
        .vir()
        .functions()
        .iter()
        .find(|f| f.id == root)
        .unwrap();
    let mut observer = Observer {
        bundle: b,
        emitted,
        steps: 0,
        epoch: 0,
        trace: vec![],
        last_exception_payload: None,
    };
    let result = observer.call(function, vec![run["n"].clone()]);
    (result, observer.last_exception_payload)
}

pub(super) fn execute_handler(
    b: &ValidatedFoundationBundle,
    emitted: &EmittedDataPhase,
    root: &str,
    run: &Value,
) -> (Result<Value, String>, Vec<String>) {
    let function = emitted
        .vir()
        .functions()
        .iter()
        .find(|f| f.id == root)
        .unwrap();
    let mut observer = Observer {
        bundle: b,
        emitted,
        steps: 0,
        epoch: 0,
        trace: vec![],
        last_exception_payload: None,
    };
    let result = observer.call(function, vec![run["n"].clone()]);
    (result, observer.trace)
}
