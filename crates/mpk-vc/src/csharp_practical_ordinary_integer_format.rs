//! Canonical decimal integer formatting. Parsers and round-trip proofs are separate.
use super::hex_codecs::{call, truth, word};
use super::*;
const NAME: &str = "Mpk.CSharp.Ordinary.IntegerFormat";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryIntegerFormatDefinition {
    pub codec_id: String,
    pub value_type_id: String,
    pub value_depth: u32,
    pub signed: bool,
    pub format_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryIntegerFormatProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryIntegerFormatDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryIntegerFormatProgram {
    pub fn definitions(&self) -> &[OrdinaryIntegerFormatDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary integer formatting")
    }
}
pub(super) fn define(
    b: &mut Builder,
    name: &str,
    inputs: &[u32],
    output: u32,
    mut body: u32,
) -> R<()> {
    // Preserve cube depths directly. Converting a depth to a host bit width
    // and then narrowing that width to u32 silently turns C32 into Bool.
    for depth in inputs.iter().rev() {
        let ty = b.cube(*depth)?;
        body = b.lam(ty, body)?;
    }
    let mut ty = b.cube(output)?;
    for depth in inputs.iter().rev() {
        let input = b.cube(*depth)?;
        ty = b.pi(input, ty)?;
    }
    b.define(name, ty, body)
}

pub(super) fn circuit(b: &mut Builder, id: &str, circuit: Circuit, output: Word) -> R<String> {
    circuit_with_block_bits(b, id, circuit, output, 5)
}
pub(super) fn circuit_with_block_bits(
    b: &mut Builder,
    id: &str,
    circuit: Circuit,
    output: Word,
    block_bits: u32,
) -> R<String> {
    let cube = |w| format!("{PREFIX}.Cube.D{}", address_bits(w as u32));
    let signature = ClosedOperationSignature {
        id: id.into(),
        tag: ClosedOperationTag::Data,
        argument_type_ids: circuit.inputs.iter().map(|v| cube(v.len())).collect(),
        normal_result_type_id: cube(output.len()),
        ordered_checks: vec![],
    };
    Ok(emit_circuit_with_block_bits(
        b,
        IntegerCircuit {
            signature,
            circuit,
            output,
            failures: vec![],
        },
        "IntegerText",
        block_bits,
    )?
    .result_definition)
}
fn helpers(b: &mut Builder) -> R<(String, String, String)> {
    let mut c = Circuit::new(&[64]);
    let input = c.inputs[0].clone();
    let mut rem = vec![F; 4];
    let mut quotient = vec![F; 64];
    let ten = vec![F, T, F, T, F];
    // At each bit, remainder is in 0..9. Therefore the next dividend is in
    // 0..19 and a five-bit subtract suffices, including u64::MAX.
    for i in (0..64).rev() {
        let mut shifted = vec![input[i]];
        shifted.extend(&rem);
        let (difference, ge) = c.sub(&shifted, &ten);
        quotient[i] = ge;
        rem = c.select(ge, &difference[..4], &shifted[..4]);
    }
    quotient.extend(rem);
    let divide = circuit(b, &format!("{NAME}.Div10"), c, quotient)?;
    let pair = b.var(6)?;
    let mut bits = vec![];
    for i in 0..64 {
        bits.push(core_read(b, pair, i, 7)?);
    }
    let zero = truth(b, false)?;
    let body = core_select(b, &bits, 6, 0, 6, zero)?;
    let body = b.wrap_selectors(6, body)?;
    define(b, &format!("{NAME}.Quotient"), &[7], 6, body)?;
    let pair = b.var(0)?;
    let mut nonzero = zero;
    for i in 0..64 {
        let bit = core_read(b, pair, i, 7)?;
        nonzero = call(b, "Std.Bool.or", vec![nonzero, bit])?;
    }
    define(b, &format!("{NAME}.MoreDigits"), &[7], 0, nonzero)?;
    let pair = b.var(4)?;
    let one = truth(b, true)?;
    let mut bits = vec![];
    for i in 0..16 {
        bits.push(if i < 4 {
            core_read(b, pair, 64 + i, 7)?
        } else if i == 4 || i == 5 {
            one
        } else {
            zero
        });
    }
    let body = core_select(b, &bits, 4, 0, 4, zero)?;
    let body = b.wrap_selectors(4, body)?;
    define(b, &format!("{NAME}.Digit"), &[7], 4, body)?;
    let mut c = Circuit::new(&[32, 1]);
    let count = c.inputs[0].clone();
    let sign = c.inputs[1][0];
    let (output, _) = c.add(&count, &vec![F; 32], sign);
    let length = circuit(b, &format!("{NAME}.Length"), c, output)?;
    let mut c = Circuit::new(&[32, 32]);
    let len = c.inputs[0].clone();
    let index = c.inputs[1].clone();
    let one = std::iter::once(T)
        .chain(std::iter::repeat_n(F, 31))
        .collect::<Vec<_>>();
    let (remaining, _) = c.sub(&len, &one);
    let (reverse, _) = c.sub(&remaining, &index);
    let visible = c.lt(&index, &len, false);
    let mut output = reverse;
    output.push(visible);
    let position = circuit(b, &format!("{NAME}.Position"), c, output)?;
    if !b.globals.contains_key(&format!("{PREFIX}.Cube.D5.Mux")) {
        b.helpers(5)?;
    }
    Ok((divide, length, position))
}
fn formatter(
    b: &mut Builder,
    token: &str,
    width: u32,
    signed: bool,
    divide: &str,
    length_fn: &str,
    position_fn: &str,
) -> R<OrdinaryIntegerFormatDefinition> {
    let depth = address_bits(width);
    let stem = format!("{NAME}.{token}");
    let value_type_id = format!("mpk.csharp.value.{token}.v1");
    let codec_id = match token {
        "duration" => "duration_ticks".into(),
        "instant" => "unix_milliseconds".into(),
        _ => format!("integer.{token}"),
    };
    BoundaryCodec::new(&codec_id, &value_type_id, None, None)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    let mut c = Circuit::new(&[width as usize]);
    let input = c.inputs[0].clone();
    let magnitude = if signed {
        let negative = c.neg(&input);
        c.select(input[width as usize - 1], &negative, &input)
    } else {
        input
    };
    let magnitude = Circuit::extend(&magnitude, 64, false);
    let magnitude = circuit(b, &format!("{stem}.Magnitude"), c, magnitude)?;
    let input = b.var(0)?;
    let negative = if signed {
        core_read(b, input, width as usize - 1, depth)?
    } else {
        truth(b, false)?
    };
    define(b, &format!("{stem}.Negative"), &[depth], 0, negative)?;
    let mut lets = vec![];
    let pair_ty = b.cube(7)?;
    let quotient = format!("{NAME}.Quotient");
    for i in 0..20 {
        let previous = b.var(0)?;
        let q = call(
            b,
            if i == 0 { &magnitude } else { &quotient },
            vec![previous],
        )?;
        let pair = call(b, divide, vec![q])?;
        lets.push((pair_ty, pair));
    }
    // Keep all twenty quotient/remainder computations outside the output's
    // selectors. Every output bit shares the same concrete arithmetic state.
    let zero = truth(b, false)?;
    let mut chars = vec![];
    for i in 0..20 {
        let pair = b.var(9 + 19 - i)?;
        let ch = call(b, &format!("{NAME}.Digit"), vec![pair])?;
        let selectors = (0..4).rev().map(|i| b.var(i)).collect::<R<Vec<_>>>()?;
        chars.push(b.app(ch, selectors)?);
    }
    let digits = core_select(b, &chars, 9, 0, 5, zero)?;
    let digits = b.wrap_selectors(9, digits)?;
    lets.push((b.cube(9)?, digits));
    let source = b.var(21)?;
    let sign = call(b, &format!("{stem}.Negative"), vec![source])?;
    lets.push((b.boolean, sign));
    let mut count = word(b, 1, 5)?;
    for i in 0..19 {
        let pair = b.var(21 - i)?;
        let more = call(b, &format!("{NAME}.MoreDigits"), vec![pair])?;
        let next = word(b, i + 2, 5)?;
        count = call(b, &format!("{PREFIX}.Cube.D5.Mux"), vec![more, next, count])?;
    }
    lets.push((b.cube(5)?, count));
    let count = b.var(0)?;
    let sign = b.var(1)?;
    let len = call(b, length_fn, vec![count, sign])?;
    lets.push((b.cube(5)?, len));

    // Under 19 text selectors: length=19, count=20, sign=21, digit cube=22.
    let len = b.var(19)?;
    let sign = b.var(21)?;
    let digits = b.var(22)?;
    let mut index_bits = vec![];
    for i in 0..32 {
        index_bits.push(if i < 14 { b.var(23 - 1 - i)? } else { zero });
    }
    let index = core_select(b, &index_bits, 5, 0, 5, zero)?;
    let index = b.wrap_selectors(5, index)?;
    let position = call(b, position_fn, vec![len, index])?;
    let visible = core_read(b, position, 32, 6)?;
    let mut digit_args = vec![];
    for i in 0..5 {
        digit_args.push(core_read(b, position, i, 6)?);
    }
    for i in 15..19 {
        digit_args.push(b.var(18 - i)?);
    }
    let digit = b.app(digits, digit_args)?;
    let mut first = truth(b, true)?;
    for i in 1..15 {
        let selector = b.var(18 - i)?;
        let not = call(b, "Std.Bool.not", vec![selector])?;
        first = call(b, "Std.Bool.and", vec![first, not])?;
    }
    let minus = call(b, "Std.Bool.and", vec![first, sign])?;
    let mut minus_bits = vec![];
    for i in 0..16 {
        minus_bits.push(truth(b, (b'-' as u16) & (1 << i) != 0)?);
    }
    let minus_char = core_select(b, &minus_bits, 19, 15, 19, zero)?;
    let data = core_mux(b, minus, minus_char, digit)?;
    let data = core_mux(b, visible, data, zero)?;
    let selectors = (0..5).rev().map(|i| b.var(i)).collect::<R<Vec<_>>>()?;
    let mut header = b.app(len, selectors)?;
    for i in 1..14 {
        let selector = b.var(18 - i)?;
        header = core_mux(b, selector, zero, header)?;
    }
    let role = b.var(18)?;
    let body = core_mux(b, role, data, header)?;
    let mut body = b.wrap_selectors(19, body)?;
    for (ty, value) in lets.into_iter().rev() {
        body = b.term(TermNode::Let { ty, value, body })?;
    }
    let format_definition = format!("{stem}.Format");
    define(b, &format_definition, &[depth], 19, body)?;
    Ok(OrdinaryIntegerFormatDefinition {
        codec_id,
        value_type_id,
        value_depth: depth,
        signed,
        format_definition,
    })
}
pub(super) fn emit_formats(
    layouts: &OrdinaryCarrierProgram,
    b: &mut Builder,
) -> R<Vec<OrdinaryIntegerFormatDefinition>> {
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
            let (divide, length, position) = shared.as_ref().unwrap();
            definitions.push(formatter(
                b, token, width, signed, divide, length, position,
            )?);
        }
    }
    Ok(definitions)
}
pub fn generate_csharp_practical_ordinary_integer_formats(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryIntegerFormatProgram> {
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut b = Builder::new()?;
    let definitions = emit_formats(&layouts, &mut b)?;
    let certificate = b.finish()?;
    let p = OrdinaryIntegerFormatProgram {
        schema: "mpk.csharp.ordinary_integer_formats.v1".into(),
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
pub fn import_csharp_practical_ordinary_integer_formats(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryIntegerFormatProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_integer_formats(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}

#[cfg(test)]
mod definition_tests {
    use super::*;
    #[test]
    fn ordinary_definition_preserves_narrow_bytes_and_wide_cube_arguments() {
        let mut current = Builder::new().unwrap();
        let mut legacy = Builder::new().unwrap();
        for depth in 0..32 {
            let name = format!("Test.Identity.D{depth}");
            let body = current.var(1).unwrap();
            define(&mut current, &name, &[depth, 0], depth, body).unwrap();
            let body = legacy.var(1).unwrap();
            let widths = [1usize << depth, 1];
            let body = bind_inputs(&mut legacy, &widths, body).unwrap();
            let output = legacy.cube(depth).unwrap();
            let ty = input_type(&mut legacy, &widths, output).unwrap();
            legacy.define(&name, ty, body).unwrap();
        }
        assert_eq!(current.finish().unwrap(), legacy.finish().unwrap());
        let mut b = Builder::new().unwrap();
        for depth in [32, 33, 64, 128, 253] {
            let body = b.var(0).unwrap();
            define(
                &mut b,
                &format!("Test.WideIdentity.D{depth}"),
                &[depth],
                depth,
                body,
            )
            .unwrap();
            let cube = b.cube(depth).unwrap();
            let expected = b.pi(cube, cube).unwrap();
            let DeclarationKind::Def { ty, .. } = b.c.declarations.last().unwrap().kind else {
                panic!()
            };
            assert_eq!(ty, expected, "C{depth} must remain the input type");
        }
        let bytes = b.finish().unwrap();
        let cert = decode_canonical_certificate(&bytes).unwrap();
        crate::csharp_practical_vc_model::validate_csharp_practical_certificate_structure(&cert)
            .unwrap();
    }
}
