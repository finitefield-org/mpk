//! Ordinary decimal conversions and rounding over the frozen product carrier.
use super::temporal::{divide_constant, literal};
use super::*;
#[path = "csharp_practical_ordinary_decimal_arithmetic.rs"]
mod arithmetic;
const WIDTH: usize = 512;
const STATE: usize = 119;
const TOKENS: &[(&str, &str)] = &[
    ("sbyte", "i8"),
    ("byte", "u8"),
    ("int16", "i16"),
    ("uint16", "u16"),
    ("int32", "i32"),
    ("uint32", "u32"),
    ("int64", "i64"),
    ("uint64", "u64"),
    ("char", "char"),
];
fn ty(token: &str) -> String {
    format!("mpk.csharp.value.{token}.v1")
}
fn signature(id: &str) -> R<ClosedOperationSignature> {
    let (args, result) = if let Some(op) = arithmetic::operation(id) {
        (
            vec![ty("decimal"); 2],
            ty(if arithmetic::comparison(op) {
                "bool"
            } else {
                "decimal"
            }),
        )
    } else if let Some(suffix) = id.strip_prefix("decimal.conversion.") {
        let (from, to) = suffix
            .split_once("_to_")
            .ok_or(OrdinaryCarrierError::Shape)?;
        let token = |name| {
            if name == "decimal" {
                Some("decimal")
            } else {
                TOKENS.iter().find(|(n, _)| *n == name).map(|(_, t)| *t)
            }
        };
        (
            vec![ty(token(from).ok_or(OrdinaryCarrierError::Shape)?)],
            ty(token(to).ok_or(OrdinaryCarrierError::Shape)?),
        )
    } else if let Some(round) = id.strip_prefix("decimal.round.") {
        let (_, arity) = round.split_once('.').ok_or(OrdinaryCarrierError::Shape)?;
        let mut args = vec![ty("decimal")];
        if arity == "2" {
            args.push(ty("i32"));
        }
        (args, ty("decimal"))
    } else if matches!(
        id,
        "decimal.plus"
            | "decimal.negate"
            | "decimal.truncate"
            | "decimal.floor"
            | "decimal.ceiling"
    ) {
        (vec![ty("decimal")], ty("decimal"))
    } else {
        return Err(OrdinaryCarrierError::Shape);
    };
    let recipe =
        NumericOperation::new(id, &args, &result, None).map_err(|_| OrdinaryCarrierError::Shape)?;
    let checks = recipe
        .exception_types()
        .into_iter()
        .map(|exception| {
            let name = match exception {
                "System.OverflowException" => "exception.overflow",
                "System.DivideByZeroException" => "exception.division_by_zero",
                "System.ArgumentOutOfRangeException" => "exception.range",
                _ => return Err(OrdinaryCarrierError::Shape),
            };
            Ok(RequiredCheck {
                id: name.into(),
                tag: RequiredCheckTag::Exception,
                failure_type_id: Some(exception.into()),
            })
        })
        .collect::<R<Vec<_>>>()?;
    Ok(ClosedOperationSignature {
        id: id.into(),
        tag: ClosedOperationTag::Data,
        argument_type_ids: args,
        normal_result_type_id: result,
        ordered_checks: checks,
    })
}
fn width(id: &str) -> R<usize> {
    if id == ty("decimal") {
        Ok(WIDTH)
    } else {
        Ok(integral_shape(id)?.0)
    }
}
// Field selectors precede child selectors; short children prepend zero padding.
// The decimal product has two field bits and seven padded child bits.
fn read(raw: &[Bit]) -> (Bit, Word, Word) {
    (
        raw[0],
        (0..8).map(|i| raw[1 + (i << 6)]).collect(),
        (0..96).map(|i| raw[2 + (i << 2)]).collect(),
    )
}
fn product(sign: Bit, scale: &[Bit], coefficient: &[Bit]) -> Word {
    let mut raw = vec![F; WIDTH];
    raw[0] = sign;
    for (i, &b) in scale.iter().enumerate() {
        raw[1 + (i << 6)] = b;
    }
    for (i, &b) in coefficient.iter().enumerate() {
        raw[2 + (i << 2)] = b;
    }
    raw
}
fn direct(id: &str) -> R<IntegerCircuit> {
    let signature = signature(id)?;
    let mut c = Circuit::new(&[width(&signature.argument_type_ids[0])?]);
    let raw = c.inputs[0].clone();
    let output = if signature.argument_type_ids[0] == ty("decimal") {
        let (sign, scale, coefficient) = read(&raw);
        let sign = if id == "decimal.negate" {
            c.not(sign)
        } else {
            sign
        };
        product(sign, &scale, &coefficient)
    } else {
        let (_, signed) = integral_shape(&signature.argument_type_ids[0])?;
        let sign = if signed { *raw.last().unwrap() } else { F };
        let negative = c.neg(&raw);
        let magnitude = c.select(sign, &negative, &raw);
        product(sign, &[F; 8], &Circuit::extend(&magnitude, 96, false))
    };
    Ok(IntegerCircuit {
        signature,
        circuit: c,
        output,
        failures: vec![],
    })
}
fn helper(
    id: &str,
    suffix: &str,
    circuit: Circuit,
    output: Word,
    checks: Vec<RequiredCheck>,
    failures: Vec<Bit>,
) -> IntegerCircuit {
    let cube = |w| format!("{PREFIX}.Cube.D{}", address_bits(w as u32));
    let signature = ClosedOperationSignature {
        id: format!("{id}.{suffix}"),
        tag: ClosedOperationTag::Data,
        argument_type_ids: circuit.inputs.iter().map(|w| cube(w.len())).collect(),
        normal_result_type_id: cube(output.len()),
        ordered_checks: checks,
    };
    IntegerCircuit {
        signature,
        circuit,
        output,
        failures,
    }
}
// Only internal helpers use this cache. Equality covers the entire circuit,
// physical input/output mapping and ordered checks, not an operation name/hash.
fn emit_shared(b: &mut Builder, p: IntegerCircuit, namespace: &str) -> R<OrdinaryScalarDefinition> {
    let gates = p
        .circuit
        .gates
        .iter()
        .map(|g| match *g {
            Gate::False => [0, 0, 0, 0],
            Gate::True => [1, 0, 0, 0],
            Gate::Input(a, b) => [2, a, b, 0],
            Gate::Not(a) => [3, a, 0, 0],
            Gate::And(a, b) => [4, a, b, 0],
            Gate::Xor(a, b) => [5, a, b, 0],
            Gate::Mux(c, t, e) => [6, c, t, e],
        })
        .collect::<Vec<_>>();
    let key = serde_json::to_vec(&serde_json::json!({
        "widths":p.circuit.inputs.iter().map(Vec::len).collect::<Vec<_>>(),
        "start":p.circuit.start,"gates":gates,"output":p.output,"failures":p.failures,
        "arguments":p.signature.argument_type_ids,"result":p.signature.normal_result_type_id,
        "tag":p.signature.tag,"checks":p.signature.ordered_checks,
    }))
    .expect("typed circuit key");
    if let Some(cached) = b.shared_scalar_circuits.get(&key) {
        let mut d = cached.clone();
        d.operation = p.signature;
        return Ok(d);
    }
    let d = emit_circuit(b, p, namespace)?;
    b.shared_scalar_circuits.insert(key, d.clone());
    Ok(d)
}
fn stages(id: &str) -> R<[IntegerCircuit; 3]> {
    let signature = signature(id)?;
    let widths = signature
        .argument_type_ids
        .iter()
        .map(|s| width(s))
        .collect::<R<Vec<_>>>()?;
    let mut c = Circuit::new(&widths);
    let raw = c.inputs[0].clone();
    let (sign, scale, coefficient) = read(&raw);
    let digits = c.inputs.get(1).cloned().unwrap_or_else(|| vec![F; 32]);
    let large = c.lt(&literal(28, 32), &digits, true);
    let invalid = c.or(digits[31], large);
    let small = c.lt(&digits[..8], &scale, false);
    let target = c.select(small, &digits[..8], &scale);
    let count = c.sub(&scale, &target).0;
    let state = coefficient
        .into_iter()
        .chain(count)
        .chain(vec![F; 5])
        .chain(std::iter::once(sign))
        .chain(target)
        .chain(std::iter::once(invalid))
        .collect();
    let initial = helper(id, "Init", c, state, vec![], vec![]);
    let mut c = Circuit::new(&[STATE]);
    let state = c.inputs[0].clone();
    let (quotient, remainder) = divide_constant(&mut c, &state[..96], 10);
    let any = c.nonzero(&state[104..108]);
    let sticky = c.or(state[108], any);
    let count = c.sub(&state[96..104], &literal(1, 8)).0;
    let next = quotient
        .into_iter()
        .chain(count)
        .chain(remainder[..4].iter().copied())
        .chain(std::iter::once(sticky))
        .chain(state[109..].iter().copied())
        .collect::<Word>();
    let enabled = c.nonzero(&state[96..104]);
    let next = c.select(enabled, &next, &state);
    let step = helper(id, "Step", c, next, vec![], vec![]);
    let mut c = Circuit::new(&[STATE]);
    let state = c.inputs[0].clone();
    let coefficient = &state[..96];
    let sign = state[109];
    let (output, failures) = if signature.normal_result_type_id != ty("decimal") {
        let (w, signed) = integral_shape(&signature.normal_result_type_id)?;
        let max = literal((1u128 << (w - usize::from(signed))) - 1, 96);
        let minimum = literal(1u128 << (w - 1), 96);
        let limit = if signed {
            c.select(sign, &minimum, &max)
        } else {
            max
        };
        let overflow = c.lt(&limit, coefficient, false);
        let nonzero = c.nonzero(coefficient);
        let negative = if signed { F } else { c.and(sign, nonzero) };
        let overflow = c.or(overflow, negative);
        let negative = c.neg(&coefficient[..w]);
        let result = c.select(sign, &negative, &coefficient[..w]);
        (result, vec![overflow])
    } else {
        let mode = if let Some(round) = id.strip_prefix("decimal.round.") {
            CodecRounding::from_id(round.split_once('.').unwrap().0)
                .map_err(|_| OrdinaryCarrierError::Shape)?
        } else {
            match id {
                "decimal.floor" => CodecRounding::ToNegativeInfinity,
                "decimal.ceiling" => CodecRounding::ToPositiveInfinity,
                _ => CodecRounding::ToZero,
            }
        };
        let digit = &state[104..108];
        let any_digit = c.nonzero(digit);
        let any = c.or(any_digit, state[108]);
        let above = c.lt(&literal(5, 4), digit, false);
        let tie = c.equal(digit, &literal(5, 4));
        let up = match mode {
            CodecRounding::ToEven => {
                let tail = c.or(state[108], coefficient[0]);
                let half = c.and(tie, tail);
                c.or(above, half)
            }
            CodecRounding::AwayFromZero => c.or(above, tie),
            CodecRounding::ToZero => F,
            CodecRounding::ToNegativeInfinity => c.and(sign, any),
            CodecRounding::ToPositiveInfinity => {
                let positive = c.not(sign);
                c.and(positive, any)
            }
        };
        let rounded = c.add(coefficient, &vec![F; 96], up).0;
        (
            product(sign, &state[110..118], &rounded),
            if signature.ordered_checks.is_empty() {
                vec![]
            } else {
                vec![state[118]]
            },
        )
    };
    let finish = helper(id, "Finish", c, output, signature.ordered_checks, failures);
    Ok([initial, step, finish])
}
pub(super) fn emit_decimal(b: &mut Builder, id: &str) -> R<OrdinaryScalarDefinition> {
    if arithmetic::operation(id).is_some() {
        return arithmetic::emit(b, id);
    }
    let signature = signature(id)?;
    if matches!(id, "decimal.plus" | "decimal.negate")
        || signature.argument_type_ids[0] != ty("decimal")
    {
        return emit_circuit(b, direct(id)?, "Decimal");
    }
    let definitions = stages(id)?
        .into_iter()
        .map(|p| emit_shared(b, p, "DecimalSteps"))
        .collect::<R<Vec<_>>>()?;
    let d = address_bits(STATE as u32);
    if !b
        .globals
        .contains_key(&format!("{PREFIX}.Cube.D{d}.Compose"))
    {
        b.helpers(d)?;
    }
    let step = b.constant(&definitions[1].result_definition)?;
    let composed = b.compose(d, &[step; 28])?;
    let widths = signature
        .argument_type_ids
        .iter()
        .map(|s| width(s))
        .collect::<R<Vec<_>>>()?;
    let args = (0..widths.len())
        .rev()
        .map(|i| b.var(i as u32))
        .collect::<R<Vec<_>>>()?;
    let initial = b.constant(&definitions[0].result_definition)?;
    let initial = b.app(initial, args)?;
    let state = b.app(composed, vec![initial])?;
    let name = format!(
        "{PREFIX}.Decimal.O{}",
        id.as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    let result_definition = format!("{name}.Result");
    let success_definition = format!("{name}.Success");
    let failures = (0..signature.ordered_checks.len())
        .map(|i| format!("{name}.Failure.F{i}"))
        .collect::<Vec<_>>();
    let outputs = std::iter::once((
        &result_definition,
        &definitions[2].result_definition,
        width(&signature.normal_result_type_id)?,
    ))
    .chain(std::iter::once((
        &success_definition,
        &definitions[2].success_definition,
        1,
    )))
    .chain(
        failures
            .iter()
            .zip(&definitions[2].ordered_failure_definitions)
            .map(|(a, b)| (a, b, 1)),
    );
    for (name, helper, output_width) in outputs {
        let f = b.constant(helper)?;
        let value = b.app(f, vec![state])?;
        let body = bind_inputs(b, &widths, value)?;
        let result = b.cube(address_bits(output_width as u32))?;
        let ty = input_type(b, &widths, result)?;
        b.define(name, ty, body)?;
    }
    Ok(OrdinaryScalarDefinition {
        operation: signature,
        result_definition,
        success_definition,
        ordered_failure_definitions: failures,
        boolean_gates: definitions.iter().map(|d| d.boolean_gates).sum(),
        state_depth: definitions.iter().map(|d| d.state_depth).max().unwrap(),
        static_transformers: definitions
            .iter()
            .map(|d| d.static_transformers)
            .sum::<usize>()
            + 28,
    })
}
/// W09 decimal scalar definitions. Literal bodies, input domains and application
/// proofs remain separate work.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryDecimalProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryScalarDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryDecimalProgram {
    pub fn definitions(&self) -> &[OrdinaryScalarDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed decimal program")
    }
}
pub fn generate_csharp_practical_ordinary_decimal(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryDecimalProgram> {
    let mut b = Builder::new()?;
    let mut definitions = vec![];
    let signatures = vir
        .operation_signatures()
        .iter()
        .filter(|s| s.id.starts_with("decimal."))
        .map(|s| (s.id.clone(), s))
        .collect::<BTreeMap<_, _>>();
    for (id, expected) in signatures {
        if &signature(&id)? != expected {
            return Err(OrdinaryCarrierError::Linkage);
        }
        definitions.push(emit_decimal(&mut b, &id)?);
    }
    let certificate = b.finish()?;
    let p = OrdinaryDecimalProgram {
        schema: "mpk.csharp.ordinary_decimal.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        definitions,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_decimal(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryDecimalProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_decimal(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    const MAX: u128 = (1u128 << 96) - 1;
    const MODES: &[&str] = &[
        "ToEven",
        "AwayFromZero",
        "ToZero",
        "ToNegativeInfinity",
        "ToPositiveInfinity",
    ];
    const FIXTURES: &[&str] = &[
        "decimal.negate",
        "decimal.floor",
        "decimal.round.ToEven.2",
        "decimal.round.AwayFromZero.1",
        "decimal.conversion.decimal_to_int64",
        "decimal.conversion.decimal_to_uint64",
        "decimal.conversion.char_to_decimal",
    ];
    pub(super) fn ids() -> Vec<String> {
        let mut ids = ["plus", "negate", "truncate", "floor", "ceiling"]
            .map(|s| format!("decimal.{s}"))
            .to_vec();
        for mode in MODES {
            for n in [1, 2] {
                ids.push(format!("decimal.round.{mode}.{n}"));
            }
        }
        for &(name, _) in TOKENS {
            ids.push(format!("decimal.conversion.{name}_to_decimal"));
            ids.push(format!("decimal.conversion.decimal_to_{name}"));
        }
        ids
    }
    pub(super) fn decimal(sign: bool, n: u128, scale: u8) -> MonomorphicValue {
        MonomorphicValue::DecimalBits {
            type_id: ty("decimal"),
            negative: sign,
            coefficient: n.to_string(),
            scale,
        }
    }
    fn integer(token: &str, raw: u128) -> MonomorphicValue {
        let (w, signed) = integral_shape(&ty(token)).unwrap();
        let raw = raw & ((1u128 << w) - 1);
        if token == "char" {
            MonomorphicValue::Char {
                type_id: ty(token),
                utf16: raw as u16,
            }
        } else if signed {
            MonomorphicValue::Signed {
                type_id: ty(token),
                value: (((raw << (128 - w)) as i128) >> (128 - w)).to_string(),
            }
        } else {
            MonomorphicValue::Unsigned {
                type_id: ty(token),
                value: raw.to_string(),
            }
        }
    }
    // Independent physical encoding of the unit-1 product contract. Do not use
    // the production read/product functions to validate their address mapping.
    pub(super) fn physical(v: &MonomorphicValue) -> Vec<bool> {
        match v {
            MonomorphicValue::DecimalBits {
                negative,
                coefficient,
                scale,
                ..
            } => {
                let n: u128 = coefficient.parse().unwrap();
                let mut bits = vec![false; 512];
                bits[0] = *negative;
                for i in 0..8 {
                    bits[1 + i * 64] = scale & (1 << i) != 0;
                }
                for i in 0..96 {
                    bits[2 + i * 4] = n & (1 << i) != 0;
                }
                bits
            }
            MonomorphicValue::Signed { type_id, value } => {
                let w = integral_shape(type_id).unwrap().0;
                let raw = value.parse::<i128>().unwrap() as u128;
                (0..w).map(|i| raw & (1 << i) != 0).collect()
            }
            MonomorphicValue::Unsigned { type_id, value } => {
                let w = integral_shape(type_id).unwrap().0;
                let raw = value.parse::<u128>().unwrap();
                (0..w).map(|i| raw & (1 << i) != 0).collect()
            }
            MonomorphicValue::Char { utf16, .. } => {
                (0..16).map(|i| utf16 & (1 << i) != 0).collect()
            }
            _ => unreachable!(),
        }
    }
    pub(super) fn prune(mut p: IntegerCircuit) -> IntegerCircuit {
        let mut roots = p
            .output
            .iter()
            .chain(&p.failures)
            .copied()
            .collect::<Vec<_>>();
        p.circuit.prune(&mut roots);
        let n = p.output.len();
        p.output.copy_from_slice(&roots[..n]);
        p.failures.copy_from_slice(&roots[p.output.len()..]);
        p
    }
    pub(super) fn observe(p: &IntegerCircuit, inputs: &[Vec<u64>]) -> (Vec<u64>, Vec<u64>) {
        let mut values: Vec<u64> = vec![];
        for gate in &p.circuit.gates {
            let v = match *gate {
                Gate::False => 0,
                Gate::True => u64::MAX,
                Gate::Input(a, b) => inputs[a][b],
                Gate::Not(a) => !values[a],
                Gate::And(a, b) => values[a] & values[b],
                Gate::Xor(a, b) => values[a] ^ values[b],
                Gate::Mux(c, t, e) => (values[c] & values[t]) | (!values[c] & values[e]),
            };
            values.push(v);
        }
        (
            p.output.iter().map(|&i| values[i]).collect(),
            p.failures.iter().map(|&i| values[i]).collect(),
        )
    }
    fn samples(id: &str, signature: &ClosedOperationSignature) -> Vec<Vec<MonomorphicValue>> {
        let mut cases = vec![];
        if signature.argument_type_ids[0] != ty("decimal") {
            let token = signature.argument_type_ids[0]
                .strip_prefix("mpk.csharp.value.")
                .unwrap()
                .strip_suffix(".v1")
                .unwrap();
            let w = integral_shape(&signature.argument_type_ids[0]).unwrap().0;
            for bit in 0..w {
                for delta in [-1i128, 0, 1] {
                    cases.push(vec![integer(token, ((1i128 << bit) + delta) as u128)]);
                    cases.push(vec![integer(token, (-((1i128 << bit) + delta)) as u128)]);
                }
            }
        } else {
            let coefficients = [
                0,
                1,
                5,
                15,
                25,
                149,
                150,
                155,
                251,
                999,
                65535,
                65536,
                1u128 << 63,
                1u128 << 64,
                MAX - 1,
                MAX,
                10u128.pow(28) - 1,
                10u128.pow(28),
            ];
            for scale in 0..=28 {
                for n in coefficients {
                    for sign in [false, true] {
                        if signature.argument_type_ids.len() == 2 {
                            for digits in [-1, 0, 1, 2, 27, 28, 29, i32::MIN, i32::MAX] {
                                cases.push(vec![
                                    decimal(sign, n, scale),
                                    integer("i32", digits as u128),
                                ]);
                            }
                        } else {
                            cases.push(vec![decimal(sign, n, scale)]);
                        }
                    }
                }
            }
            if id.contains("ToEven") || id.contains("AwayFromZero") {
                for n in [144, 145, 146, 149, 150, 151, 154, 155, 156, 245, 250, 251] {
                    cases.push(if signature.argument_type_ids.len() == 2 {
                        vec![decimal(false, n, 2), integer("i32", 0)]
                    } else {
                        vec![decimal(false, n, 2)]
                    });
                }
            }
        }
        cases
    }
    #[test]
    fn decimal_circuits_match_numeric_oracle() {
        let bundle = validate_registered_foundation_bundle(
            registered_foundation_descriptor_transport(),
            registered_foundation_definitions_transport(),
        )
        .unwrap();
        let roots = serde_json::json!(["decimal", "i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64", "char"].map(|id|serde_json::json!({"origin":"semantic_binding","provenance_id":format!("decimal.{id}"),"type":{"kind":"primitive","id":id}})));
        let bytes =
            canonical_closed_root_set_transport(&bundle, &roots, &serde_json::json!({})).unwrap();
        let roots = validate_closed_root_set(&bundle, &bytes).unwrap();
        let closed = derive_closed_instances(&bundle, &roots).unwrap();
        let mut count = 0;
        for id in ids() {
            let signature = signature(&id).unwrap();
            let recipe = NumericOperation::new(
                &id,
                &signature.argument_type_ids,
                &signature.normal_result_type_id,
                None,
            )
            .unwrap();
            let direct = if matches!(id.as_str(), "decimal.plus" | "decimal.negate")
                || signature.argument_type_ids[0] != ty("decimal")
            {
                Some(prune(direct(&id).unwrap()))
            } else {
                None
            };
            let stages = if direct.is_none() {
                Some(stages(&id).unwrap().map(prune))
            } else {
                None
            };
            for batch in samples(&id, &signature).chunks(64) {
                let mut inputs = signature
                    .argument_type_ids
                    .iter()
                    .map(|s| vec![0u64; width(s).unwrap()])
                    .collect::<Vec<_>>();
                for (lane, args) in batch.iter().enumerate() {
                    for (i, v) in args.iter().enumerate() {
                        for (j, on) in physical(v).iter().enumerate() {
                            if *on {
                                inputs[i][j] |= 1 << lane;
                            }
                        }
                    }
                }
                let (output, failures) = if let Some(p) = &direct {
                    observe(p, &inputs)
                } else {
                    let [initial, step, finish] = stages.as_ref().unwrap();
                    let mut state = observe(initial, &inputs).0;
                    for _ in 0..28 {
                        state = observe(step, &[state]).0;
                    }
                    assert!(state[96..104].iter().all(|&v| v == 0));
                    assert_eq!(
                        observe(step, &[state.clone()]).0,
                        state,
                        "zero-counter identity {id}"
                    );
                    observe(finish, &[state])
                };
                for (lane, args) in batch.iter().enumerate() {
                    let error = failures.iter().any(|v| v & (1 << lane) != 0);
                    match recipe.evaluate(&bundle, &roots, &closed, args) {
                        Ok(v) => {
                            assert!(!error, "{id} {args:?}");
                            let expected = physical(&v);
                            assert_eq!(output.len(), expected.len());
                            for (i, on) in expected.iter().enumerate() {
                                assert_eq!(
                                    output[i] & (1 << lane) != 0,
                                    *on,
                                    "{id} {args:?} bit {i}"
                                );
                            }
                        }
                        Err(e) => {
                            assert!(matches!(e, NumericError::Range | NumericError::Overflow));
                            assert!(error, "missing {e:?}: {id} {args:?}");
                            assert_eq!(
                                signature.ordered_checks[0].failure_type_id.as_deref(),
                                e.exception_type()
                            );
                        }
                    }
                    count += 1;
                }
            }
        }
        eprintln!("decimal oracle cases: {count}");
    }
    #[test]
    fn decimal_circuits_emit_supported_signatures() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/decimal-circuits");
        let out = std::env::var_os("MPK_W09_DECIMAL_OUT").map(std::path::PathBuf::from);
        if let Some(dir) = &out {
            std::fs::create_dir_all(dir).unwrap();
        }
        let mut metrics = vec![];
        for id in ids() {
            let mut b = Builder::new().unwrap();
            let definition = emit_decimal(&mut b, &id).unwrap();
            let bytes = b.finish().unwrap_or_else(|e| panic!("{id}: {e:?}"));
            let cert = decode_canonical_certificate(&bytes).unwrap();
            metrics.push(serde_json::json!({"id":id,"definition":definition,"terms":cert.term_table.len(),"declarations":cert.declarations.len(),"hash":mpk_cert::hash_hex(&mpk_cert::certificate_hash(&bytes))}));
            if FIXTURES.contains(&id.as_str()) {
                let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
                if let Some(dir) = &out {
                    std::fs::write(dir.join(format!("{id}.hex")), hex).unwrap();
                } else {
                    assert_eq!(
                        std::fs::read_to_string(root.join(format!("{id}.hex"))).unwrap(),
                        hex,
                        "{id}"
                    );
                }
            }
        }
        if let Some(dir) = &out {
            std::fs::write(
                dir.join("metrics.json"),
                serde_json::to_vec_pretty(&metrics).unwrap(),
            )
            .unwrap();
        } else {
            let expected: Value =
                serde_json::from_slice(&std::fs::read(root.join("metrics.json")).unwrap()).unwrap();
            assert_eq!(serde_json::json!(metrics), expected);
        }
        for unknown in [
            "decimal.round",
            "decimal.round.ToEven.3",
            "decimal.round.unknown.1",
            "decimal.conversion.int32_to_uint32",
            "decimal.conversion.decimal_to_bool",
            "decimal.power",
        ] {
            assert!(signature(unknown).is_err(), "{unknown}");
        }
    }
    #[test]
    fn decimal_circuits_core_product_rounding_and_failure() {
        std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                use super::super::super::tests::{apply, bit, run, V};
                for (id, cases) in [
                    (
                        "decimal.negate",
                        vec![(vec![decimal(false, 0, 28)], Some(decimal(true, 0, 28)))],
                    ),
                    (
                        "decimal.conversion.char_to_decimal",
                        vec![(vec![integer("char", 65535)], Some(decimal(false, 65535, 0)))],
                    ),
                    (
                        "decimal.round.ToEven.2",
                        vec![
                            (
                                vec![decimal(false, 245, 2), integer("i32", 0)],
                                Some(decimal(false, 2, 0)),
                            ),
                            (
                                vec![decimal(true, 251, 2), integer("i32", 0)],
                                Some(decimal(true, 3, 0)),
                            ),
                            (vec![decimal(false, MAX, 28), integer("i32", 29)], None),
                        ],
                    ),
                    (
                        "decimal.conversion.decimal_to_uint64",
                        vec![
                            (vec![decimal(true, 1, 28)], Some(integer("u64", 0))),
                            (vec![decimal(true, 1, 0)], None),
                            (vec![decimal(false, 1u128 << 64, 0)], None),
                        ],
                    ),
                ] {
                    let mut b = Builder::new().unwrap();
                    let d = emit_decimal(&mut b, id).unwrap();
                    let cert = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
                    for (args, expected) in cases {
                        let args = args
                            .iter()
                            .map(|v| V::Cube(physical(v)))
                            .collect::<Vec<_>>();
                        assert_eq!(
                            bit(run(&cert, &d.success_definition, args.clone())),
                            expected.is_some()
                        );
                        for failure in &d.ordered_failure_definitions {
                            assert_eq!(bit(run(&cert, failure, args.clone())), expected.is_none());
                        }
                        let output = run(&cert, &d.result_definition, args);
                        let width = width(&d.operation.normal_result_type_id).unwrap();
                        let expected = expected
                            .as_ref()
                            .map(physical)
                            .unwrap_or_else(|| vec![false; width]);
                        for (i, on) in expected.iter().enumerate() {
                            let mut v = output.clone();
                            for j in 0..address_bits(width as u32) {
                                v = apply(&cert, v, V::Bit(i & (1 << j) != 0));
                            }
                            assert_eq!(bit(v), *on, "{id} bit {i}");
                        }
                    }
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
