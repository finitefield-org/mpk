//! Canonical quoted JSON string decoding into the admitted UTF-16 carrier.
//! Whole-document and prefix results are private ordinary relations.
//! A prefix success reports its exact consumed byte count; trailing tokens remain unchecked.
use super::boundary_document::emit as emit_document;
use super::hex_codecs::{call, truth, word};
use super::integer_format::{circuit, define};
use super::temporal::literal;
use super::*;
const NAME: &str = "Mpk.CSharp.Ordinary.JsonStringParse";
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonStringParseDefinition {
    pub document: OrdinaryBoundaryDocumentDefinition,
    pub fragments: OrdinaryBoundaryFragmentDefinition,
    pub state_depth: u32,
    pub value_depth: u32,
    /// Logical two-packet steps; adjacent pairs share an ordinary StepFour.
    pub scan_steps: u32,
    pub static_transformers: u32,
    pub valid_definition: String,
    pub value_definition: String,
    pub prefix_valid_definition: String,
    pub prefix_value_definition: String,
    pub consumed_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonStringParseProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    boundary_program_sha256: String,
    definition: Option<OrdinaryJsonStringParseDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryJsonStringParseProgram {
    pub fn definition(&self) -> Option<&OrdinaryJsonStringParseDefinition> {
        self.definition.as_ref()
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary JSON string parse")
    }
}
fn name(s: &str) -> String {
    format!("{NAME}.{s}")
}
fn invoke(b: &mut Builder, s: &str, a: Vec<u32>) -> R<u32> {
    call(b, &name(s), a)
}
fn range(c: &mut Circuit, x: &[Bit], lo: u128, hi: u128) -> Bit {
    let a = c.lt(x, &literal(lo, x.len()), false);
    let z = c.lt(&literal(hi, x.len()), x, false);
    let bad = c.or(a, z);
    c.not(bad)
}
fn all(c: &mut Circuit, bits: &[Bit]) -> Bit {
    bits.iter().fold(T, |a, &x| c.and(a, x))
}
// Packet: consumed bytes 0..3, output units 3..5, valid 5, escaped-high 6,
// closing quote 7, two UTF-16 units 8..40, escaped-low 40; other bits zero.
fn make_packet(consumed: u128, units: &[Word], high: Bit, low: Bit, close: Bit) -> Word {
    let mut p = literal(consumed, 3);
    p.extend(literal(units.len() as u128, 2));
    p.extend([T, high, close]);
    for unit in units {
        p.extend(unit);
    }
    p.resize(40, F);
    p.push(low);
    p.resize(64, F);
    p
}
fn emit_packet(b: &mut Builder) -> R<String> {
    let mut c = Circuit::new(&[64, 32]);
    let input = c.inputs[0].clone();
    let remaining = c.inputs[1].clone();
    let bytes = (0..6)
        .map(|i| input[i * 8..(i + 1) * 8].to_vec())
        .collect::<Vec<_>>();
    let first = &bytes[0];
    let mut out = literal(0, 64);
    let mut enough = vec![];
    for n in 1..=6 {
        let less = c.lt(&remaining, &literal(n, 32), false);
        enough.push(c.not(less));
    }
    let quote = c.equal(first, &literal(34, 8));
    let slash = c.equal(first, &literal(92, 8));
    let regular = range(&mut c, first, 32, 127);
    let special = c.or(quote, slash);
    let ordinary = c.not(special);
    let valid = all(&mut c, &[regular, ordinary, enough[0]]);
    let mut unit = first.clone();
    unit.resize(16, F);
    out = c.select(valid, &make_packet(1, &[unit], F, F, F), &out);
    // A closing quote ends the token. The whole-document wrapper separately
    // requires the final cursor to equal the full document length.
    let close = c.and(quote, enough[0]);
    out = c.select(close, &make_packet(1, &[], F, F, T), &out);
    let escaped_quote = c.equal(&bytes[1], &literal(34, 8));
    let escaped_slash = c.equal(&bytes[1], &literal(92, 8));
    let escaped = c.or(escaped_quote, escaped_slash);
    let valid = all(&mut c, &[slash, escaped, enough[1]]);
    let mut unit = bytes[1].clone();
    unit.resize(16, F);
    out = c.select(valid, &make_packet(2, &[unit], F, F, F), &out);
    let u = c.equal(&bytes[1], &literal(117, 8));
    let mut hex_valid = T;
    let mut nibbles = vec![];
    for byte in &bytes[2..6] {
        let digit = range(&mut c, byte, 48, 57);
        let hex = range(&mut c, byte, 97, 102);
        let good = c.or(digit, hex);
        hex_valid = c.and(hex_valid, good);
        let decimal = c.sub(byte, &literal(48, 8)).0;
        let alpha = c.sub(byte, &literal(87, 8)).0;
        nibbles.push(c.select(digit, &decimal[..4], &alpha[..4]));
    }
    let unit = nibbles.into_iter().rev().flatten().collect::<Word>();
    let control = c.lt(&unit, &literal(32, 16), false);
    let high = range(&mut c, &unit, 0xd800, 0xdbff);
    let low = range(&mut c, &unit, 0xdc00, 0xdfff);
    let surrogate = c.or(high, low);
    let allowed = c.or(control, surrogate);
    let valid = all(&mut c, &[slash, u, hex_valid, allowed, enough[5]]);
    out = c.select(valid, &make_packet(6, &[unit], high, low, F), &out);
    let cont = (1..4)
        .map(|i| range(&mut c, &bytes[i], 0x80, 0xbf))
        .collect::<Vec<_>>();
    let lead = range(&mut c, first, 0xc2, 0xdf);
    let valid = all(&mut c, &[lead, cont[0], enough[1]]);
    let mut unit = bytes[1][..6].to_vec();
    unit.extend(&first[..5]);
    unit.resize(16, F);
    out = c.select(valid, &make_packet(2, &[unit], F, F, F), &out);
    let lead = range(&mut c, first, 0xe0, 0xef);
    let e0 = c.equal(first, &literal(0xe0, 8));
    let ed = c.equal(first, &literal(0xed, 8));
    let low = c.select(e0, &literal(0xa0, 8), &literal(0x80, 8));
    let high = c.select(ed, &literal(0x9f, 8), &literal(0xbf, 8));
    let below = c.lt(&bytes[1], &low, false);
    let above = c.lt(&high, &bytes[1], false);
    let bad = c.or(below, above);
    let second = c.not(bad);
    let valid = all(&mut c, &[lead, second, cont[1], enough[2]]);
    let mut unit = bytes[2][..6].to_vec();
    unit.extend(&bytes[1][..6]);
    unit.extend(&first[..4]);
    out = c.select(valid, &make_packet(3, &[unit], F, F, F), &out);
    let lead = range(&mut c, first, 0xf0, 0xf4);
    let f0 = c.equal(first, &literal(0xf0, 8));
    let f4 = c.equal(first, &literal(0xf4, 8));
    let low = c.select(f0, &literal(0x90, 8), &literal(0x80, 8));
    let high = c.select(f4, &literal(0x8f, 8), &literal(0xbf, 8));
    let below = c.lt(&bytes[1], &low, false);
    let above = c.lt(&high, &bytes[1], false);
    let bad = c.or(below, above);
    let second = c.not(bad);
    let valid = all(&mut c, &[lead, second, cont[1], cont[2], enough[3]]);
    let mut scalar = bytes[3][..6].to_vec();
    scalar.extend(&bytes[2][..6]);
    scalar.extend(&bytes[1][..6]);
    scalar.extend(&first[..3]);
    let offset = c.sub(&scalar, &literal(0x10000, 21)).0;
    let mut low = offset[..10].to_vec();
    low.extend(literal(0xdc00 >> 10, 6));
    let mut high = offset[10..20].to_vec();
    high.extend(literal(0xd800 >> 10, 6));
    out = c.select(valid, &make_packet(4, &[high, low], F, F, F), &out);
    circuit(b, &name("Packet"), c, out)
}
pub(super) fn emit(
    b: &mut Builder,
    fragments: OrdinaryBoundaryFragmentDefinition,
) -> R<OrdinaryJsonStringParseDefinition> {
    let document = fragments.document.clone();
    if !b
        .globals
        .contains_key(&format!("{PREFIX}.Cube.D19.Compose"))
    {
        b.helpers(19)?;
    }
    let mut d = OrdinaryJsonStringParseDefinition {
        document,
        fragments,
        state_depth: 19,
        value_depth: 19,
        scan_steps: 8193,
        static_transformers: 0,
        valid_definition: name("Valid"),
        value_definition: name("Value"),
        prefix_valid_definition: name("PrefixValid"),
        prefix_value_definition: name("PrefixValue"),
        consumed_definition: name("Consumed"),
    };
    let packet_name = emit_packet(b)?;
    // State C19: role-zero C7 header (cursor, decoded length, escaped-high,
    // failed, done); role-one C18 UTF-16 storage. Header padding is zero.
    let state = b.var(7)?;
    let zero = truth(b, false)?;
    let mut args = vec![zero; 12];
    args.extend(b.selectors(7)?);
    let body = b.app(state, args)?;
    let body = b.wrap_selectors(7, body)?;
    define(b, &name("Header"), &[19], 7, body)?;
    let state = b.var(18)?;
    let mut args = vec![truth(b, true)?];
    args.extend(b.selectors(18)?);
    let body = b.app(state, args)?;
    let body = b.wrap_selectors(18, body)?;
    define(b, &name("Units"), &[19], 18, body)?;
    let head = b.var(20)?;
    let units = b.var(19)?;
    let selectors = b.selectors(19)?;
    let zero = truth(b, false)?;
    let data = b.app(units, selectors[1..].to_vec())?;
    let mut header = b.app(head, selectors[12..].to_vec())?;
    for &sel in &selectors[1..12] {
        header = core_mux(b, sel, zero, header)?;
    }
    let body = core_mux(b, selectors[0], data, header)?;
    let body = b.wrap_selectors(19, body)?;
    define(b, &name("MakeState"), &[7, 18], 19, body)?;
    for (suffix, offset) in [("Cursor", 0), ("Length", 32)] {
        let head = b.var(5)?;
        let bits = (0..32)
            .map(|i| core_read(b, head, i + offset, 7))
            .collect::<R<Vec<_>>>()?;
        let zero = truth(b, false)?;
        let body = core_select(b, &bits, 5, 0, 5, zero)?;
        let body = b.wrap_selectors(5, body)?;
        define(b, &name(suffix), &[7], 5, body)?;
    }
    let mut c = Circuit::new(&[128]);
    let head = c.inputs[0].clone();
    let stop = c.or(head[65], head[66]);
    let more = c.not(stop);
    let more_name = circuit(b, &name("MoreCircuit"), c, vec![more])?;
    let state = b.var(0)?;
    let head = invoke(b, "Header", vec![state])?;
    let body = call(b, &more_name, vec![head])?;
    define(b, &name("More"), &[19], 0, body)?;
    let mut c = Circuit::new(&[128, 64]);
    let head = c.inputs[0].clone();
    let packet = c.inputs[1].clone();
    let mut consumed = packet[..3].to_vec();
    consumed.resize(32, F);
    let cursor = c.add(&head[..32], &consumed, F).0;
    let mut count = packet[3..5].to_vec();
    count.resize(32, F);
    let length = c.add(&head[32..64], &count, F).0;
    let bounded = c.lt(&length, &literal(16385, 32), false);
    let escaped_pair = c.and(head[64], packet[40]);
    let canonical = c.not(escaped_pair);
    let good = all(&mut c, &[packet[5], bounded, canonical]);
    let bad = c.not(good);
    let failed = c.or(head[65], bad);
    let mut updated = cursor;
    updated.extend(length);
    updated.push(packet[6]);
    updated.push(failed);
    updated.push(packet[7]);
    updated.resize(128, F);
    let update_name = circuit(b, &name("UpdatedHeader"), c, updated)?;
    // Unit write selects only valid packet output slots and never wraps an
    // overflow index into the physical 16,384-unit storage.
    let mut c = Circuit::new(&[32, 128, 64, 128]);
    let index = c.inputs[0].clone();
    let old = c.inputs[1].clone();
    let packet = c.inputs[2].clone();
    let new = c.inputs[3].clone();
    let below = c.lt(&index, &old[32..64], false);
    let after = c.not(below);
    let diff = c.sub(&index, &old[32..64]).0;
    let mut count = packet[3..5].to_vec();
    count.resize(32, F);
    let within = c.lt(&diff, &count, false);
    let good = c.not(new[65]);
    let write = all(&mut c, &[after, within, good]);
    let first = c.equal(&diff, &literal(0, 32));
    let unit = c.select(first, &packet[8..24], &packet[24..40]);
    let mut output = unit;
    output.push(write);
    output.resize(32, F);
    let write_name = circuit(b, &name("WriteUnit"), c, output)?;
    let units = b.var(21)?;
    let head = b.var(20)?;
    let packet = b.var(19)?;
    let updated = b.var(18)?;
    let zero = truth(b, false)?;
    let bits = (0..14).map(|i| b.var(22 - i)).collect::<R<Vec<_>>>()?;
    let index = core_select(b, &bits, 5, 0, 5, zero)?;
    let index = b.wrap_selectors(5, index)?;
    let write = call(b, &write_name, vec![index, head, packet, updated])?;
    let active = core_read(b, write, 16, 5)?;
    let selectors = b.selectors(18)?;
    let old = b.app(units, selectors.clone())?;
    let mut args = selectors[14..].to_vec();
    args.push(zero);
    let new = b.app(write, args)?;
    let body = core_mux(b, active, new, old)?;
    let body = b.wrap_selectors(18, body)?;
    define(b, &name("Append"), &[18, 7, 6, 7], 18, body)?;
    let state = b.var(1)?;
    let packet = b.var(0)?;
    let head = invoke(b, "Header", vec![state])?;
    let updated = call(b, &update_name, vec![head, packet])?;
    let units = invoke(b, "Units", vec![state])?;
    let units = invoke(b, "Append", vec![units, head, packet, updated])?;
    let body = invoke(b, "MakeState", vec![updated, units])?;
    define(b, &name("Update"), &[19, 6], 19, body)?;
    let mut offsets = vec![];
    for i in 0..6 {
        let mut c = Circuit::new(&[32]);
        let cursor = c.inputs[0].clone();
        let next = c.add(&cursor, &literal(i, 32), F).0;
        offsets.push(circuit(b, &name(&format!("Index{i}")), c, next)?);
    }
    let zero = truth(b, false)?;
    let source = b.var(7)?;
    let cursor = b.var(6)?;
    let mut byte_bits = vec![];
    for offset in &offsets {
        let index = call(b, offset, vec![cursor])?;
        let byte = call(b, &d.document.read_byte_definition, vec![source, index])?;
        for k in 0..8 {
            byte_bits.push(core_read(b, byte, k, 3)?);
        }
    }
    let body = core_select(b, &byte_bits, 6, 0, 6, zero)?;
    let body = b.wrap_selectors(6, body)?;
    define(b, &name("Capture"), &[24, 5], 6, body)?;
    let mut c = Circuit::new(&[32, 32]);
    let len = c.inputs[0].clone();
    let cursor = c.inputs[1].clone();
    let remaining = c.sub(&len, &cursor).0;
    let remaining_name = circuit(b, &name("Remaining"), c, remaining)?;
    let source = b.var(1)?;
    let state = b.var(0)?;
    let head = invoke(b, "Header", vec![state])?;
    let cursor = invoke(b, "Cursor", vec![head])?;
    let capture = invoke(b, "Capture", vec![source, cursor])?;
    let length = call(b, &d.document.length_definition, vec![source])?;
    let remaining = call(b, &remaining_name, vec![length, cursor])?;
    let packet = call(b, &packet_name, vec![capture, remaining])?;
    let next = invoke(b, "Update", vec![state, packet])?;
    let more = invoke(b, "More", vec![state])?;
    let body = call(
        b,
        &format!("{PREFIX}.Cube.D19.Mux"),
        vec![more, next, state],
    )?;
    define(b, &name("Step"), &[24, 19], 19, body)?;
    let state_ty = b.cube(19)?;
    let transformer = b.pi(state_ty, state_ty)?;
    let f = b.var(2)?;
    let s = b.var(0)?;
    let first = b.app(f, vec![s])?;
    let g = b.var(2)?;
    let middle = b.var(0)?;
    let next = b.app(g, vec![middle])?;
    let more = invoke(b, "More", vec![middle])?;
    let body = call(
        b,
        &format!("{PREFIX}.Cube.D19.Mux"),
        vec![more, next, middle],
    )?;
    let body = b.term(TermNode::Let {
        ty: state_ty,
        value: first,
        body,
    })?;
    let body = b.lam(state_ty, body)?;
    let body = b.lam(transformer, body)?;
    let body = b.lam(transformer, body)?;
    let ty = b.pi(transformer, transformer)?;
    let ty = b.pi(transformer, ty)?;
    b.define(&name("Compose"), ty, body)?;
    let invalid = b.var(7)?;
    let zero = truth(b, false)?;
    let one = truth(b, true)?;
    let mut bits = vec![zero; 128];
    bits[0] = one;
    bits[65] = invalid;
    let head = core_select(b, &bits, 7, 0, 7, zero)?;
    let head = b.wrap_selectors(7, head)?;
    let units = b.wrap_selectors(18, zero)?;
    let body = invoke(b, "MakeState", vec![head, units])?;
    define(b, &name("Initial"), &[0], 19, body)?;
    let mut c = Circuit::new(&[32, 8]);
    let len = c.inputs[0].clone();
    let first = c.inputs[1].clone();
    let low = c.lt(&len, &literal(2, 32), false);
    let enough = c.not(low);
    let bounded = c.lt(&len, &literal(1048577, 32), false);
    let quote = c.equal(&first, &literal(34, 8));
    let good = all(&mut c, &[enough, bounded, quote]);
    let bad = c.not(good);
    let initial_bad = circuit(b, &name("InitialBad"), c, vec![bad])?;
    let source = b.var(0)?;
    let len = call(b, &d.document.length_definition, vec![source])?;
    let zero_index = word(b, 0, 5)?;
    let first = call(
        b,
        &d.document.read_byte_definition,
        vec![source, zero_index],
    )?;
    let bad = call(b, &initial_bad, vec![len, first])?;
    let initial = invoke(b, "Initial", vec![bad])?;
    let step = invoke(b, "Step", vec![source])?;
    let compose = b.constant(&name("Compose"))?;
    let pair = b.compose_term(19, compose, &[step, step])?;
    let source_ty = b.cube(24)?;
    let body = b.lam(source_ty, pair)?;
    let ty = b.pi(source_ty, transformer)?;
    b.define(&name("StepTwo"), ty, body)?;
    let pair = invoke(b, "StepTwo", vec![source])?;
    let four = b.compose_term(19, compose, &[pair, pair])?;
    let body = b.lam(source_ty, four)?;
    let ty = b.pi(source_ty, transformer)?;
    b.define(&name("StepFour"), ty, body)?;
    let four = invoke(b, "StepFour", vec![source])?;
    // Exactly 16,386 packets as before: 4,096 groups of four and a final pair.
    // Charge every composition occurrence, including both shared group bodies.
    let mut groups = vec![four; 4096];
    groups.push(pair);
    let pipeline = b.compose_term(19, compose, &groups)?;
    let body = b.app(pipeline, vec![initial])?;
    define(b, &name("Scan"), &[24], 19, body)?;
    let mut c = Circuit::new(&[128]);
    let head = c.inputs[0].clone();
    let no_failure = c.not(head[65]);
    let valid = c.and(no_failure, head[66]);
    let finished = circuit(b, &name("Finished"), c, vec![valid])?;
    let mut c = Circuit::new(&[128, 32]);
    let head = c.inputs[0].clone();
    let document_length = c.inputs[1].clone();
    let no_failure = c.not(head[65]);
    let at_end = c.equal(&head[..32], &document_length);
    let valid = all(&mut c, &[no_failure, head[66], at_end]);
    let whole_finished = circuit(b, &name("WholeFinished"), c, vec![valid])?;
    for (definition, whole) in [
        (&d.valid_definition, true),
        (&d.prefix_valid_definition, false),
    ] {
        let source = b.var(0)?;
        let state = invoke(b, "Scan", vec![source])?;
        let head = invoke(b, "Header", vec![state])?;
        let body = if whole {
            let length = call(b, &d.document.length_definition, vec![source])?;
            call(b, &whole_finished, vec![head, length])?
        } else {
            call(b, &finished, vec![head])?
        };
        define(b, definition, &[24], 0, body)?;
    }
    for (definition, whole) in [
        (&d.value_definition, true),
        (&d.prefix_value_definition, false),
    ] {
        let source = b.var(0)?;
        let scanned = invoke(b, "Scan", vec![source])?;
        // Source, shared scanned state, then the nineteen value selectors.
        let state = b.var(19)?;
        let head = invoke(b, "Header", vec![state])?;
        let length = invoke(b, "Length", vec![head])?;
        let units = invoke(b, "Units", vec![state])?;
        let selectors = b.selectors(19)?;
        let data = b.app(units, selectors[1..].to_vec())?;
        let mut header = b.app(length, selectors[14..].to_vec())?;
        for &sel in &selectors[1..14] {
            header = core_mux(b, sel, zero, header)?;
        }
        let value = core_mux(b, selectors[0], data, header)?;
        let valid = if whole {
            let source = b.var(20)?;
            let document_length = call(b, &d.document.length_definition, vec![source])?;
            call(b, &whole_finished, vec![head, document_length])?
        } else {
            call(b, &finished, vec![head])?
        };
        let body = core_mux(b, valid, value, zero)?;
        let body = b.wrap_selectors(19, body)?;
        let body = b.term(TermNode::Let {
            ty: state_ty,
            value: scanned,
            body,
        })?;
        define(b, definition, &[24], 19, body)?;
    }
    let source = b.var(0)?;
    let scanned = invoke(b, "Scan", vec![source])?;
    let state = b.var(5)?;
    let head = invoke(b, "Header", vec![state])?;
    let cursor = invoke(b, "Cursor", vec![head])?;
    let selectors = b.selectors(5)?;
    let cursor_bit = b.app(cursor, selectors)?;
    let valid = call(b, &finished, vec![head])?;
    let body = core_mux(b, valid, cursor_bit, zero)?;
    let body = b.wrap_selectors(5, body)?;
    let body = b.term(TermNode::Let {
        ty: state_ty,
        value: scanned,
        body,
    })?;
    define(b, &d.consumed_definition, &[24], 5, body)?;
    d.static_transformers = b.static_transformers as u32;
    Ok(d)
}
// Append framing to the shared lexical environment. The standalone prefix
// parser stays byte-identical. Scan is bound once, retaining its complete
// canonical UTF-8/escape/UTF-16 checks and decoded-unit bound.
pub(super) fn emit_framed(
    b: &mut Builder,
    d: &OrdinaryJsonStringParseDefinition,
) -> R<[String; 3]> {
    let mut c = Circuit::new(&[32, 32]);
    let length = c.inputs[0].clone();
    let start = c.inputs[1].clone();
    let remaining = c.sub(&length, &start).0;
    let remaining = circuit(b, &name("Frame.Remaining"), c, remaining)?;
    let mut c = Circuit::new(&[128, 32]);
    let cursor = c.inputs[0][..32].to_vec();
    let start = c.inputs[1].clone();
    let end = c.add(&start, &cursor, F).0;
    let end_definition = circuit(b, &name("Frame.End"), c, end)?;
    let mut c = Circuit::new(&[128, 32, 32, 8, 8]);
    let head = c.inputs[0].clone();
    let start = c.inputs[1].clone();
    let length = c.inputs[2].clone();
    let ending = c.inputs[3].clone();
    let next = c.inputs[4].clone();
    let (end, overflow) = c.add(&start, &head[..32], F);
    let no_overflow = c.not(overflow);
    let bounded = c.lt(&length, &literal(1_048_577, 32), false);
    let reversed = c.lt(&length, &start, false);
    let start_ok = c.not(reversed);
    let past = c.lt(&length, &end, false);
    let end_ok = c.not(past);
    let empty = c.equal(&head[..32], &literal(0, 32));
    let nonempty = c.not(empty);
    let healthy = c.not(head[65]);
    let whole = c.equal(&end, &length);
    let eof = c.equal(&ending, &literal(0, 8));
    let mut delimiter = c.and(eof, whole);
    let exists = c.lt(&end, &length, false);
    for (tag, byte) in [(1, b','), (2, b']'), (3, b'}'), (4, b':')] {
        let selected = c.equal(&ending, &literal(tag, 8));
        let matches = c.equal(&next, &literal(byte as u128, 8));
        let accepted = c.and(selected, matches);
        let accepted = c.and(accepted, exists);
        delimiter = c.or(delimiter, accepted);
    }
    let valid = all(
        &mut c,
        &[
            head[66],
            healthy,
            no_overflow,
            bounded,
            start_ok,
            end_ok,
            nonempty,
            delimiter,
        ],
    );
    let mut packet = vec![T, whole];
    packet.extend(end);
    packet.extend(&head[32..64]);
    packet.resize(128, F);
    let packet = c.select(valid, &packet, &literal(0, 128));
    let finish = circuit(b, &name("Frame.Finish"), c, packet)?;

    // Pack the framed header and the ordinary C19 string in a C20 result.
    // Validity gates both roles, including every inactive storage/padding bit.
    let framed_head = b.var(21)?;
    let scanned = b.var(20)?;
    let selectors = b.selectors(20)?;
    let zero = truth(b, false)?;
    let head = invoke(b, "Header", vec![scanned])?;
    let length = invoke(b, "Length", vec![head])?;
    let units = invoke(b, "Units", vec![scanned])?;
    let data = b.app(units, selectors[2..].to_vec())?;
    let mut length_bit = b.app(length, selectors[15..].to_vec())?;
    for &sel in &selectors[2..15] {
        length_bit = core_mux(b, sel, zero, length_bit)?;
    }
    let value = core_mux(b, selectors[1], data, length_bit)?;
    let mut header_bit = b.app(framed_head, selectors[13..].to_vec())?;
    for &sel in &selectors[1..13] {
        header_bit = core_mux(b, sel, zero, header_bit)?;
    }
    let body = core_mux(b, selectors[0], value, header_bit)?;
    let valid = core_read(b, framed_head, 0, 7)?;
    let body = core_mux(b, valid, body, zero)?;
    let body = b.wrap_selectors(20, body)?;
    define(b, &name("Frame.Pack"), &[7, 19], 20, body)?;

    let doc = b.var(2)?;
    let start = b.var(1)?;
    let length = call(b, &d.document.length_definition, vec![doc])?;
    let count = call(b, &remaining, vec![length, start])?;
    let suffix = call(b, &d.fragments.slice_definition, vec![doc, start, count])?;
    let scanned = invoke(b, "Scan", vec![suffix])?;
    // Under the scan Let: doc/start/ending/scanned.
    let doc = b.var(3)?;
    let start = b.var(2)?;
    let ending = b.var(1)?;
    let state = b.var(0)?;
    let head = invoke(b, "Header", vec![state])?;
    let end = call(b, &end_definition, vec![head, start])?;
    let next = call(b, &d.document.read_byte_definition, vec![doc, end])?;
    let length = call(b, &d.document.length_definition, vec![doc])?;
    let framed_head = call(b, &finish, vec![head, start, length, ending, next])?;
    let body = invoke(b, "Frame.Pack", vec![framed_head, state])?;
    let ty = b.cube(19)?;
    let body = b.term(TermNode::Let {
        ty,
        value: scanned,
        body,
    })?;
    let parse = name("Frame.Parse");
    define(b, &parse, &[24, 5, 3], 20, body)?;

    let result = b.var(7)?;
    let mut selectors = vec![zero; 13];
    selectors.extend(b.selectors(7)?);
    let body = b.app(result, selectors)?;
    let body = b.wrap_selectors(7, body)?;
    let header = name("Frame.Header");
    define(b, &header, &[20], 7, body)?;
    let result = b.var(19)?;
    let mut selectors = vec![truth(b, true)?];
    selectors.extend(b.selectors(19)?);
    let body = b.app(result, selectors)?;
    let body = b.wrap_selectors(19, body)?;
    let value = name("Frame.Value");
    define(b, &value, &[20], 19, body)?;
    Ok([parse, header, value])
}

pub fn generate_csharp_practical_ordinary_json_string_parsers(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryJsonStringParseProgram> {
    let boundary = generate_boundary_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let mut b = Builder::new()?;
    let definition = if boundary.contracts().is_empty() {
        None
    } else {
        let document = emit_document(&mut b)?;
        let fragments = super::boundary_fragments::emit(&mut b, document)?;
        Some(emit(&mut b, fragments)?)
    };
    let certificate = b.finish()?;
    let p = OrdinaryJsonStringParseProgram {
        schema: "mpk.csharp.ordinary_json_string_parsers.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        boundary_program_sha256: boundary.hash(),
        definition,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_json_string_parsers(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryJsonStringParseProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_json_string_parsers(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
