//! Canonical decimal parsing with ordered errors and ordinary finite circuits.
use super::hex_codecs::{call, truth, word};
use super::integer_format::{circuit, define};
use super::*;

const NAME: &str = "Mpk.CSharp.Ordinary.DecimalParse";
const VALUE: &str = "mpk.csharp.value.decimal.v1";
const MODES: [&str; 5] = [
    "ToEven",
    "AwayFromZero",
    "ToZero",
    "ToNegativeInfinity",
    "ToPositiveInfinity",
];

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryDecimalParseDefinition {
    pub codec_id: String,
    pub value_type_id: String,
    pub value_depth: u32,
    pub scale: Option<u8>,
    pub rounding: Option<String>,
    pub parse_result_shape: OrdinaryShape,
    pub parse_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryDecimalParseProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryDecimalParseDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryDecimalParseProgram {
    pub fn definitions(&self) -> &[OrdinaryDecimalParseDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary decimal parsers")
    }
}
fn constant(n: u32, width: usize) -> Word {
    (0..width)
        .map(|i| if i < 32 && n & (1 << i) != 0 { T } else { F })
        .collect()
}
fn invoke(b: &mut Builder, suffix: &str, args: Vec<u32>) -> R<u32> {
    call(b, &format!("{NAME}.{suffix}"), args)
}
fn and(b: &mut Builder, x: u32, y: u32) -> R<u32> {
    let no = truth(b, false)?;
    core_mux(b, x, y, no)
}
fn or(b: &mut Builder, x: u32, y: u32) -> R<u32> {
    let yes = truth(b, true)?;
    core_mux(b, x, yes, y)
}
fn not(b: &mut Builder, x: u32) -> R<u32> {
    let yes = truth(b, true)?;
    let no = truth(b, false)?;
    core_mux(b, x, no, yes)
}
fn mux(b: &mut Builder, depth: u32, cond: u32, yes: u32, no: u32) -> R<u32> {
    call(
        b,
        &format!("{PREFIX}.Cube.D{depth}.Mux"),
        vec![cond, yes, no],
    )
}
// Embed the enclosing little-endian index selectors as a full u32 word.
fn index_word(b: &mut Builder, total: u32, start: u32, count: u32) -> R<u32> {
    let bits = (0..count)
        .map(|i| b.var(total + 4 - start - i))
        .collect::<R<Vec<_>>>()?;
    let zero = truth(b, false)?;
    let body = core_select(b, &bits, 5, 0, 5, zero)?;
    b.wrap_selectors(5, body)
}
fn emit_text(b: &mut Builder) -> R<()> {
    for d in [3, 4, 5, 8] {
        if !b.globals.contains_key(&format!("{PREFIX}.Cube.D{d}.Mux")) {
            b.helpers(d)?;
        }
    }
    let text = b.var(5)?;
    let zero = truth(b, false)?;
    let mut args = vec![zero; 14];
    args.extend(b.selectors(5)?);
    let body = b.app(text, args)?;
    let body = b.wrap_selectors(5, body)?;
    define(b, &format!("{NAME}.Length"), &[19], 5, body)?;
    let text = b.var(5)?;
    let index = b.var(4)?;
    let mut args = vec![truth(b, true)?];
    for i in 0..14 {
        args.push(core_read(b, index, i, 5)?);
    }
    args.extend(b.selectors(4)?);
    let body = b.app(text, args)?;
    let body = b.wrap_selectors(4, body)?;
    define(b, &format!("{NAME}.ReadAt"), &[19, 5], 4, body)?;

    let mut c = Circuit::new(&[16]);
    let ch = c.inputs[0].clone();
    let below = c.lt(&ch, &constant(48, 16), false);
    let lower = c.not(below);
    let upper = c.lt(&ch, &constant(58, 16), false);
    let digit = c.and(lower, upper);
    let minus = c.equal(&ch, &constant(45, 16));
    let plus = c.equal(&ch, &constant(43, 16));
    let sign = c.or(minus, plus);
    let point = c.equal(&ch, &constant(46, 16));
    let zero = c.equal(&ch, &constant(48, 16));
    let nonzero = c.not(zero);
    let nonzero = c.and(digit, nonzero);
    let classify = circuit(
        b,
        &format!("{NAME}.ClassifyCircuit"),
        c,
        vec![digit, sign, minus, plus, point, zero, nonzero],
    )?;
    let ch = b.var(0)?;
    let body = call(b, &classify, vec![ch])?;
    define(b, &format!("{NAME}.Classify"), &[4], 3, body)?;
    for (name, bit) in [("Sign", 1), ("Minus", 2), ("Plus", 3)] {
        let text = b.var(0)?;
        let index = word(b, 0, 5)?;
        let ch = invoke(b, "ReadAt", vec![text, index])?;
        let class = invoke(b, "Classify", vec![ch])?;
        let body = core_read(b, class, bit, 3)?;
        define(b, &format!("{NAME}.First{name}"), &[19], 0, body)?;
    }
    let text = b.var(0)?;
    let sign = invoke(b, "FirstSign", vec![text])?;
    let one = word(b, 1, 5)?;
    let zero = word(b, 0, 5)?;
    let index = mux(b, 5, sign, one, zero)?;
    let ch = invoke(b, "ReadAt", vec![text, index])?;
    let class = invoke(b, "Classify", vec![ch])?;
    let body = core_read(b, class, 5, 3)?;
    define(b, &format!("{NAME}.FirstBodyZero"), &[19], 0, body)?;
    let mut c = Circuit::new(&[32]);
    let len = c.inputs[0].clone();
    let last = c.sub(&len, &constant(1, 32)).0;
    let last = circuit(b, &format!("{NAME}.LastIndex"), c, last)?;
    let text = b.var(0)?;
    let len = invoke(b, "Length", vec![text])?;
    let index = call(b, &last, vec![len])?;
    let ch = invoke(b, "ReadAt", vec![text, index])?;
    let class = invoke(b, "Classify", vec![ch])?;
    let body = core_read(b, class, 5, 3)?;
    define(b, &format!("{NAME}.LastZero"), &[19], 0, body)?;

    let mut c = Circuit::new(&[8, 32]);
    let class = c.inputs[0].clone();
    let index = c.inputs[1].clone();
    let next = c.add(&index, &constant(1, 32), F).0;
    let result = c.select(class[4], &next, &constant(0, 32));
    let point_word = circuit(b, &format!("{NAME}.PointWord"), c, result)?;
    let text = b.var(14)?;
    let index = index_word(b, 14, 0, 14)?;
    let ch = invoke(b, "ReadAt", vec![text, index])?;
    let class = invoke(b, "Classify", vec![ch])?;
    let body = call(b, &point_word, vec![class, index])?;
    // Only index binders: share the returned C5 across its output selectors.
    let body = b.wrap_selectors(14, body)?;
    define(b, &format!("{NAME}.PointWords"), &[19], 19, body)?;
    let fold = super::super::structural::emit_aggregate_fold(b, 14)?;
    let text = b.var(0)?;
    let predicate = invoke(b, "PointWords", vec![text])?;
    let len = invoke(b, "Length", vec![text])?;
    let body = call(b, &fold.first_definition, vec![predicate, len])?;
    define(b, &format!("{NAME}.FirstPoint"), &[19], 5, body)?;
    let mut c = Circuit::new(&[8, 32, 32]);
    let class = c.inputs[0].clone();
    let index = c.inputs[1].clone();
    let point = c.inputs[2].clone();
    let first = c.equal(&index, &constant(0, 32));
    let sign = c.and(class[1], first);
    let next = c.add(&index, &constant(1, 32), F).0;
    let at_point = c.equal(&next, &point);
    let dot = c.and(class[4], at_point);
    let extra = c.or(sign, dot);
    let valid = c.or(class[0], extra);
    let syntax = circuit(b, &format!("{NAME}.CharacterSyntax"), c, vec![valid])?;
    let text = b.var(15)?;
    let point = b.var(14)?;
    let index = index_word(b, 14, 0, 14)?;
    let ch = invoke(b, "ReadAt", vec![text, index])?;
    let class = invoke(b, "Classify", vec![ch])?;
    let body = call(b, &syntax, vec![class, index, point])?;
    let body = b.wrap_selectors(14, body)?;
    define(b, &format!("{NAME}.SyntaxPred"), &[19, 5], 14, body)?;
    let text = b.var(1)?;
    let point = b.var(0)?;
    let pred = invoke(b, "SyntaxPred", vec![text, point])?;
    let len = invoke(b, "Length", vec![text])?;
    let body = call(b, &fold.all_definition, vec![pred, len])?;
    define(b, &format!("{NAME}.AllSyntax"), &[19, 5], 0, body)?;
    let text = b.var(14)?;
    let index = index_word(b, 14, 0, 14)?;
    let ch = invoke(b, "ReadAt", vec![text, index])?;
    let class = invoke(b, "Classify", vec![ch])?;
    let body = core_read(b, class, 6, 3)?;
    let body = b.wrap_selectors(14, body)?;
    define(b, &format!("{NAME}.NonzeroPred"), &[19], 14, body)?;
    let text = b.var(0)?;
    let pred = invoke(b, "NonzeroPred", vec![text])?;
    let len = invoke(b, "Length", vec![text])?;
    let body = call(b, &fold.any_definition, vec![pred, len])?;
    define(b, &format!("{NAME}.AnyNonzero"), &[19], 0, body)?;

    let mut c = Circuit::new(&[32, 32, 1]);
    let len = c.inputs[0].clone();
    let point = c.inputs[1].clone();
    let mut offset = constant(0, 32);
    offset[0] = c.inputs[2][0];
    let has_point = c.nonzero(&point);
    let before = c.sub(&point, &constant(1, 32)).0;
    let before = c.select(has_point, &before, &len);
    let integer = c.sub(&before, &offset).0;
    let after = c.sub(&len, &point).0;
    let fraction = c.select(has_point, &after, &constant(0, 32));
    let parts = circuit(
        b,
        &format!("{NAME}.PartsCircuit"),
        c,
        integer.into_iter().chain(fraction).collect(),
    )?;
    let text = b.var(1)?;
    let point = b.var(0)?;
    let len = invoke(b, "Length", vec![text])?;
    let sign = invoke(b, "FirstSign", vec![text])?;
    let body = call(b, &parts, vec![len, point, sign])?;
    define(b, &format!("{NAME}.Parts"), &[19, 5], 6, body)?;
    let parts = b.var(5)?;
    let mut args = b.selectors(5)?;
    args.push(truth(b, true)?);
    let body = b.app(parts, args)?;
    let body = b.wrap_selectors(5, body)?;
    define(b, &format!("{NAME}.Fraction"), &[6], 5, body)
}

fn emit_number(b: &mut Builder) -> R<()> {
    // Every potentially accepted spelling has <=59 characters and <=57
    // significant digits. 192 bits cover 10^57; larger inputs cannot become
    // 96-bit decimals after removing at most 28 fractional zeroes.
    let mut c = Circuit::new(&[256, 16, 1]);
    let state = c.inputs[0].clone();
    let ch = c.inputs[1].clone();
    let active = c.inputs[2][0];
    let below = c.lt(&ch, &constant(48, 16), false);
    let lower = c.not(below);
    let upper = c.lt(&ch, &constant(58, 16), false);
    let digit = c.and(lower, upper);
    let enabled = c.and(active, digit);
    let acc = &state[..192];
    let mut twice = vec![F];
    twice.extend(&acc[..191]);
    let mut eight = vec![F; 3];
    eight.extend(&acc[..189]);
    let lost = c.nonzero(&acc[189..]);
    let (sum, a) = c.add(&twice, &eight, F);
    let (sum, v) = c.add(&sum, &Circuit::extend(&ch[..4], 192, false), F);
    let carry = c.or(a, v);
    let carry = c.or(carry, lost);
    let carry = c.or(carry, state[192]);
    let mut next = sum;
    next.push(carry);
    let output = c.select(enabled, &next, &state[..193]);
    let step = circuit(b, &format!("{NAME}.Step"), c, output)?;
    let mut c = Circuit::new(&[32, 32]);
    let index = c.inputs[0].clone();
    let len = c.inputs[1].clone();
    let active = c.lt(&index, &len, false);
    let active = circuit(b, &format!("{NAME}.Active"), c, vec![active])?;
    let text = b.var(0)?;
    let len = invoke(b, "Length", vec![text])?;
    let mut lets = vec![(b.cube(5)?, len)];
    for i in 0..59 {
        let text = b.var(i + 1)?;
        let len = b.var(i)?;
        let index = word(b, i, 5)?;
        let ch = invoke(b, "ReadAt", vec![text, index])?;
        let enabled = call(b, &active, vec![index, len])?;
        let old = if i == 0 { word(b, 0, 8)? } else { b.var(0)? };
        let value = call(b, &step, vec![old, ch, enabled])?;
        lets.push((b.cube(8)?, value));
    }
    let mut body = b.var(0)?;
    for (ty, value) in lets.into_iter().rev() {
        body = b.term(TermNode::Let { ty, value, body })?;
    }
    define(b, &format!("{NAME}.Accumulate"), &[19], 8, body)?;

    let c = Circuit::new(&[256, 32]);
    let mut output = c.inputs[0][..193].to_vec();
    output.extend(&c.inputs[1]);
    let init = circuit(b, &format!("{NAME}.TrimInit"), c, output)?;
    let mut c = Circuit::new(&[256]);
    let state = c.inputs[0].clone();
    let mut remainder = vec![F; 4];
    let mut quotient = vec![F; 192];
    for i in (0..192).rev() {
        let mut next = vec![state[i]];
        next.extend(&remainder);
        let (difference, ge) = c.sub(&next, &constant(10, 5));
        quotient[i] = ge;
        remainder = c.select(ge, &difference[..4], &next[..4]);
    }
    let excessive = c.nonzero(&state[96..192]);
    let scale = &state[193..225];
    let fractional = c.nonzero(scale);
    let divisible = c.equal(&remainder, &constant(0, 4));
    let enabled = c.and(excessive, fractional);
    let enabled = c.and(enabled, divisible);
    let coefficient = c.select(enabled, &quotient, &state[..192]);
    let decreased = c.sub(scale, &constant(1, 32)).0;
    let scale = c.select(enabled, &decreased, scale);
    let mut output = coefficient;
    output.push(state[192]);
    output.extend(scale);
    let step = circuit(b, &format!("{NAME}.TrimStep"), c, output)?;
    let step = b.constant(&step)?;
    let pipeline = b.compose(8, &[step; 28])?;
    let state = b.var(1)?;
    let scale = b.var(0)?;
    let initial = call(b, &init, vec![state, scale])?;
    let body = b.app(pipeline, vec![initial])?;
    define(b, &format!("{NAME}.Trim"), &[8, 5], 8, body)?;
    let mut c = Circuit::new(&[256]);
    let state = c.inputs[0].clone();
    let excess = c.nonzero(&state[96..193]);
    let good = c.not(excess);
    let range = circuit(b, &format!("{NAME}.RangeCircuit"), c, vec![good])?;
    let state = b.var(0)?;
    let body = call(b, &range, vec![state])?;
    define(b, &format!("{NAME}.Range"), &[8], 0, body)?;
    let c = Circuit::new(&[256, 1]);
    let state = &c.inputs[0];
    let mut output = vec![F; 512];
    output[0] = c.inputs[1][0];
    for i in 0..8 {
        output[1 + (i << 6)] = state[193 + i];
    }
    for i in 0..96 {
        output[2 + (i << 2)] = state[i];
    }
    let value = circuit(b, &format!("{NAME}.ValueCircuit"), c, output)?;
    let state = b.var(1)?;
    let negative = b.var(0)?;
    let body = call(b, &value, vec![state, negative])?;
    define(b, &format!("{NAME}.Value"), &[8, 0], 9, body)
}

fn emit_core(b: &mut Builder) -> R<()> {
    emit_text(b)?;
    emit_number(b)?;
    let mut c = Circuit::new(&[32, 32, 64]);
    let len = c.inputs[0].clone();
    let point = c.inputs[1].clone();
    let parts = c.inputs[2].clone();
    let nonempty = c.nonzero(&len);
    let integer = c.nonzero(&parts[..32]);
    let fractional = c.nonzero(&parts[32..]);
    let no_point = c.equal(&point, &constant(0, 32));
    let tail = c.or(no_point, fractional);
    let valid = c.and(nonempty, integer);
    let valid = c.and(valid, tail);
    let basic = circuit(b, &format!("{NAME}.BasicSyntax"), c, vec![valid])?;
    let mut c = Circuit::new(&[64, 1, 32]);
    let parts = c.inputs[0].clone();
    let normalized = c.inputs[1][0];
    let target = c.inputs[2].clone();
    let within = c.lt(&parts[32..], &constant(29, 32), false);
    let exact = c.equal(&parts[32..], &target);
    let exact = c.or(normalized, exact);
    let valid = c.and(within, exact);
    let precision = circuit(b, &format!("{NAME}.Precision"), c, vec![valid])?;
    let mut c = Circuit::new(&[64]);
    let parts = c.inputs[0].clone();
    let many = c.lt(&constant(1, 32), &parts[..32], false);
    let many = circuit(b, &format!("{NAME}.ManyInteger"), c, vec![many])?;
    let mut c = Circuit::new(&[64]);
    let parts = c.inputs[0].clone();
    let fractional = c.nonzero(&parts[32..]);
    let fractional = circuit(b, &format!("{NAME}.HasFraction"), c, vec![fractional])?;
    let mut tests = vec![];
    for (name, bound) in [("Short", 60), ("Bounded", 16385)] {
        let mut c = Circuit::new(&[32]);
        let len = c.inputs[0].clone();
        let valid = c.lt(&len, &constant(bound, 32), false);
        tests.push(circuit(b, &format!("{NAME}.{name}"), c, vec![valid])?);
    }
    // Text, first point, part lengths, trimmed state, normalized, target.
    let text = b.var(5)?;
    let point = b.var(4)?;
    let parts = b.var(3)?;
    let state = b.var(2)?;
    let normalized = b.var(1)?;
    let target = b.var(0)?;
    let len = invoke(b, "Length", vec![text])?;
    let short = call(b, &tests[0], vec![len])?;
    let in_range = invoke(b, "Range", vec![state])?;
    let in_range = and(b, short, in_range)?;
    let success = word(b, 0, 3)?;
    let range_error = word(b, 5, 3)?;
    let result = mux(b, 3, in_range, success, range_error)?;
    let precise = call(b, &precision, vec![parts, normalized, target])?;
    let precision_error = word(b, 4, 3)?;
    let result = mux(b, 3, precise, result, precision_error)?;
    let plus = invoke(b, "FirstPlus", vec![text])?;
    let multiple = call(b, &many, vec![parts])?;
    let zero = invoke(b, "FirstBodyZero", vec![text])?;
    let redundant = and(b, multiple, zero)?;
    let noncanonical = or(b, plus, redundant)?;
    let negative = invoke(b, "FirstMinus", vec![text])?;
    let nonzero = invoke(b, "AnyNonzero", vec![text])?;
    let allzero = not(b, nonzero)?;
    let negative_zero = and(b, negative, allzero)?;
    let noncanonical = or(b, noncanonical, negative_zero)?;
    let has_fraction = call(b, &fractional, vec![parts])?;
    let last_zero = invoke(b, "LastZero", vec![text])?;
    let redundant_tail = and(b, has_fraction, last_zero)?;
    let redundant_tail = and(b, normalized, redundant_tail)?;
    let noncanonical = or(b, noncanonical, redundant_tail)?;
    let noncanonical_error = word(b, 3, 3)?;
    let result = mux(b, 3, noncanonical, noncanonical_error, result)?;
    let basic = call(b, &basic, vec![len, point, parts])?;
    let characters = invoke(b, "AllSyntax", vec![text, point])?;
    let syntax = and(b, basic, characters)?;
    let syntax_error = word(b, 2, 3)?;
    let result = mux(b, 3, syntax, result, syntax_error)?;
    let bounded = call(b, &tests[1], vec![len])?;
    let bound_error = word(b, 1, 3)?;
    let result = mux(b, 3, bounded, result, bound_error)?;
    define(
        b,
        &format!("{NAME}.Decision"),
        &[19, 5, 6, 8, 0, 5],
        3,
        result,
    )?;

    let mut c = Circuit::new(&[8]);
    let code = c.inputs[0].clone();
    let failed = c.nonzero(&code[..3]);
    let failed = circuit(b, &format!("{NAME}.Failed"), c, vec![failed])?;
    let mut c = Circuit::new(&[8]);
    let code = c.inputs[0].clone();
    let error = c.sub(&code[..3], &constant(1, 3)).0;
    let error = circuit(
        b,
        &format!("{NAME}.ErrorCode"),
        c,
        Circuit::extend(&error, 32, false),
    )?;
    let code = b.var(11)?;
    let value = b.var(10)?;
    let failed = call(b, &failed, vec![code])?;
    let zero = truth(b, false)?;
    let mut header = failed;
    for i in 1..10 {
        let selector = b.var(9 - i)?;
        header = core_mux(b, selector, zero, header)?;
    }
    let value_args = (0..9).rev().map(|i| b.var(i)).collect::<R<Vec<_>>>()?;
    let value_payload = b.app(value, value_args)?;
    let error = call(b, &error, vec![code])?;
    let error_args = (0..5).rev().map(|i| b.var(i)).collect::<R<Vec<_>>>()?;
    let mut error_payload = b.app(error, error_args)?;
    for i in 1..5 {
        let selector = b.var(9 - i)?;
        error_payload = core_mux(b, selector, zero, error_payload)?;
    }
    let payload = core_mux(b, failed, error_payload, value_payload)?;
    let role = b.var(9)?;
    let body = core_mux(b, role, payload, header)?;
    let body = b.wrap_selectors(10, body)?;
    define(b, &format!("{NAME}.Pack"), &[3, 9], 10, body)?;

    let text = b.var(2)?;
    let point = invoke(b, "FirstPoint", vec![text])?;
    let mut lets = vec![(b.cube(5)?, point)];
    let text = b.var(3)?;
    let point = b.var(0)?;
    let parts = invoke(b, "Parts", vec![text, point])?;
    lets.push((b.cube(6)?, parts));
    let text = b.var(4)?;
    let acc = invoke(b, "Accumulate", vec![text])?;
    lets.push((b.cube(8)?, acc));
    let parts = b.var(1)?;
    let acc = b.var(0)?;
    let fraction = invoke(b, "Fraction", vec![parts])?;
    let trimmed = invoke(b, "Trim", vec![acc, fraction])?;
    lets.push((b.cube(8)?, trimmed));
    let text = b.var(6)?;
    let normalized = b.var(5)?;
    let target = b.var(4)?;
    let point = b.var(3)?;
    let parts = b.var(2)?;
    let state = b.var(0)?;
    let code = invoke(
        b,
        "Decision",
        vec![text, point, parts, state, normalized, target],
    )?;
    lets.push((b.cube(3)?, code));
    let text = b.var(7)?;
    let state = b.var(1)?;
    let code = b.var(0)?;
    let negative = invoke(b, "FirstMinus", vec![text])?;
    let value = invoke(b, "Value", vec![state, negative])?;
    let mut body = invoke(b, "Pack", vec![code, value])?;
    for (ty, value) in lets.into_iter().rev() {
        body = b.term(TermNode::Let { ty, value, body })?;
    }
    define(b, &format!("{NAME}.Parse"), &[19, 0, 5], 10, body)
}

pub(super) fn emit_parser(b: &mut Builder) -> R<Vec<OrdinaryDecimalParseDefinition>> {
    if !b.globals.contains_key(&format!("{NAME}.Parse")) {
        emit_core(b)?;
    }
    let mut configs = vec![(None, None)];
    for scale in 0..=28 {
        for mode in MODES {
            configs.push((Some(scale), Some(mode)));
        }
    }
    let mut definitions = vec![];
    for (scale, rounding) in configs {
        let codec_id = if scale.is_none() {
            "decimal.normalized"
        } else {
            "decimal.fixed"
        };
        BoundaryCodec::new(codec_id, VALUE, scale, rounding)
            .map_err(|_| OrdinaryCarrierError::Linkage)?;
        let suffix = scale.map_or_else(
            || "Normalized".into(),
            |s| format!("Fixed.S{s}.{}", rounding.unwrap()),
        );
        let parse_definition = format!("{NAME}.{suffix}.Parse");
        if !b.globals.contains_key(&parse_definition) {
            let text = b.var(0)?;
            let normalized = truth(b, scale.is_none())?;
            let target = word(b, u32::from(scale.unwrap_or(0)), 5)?;
            let body = invoke(b, "Parse", vec![text, normalized, target])?;
            define(b, &parse_definition, &[19], 10, body)?;
        }
        let arms = [
            (0, "ok", VALUE),
            (1, "error", "mpk.csharp.value.parse_error.v1"),
        ]
        .into_iter()
        .map(|(tag, id, type_id)| OrdinaryArm {
            tag,
            id: id.into(),
            fields: vec![OrdinaryField {
                id: "0".into(),
                shape: OrdinaryShape::Reference {
                    type_id: type_id.into(),
                },
            }],
        })
        .collect();
        definitions.push(OrdinaryDecimalParseDefinition {
            codec_id: codec_id.into(),
            value_type_id: VALUE.into(),
            value_depth: 9,
            scale,
            rounding: rounding.map(Into::into),
            parse_result_shape: OrdinaryShape::Sum { arms },
            parse_definition,
        });
    }
    Ok(definitions)
}

pub fn generate_csharp_practical_ordinary_decimal_parsers(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryDecimalParseProgram> {
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut b = Builder::new()?;
    let mut definitions = vec![];
    if let Some(c) = layouts.carriers().iter().find(|c| c.type_id == VALUE) {
        if c.depth != 9 || c.shape != primitive("decimal", vir)? {
            return Err(OrdinaryCarrierError::Shape);
        }
        definitions = emit_parser(&mut b)?;
    }
    let certificate = b.finish()?;
    let p = OrdinaryDecimalParseProgram {
        schema: "mpk.csharp.ordinary_decimal_parsers.v1".into(),
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
pub fn import_csharp_practical_ordinary_decimal_parsers(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryDecimalParseProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_decimal_parsers(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decimal_parser_shared_emission_preserves_standalone_bytes() {
        let mut b = Builder::new().unwrap();
        let first = emit_parser(&mut b).unwrap();
        let count = b.static_transformers;
        assert_eq!(emit_parser(&mut b).unwrap(), first);
        assert_eq!(b.static_transformers, count);
        let bytes = b.finish().unwrap();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/decimal-parsers/binding-vc-money.hex");
        let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
        assert_eq!(std::fs::read_to_string(path).unwrap(), hex);
    }
}
