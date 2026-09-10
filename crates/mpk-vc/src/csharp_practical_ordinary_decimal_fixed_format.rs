//! Fixed decimal formatting, with all 29 scales and five explicit rounding modes.
use super::decimal_format::emit_decimal_digit_helpers;
use super::hex_codecs::{call, truth, word};
use super::integer_format::{circuit, define};
use super::*;
const NAME: &str = "Mpk.CSharp.Ordinary.DecimalFixedFormat";
const VALUE: &str = "mpk.csharp.value.decimal.v1";
const MODES: [&str; 5] = [
    "ToEven",
    "AwayFromZero",
    "ToZero",
    "ToNegativeInfinity",
    "ToPositiveInfinity",
];
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryDecimalFixedFormatDefinition {
    pub codec_id: String,
    pub value_type_id: String,
    pub value_depth: u32,
    pub scale: u8,
    pub rounding: String,
    pub format_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryDecimalFixedFormatProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryDecimalFixedFormatDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryDecimalFixedFormatProgram {
    pub fn definitions(&self) -> &[OrdinaryDecimalFixedFormatDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary fixed decimal format")
    }
}
fn constant(n: u32) -> Word {
    (0..32)
        .map(|i| if n & (1 << i) != 0 { T } else { F })
        .collect()
}
fn helpers(b: &mut Builder) -> R<(String, String)> {
    let mut c = Circuit::new(&[512, 32, 32]);
    let raw = c.inputs[0].clone();
    let target = c.inputs[1].clone();
    let digits = c.inputs[2].clone();
    let coefficient = (0..96).map(|i| raw[2 + (i << 2)]).collect::<Word>();
    let nonzero = c.nonzero(&coefficient);
    let negative = c.and(raw[0], nonzero);
    let scale = (0..8).map(|i| raw[1 + (i << 6)]).collect::<Word>();
    let scale = Circuit::extend(&scale, 32, false);
    let small = c.lt(&scale, &digits, false);
    let integer = c.sub(&digits, &scale).0;
    let integer = c.select(small, &integer, &constant(1));
    let point = c.nonzero(&target);
    let length = c.add(&integer, &target, point).0;
    let length = c.add(&length, &constant(0), negative).0;
    let output = length
        .into_iter()
        .chain(scale)
        .chain(integer)
        .chain([negative, point])
        .collect();
    let layout = circuit(b, &format!("{NAME}.Layout"), c, output)?;
    let mut c = Circuit::new(&[128, 32]);
    let shape = c.inputs[0].clone();
    let index = c.inputs[1].clone();
    let mut offset = constant(0);
    offset[0] = shape[96];
    let pos = c.sub(&index, &offset).0;
    let first = c.equal(&index, &constant(0));
    let minus = c.and(first, shape[96]);
    let integer = &shape[64..96];
    let scale = &shape[32..64];
    let point = c.equal(&pos, integer);
    let point = c.and(shape[97], point);
    let after = c.lt(integer, &pos, false);
    let after = c.and(shape[97], after);
    let base = c.add(scale, integer, F).0;
    let appended = c.lt(&base, &pos, false);
    let appended = c.and(after, appended);
    let reverse = c.sub(&base, &pos).0;
    let before = c.sub(&reverse, &constant(1)).0;
    let reverse = c.select(after, &reverse, &before);
    let visible = c.lt(&index, &shape[..32], false);
    let mut output = reverse;
    output.extend([visible, minus, point, appended]);
    let position = circuit(b, &format!("{NAME}.Position"), c, output)?;
    Ok((layout, position))
}
fn emit(b: &mut Builder) -> R<Vec<OrdinaryDecimalFixedFormatDefinition>> {
    let digits = emit_decimal_digit_helpers(b)?;
    emit_with_digits(b, digits)
}
pub(super) fn emit_with_digits(
    b: &mut Builder,
    digits: super::decimal_format::DecimalDigitHelpers,
) -> R<Vec<OrdinaryDecimalFixedFormatDefinition>> {
    let (layout, position) = helpers(b)?;
    let mut definitions = vec![];
    for mode in MODES {
        // Existing ordinary decimal rounding preserves the original scale when
        // target >= scale. Only the text renderer extends the fractional zeros.
        let round = super::decimal::emit_decimal(b, &format!("decimal.round.{mode}.2"))?;
        let source = b.var(1)?;
        let target = b.var(0)?;
        let rounded = call(b, &round.result_definition, vec![source, target])?;
        let mut lets = vec![(b.cube(9)?, rounded)];
        for i in 0..29 {
            let previous = b.var(0)?;
            let input = if i == 0 {
                call(b, &digits.magnitude, vec![previous])?
            } else {
                previous
            };
            let pair = call(b, &digits.divide, vec![input])?;
            lets.push((b.cube(7)?, pair));
        }
        let mut chars = vec![];
        for i in 0..29 {
            let pair = b.var(9 + 28 - i)?;
            let ch = call(b, &digits.digit, vec![pair])?;
            let args = (0..4).rev().map(|i| b.var(i)).collect::<R<Vec<_>>>()?;
            chars.push(b.app(ch, args)?);
        }
        let text_digits = core_select(b, &chars, 9, 0, 5, digits.zero)?;
        let text_digits = b.wrap_selectors(9, text_digits)?;
        lets.push((b.cube(9)?, text_digits));
        let mut count = word(b, 1, 5)?;
        for i in 0..28 {
            let pair = b.var(29 - i)?;
            let more = call(b, &digits.more, vec![pair])?;
            let next = word(b, i + 2, 5)?;
            count = call(b, &format!("{PREFIX}.Cube.D5.Mux"), vec![more, next, count])?;
        }
        lets.push((b.cube(5)?, count));
        // 32 lets: count=0, digits=1, rounded=31, target=32, source=33.
        let rounded = b.var(31)?;
        let target = b.var(32)?;
        let count = b.var(0)?;
        let shape = call(b, &layout, vec![rounded, target, count])?;
        lets.push((b.cube(7)?, shape));
        let shape = b.var(19)?;
        let stored = b.var(21)?;
        let mut index_bits = vec![];
        for i in 0..32 {
            index_bits.push(if i < 14 { b.var(22 - i)? } else { digits.zero });
        }
        let index = core_select(b, &index_bits, 5, 0, 5, digits.zero)?;
        let index = b.wrap_selectors(5, index)?;
        let selected = call(b, &position, vec![shape, index])?;
        let visible = core_read(b, selected, 32, 6)?;
        let minus = core_read(b, selected, 33, 6)?;
        let point = core_read(b, selected, 34, 6)?;
        let appended = core_read(b, selected, 35, 6)?;
        let mut args = vec![];
        for i in 0..5 {
            args.push(core_read(b, selected, i, 6)?);
        }
        for i in 15..19 {
            args.push(b.var(18 - i)?);
        }
        let digit = b.app(stored, args)?;
        let mut punctuation = |ch: u8| -> R<u32> {
            let mut bits = vec![];
            for i in 0..16 {
                bits.push(truth(b, (ch as u16) & (1 << i) != 0)?);
            }
            core_select(b, &bits, 19, 15, 19, digits.zero)
        };
        let zero_char = punctuation(b'0')?;
        let dot = punctuation(b'.')?;
        let dash = punctuation(b'-')?;
        let data = core_mux(b, appended, zero_char, digit)?;
        let data = core_mux(b, point, dot, data)?;
        let data = core_mux(b, minus, dash, data)?;
        let data = core_mux(b, visible, data, digits.zero)?;
        let mut args = (0..5).rev().map(|i| b.var(i)).collect::<R<Vec<_>>>()?;
        args.extend([digits.zero, digits.zero]);
        let mut header = b.app(shape, args)?;
        for i in 1..14 {
            let v = b.var(18 - i)?;
            header = core_mux(b, v, digits.zero, header)?;
        }
        let role = b.var(18)?;
        let body = core_mux(b, role, data, header)?;
        let mut body = b.wrap_selectors(19, body)?;
        for (ty, value) in lets.into_iter().rev() {
            body = b.term(TermNode::Let { ty, value, body })?;
        }
        let generic = format!("{NAME}.{mode}.ScaleArgument");
        define(b, &generic, &[9, 5], 19, body)?;
        for scale in 0..=28u8 {
            BoundaryCodec::new("decimal.fixed", VALUE, Some(scale), Some(mode))
                .map_err(|_| OrdinaryCarrierError::Linkage)?;
            let source = b.var(0)?;
            let target = word(b, scale.into(), 5)?;
            let body = call(b, &generic, vec![source, target])?;
            let format_definition = format!("{NAME}.{mode}.Scale{scale}");
            define(b, &format_definition, &[9], 19, body)?;
            definitions.push(OrdinaryDecimalFixedFormatDefinition {
                codec_id: "decimal.fixed".into(),
                value_type_id: VALUE.into(),
                value_depth: 9,
                scale,
                rounding: mode.into(),
                format_definition,
            });
        }
    }
    Ok(definitions)
}
pub fn generate_csharp_practical_ordinary_decimal_fixed_formats(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryDecimalFixedFormatProgram> {
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut b = Builder::new()?;
    let mut definitions = vec![];
    if let Some(c) = layouts.carriers().iter().find(|c| c.type_id == VALUE) {
        if c.depth != 9 || c.shape != primitive("decimal", vir)? {
            return Err(OrdinaryCarrierError::Shape);
        }
        definitions = emit(&mut b)?;
    }
    let certificate = b.finish()?;
    let p = OrdinaryDecimalFixedFormatProgram {
        schema: "mpk.csharp.ordinary_decimal_fixed_formats.v1".into(),
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
pub fn import_csharp_practical_ordinary_decimal_fixed_formats(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryDecimalFixedFormatProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_decimal_fixed_formats(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
