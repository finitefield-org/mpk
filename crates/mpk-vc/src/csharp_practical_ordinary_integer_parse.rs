//! Bounded canonical decimal integer parsers as ordinary definitions.
use super::hex_codecs::{call, truth, word};
use super::integer_format::{circuit, define};
use super::*;
const NAME: &str = "Mpk.CSharp.Ordinary.IntegerParse";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryIntegerParseDefinition {
    pub codec_id: String,
    pub value_type_id: String,
    pub value_depth: u32,
    pub signed: bool,
    pub parse_result_shape: OrdinaryShape,
    pub parse_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryIntegerParseProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryIntegerParseDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryIntegerParseProgram {
    pub fn definitions(&self) -> &[OrdinaryIntegerParseDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary integer parsers")
    }
}
fn constant(n: u128, width: usize) -> Word {
    (0..width)
        .map(|i| if n & (1u128 << i) != 0 { T } else { F })
        .collect()
}
fn invoke(b: &mut Builder, suffix: &str, args: Vec<u32>) -> R<u32> {
    call(b, &format!("{NAME}.{suffix}"), args)
}
fn negate(b: &mut Builder, x: u32) -> R<u32> {
    call(b, "Std.Bool.not", vec![x])
}
fn and(b: &mut Builder, x: u32, y: u32) -> R<u32> {
    call(b, "Std.Bool.and", vec![x, y])
}
fn or(b: &mut Builder, x: u32, y: u32) -> R<u32> {
    call(b, "Std.Bool.or", vec![x, y])
}
fn mux_cube(b: &mut Builder, depth: u32, c: u32, t: u32, f: u32) -> R<u32> {
    call(b, &format!("{PREFIX}.Cube.D{depth}.Mux"), vec![c, t, f])
}
fn at(b: &mut Builder, text: u32, index: usize) -> R<u32> {
    let mut args = vec![truth(b, true)?];
    for i in 0..14 {
        args.push(truth(b, index & (1 << i) != 0)?);
    }
    b.app(text, args)
}
struct Shared {
    step: String,
    remaining: String,
    small: String,
    bounded: String,
}
fn helpers(b: &mut Builder) -> R<Shared> {
    for d in [3, 4, 7] {
        if !b.globals.contains_key(&format!("{PREFIX}.Cube.D{d}.Mux")) {
            b.helpers(d)?;
        }
    }
    let mut c = Circuit::new(&[16]);
    let input = c.inputs[0].clone();
    let low = c.lt(&input, &constant(48, 16), false);
    let high = c.lt(&input, &constant(58, 16), false);
    let low = c.not(low);
    let good = c.and(low, high);
    let digit = circuit(b, &format!("{NAME}.DigitCircuit"), c, vec![good])?;
    let x = b.var(0)?;
    let body = call(b, &digit, vec![x])?;
    define(b, &format!("{NAME}.Digit"), &[4], 0, body)?;
    for (name, ch) in [("Plus", b'+'), ("Minus", b'-'), ("Zero", b'0')] {
        let mut c = Circuit::new(&[16]);
        let input = c.inputs[0].clone();
        let same = c.equal(&input, &constant(ch as u128, 16));
        let classify = circuit(b, &format!("{NAME}.{name}Circuit"), c, vec![same])?;
        let x = b.var(0)?;
        let body = call(b, &classify, vec![x])?;
        define(b, &format!("{NAME}.{name}"), &[4], 0, body)?;
    }
    let x = b.var(0)?;
    let plus = invoke(b, "Plus", vec![x])?;
    let minus = invoke(b, "Minus", vec![x])?;
    let sign = or(b, plus, minus)?;
    define(b, &format!("{NAME}.Sign"), &[4], 0, sign)?;
    for name in ["Plus", "Minus", "Sign"] {
        let text = b.var(0)?;
        let first = at(b, text, 0)?;
        let body = invoke(b, name, vec![first])?;
        define(b, &format!("{NAME}.First{name}"), &[19], 0, body)?;
    }
    let text = b.var(5)?;
    let zero = truth(b, false)?;
    let mut args = vec![zero; 14];
    args.extend(b.selectors(5)?);
    let body = b.app(text, args)?;
    let body = b.wrap_selectors(5, body)?;
    define(b, &format!("{NAME}.Length"), &[19], 5, body)?;
    let mut c = Circuit::new(&[32, 1]);
    let len = c.inputs[0].clone();
    let sign = c.inputs[1][0];
    let mut offset = vec![F; 32];
    offset[0] = sign;
    let body_len = c.sub(&len, &offset).0;
    let length_fn = circuit(b, &format!("{NAME}.BodyLengthCircuit"), c, body_len)?;
    let text = b.var(0)?;
    let len = invoke(b, "Length", vec![text])?;
    let sign = invoke(b, "FirstSign", vec![text])?;
    let body = call(b, &length_fn, vec![len, sign])?;
    define(b, &format!("{NAME}.BodyLength"), &[19], 5, body)?;
    let mut c = Circuit::new(&[32]);
    let len = c.inputs[0].clone();
    let ok = c.lt(&len, &constant(16385, 32), false);
    let bounded = circuit(b, &format!("{NAME}.Bounded"), c, vec![ok])?;
    let mut c = Circuit::new(&[32]);
    let len = c.inputs[0].clone();
    let ok = c.lt(&len, &constant(21, 32), false);
    let small = circuit(b, &format!("{NAME}.Small"), c, vec![ok])?;
    let mut c = Circuit::new(&[32, 32]);
    let left = c.inputs[0].clone();
    let right = c.inputs[1].clone();
    let less = c.lt(&left, &right, false);
    let remaining = circuit(b, &format!("{NAME}.Less"), c, vec![less])?;

    let text = b.var(14)?;
    let mut args = vec![truth(b, true)?];
    args.extend(b.selectors(14)?);
    let ch = b.app(text, args)?;
    let good = invoke(b, "Digit", vec![ch])?;
    let sign = invoke(b, "Sign", vec![ch])?;
    let mut first = truth(b, true)?;
    for i in 0..14 {
        let v = b.var(i)?;
        let v = negate(b, v)?;
        first = and(b, first, v)?;
    }
    let initial_sign = and(b, first, sign)?;
    let valid = or(b, good, initial_sign)?;
    let body = b.wrap_selectors(14, valid)?;
    define(b, &format!("{NAME}.Characters"), &[19], 14, body)?;
    let fold = super::super::structural::emit_aggregate_fold(b, 14)?;
    let text = b.var(0)?;
    let predicate = invoke(b, "Characters", vec![text])?;
    let len = invoke(b, "Length", vec![text])?;
    let body = call(b, &fold.all_definition, vec![predicate, len])?;
    define(b, &format!("{NAME}.AllCharacters"), &[19], 0, body)?;
    for (name, n) in [("Empty", 0), ("One", 1)] {
        let mut c = Circuit::new(&[32]);
        let len = c.inputs[0].clone();
        let same = c.equal(&len, &constant(n, 32));
        let test = circuit(b, &format!("{NAME}.{name}Length"), c, vec![same])?;
        let text = b.var(0)?;
        let len = invoke(b, "Length", vec![text])?;
        let body = call(b, &test, vec![len])?;
        define(b, &format!("{NAME}.{name}"), &[19], 0, body)?;
    }
    for unsigned in [false, true] {
        let text = b.var(0)?;
        let empty = invoke(b, "Empty", vec![text])?;
        let one = invoke(b, "One", vec![text])?;
        let sign = invoke(b, "FirstSign", vec![text])?;
        let sign_only = and(b, one, sign)?;
        let invalid = or(b, empty, sign_only)?;
        let invalid = if unsigned {
            let negative = invoke(b, "FirstMinus", vec![text])?;
            or(b, invalid, negative)?
        } else {
            invalid
        };
        let basic = negate(b, invalid)?;
        let all = invoke(b, "AllCharacters", vec![text])?;
        let valid = and(b, basic, all)?;
        define(
            b,
            &format!("{NAME}.Syntax.U{}", u8::from(unsigned)),
            &[19],
            0,
            valid,
        )?;
    }
    let mut c = Circuit::new(&[32, 1, 1, 1]);
    let len = c.inputs[0].clone();
    let negative = c.inputs[1][0];
    let plus = c.inputs[2][0];
    let zero = c.inputs[3][0];
    let one = c.equal(&len, &constant(1, 32));
    let many = c.lt(&constant(1, 32), &len, false);
    let negative_zero = c.and(negative, one);
    let redundant = c.or(negative_zero, many);
    let redundant = c.and(redundant, zero);
    let noncanonical = c.or(plus, redundant);
    let canonical = c.not(noncanonical);
    let canon = circuit(b, &format!("{NAME}.CanonicalCircuit"), c, vec![canonical])?;
    let text = b.var(0)?;
    let sign = invoke(b, "FirstSign", vec![text])?;
    let first = at(b, text, 0)?;
    let after = at(b, text, 1)?;
    let body_char = mux_cube(b, 4, sign, after, first)?;
    let zero_char = invoke(b, "Zero", vec![body_char])?;
    let negative = invoke(b, "FirstMinus", vec![text])?;
    let plus = invoke(b, "FirstPlus", vec![text])?;
    let length = invoke(b, "BodyLength", vec![text])?;
    let body = call(b, &canon, vec![length, negative, plus, zero_char])?;
    define(b, &format!("{NAME}.Canonical"), &[19], 0, body)?;
    let mut c = Circuit::new(&[128, 16]);
    let state = c.inputs[0].clone();
    let digit = c.inputs[1][..4].to_vec();
    let acc = state[..64].to_vec();
    let mut twice = vec![F];
    twice.extend(&acc[..63]);
    let mut eight = vec![F; 3];
    eight.extend(&acc[..61]);
    let shifted = c.nonzero(&acc[61..]);
    let (sum, a) = c.add(&twice, &eight, F);
    let (value, v) = c.add(&sum, &Circuit::extend(&digit, 64, false), F);
    let carry = c.or(a, v);
    let carry = c.or(carry, shifted);
    let carry = c.or(carry, state[64]);
    let mut output = value;
    output.push(carry);
    let step = circuit(b, &format!("{NAME}.Step"), c, output)?;
    let shared = Shared {
        step,
        remaining,
        small,
        bounded,
    };
    let mut lets = vec![];
    let text = b.var(0)?;
    let len = invoke(b, "BodyLength", vec![text])?;
    lets.push((b.cube(5)?, len));
    let state_type = b.cube(7)?;
    for i in 0..20u32 {
        let text = b.var(i + 1)?;
        let len = b.var(i)?;
        let old = if i == 0 { word(b, 0, 7)? } else { b.var(0)? };
        let sign = invoke(b, "FirstSign", vec![text])?;
        let first = at(b, text, i as usize)?;
        let after = at(b, text, i as usize + 1)?;
        let ch = mux_cube(b, 4, sign, after, first)?;
        let changed = call(b, &shared.step, vec![old, ch])?;
        let index = word(b, i, 5)?;
        let active = call(b, &shared.remaining, vec![index, len])?;
        let body = mux_cube(b, 7, active, changed, old)?;
        lets.push((state_type, body));
    }
    let mut body = b.var(0)?;
    for (ty, value) in lets.into_iter().rev() {
        body = b.term(TermNode::Let { ty, value, body })?;
    }
    define(b, &format!("{NAME}.State"), &[19], 7, body)?;
    Ok(shared)
}
fn parser(
    b: &mut Builder,
    shared: &Shared,
    token: &str,
    width: u32,
    signed: bool,
) -> R<OrdinaryIntegerParseDefinition> {
    let stem = format!("{NAME}.{token}");
    let depth = address_bits(width);
    let value_type_id = format!("mpk.csharp.value.{token}.v1");
    let codec_id = match token {
        "duration" => "duration_ticks".into(),
        "instant" => "unix_milliseconds".into(),
        _ => format!("integer.{token}"),
    };
    BoundaryCodec::new(&codec_id, &value_type_id, None, None)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    let mut c = Circuit::new(&[128, 1]);
    let state = c.inputs[0].clone();
    let sign = c.inputs[1][0];
    let max = if signed {
        (1u128 << (width - 1)) - 1
    } else {
        (1u128 << width) - 1
    };
    let maximum = if signed {
        c.select(sign, &constant(max + 1, 64), &constant(max, 64))
    } else {
        constant(max, 64)
    };
    let excess = c.lt(&maximum, &state[..64], false);
    let bad = c.or(excess, state[64]);
    let good = c.not(bad);
    let range = circuit(b, &format!("{stem}.Range"), c, vec![good])?;
    let mut c = Circuit::new(&[128, 1]);
    let state = c.inputs[0].clone();
    let sign = c.inputs[1][0];
    let raw = state[..width as usize].to_vec();
    let value = if signed {
        let negative = c.neg(&raw);
        c.select(sign, &negative, &raw)
    } else {
        raw
    };
    let value_fn = circuit(b, &format!("{stem}.Value"), c, value)?;
    // Decision uses 0 for success and parse_error+1 otherwise. Conditional
    // branches retain bound/syntax/canonical/range precedence and laziness.
    let text = b.var(1)?;
    let state = b.var(0)?;
    let sign = invoke(b, "FirstMinus", vec![text])?;
    let in_range = call(b, &range, vec![state, sign])?;
    let len = invoke(b, "BodyLength", vec![text])?;
    let short = call(b, &shared.small, vec![len])?;
    let in_range = and(b, short, in_range)?;
    let success = word(b, 0, 3)?;
    let range_error = word(b, 5, 3)?;
    let outcome = mux_cube(b, 3, in_range, success, range_error)?;
    let canonical = invoke(b, "Canonical", vec![text])?;
    let noncanonical = word(b, 3, 3)?;
    let outcome = mux_cube(b, 3, canonical, outcome, noncanonical)?;
    let syntax = invoke(b, &format!("Syntax.U{}", u8::from(!signed)), vec![text])?;
    let syntax_error = word(b, 2, 3)?;
    let outcome = mux_cube(b, 3, syntax, outcome, syntax_error)?;
    let len = invoke(b, "Length", vec![text])?;
    let bounded = call(b, &shared.bounded, vec![len])?;
    let bound_error = word(b, 1, 3)?;
    let outcome = mux_cube(b, 3, bounded, outcome, bound_error)?;
    define(b, &format!("{stem}.Decision"), &[19, 7], 3, outcome)?;
    let result_depth = depth.max(5) + 1;
    let code = b.var(result_depth + 1)?;
    let value = b.var(result_depth)?;
    let zero = truth(b, false)?;
    let mut failed = zero;
    for i in 0..3 {
        let bit = core_read(b, code, i, 3)?;
        failed = or(b, failed, bit)?;
    }
    let mut header = failed;
    for i in 1..result_depth {
        let v = b.var(result_depth - 1 - i)?;
        header = core_mux(b, v, zero, header)?;
    }
    let value_args = (result_depth - depth..result_depth)
        .map(|i| b.var(result_depth - 1 - i))
        .collect::<R<Vec<_>>>()?;
    let mut payload = b.app(value, value_args)?;
    for i in 1..result_depth - depth {
        let v = b.var(result_depth - 1 - i)?;
        payload = core_mux(b, v, zero, payload)?;
    }
    // error codes 0,1,2,4 correspond to decision tags 1,2,3,5.
    let mut error_bits = vec![];
    for wanted in [2u32, 3, 5] {
        let mut selected = truth(b, true)?;
        for i in 0..3 {
            let v = core_read(b, code, i, 3)?;
            let v = if wanted & (1 << i) == 0 {
                negate(b, v)?
            } else {
                v
            };
            selected = and(b, selected, v)?;
        }
        error_bits.push(selected);
    }
    let mut error = core_select(
        b,
        &error_bits,
        result_depth,
        result_depth - 5,
        result_depth,
        zero,
    )?;
    for i in 1..result_depth - 5 {
        let v = b.var(result_depth - 1 - i)?;
        error = core_mux(b, v, zero, error)?;
    }
    let payload = core_mux(b, failed, error, payload)?;
    let role = b.var(result_depth - 1)?;
    let body = core_mux(b, role, payload, header)?;
    let body = b.wrap_selectors(result_depth, body)?;
    define(b, &format!("{stem}.Pack"), &[3, depth], result_depth, body)?;
    let text = b.var(0)?;
    let state = invoke(b, "State", vec![text])?;
    let text = b.var(1)?;
    let s = b.var(0)?;
    let decision = call(b, &format!("{stem}.Decision"), vec![text, s])?;
    let text = b.var(2)?;
    let s = b.var(1)?;
    let sign = invoke(b, "FirstMinus", vec![text])?;
    let value = call(b, &value_fn, vec![s, sign])?;
    let code = b.var(0)?;
    let body = call(b, &format!("{stem}.Pack"), vec![code, value])?;
    let decision_type = b.cube(3)?;
    let state_type = b.cube(7)?;
    let body = b.term(TermNode::Let {
        ty: decision_type,
        value: decision,
        body,
    })?;
    let body = b.term(TermNode::Let {
        ty: state_type,
        value: state,
        body,
    })?;
    let parse_definition = format!("{stem}.Parse");
    define(b, &parse_definition, &[19], result_depth, body)?;
    let parse_result_shape = OrdinaryShape::Sum {
        arms: vec![
            OrdinaryArm {
                tag: 0,
                id: "ok".into(),
                fields: vec![OrdinaryField {
                    id: "0".into(),
                    shape: OrdinaryShape::Reference {
                        type_id: value_type_id.clone(),
                    },
                }],
            },
            OrdinaryArm {
                tag: 1,
                id: "error".into(),
                fields: vec![OrdinaryField {
                    id: "0".into(),
                    shape: OrdinaryShape::Reference {
                        type_id: "mpk.csharp.value.parse_error.v1".into(),
                    },
                }],
            },
        ],
    };
    Ok(OrdinaryIntegerParseDefinition {
        codec_id,
        value_type_id,
        value_depth: depth,
        signed,
        parse_result_shape,
        parse_definition,
    })
}
pub(super) fn emit_parsers(
    layouts: &OrdinaryCarrierProgram,
    b: &mut Builder,
) -> R<Vec<OrdinaryIntegerParseDefinition>> {
    let mut shared = None;
    let mut definitions = vec![];
    for (token, width, signed) in [
        ("i8", 8, true),
        ("u8", 8, false),
        ("i16", 16, true),
        ("u16", 16, false),
        ("i32", 32, true),
        ("u32", 32, false),
        ("i64", 64, true),
        ("u64", 64, false),
        ("duration", 64, true),
        ("instant", 64, true),
    ] {
        if let Some(c) = layouts
            .carriers()
            .iter()
            .find(|c| c.type_id == format!("mpk.csharp.value.{token}.v1"))
        {
            if c.depth != address_bits(width) || c.shape != (OrdinaryShape::Bits { width }) {
                return Err(OrdinaryCarrierError::Shape);
            }
            if shared.is_none() {
                shared = Some(helpers(b)?);
            }
            definitions.push(parser(b, shared.as_ref().unwrap(), token, width, signed)?);
        }
    }
    Ok(definitions)
}
pub fn generate_csharp_practical_ordinary_integer_parsers(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryIntegerParseProgram> {
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut b = Builder::new()?;
    let definitions = emit_parsers(&layouts, &mut b)?;
    let certificate = b.finish()?;
    let p = OrdinaryIntegerParseProgram {
        schema: "mpk.csharp.ordinary_integer_parsers.v1".into(),
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
pub fn import_csharp_practical_ordinary_integer_parsers(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryIntegerParseProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_integer_parsers(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
