//! Fixed Gregorian date and seven-fractional-digit time codecs in ordinary core.
use super::hex_codecs::{call, truth, word};
use super::integer_format::{circuit, define};
use super::temporal::literal;
use super::*;

const NAME: &str = "Mpk.CSharp.Ordinary.CalendarCodec";
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryCalendarCodecDefinition {
    pub codec_id: String,
    pub value_type_id: String,
    pub value_depth: u32,
    pub text_length: u32,
    pub parse_result_shape: OrdinaryShape,
    pub parse_definition: String,
    pub format_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryCalendarCodecProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryCalendarCodecDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryCalendarCodecProgram {
    pub fn definitions(&self) -> &[OrdinaryCalendarCodecDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary calendar codecs")
    }
}
fn and(b: &mut Builder, a: u32, c: u32) -> R<u32> {
    let no = truth(b, false)?;
    core_mux(b, a, c, no)
}
fn invoke(b: &mut Builder, suffix: &str, args: Vec<u32>) -> R<u32> {
    call(b, &format!("{NAME}.{suffix}"), args)
}
fn character(raw: &[Bit], index: usize) -> Word {
    (0..16).map(|i| raw[index | (i << 4)]).collect()
}
fn field(c: &mut Circuit, raw: &[Bit], start: usize, digits: usize) -> Word {
    let mut value = literal(0, 32);
    for i in start..start + digits {
        let ch = character(raw, i);
        let shifted = c.multiply(&value, &literal(10, 32), 32);
        value = c.add(&shifted, &Circuit::extend(&ch[..4], 32, false), F).0;
    }
    value
}
fn put_character(output: &mut [Bit], index: usize, character: &[Bit]) {
    for (i, bit) in character.iter().enumerate() {
        output[index | (i << 4)] = *bit;
    }
}
// The remainder is always below the constant divisor. Store only its
// divisor-width bits; the extra bit belongs to the shifted trial dividend.
// Before enough input bits have arrived to reach the divisor, the quotient
// bit is identically zero. These bounds apply to every input bit pattern.
fn divide_for_format(c: &mut Circuit, input: &[Bit], divisor: u128) -> (Word, Word) {
    assert!(divisor > 0 && divisor < (1u128 << 127));
    let width = (128 - divisor.leading_zeros()) as usize;
    let mut remainder = vec![F; width];
    let mut quotient = vec![F; input.len()];
    let divisor_word = literal(divisor, width + 1);
    for (consumed, i) in (0..input.len()).rev().enumerate() {
        let mut trial = vec![input[i]];
        trial.extend(&remainder);
        if consumed + 1 < width {
            remainder.copy_from_slice(&trial[..width]);
        } else {
            let (difference, no_borrow) = c.sub(&trial, &divisor_word);
            quotient[i] = no_borrow;
            remainder = c.select(no_borrow, &difference[..width], &trial[..width]);
        }
    }
    // Preserve the original restoring divider's public word widths.
    remainder.push(F);
    (quotient, remainder)
}
fn decimal_digits(c: &mut Circuit, value: &[Bit], digits: usize) -> Word {
    let mut bcd = vec![F; digits * 4];
    for &bit in value.iter().rev() {
        for digit in bcd.chunks_exact_mut(4) {
            // Each incoming nibble is a decimal digit (0..9). Add three
            // exactly when it is at least five, before the binary shift.
            // The shift preserves this invariant, including discarded high
            // decimal carry: the fixed digits represent input modulo 10^digits.
            let low = c.mux(digit[1], T, digit[0]);
            let five = c.and(digit[2], low);
            let adjust = c.mux(digit[3], T, five);
            let no_units = c.not(digit[0]);
            let carry_one = c.and(adjust, no_units);
            let carry_two = c.and(adjust, low);
            let corrected = [
                c.xor(digit[0], adjust),
                c.xor(digit[1], carry_one),
                c.xor(digit[2], carry_two),
                adjust,
            ];
            digit.copy_from_slice(&corrected);
        }
        bcd.rotate_right(1);
        bcd[0] = bit;
    }
    bcd
}
fn put_digits(c: &mut Circuit, output: &mut [Bit], start: usize, digits: usize, value: Word) {
    let bcd = decimal_digits(c, &value, digits);
    for (digit, index) in (start..start + digits).rev().enumerate() {
        let nibble = &bcd[digit * 4..digit * 4 + 4];
        let ch = c
            .add(&Circuit::extend(nibble, 16, false), &literal(48, 16), F)
            .0;
        put_character(output, index, &ch);
    }
}

fn shared(b: &mut Builder) -> R<()> {
    if b.globals.contains_key(&format!("{NAME}.Capture")) {
        return Ok(());
    }
    if !b.globals.contains_key(&format!("{PREFIX}.Cube.D3.Mux")) {
        b.helpers(3)?;
    }
    // Capture sixteen characters as C8: four index bits, four character bits.
    let text = b.var(8)?;
    let mut args = vec![truth(b, true)?];
    for i in 0..4 {
        args.push(b.var(7 - i)?);
    }
    args.extend(vec![truth(b, false)?; 10]);
    for i in 4..8 {
        args.push(b.var(7 - i)?);
    }
    let body = b.app(text, args)?;
    let body = b.wrap_selectors(8, body)?;
    define(b, &format!("{NAME}.Capture"), &[19], 8, body)?;
    let text = b.var(5)?;
    let mut args = vec![truth(b, false)?; 14];
    args.extend(b.selectors(5)?);
    let body = b.app(text, args)?;
    let body = b.wrap_selectors(5, body)?;
    define(b, &format!("{NAME}.Length"), &[19], 5, body)
}
pub(super) fn codec(b: &mut Builder, token: &str) -> R<OrdinaryCalendarCodecDefinition> {
    shared(b)?;
    let date = token == "date";
    let depth = if date { 5 } else { 6 };
    let length = if date { 10 } else { 16 };
    let value_type_id = format!("mpk.csharp.value.{token}.v1");
    BoundaryCodec::new(token, &value_type_id, None, None)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    let stem = format!("{NAME}.{token}");
    let punctuation = if date {
        vec![(4, b'-'), (7, b'-')]
    } else {
        vec![(2, b':'), (5, b':'), (8, b'.')]
    };
    let mut c = Circuit::new(&[256]);
    let raw = c.inputs[0].clone();
    let mut good = T;
    for i in 0..length {
        let ch = character(&raw, i);
        let valid = if let Some((_, mark)) = punctuation.iter().find(|(index, _)| *index == i) {
            c.equal(&ch, &literal(*mark as u128, 16))
        } else {
            let below = c.lt(&ch, &literal(48, 16), false);
            let above = c.lt(&literal(57, 16), &ch, false);
            let invalid = c.or(below, above);
            c.not(invalid)
        };
        good = c.and(good, valid);
    }
    let syntax = circuit(b, &format!("{stem}.Characters"), c, vec![good])?;
    let mut c = Circuit::new(&[256]);
    let raw = c.inputs[0].clone();
    let groups = if date {
        vec![(0, 4), (5, 2), (8, 2)]
    } else {
        vec![(0, 2), (3, 2), (6, 2), (9, 7)]
    };
    let mut parts = vec![];
    for (start, digits) in groups {
        parts.extend(field(&mut c, &raw, start, digits));
    }
    parts.resize(128, F);
    let parts = circuit(b, &format!("{stem}.Parts"), c, parts)?;
    let mut c = Circuit::new(&[128]);
    let parts_input = c.inputs[0].clone();
    let a = &parts_input[..32];
    let m = &parts_input[32..64];
    let d = &parts_input[64..96];
    let in_range = if date {
        let year_zero = c.equal(a, &literal(0, 32));
        let year_excess = c.lt(&literal(9999, 32), a, false);
        let month_zero = c.equal(m, &literal(0, 32));
        let month_excess = c.lt(&literal(12, 32), m, false);
        let day_zero = c.equal(d, &literal(0, 32));
        let month_days = super::calendar::month_length(&mut c, a, m);
        let day_excess = c.lt(&month_days, d, false);
        let mut bad = F;
        for flag in [
            year_zero,
            year_excess,
            month_zero,
            month_excess,
            day_zero,
            day_excess,
        ] {
            bad = c.or(bad, flag);
        }
        c.not(bad)
    } else {
        let hour_bad = c.lt(&literal(23, 32), a, false);
        let minute_bad = c.lt(&literal(59, 32), m, false);
        let second_bad = c.lt(&literal(59, 32), d, false);
        let bad = c.or(hour_bad, minute_bad);
        let bad = c.or(bad, second_bad);
        c.not(bad)
    };
    let range = circuit(b, &format!("{stem}.Range"), c, vec![in_range])?;
    let mut c = Circuit::new(&[128]);
    let input = c.inputs[0].clone();
    let value = if date {
        super::calendar::date_number(&mut c, &input[..32], &input[32..64], &input[64..96])
    } else {
        let mut ticks = literal(0, 64);
        for (offset, multiplier) in [
            (0, 36_000_000_000u128),
            (32, 600_000_000),
            (64, 10_000_000),
            (96, 1),
        ] {
            let value = Circuit::extend(&input[offset..offset + 32], 64, false);
            let value = c.multiply(&value, &literal(multiplier, 64), 64);
            ticks = c.add(&ticks, &value, F).0;
        }
        ticks
    };
    let value_fn = circuit(b, &format!("{stem}.Value"), c, value)?;
    let mut c = Circuit::new(&[32]);
    let len = c.inputs[0].clone();
    let same = c.equal(&len, &literal(length as u128, 32));
    let same_length = circuit(b, &format!("{stem}.SameLength"), c, vec![same])?;
    let mut c = Circuit::new(&[32]);
    let len = c.inputs[0].clone();
    let bounded = c.lt(&len, &literal(16385, 32), false);
    let bounded = circuit(b, &format!("{stem}.Bounded"), c, vec![bounded])?;
    let text = b.var(2)?;
    let captured = b.var(1)?;
    let parts_value = b.var(0)?;
    let in_range = call(b, &range, vec![parts_value])?;
    let success = word(b, 0, 3)?;
    let range_error = word(b, 5, 3)?;
    let mux = format!("{PREFIX}.Cube.D3.Mux");
    let outcome = call(b, &mux, vec![in_range, success, range_error])?;
    let len = invoke(b, "Length", vec![text])?;
    let same = call(b, &same_length, vec![len])?;
    let syntax = call(b, &syntax, vec![captured])?;
    let syntax = and(b, same, syntax)?;
    let syntax_error = word(b, 2, 3)?;
    let outcome = call(b, &mux, vec![syntax, outcome, syntax_error])?;
    let bounded = call(b, &bounded, vec![len])?;
    let bound_error = word(b, 1, 3)?;
    let outcome = call(b, &mux, vec![bounded, outcome, bound_error])?;
    define(b, &format!("{stem}.Decision"), &[19, 8, 7], 3, outcome)?;
    let mut c = Circuit::new(&[8]);
    let code = c.inputs[0].clone();
    let failed = c.nonzero(&code[..3]);
    let failed = circuit(b, &format!("{stem}.Failed"), c, vec![failed])?;
    let mut c = Circuit::new(&[8]);
    let code = c.inputs[0].clone();
    let error = c.sub(&code[..3], &literal(1, 3)).0;
    let error = circuit(
        b,
        &format!("{stem}.ErrorCode"),
        c,
        Circuit::extend(&error, 32, false),
    )?;
    let result_depth = depth + 1;
    let code = b.var(result_depth + 1)?;
    let value = b.var(result_depth)?;
    let failed = call(b, &failed, vec![code])?;
    let zero = truth(b, false)?;
    let mut header = failed;
    for i in 1..result_depth {
        let selector = b.var(result_depth - 1 - i)?;
        header = core_mux(b, selector, zero, header)?;
    }
    let args = (0..depth).rev().map(|i| b.var(i)).collect::<R<Vec<_>>>()?;
    let value = b.app(value, args)?;
    let error = call(b, &error, vec![code])?;
    let args = (0..5).rev().map(|i| b.var(i)).collect::<R<Vec<_>>>()?;
    let mut error = b.app(error, args)?;
    for i in 1..result_depth - 5 {
        let selector = b.var(result_depth - 1 - i)?;
        error = core_mux(b, selector, zero, error)?;
    }
    let payload = core_mux(b, failed, error, value)?;
    let role = b.var(result_depth - 1)?;
    let body = core_mux(b, role, payload, header)?;
    let body = b.wrap_selectors(result_depth, body)?;
    define(b, &format!("{stem}.Pack"), &[3, depth], result_depth, body)?;
    let text = b.var(0)?;
    let captured = invoke(b, "Capture", vec![text])?;
    let mut lets = vec![(b.cube(8)?, captured)];
    let captured = b.var(0)?;
    let part_values = call(b, &parts, vec![captured])?;
    lets.push((b.cube(7)?, part_values));
    let text = b.var(2)?;
    let captured = b.var(1)?;
    let part_values = b.var(0)?;
    let code = call(
        b,
        &format!("{stem}.Decision"),
        vec![text, captured, part_values],
    )?;
    lets.push((b.cube(3)?, code));
    let parts_value = b.var(1)?;
    let code = b.var(0)?;
    let value = call(b, &value_fn, vec![parts_value])?;
    let mut body = call(b, &format!("{stem}.Pack"), vec![code, value])?;
    for (ty, value) in lets.into_iter().rev() {
        body = b.term(TermNode::Let { ty, value, body })?;
    }
    let parse_definition = format!("{stem}.Parse");
    define(b, &parse_definition, &[19], result_depth, body)?;

    let mut c = Circuit::new(&[1 << depth]);
    let input = c.inputs[0].clone();
    let mut output = vec![F; 256];
    for (index, mark) in punctuation {
        put_character(&mut output, index, &literal(mark as u128, 16));
    }
    if date {
        let (year, month, day) = super::calendar::date_parts(&mut c, &input);
        put_digits(&mut c, &mut output, 0, 4, year);
        put_digits(&mut c, &mut output, 5, 2, month);
        put_digits(&mut c, &mut output, 8, 2, day);
    } else {
        let (seconds, fractional) = divide_for_format(&mut c, &input, 10_000_000);
        let (minutes, seconds) = divide_for_format(&mut c, &seconds, 60);
        let (hours, minutes) = divide_for_format(&mut c, &minutes, 60);
        put_digits(&mut c, &mut output, 0, 2, hours);
        put_digits(&mut c, &mut output, 3, 2, minutes);
        put_digits(&mut c, &mut output, 6, 2, seconds);
        put_digits(&mut c, &mut output, 9, 7, fractional);
    }
    // Emit 128 actual Boolean gates per transformer; no transformer calls
    // are hidden inside a wrapper and every emitted block is charged.
    let characters = super::integer_format::circuit_with_block_bits(
        b,
        &format!("{stem}.FormatCharacters"),
        c,
        output,
        7,
    )?;
    let source = b.var(0)?;
    let characters = call(b, &characters, vec![source])?;
    // Bind the entire small character cube before any text output selectors.
    let chars = b.var(19)?;
    let mut args = (1..5).map(|i| b.var(18 - i)).collect::<R<Vec<_>>>()?;
    args.extend((15..19).map(|i| b.var(18 - i)).collect::<R<Vec<_>>>()?);
    let mut data = b.app(chars, args)?;
    for i in 5..15 {
        let selector = b.var(18 - i)?;
        data = core_mux(b, selector, zero, data)?;
    }
    let header_bits = (0..32)
        .map(|i| truth(b, length & (1usize << i) != 0))
        .collect::<R<Vec<_>>>()?;
    let mut header = core_select(b, &header_bits, 19, 14, 19, zero)?;
    for i in 1..14 {
        let selector = b.var(18 - i)?;
        header = core_mux(b, selector, zero, header)?;
    }
    let role = b.var(18)?;
    let body = core_mux(b, role, data, header)?;
    let body = b.wrap_selectors(19, body)?;
    let chars_ty = b.cube(8)?;
    let body = b.term(TermNode::Let {
        ty: chars_ty,
        value: characters,
        body,
    })?;
    let format_definition = format!("{stem}.Format");
    define(b, &format_definition, &[depth], 19, body)?;
    let arms = [
        (0, "ok", value_type_id.as_str()),
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
    Ok(OrdinaryCalendarCodecDefinition {
        codec_id: token.into(),
        value_type_id,
        value_depth: depth,
        text_length: length as u32,
        parse_result_shape: OrdinaryShape::Sum { arms },
        parse_definition,
        format_definition,
    })
}
pub(super) fn emit_codecs(
    layouts: &OrdinaryCarrierProgram,
    b: &mut Builder,
) -> R<Vec<OrdinaryCalendarCodecDefinition>> {
    let mut definitions = vec![];
    for (token, width) in [("date", 32), ("time", 64)] {
        if let Some(c) = layouts
            .carriers()
            .iter()
            .find(|c| c.type_id == format!("mpk.csharp.value.{token}.v1"))
        {
            if c.depth != address_bits(width) || c.shape != (OrdinaryShape::Bits { width }) {
                return Err(OrdinaryCarrierError::Shape);
            }
            definitions.push(codec(b, token)?);
        }
    }
    Ok(definitions)
}
pub fn generate_csharp_practical_ordinary_calendar_codecs(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryCalendarCodecProgram> {
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut b = Builder::new()?;
    let definitions = emit_codecs(&layouts, &mut b)?;
    let certificate = b.finish()?;
    let p = OrdinaryCalendarCodecProgram {
        schema: "mpk.csharp.ordinary_calendar_codecs.v1".into(),
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
pub fn import_csharp_practical_ordinary_calendar_codecs(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryCalendarCodecProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_calendar_codecs(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn calendar_decimal_digit_circuit_costs() {
        for (width, digits) in [(12, 4), (32, 4), (32, 2), (64, 2), (24, 7)] {
            let mut bcd = Circuit::new(&[width]);
            let input = bcd.inputs[0].clone();
            let mut bcd_out = decimal_digits(&mut bcd, &input, digits);
            bcd.prune(&mut bcd_out);
            let mut restoring = Circuit::new(&[width]);
            let mut value = restoring.inputs[0].clone();
            let mut old_out = vec![];
            for _ in 0..digits {
                let (q, r) = super::super::temporal::divide_constant(&mut restoring, &value, 10);
                old_out.extend(&r[..4]);
                value = q;
            }
            restoring.prune(&mut old_out);
            let max = (1u128 << width) - 1;
            let mut values = BTreeSet::from([0, 1, 9, 10, 99, 100, max, max - 1]);
            if width == 12 {
                values.extend(0..=max);
            }
            for bit in 0..width {
                let value = 1u128 << bit;
                values.extend([value - 1, value, (value + 1).min(max)]);
            }
            for value in values {
                let a = bcd.evaluate(&[value]);
                let b = restoring.evaluate(&[value]);
                for (i, (&x, &y)) in bcd_out.iter().zip(&old_out).enumerate() {
                    assert_eq!(a[x], b[y], "{width}/{digits}: {value} bit {i}");
                    let digit = value / 10u128.pow((i / 4) as u32) % 10;
                    assert_eq!(a[x], digit & (1 << (i % 4)) != 0);
                }
            }
            eprintln!(
                "format digits width {width} count {digits}: restoring {}, double dabble {} gates",
                restoring.gates.len(),
                bcd.gates.len()
            );
        }
    }
    #[test]
    fn calendar_format_divider_preserves_full_width_arithmetic() {
        let mut cases = 0;
        for width in [12, 32, 64] {
            for divisor in [10, 60, 10_000_000] {
                let mut old = Circuit::new(&[width]);
                let input = old.inputs[0].clone();
                let (old_q, old_r) =
                    super::super::temporal::divide_constant(&mut old, &input, divisor);
                let mut new = Circuit::new(&[width]);
                let input = new.inputs[0].clone();
                let (new_q, new_r) = divide_for_format(&mut new, &input, divisor);
                assert_eq!((old_q.len(), old_r.len()), (new_q.len(), new_r.len()));
                let max = (1u128 << width) - 1;
                let mut inputs = BTreeSet::from([0, 1, max, max - 1]);
                if width == 12 {
                    inputs.extend(0..=max);
                }
                for bit in 0..width {
                    let value = 1u128 << bit;
                    inputs.extend([value - 1, value, (value + 1).min(max)]);
                }
                for quotient in [0, 1, 2, 9, 10, max / divisor] {
                    let value = quotient * divisor;
                    for n in [value.saturating_sub(1), value, value + 1] {
                        if n <= max {
                            inputs.insert(n);
                        }
                    }
                }
                let mut seed = 0x51f4_207du64;
                for _ in 0..128 {
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                    inputs.insert(u128::from(seed) & max);
                }
                let read = |values: &[bool], word: &[Bit]| {
                    word.iter()
                        .enumerate()
                        .fold(0u128, |n, (i, &b)| n | (u128::from(values[b]) << i))
                };
                for input in inputs {
                    let a = old.evaluate(&[input]);
                    let b = new.evaluate(&[input]);
                    let expected = (input / divisor, input % divisor);
                    assert_eq!((read(&a, &old_q), read(&a, &old_r)), expected);
                    assert_eq!(
                        (read(&b, &new_q), read(&b, &new_r)),
                        expected,
                        "{width}-bit {input}/{divisor}"
                    );
                    cases += 1;
                }
                eprintln!(
                    "calendar divider {width}/{divisor}: {} -> {} gates",
                    old.gates.len(),
                    new.gates.len()
                );
            }
        }
        eprintln!("calendar divider full-width arithmetic: {cases} cases");
    }

    #[test]
    fn calendar_json_cumulative_environment_limits() {
        let mut metrics = vec![];
        for tokens in [vec!["date"], vec!["time"], vec!["date", "time"]] {
            let mut b = Builder::new().unwrap();
            super::super::emit_boundary_json(&mut b).unwrap();
            for token in &tokens {
                codec(&mut b, token).unwrap();
            }
            let format_blocks = tokens
                .iter()
                .map(|token| {
                    let id = format!("{NAME}.{token}.FormatCharacters");
                    let encoded = id
                        .as_bytes()
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<String>();
                    let prefix = format!("{PREFIX}.IntegerText.O{encoded}.Block.B");
                    (
                        token,
                        b.globals
                            .keys()
                            .filter(|name| name.starts_with(&prefix))
                            .count(),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            let static_transformers = b.static_transformers;
            let bytes = b.finish().unwrap();
            let cert = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
            crate::csharp_practical_vc_model::validate_csharp_practical_certificate_structure(
                &cert,
            )
            .unwrap();
            metrics.push(serde_json::json!({
                "tokens": tokens,
                "terms": cert.term_table.len(),
                "declarations": cert.declarations.len(),
                "static_transformers": static_transformers,
                "format_gate_blocks": format_blocks,
                "certificate_sha256": mpk_cert::hash_hex(&mpk_cert::certificate_hash(&bytes)),
            }));
        }
        eprintln!("{}", serde_json::to_string_pretty(&metrics).unwrap());
    }
}
