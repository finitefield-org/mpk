//! Normalized decimal text from the explicit sign/scale/coefficient carrier.
use super::hex_codecs::{call, truth, word};
use super::integer_format::{circuit, define};
use super::*;
const NAME: &str = "Mpk.CSharp.Ordinary.DecimalFormat";
const VALUE: &str = "mpk.csharp.value.decimal.v1";
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryDecimalFormatDefinition {
    pub codec_id: String,
    pub value_type_id: String,
    pub value_depth: u32,
    pub format_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryDecimalFormatProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryDecimalFormatDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryDecimalFormatProgram {
    pub fn definitions(&self) -> &[OrdinaryDecimalFormatDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary decimal format")
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
#[derive(Clone)]
pub(super) struct DecimalDigitHelpers {
    pub(super) divide: String,
    pub(super) magnitude: String,
    pub(super) more: String,
    pub(super) digit: String,
    pub(super) zero: u32,
}
pub(super) fn emit_decimal_digit_helpers(b: &mut Builder) -> R<DecimalDigitHelpers> {
    for depth in [5, 7] {
        if !b
            .globals
            .contains_key(&format!("{PREFIX}.Cube.D{depth}.Mux"))
        {
            b.helpers(depth)?;
        }
    }
    // A packed private pair contains the 96-bit quotient and four-bit remainder.
    let mut c = Circuit::new(&[128]);
    let input = c.inputs[0].clone();
    let mut quotient = vec![F; 96];
    let mut rem = vec![F; 4];
    for i in (0..96).rev() {
        let mut shifted = vec![input[i]];
        shifted.extend(&rem);
        let (difference, ge) = c.sub(&shifted, &constant(10, 5));
        quotient[i] = ge;
        rem = c.select(ge, &difference[..4], &shifted[..4]);
    }
    quotient.extend(rem);
    let divide = circuit(b, &format!("{NAME}.Div10"), c, quotient)?;
    let c = Circuit::new(&[512]);
    let raw = c.inputs[0].clone();
    let coefficient = (0..96).map(|i| raw[2 + (i << 2)]).collect::<Word>();
    let magnitude = circuit(b, &format!("{NAME}.Coefficient"), c, coefficient)?;
    // Div10 ignores padding beyond bit 95, so the previous pair is the next
    // dividend without an extra quotient projection.
    let pair = b.var(4)?;
    let zero = truth(b, false)?;
    let one = truth(b, true)?;
    let mut bits = vec![];
    for i in 0..16 {
        bits.push(if i < 4 {
            core_read(b, pair, 96 + i, 7)?
        } else if i == 4 || i == 5 {
            one
        } else {
            zero
        });
    }
    let body = core_select(b, &bits, 4, 0, 4, zero)?;
    let body = b.wrap_selectors(4, body)?;
    define(b, &format!("{NAME}.Digit"), &[7], 4, body)?;
    let mut c = Circuit::new(&[128]);
    let pair = c.inputs[0].clone();
    let more = c.nonzero(&pair[..96]);
    let more = circuit(b, &format!("{NAME}.More"), c, vec![more])?;

    Ok(DecimalDigitHelpers {
        divide,
        magnitude,
        more,
        digit: format!("{NAME}.Digit"),
        zero,
    })
}
fn emit(b: &mut Builder) -> R<OrdinaryDecimalFormatDefinition> {
    let digits = emit_decimal_digit_helpers(b)?;
    emit_with_digits(b, digits)
}
pub(super) fn emit_with_digits(
    b: &mut Builder,
    digits: DecimalDigitHelpers,
) -> R<OrdinaryDecimalFormatDefinition> {
    let DecimalDigitHelpers {
        divide,
        magnitude,
        more,
        zero,
        ..
    } = digits;
    // Scan low digits while the original scale permits removing trailing zeros.
    // Count stops at the first nonzero digit, represented by a sticky flag.
    let mut c = Circuit::new(&[64, 128, 512]);
    let state = c.inputs[0].clone();
    let pair = c.inputs[1].clone();
    let raw = c.inputs[2].clone();
    let scale = (0..8).map(|i| raw[1 + (i << 6)]).collect::<Word>();
    let scale = Circuit::extend(&scale, 32, false);
    let within = c.lt(&state[..32], &scale, false);
    let nonzero = c.nonzero(&pair[96..100]);
    let stopped = c.or(state[32], nonzero);
    let enabled = c.not(stopped);
    let enabled = c.and(within, enabled);
    let incremented = c.add(&state[..32], &constant(1, 32), F).0;
    let count = c.select(enabled, &incremented, &state[..32]);
    let mut output = count;
    output.push(stopped);
    let trim = circuit(b, &format!("{NAME}.Trim"), c, output)?;

    // Layout packs length, effective scale, integer digit count and sign.
    let mut c = Circuit::new(&[512, 64, 32]);
    let raw = c.inputs[0].clone();
    let trimmed = c.inputs[1][..32].to_vec();
    let digits = c.inputs[2].clone();
    let coefficient = (0..96).map(|i| raw[2 + (i << 2)]).collect::<Word>();
    let nonzero = c.nonzero(&coefficient);
    let negative = c.and(raw[0], nonzero);
    let scale = (0..8).map(|i| raw[1 + (i << 6)]).collect::<Word>();
    let scale = Circuit::extend(&scale, 32, false);
    let scale = c.sub(&scale, &trimmed).0;
    let effective_digits = c.sub(&digits, &trimmed).0;
    let effective_digits = c.select(nonzero, &effective_digits, &constant(1, 32));
    let small = c.lt(&scale, &effective_digits, false);
    let integer_digits = c.sub(&effective_digits, &scale).0;
    let integer_digits = c.select(small, &integer_digits, &constant(1, 32));
    let has_point = c.nonzero(&scale);
    let length = c.add(&integer_digits, &scale, has_point).0;
    let length = c.add(&length, &constant(0, 32), negative).0;
    let output = length
        .into_iter()
        .chain(scale)
        .chain(integer_digits)
        .chain([negative])
        .collect();
    let layout = circuit(b, &format!("{NAME}.Layout"), c, output)?;

    // Per-character selection is ordinary arithmetic over the full 14-bit index.
    let mut c = Circuit::new(&[128, 64, 32]);
    let layout_bits = c.inputs[0].clone();
    let trimmed = c.inputs[1][..32].to_vec();
    let index = c.inputs[2].clone();
    let negative = layout_bits[96];
    let mut sign_offset = constant(0, 32);
    sign_offset[0] = negative;
    let pos = c.sub(&index, &sign_offset).0;
    let first = c.equal(&index, &constant(0, 32));
    let minus = c.and(negative, first);
    let scale = &layout_bits[32..64];
    let integer_digits = &layout_bits[64..96];
    let has_point = c.nonzero(scale);
    let point_position = c.equal(&pos, integer_digits);
    let point = c.and(has_point, point_position);
    let after_point = c.lt(integer_digits, &pos, false);
    let after_point = c.and(has_point, after_point);
    let base = c.add(scale, integer_digits, F).0;
    let base = c.add(&base, &trimmed, F).0;
    let reverse = c.sub(&base, &pos).0;
    let before = c.sub(&reverse, &constant(1, 32)).0;
    let reverse = c.select(after_point, &reverse, &before);
    let visible = c.lt(&index, &layout_bits[..32], false);
    let mut output = reverse;
    output.extend([visible, minus, point]);
    let position = circuit(b, &format!("{NAME}.Position"), c, output)?;

    let mut lets = vec![];
    let pair_type = b.cube(7)?;
    for i in 0..29 {
        let previous = b.var(0)?;
        let dividend = if i == 0 {
            call(b, &magnitude, vec![previous])?
        } else {
            previous
        };
        let pair = call(b, &divide, vec![dividend])?;
        lets.push((pair_type, pair));
    }
    let mut chars = vec![];
    for i in 0..29 {
        let pair = b.var(9 + 28 - i)?;
        let ch = invoke(b, "Digit", vec![pair])?;
        let selectors = (0..4).rev().map(|i| b.var(i)).collect::<R<Vec<_>>>()?;
        chars.push(b.app(ch, selectors)?);
    }
    let digits = core_select(b, &chars, 9, 0, 5, zero)?;
    let digits = b.wrap_selectors(9, digits)?;
    lets.push((b.cube(9)?, digits));
    let mut count = word(b, 1, 5)?;
    for i in 0..28 {
        let pair = b.var(29 - i)?;
        let nonzero = call(b, &more, vec![pair])?;
        let next = word(b, i + 2, 5)?;
        count = call(
            b,
            &format!("{PREFIX}.Cube.D5.Mux"),
            vec![nonzero, next, count],
        )?;
    }
    lets.push((b.cube(5)?, count));
    // 31 lets now: digit count, digit cube, pairs 28..0. The source is index 31.
    let trim_type = b.cube(6)?;
    for i in 0..28 {
        let state = if i == 0 { word(b, 0, 6)? } else { b.var(0)? };
        let pair = b.var(30)?; // original pair i shifts by i added trim states
        let source = b.var(31 + i)?;
        let next = call(b, &trim, vec![state, pair, source])?;
        lets.push((trim_type, next));
    }
    // 59 lets: final trim=0, digit count=28, digits=29, original source=59.
    let source = b.var(59)?;
    let trimmed = b.var(0)?;
    let count = b.var(28)?;
    let shape = call(b, &layout, vec![source, trimmed, count])?;
    lets.push((b.cube(7)?, shape));
    // Under nineteen output selectors: layout=19, trimmed=20, digits=49.
    let shape = b.var(19)?;
    let trimmed = b.var(20)?;
    let digits = b.var(49)?;
    let mut index_bits = vec![];
    for i in 0..32 {
        index_bits.push(if i < 14 { b.var(22 - i)? } else { zero });
    }
    let index = core_select(b, &index_bits, 5, 0, 5, zero)?;
    let index = b.wrap_selectors(5, index)?;
    let selected = call(b, &position, vec![shape, trimmed, index])?;
    let visible = core_read(b, selected, 32, 6)?;
    let minus = core_read(b, selected, 33, 6)?;
    let point = core_read(b, selected, 34, 6)?;
    let mut args = vec![];
    for i in 0..5 {
        args.push(core_read(b, selected, i, 6)?);
    }
    for i in 15..19 {
        args.push(b.var(18 - i)?);
    }
    let digit = b.app(digits, args)?;
    let mut punctuation = |ch: u8| -> R<u32> {
        let mut bits = vec![];
        for i in 0..16 {
            bits.push(truth(b, (ch as u16) & (1 << i) != 0)?);
        }
        core_select(b, &bits, 19, 15, 19, zero)
    };
    let dot = punctuation(b'.')?;
    let dash = punctuation(b'-')?;
    let data = core_mux(b, point, dot, digit)?;
    let data = core_mux(b, minus, dash, data)?;
    let data = core_mux(b, visible, data, zero)?;
    let mut args = (0..5).rev().map(|i| b.var(i)).collect::<R<Vec<_>>>()?;
    args.extend([zero, zero]);
    let mut header = b.app(shape, args)?;
    for i in 1..14 {
        let v = b.var(18 - i)?;
        header = core_mux(b, v, zero, header)?;
    }
    let role = b.var(18)?;
    let body = core_mux(b, role, data, header)?;
    let mut body = b.wrap_selectors(19, body)?;
    for (ty, value) in lets.into_iter().rev() {
        body = b.term(TermNode::Let { ty, value, body })?;
    }
    let format_definition = format!("{NAME}.Normalized");
    define(b, &format_definition, &[9], 19, body)?;
    Ok(OrdinaryDecimalFormatDefinition {
        codec_id: "decimal.normalized".into(),
        value_type_id: VALUE.into(),
        value_depth: 9,
        format_definition,
    })
}
pub fn generate_csharp_practical_ordinary_decimal_formats(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryDecimalFormatProgram> {
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut b = Builder::new()?;
    let mut definitions = vec![];
    if let Some(c) = layouts.carriers().iter().find(|c| c.type_id == VALUE) {
        if c.depth != 9 || c.shape != primitive("decimal", vir)? {
            return Err(OrdinaryCarrierError::Shape);
        }
        definitions.push(emit(&mut b)?);
    }
    let certificate = b.finish()?;
    let p = OrdinaryDecimalFormatProgram {
        schema: "mpk.csharp.ordinary_decimal_formats.v1".into(),
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
pub fn import_csharp_practical_ordinary_decimal_formats(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryDecimalFormatProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_decimal_formats(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
