//! Fixed hexadecimal boundary codecs as ordinary definitions.
//! Source/native commutation and universal round-trip proofs remain obligations.
use super::*;

const TEXT_DEPTH: u32 = 19;
const HEX_PREFIX: &str = "Mpk.CSharp.Ordinary.HexCodec";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryHexCodecDefinition {
    pub codec_id: String,
    pub value_type_id: String,
    pub value_depth: u32,
    /// Closed ordinary result carrier: tag 0/value or tag 1/parse_error.
    pub parse_result_shape: OrdinaryShape,
    pub parse_definition: String,
    pub format_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryHexCodecProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryHexCodecDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryHexCodecProgram {
    pub fn definitions(&self) -> &[OrdinaryHexCodecDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary fixed hex codec program")
    }
}
pub(super) fn call(b: &mut Builder, name: &str, args: Vec<u32>) -> R<u32> {
    let f = b.constant(name)?;
    b.app(f, args)
}
fn define(b: &mut Builder, name: &str, input: u32, output: u32, body: u32) -> R<()> {
    let arg = b.cube(input)?;
    let result = b.cube(output)?;
    let ty = b.pi(arg, result)?;
    let body = b.lam(arg, body)?;
    b.define(name, ty, body)
}
pub(super) fn truth(b: &mut Builder, v: bool) -> R<u32> {
    b.constant(if v { "Std.Bool.true" } else { "Std.Bool.false" })
}
fn both(b: &mut Builder, a: u32, c: u32) -> R<u32> {
    call(b, "Std.Bool.and", vec![a, c])
}
fn either(b: &mut Builder, a: u32, c: u32) -> R<u32> {
    call(b, "Std.Bool.or", vec![a, c])
}
fn not(b: &mut Builder, a: u32) -> R<u32> {
    call(b, "Std.Bool.not", vec![a])
}
fn equals(b: &mut Builder, value: u32, width: u32, n: u128) -> R<u32> {
    let mut result = truth(b, true)?;
    for i in 0..width {
        let bit = core_read(b, value, i as usize, address_bits(width))?;
        let bit = if n & (1 << i) == 0 { not(b, bit)? } else { bit };
        result = both(b, result, bit)?;
    }
    Ok(result)
}
fn helpers(b: &mut Builder) -> R<()> {
    if b.globals.contains_key(&format!("{HEX_PREFIX}.EncodeDigit")) {
        return Ok(());
    }
    // Full UTF-16 code unit classification, never a truncated low-byte test.
    for class in ["Digit", "Upper", "Hyphen"] {
        let input = b.var(0)?;
        let mut yes = truth(b, false)?;
        let chars: Vec<u8> = match class {
            "Digit" => b"0123456789abcdefABCDEF".to_vec(),
            "Upper" => b"ABCDEF".to_vec(),
            _ => vec![b'-'],
        };
        for c in chars {
            let selected = equals(b, input, 16, c as u128)?;
            yes = either(b, yes, selected)?;
        }
        define(b, &format!("{HEX_PREFIX}.{class}"), 4, 0, yes)?;
    }
    let input = b.var(2)?;
    let mut output = vec![];
    for i in 0..4 {
        let mut bit = truth(b, false)?;
        for (n, c) in b"0123456789abcdef".iter().enumerate() {
            if n & (1 << i) != 0 {
                let selected = equals(b, input, 16, *c as u128)?;
                bit = either(b, bit, selected)?;
            }
        }
        output.push(bit);
    }
    let zero = truth(b, false)?;
    let body = core_select(b, &output, 2, 0, 2, zero)?;
    let body = b.wrap_selectors(2, body)?;
    define(b, &format!("{HEX_PREFIX}.DecodeDigit"), 4, 2, body)?;
    let input = b.var(4)?;
    let mut output = vec![];
    for i in 0..16 {
        let mut bit = truth(b, false)?;
        for (n, c) in b"0123456789abcdef".iter().enumerate() {
            if (*c as u16) & (1 << i) != 0 {
                let selected = equals(b, input, 4, n as u128)?;
                bit = either(b, bit, selected)?;
            }
        }
        output.push(bit);
    }
    let body = core_select(b, &output, 4, 0, 4, zero)?;
    let body = b.wrap_selectors(4, body)?;
    define(b, &format!("{HEX_PREFIX}.EncodeDigit"), 2, 4, body)?;

    Ok(())
}
fn read_at(b: &mut Builder, text: u32, position: usize) -> R<u32> {
    // A fixed position selects the full 16-bit cell; inactive cells never
    // become accepted input because exact length is separately required.
    let mut args = vec![truth(b, true)?];
    for i in 0..14 {
        args.push(truth(b, position & (1 << i) != 0)?);
    }
    b.app(text, args)
}
pub(super) fn word(b: &mut Builder, n: u32, depth: u32) -> R<u32> {
    let bits = (0..1usize << depth)
        .map(|i| truth(b, i < 32 && n & (1 << i) != 0))
        .collect::<R<Vec<_>>>()?;
    let zero = truth(b, false)?;
    let body = core_select(b, &bits, depth, 0, depth, zero)?;
    b.wrap_selectors(depth, body)
}
pub(super) fn codec(
    b: &mut Builder,
    id: &str,
    token: &str,
    width: u32,
) -> R<OrdinaryHexCodecDefinition> {
    helpers(b)?;
    let depth = address_bits(width);
    let value_type_id = format!("mpk.csharp.value.{token}.v1");
    let stem = format!("{HEX_PREFIX}.{}", id.replace('.', ".Codec."));
    let len = if id == "guid.d" {
        36
    } else {
        width as usize / 4
    };
    let hyphens: &[usize] = if id == "guid.d" {
        &[8, 13, 18, 23]
    } else {
        &[]
    };
    let positions: Vec<usize> = (0..len).filter(|i| !hyphens.contains(i)).collect();
    let source = b.var(0)?;
    let mut length_bits = vec![];
    for i in 0..32 {
        length_bits.push(core_read(b, source, i << 14, TEXT_DEPTH)?);
    }
    let mut exact = truth(b, true)?;
    for (i, &v) in length_bits.iter().enumerate() {
        let test = if len & (1 << i) == 0 { not(b, v)? } else { v };
        exact = both(b, exact, test)?;
    }
    let mut high = truth(b, false)?;
    for &v in &length_bits[15..] {
        high = either(b, high, v)?;
    }
    let mut low = truth(b, false)?;
    for &v in &length_bits[..14] {
        low = either(b, low, v)?;
    }
    let excess = both(b, length_bits[14], low)?;
    let over = either(b, high, excess)?;
    let mut syntax_ok = exact;
    let mut uppercase = truth(b, false)?;
    for i in 0..len {
        let ch = read_at(b, source, i)?;
        let kind = if hyphens.contains(&i) {
            "Hyphen"
        } else {
            "Digit"
        };
        let good = call(b, &format!("{HEX_PREFIX}.{kind}"), vec![ch])?;
        syntax_ok = both(b, syntax_ok, good)?;
        let upper = call(b, &format!("{HEX_PREFIX}.Upper"), vec![ch])?;
        uppercase = either(b, uppercase, upper)?;
    }
    let in_bound = not(b, over)?;
    let canonical = not(b, uppercase)?;
    let success = both(b, syntax_ok, canonical)?;
    let success = both(b, in_bound, success)?;
    define(b, &format!("{stem}.Success"), TEXT_DEPTH, 0, success)?;
    // Ordered error tags: input_bound=0, syntax=1, noncanonical=2.
    let syntax = not(b, syntax_ok)?;
    let tag1 = both(b, in_bound, syntax)?;
    let tag2 = both(b, in_bound, syntax_ok)?;
    let tag2 = both(b, tag2, uppercase)?;
    define(b, &format!("{stem}.ErrorBit0"), TEXT_DEPTH, 0, tag1)?;
    define(b, &format!("{stem}.ErrorBit1"), TEXT_DEPTH, 0, tag2)?;

    let text = b.var(depth)?;
    let mut payload = vec![];
    for k in 0..width as usize {
        let ch = read_at(b, text, positions[positions.len() - 1 - k / 4])?;
        let nibble = call(b, &format!("{HEX_PREFIX}.DecodeDigit"), vec![ch])?;
        payload.push(core_read(b, nibble, k % 4, 2)?);
    }
    let zero = truth(b, false)?;
    let value = core_select(b, &payload, depth, 0, depth, zero)?;
    let value = b.wrap_selectors(depth, value)?;
    define(b, &format!("{stem}.Value"), TEXT_DEPTH, depth, value)?;

    let result_depth = depth.max(5) + 1;
    let text = b.var(result_depth)?;
    let success = call(b, &format!("{stem}.Success"), vec![text])?;
    let failed = not(b, success)?;
    let mut header = zero;
    for i in (1..result_depth).rev() {
        let selector = b.var(result_depth - 1 - i)?;
        header = either(b, header, selector)?;
    }
    let header_zero = not(b, header)?;
    let header = both(b, header_zero, failed)?;
    let value = call(b, &format!("{stem}.Value"), vec![text])?;
    let mut value_args = vec![];
    for i in result_depth - depth..result_depth {
        value_args.push(b.var(result_depth - 1 - i)?);
    }
    let value = b.app(value, value_args)?;
    let e0 = call(b, &format!("{stem}.ErrorBit0"), vec![text])?;
    let e1 = call(b, &format!("{stem}.ErrorBit1"), vec![text])?;
    let error = core_select(
        b,
        &[e0, e1],
        result_depth,
        result_depth - 5,
        result_depth,
        zero,
    )?;
    let mut error_padding = truth(b, true)?;
    for i in 1..result_depth - 5 {
        let v = b.var(result_depth - 1 - i)?;
        let v = not(b, v)?;
        error_padding = both(b, error_padding, v)?;
    }
    let error = both(b, error_padding, error)?;
    let payload = core_mux(b, success, value, error)?;
    let role = b.var(result_depth - 1)?;
    let body = core_mux(b, role, payload, header)?;
    let body = b.wrap_selectors(result_depth, body)?;
    let parse_definition = format!("{stem}.Parse");
    define(b, &parse_definition, TEXT_DEPTH, result_depth, body)?;

    let mut chars = vec![];
    for i in 0..len {
        let name = format!("{stem}.Format.C{i}");
        let ch = if let Some(n) = positions.iter().position(|p| *p == i) {
            let input = b.var(2)?;
            let mut nibble = vec![];
            for j in 0..4 {
                nibble.push(core_read(
                    b,
                    input,
                    (positions.len() - 1 - n) * 4 + j,
                    depth,
                )?);
            }
            let body = core_select(b, &nibble, 2, 0, 2, zero)?;
            let body = b.wrap_selectors(2, body)?;
            let body = call(b, &format!("{HEX_PREFIX}.EncodeDigit"), vec![body])?;
            body
        } else {
            word(b, b'-' as u32, 4)?
        };
        define(b, &name, depth, 4, ch)?;
        chars.push(name);
    }
    let input = b.var(TEXT_DEPTH)?;
    let mut char_bits = vec![];
    for name in chars {
        let ch = call(b, &name, vec![input])?;
        let mut selectors = vec![];
        for i in 15..19 {
            selectors.push(b.var(TEXT_DEPTH - 1 - i)?);
        }
        char_bits.push(b.app(ch, selectors)?);
    }
    let data = core_select(b, &char_bits, TEXT_DEPTH, 1, 15, zero)?;
    let header_bits = (0..32)
        .map(|i| truth(b, len & (1 << i) != 0))
        .collect::<R<Vec<_>>>()?;
    let mut header = core_select(b, &header_bits, TEXT_DEPTH, 14, 19, zero)?;
    for i in 1..14 {
        let selector = b.var(TEXT_DEPTH - 1 - i)?;
        header = core_mux(b, selector, zero, header)?;
    }
    let role = b.var(TEXT_DEPTH - 1)?;
    let body = core_mux(b, role, data, header)?;
    let body = b.wrap_selectors(TEXT_DEPTH, body)?;
    let format_definition = format!("{stem}.Format");
    define(b, &format_definition, depth, TEXT_DEPTH, body)?;
    Ok(OrdinaryHexCodecDefinition {
        codec_id: id.into(),
        value_type_id: value_type_id.clone(),
        value_depth: depth,
        parse_result_shape: OrdinaryShape::Sum {
            arms: vec![
                OrdinaryArm {
                    tag: 0,
                    id: "ok".into(),
                    fields: vec![OrdinaryField {
                        id: "0".into(),
                        shape: OrdinaryShape::Reference {
                            type_id: value_type_id,
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
        },
        parse_definition,
        format_definition,
    })
}
pub(super) fn emit_codecs(
    layouts: &OrdinaryCarrierProgram,
    b: &mut Builder,
) -> R<Vec<OrdinaryHexCodecDefinition>> {
    let mut definitions = vec![];
    for (id, token, width) in [
        ("binary32", "f32", 32),
        ("binary64", "f64", 64),
        ("guid.n", "guid", 128),
        ("guid.d", "guid", 128),
    ] {
        if let Some(c) = layouts
            .carriers()
            .iter()
            .find(|c| c.type_id == format!("mpk.csharp.value.{token}.v1"))
        {
            if c.depth != address_bits(width) || c.shape != (OrdinaryShape::Bits { width }) {
                return Err(OrdinaryCarrierError::Shape);
            }
            BoundaryCodec::new(id, &c.type_id, None, None)
                .map_err(|_| OrdinaryCarrierError::Linkage)?;
            definitions.push(codec(b, id, token, width)?);
        }
    }
    Ok(definitions)
}
pub fn generate_csharp_practical_ordinary_hex_codecs(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryHexCodecProgram> {
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut b = Builder::new()?;
    let definitions = emit_codecs(&layouts, &mut b)?;
    let certificate = b.finish()?;
    let p = OrdinaryHexCodecProgram {
        schema: "mpk.csharp.ordinary_hex_codecs.v1".into(),
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
pub fn import_csharp_practical_ordinary_hex_codecs(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryHexCodecProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_hex_codecs(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
