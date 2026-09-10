//! Canonical quoted UTF-16 string encoding into the private byte document.
//! Source linkage and universal boundary proofs remain separate obligations.
use super::boundary_document::emit as emit_document;
use super::hex_codecs::{call, truth, word};
use super::integer_format::{circuit, define};
use super::temporal::literal;
use super::*;
const NAME: &str = "Mpk.CSharp.Ordinary.JsonString";
const STATE: u32 = 24;
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonStringDefinition {
    pub document: OrdinaryBoundaryDocumentDefinition,
    pub input_depth: u32,
    pub state_depth: u32,
    pub scan_steps: u32,
    pub static_transformers: u32,
    pub bounded_definition: String,
    pub packet_definition: String,
    pub format_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonStringProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    boundary_program_sha256: String,
    definition: Option<OrdinaryJsonStringDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryJsonStringProgram {
    pub fn definition(&self) -> Option<&OrdinaryJsonStringDefinition> {
        self.definition.as_ref()
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary JSON string")
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
fn packet(b: &mut Builder) -> R<String> {
    let mut c = Circuit::new(&[16, 16, 1]);
    let u = c.inputs[0].clone();
    let v = c.inputs[1].clone();
    let next = c.inputs[2][0];
    let high = range(&mut c, &u, 0xd800, 0xdbff);
    let low = range(&mut c, &v, 0xdc00, 0xdfff);
    let pair = c.and(high, low);
    let pair = c.and(pair, next);
    let surrogate = range(&mut c, &u, 0xd800, 0xdfff);
    let control = c.lt(&u, &literal(32, 16), false);
    let escape = c.or(surrogate, control);
    let quote = c.equal(&u, &literal(34, 16));
    let slash = c.equal(&u, &literal(92, 16));
    let short = c.or(quote, slash);
    let ascii = c.lt(&u, &literal(128, 16), false);
    let two = c.lt(&u, &literal(2048, 16), false);
    let mut bytes = vec![literal(0, 8); 6];
    let mut first = u[12..16].to_vec();
    first.extend([F, T, T, T]);
    let mut second = u[6..12].to_vec();
    second.extend([F, T]);
    let mut third = u[..6].to_vec();
    third.extend([F, T]);
    bytes[0] = first;
    bytes[1] = second;
    bytes[2] = third;
    let mut first = u[6..11].to_vec();
    first.extend([F, T, T]);
    bytes[0] = c.select(two, &first, &bytes[0]);
    let mut second = u[..6].to_vec();
    second.extend([F, T]);
    bytes[1] = c.select(two, &second, &bytes[1]);
    bytes[2] = c.select(two, &literal(0, 8), &bytes[2]);
    bytes[0] = c.select(ascii, &u[..8], &bytes[0]);
    for byte in bytes.iter_mut().take(3).skip(1) {
        *byte = c.select(ascii, &literal(0, 8), byte);
    }
    let mut length = c.select(two, &literal(2, 3), &literal(3, 3));
    length = c.select(ascii, &literal(1, 3), &length);
    bytes[0] = c.select(short, &literal(92, 8), &bytes[0]);
    bytes[1] = c.select(short, &u[..8], &bytes[1]);
    length = c.select(short, &literal(2, 3), &length);
    let mut escaped = vec![literal(92, 8), literal(117, 8)];
    for start in [12, 8, 4, 0] {
        let nibble = &u[start..start + 4];
        let digit = c.lt(nibble, &literal(10, 4), false);
        let mut wide = nibble.to_vec();
        wide.resize(8, F);
        let dec = c.add(&wide, &literal(48, 8), F).0;
        let hex = c.add(&wide, &literal(87, 8), F).0;
        escaped.push(c.select(digit, &dec, &hex));
    }
    for i in 0..6 {
        bytes[i] = c.select(escape, &escaped[i], &bytes[i]);
    }
    length = c.select(escape, &literal(6, 3), &length);
    // Scalar = 0x10000 + (high low-ten-bits << 10) + low low-ten-bits.
    let mut scalar = v[..10].to_vec();
    scalar.extend(&u[..10]);
    scalar.push(F);
    scalar = c.add(&scalar, &literal(0x10000, 21), F).0;
    let mut paired = vec![];
    let mut first = scalar[18..21].to_vec();
    first.extend([F, T, T, T, T]);
    paired.push(first);
    for start in [12, 6, 0] {
        let mut byte = scalar[start..start + 6].to_vec();
        byte.extend([F, T]);
        paired.push(byte);
    }
    paired.resize(6, literal(0, 8));
    for i in 0..6 {
        bytes[i] = c.select(pair, &paired[i], &bytes[i]);
    }
    length = c.select(pair, &literal(4, 3), &length);
    // C6 packet: output count 0..3, input advance 3..5, zero padding 5..8,
    // then six bytes in consecutive eight-bit slots; final eight bits zero.
    let mut out = length;
    out.extend(c.select(pair, &literal(2, 2), &literal(1, 2)));
    out.resize(8, F);
    for byte in bytes {
        out.extend(byte);
    }
    out.resize(64, F);
    circuit(b, &name("Packet"), c, out)
}
fn emit(b: &mut Builder) -> R<OrdinaryJsonStringDefinition> {
    let document = emit_document(b)?;
    b.helpers(STATE)?;
    let packet_definition = packet(b)?;
    let mut d = OrdinaryJsonStringDefinition {
        document,
        input_depth: 19,
        state_depth: STATE,
        scan_steps: 8192,
        static_transformers: 0,
        bounded_definition: name("Bounded"),
        packet_definition: packet_definition.clone(),
        format_definition: name("Format"),
    };
    // Text has the existing C19 layout. State has C24 byte data and a C6
    // header containing the input cursor followed by the output byte count.
    for (suffix, depth, header) in [("TextLength", 19, 5), ("Header", 24, 6)] {
        let source = b.var(header)?;
        let zero = truth(b, false)?;
        let mut args = vec![zero; (depth - header) as usize];
        args.extend(b.selectors(header)?);
        let body = b.app(source, args)?;
        let body = b.wrap_selectors(header, body)?;
        define(b, &name(suffix), &[depth], header, body)?;
    }
    let state = b.var(23)?;
    let mut args = vec![truth(b, true)?];
    args.extend(b.selectors(23)?);
    let body = b.app(state, args)?;
    let body = b.wrap_selectors(23, body)?;
    define(b, &name("Bytes"), &[24], 23, body)?;
    let header = b.var(25)?;
    let bytes = b.var(24)?;
    let zero = truth(b, false)?;
    let selectors = b.selectors(24)?;
    let data = b.app(bytes, selectors[1..].to_vec())?;
    let mut head = b.app(header, selectors[18..].to_vec())?;
    for &selector in &selectors[1..18] {
        head = core_mux(b, selector, zero, head)?;
    }
    let body = core_mux(b, selectors[0], data, head)?;
    let body = b.wrap_selectors(24, body)?;
    define(b, &name("MakeState"), &[6, 23], 24, body)?;
    for (suffix, offset) in [("InputCursor", 0), ("OutputCount", 32)] {
        let header = b.var(5)?;
        let zero = truth(b, false)?;
        let bits = (0..32)
            .map(|i| core_read(b, header, i + offset, 6))
            .collect::<R<Vec<_>>>()?;
        let body = core_select(b, &bits, 5, 0, 5, zero)?;
        let body = b.wrap_selectors(5, body)?;
        define(b, &name(suffix), &[6], 5, body)?;
    }
    let mut c = Circuit::new(&[32]);
    let len = c.inputs[0].clone();
    let good = c.lt(&len, &literal(16385, 32), false);
    let bound = circuit(b, &name("BoundCircuit"), c, vec![good])?;
    let text = b.var(0)?;
    let len = invoke(b, "TextLength", vec![text])?;
    let body = call(b, &bound, vec![len])?;
    define(b, &d.bounded_definition, &[19], 0, body)?;
    let mut c = Circuit::new(&[32, 32]);
    let i = c.inputs[0].clone();
    let len = c.inputs[1].clone();
    let active = c.lt(&i, &len, false);
    let in_storage = c.lt(&i, &literal(16384, 32), false);
    let active = c.and(active, in_storage);
    let active_name = circuit(b, &name("Active"), c, vec![active])?;
    let text = b.var(1)?;
    let state = b.var(0)?;
    let len = invoke(b, "TextLength", vec![text])?;
    let head = invoke(b, "Header", vec![state])?;
    let cursor = invoke(b, "InputCursor", vec![head])?;
    let body = call(b, &active_name, vec![cursor, len])?;
    define(b, &name("More"), &[19, 24], 0, body)?;
    let text = b.var(5)?;
    let index = b.var(4)?;
    let len = invoke(b, "TextLength", vec![text])?;
    let active = call(b, &active_name, vec![index, len])?;
    let mut args = vec![truth(b, true)?];
    for i in 0..14 {
        args.push(core_read(b, index, i, 5)?);
    }
    args.extend(b.selectors(4)?);
    let raw = b.app(text, args)?;
    let zero = truth(b, false)?;
    let body = core_mux(b, active, raw, zero)?;
    let body = b.wrap_selectors(4, body)?;
    define(b, &name("ReadUnit"), &[19, 5], 4, body)?;
    let mut c = Circuit::new(&[32]);
    let i = c.inputs[0].clone();
    let next = c.add(&i, &literal(1, 32), F).0;
    let next_name = circuit(b, &name("Next"), c, next)?;
    let mut c = Circuit::new(&[64, 64]);
    let head = c.inputs[0].clone();
    let packet = c.inputs[1].clone();
    let mut consumed = packet[3..5].to_vec();
    consumed.resize(32, F);
    let mut count = packet[..3].to_vec();
    count.resize(32, F);
    let mut updated = c.add(&head[..32], &consumed, F).0;
    updated.extend(c.add(&head[32..], &count, F).0);
    let updated_name = circuit(b, &name("UpdatedHeader"), c, updated)?;
    let mut c = Circuit::new(&[32, 64, 64]);
    let index = c.inputs[0].clone();
    let head = c.inputs[1].clone();
    let packet = c.inputs[2].clone();
    let below = c.lt(&index, &head[32..], false);
    let after = c.not(below);
    let diff = c.sub(&index, &head[32..]).0;
    let mut count = packet[..3].to_vec();
    count.resize(32, F);
    let within = c.lt(&diff, &count, false);
    let write = c.and(after, within);
    let mut byte = literal(0, 8);
    for i in 0..6 {
        let selected = c.equal(&diff, &literal(i as u128, 32));
        byte = c.select(selected, &packet[8 + i * 8..16 + i * 8], &byte);
    }
    byte.push(write);
    byte.resize(16, F);
    let write_name = circuit(b, &name("WriteByte"), c, byte)?;
    let bytes = b.var(25)?;
    let head = b.var(24)?;
    let packet = b.var(23)?;
    let zero = truth(b, false)?;
    let bits = (0..20).map(|i| b.var(27 - i)).collect::<R<Vec<_>>>()?;
    let index = core_select(b, &bits, 5, 0, 5, zero)?;
    let index = b.wrap_selectors(5, index)?;
    let write = call(b, &write_name, vec![index, head, packet])?;
    let active = core_read(b, write, 8, 4)?;
    let selectors = b.selectors(23)?;
    let old = b.app(bytes, selectors.clone())?;
    let mut args = selectors[20..].to_vec();
    args.push(zero);
    let new = b.app(write, args)?;
    let body = core_mux(b, active, new, old)?;
    let body = b.wrap_selectors(23, body)?;
    define(b, &name("Append"), &[23, 6, 6], 23, body)?;
    let state = b.var(1)?;
    let packet = b.var(0)?;
    let head = invoke(b, "Header", vec![state])?;
    let updated = call(b, &updated_name, vec![head, packet])?;
    let bytes = invoke(b, "Bytes", vec![state])?;
    let bytes = invoke(b, "Append", vec![bytes, head, packet])?;
    let body = invoke(b, "MakeState", vec![updated, bytes])?;
    define(b, &name("Update"), &[24, 6], 24, body)?;
    let text = b.var(1)?;
    let state = b.var(0)?;
    let head = invoke(b, "Header", vec![state])?;
    let cursor = invoke(b, "InputCursor", vec![head])?;
    let next = call(b, &next_name, vec![cursor])?;
    let first = invoke(b, "ReadUnit", vec![text, cursor])?;
    let second = invoke(b, "ReadUnit", vec![text, next])?;
    let len = invoke(b, "TextLength", vec![text])?;
    let has_next = call(b, &active_name, vec![next, len])?;
    let packet = call(b, &packet_definition, vec![first, second, has_next])?;
    let updated = invoke(b, "Update", vec![state, packet])?;
    let active = invoke(b, "More", vec![text, state])?;
    let body = call(
        b,
        &format!("{PREFIX}.Cube.D24.Mux"),
        vec![active, updated, state],
    )?;
    define(b, &name("Step"), &[19, 24], 24, body)?;
    let state_ty = b.cube(24)?;
    let transformer = b.pi(state_ty, state_ty)?;
    let text_ty = b.cube(19)?;
    let f = b.var(2)?;
    let state = b.var(0)?;
    let first = b.app(f, vec![state])?;
    let text = b.var(4)?;
    let g = b.var(2)?;
    let middle = b.var(0)?;
    let next = b.app(g, vec![middle])?;
    let active = invoke(b, "More", vec![text, middle])?;
    let body = call(
        b,
        &format!("{PREFIX}.Cube.D24.Mux"),
        vec![active, next, middle],
    )?;
    let body = b.term(TermNode::Let {
        ty: state_ty,
        value: first,
        body,
    })?;
    let body = b.lam(state_ty, body)?;
    let body = b.lam(transformer, body)?;
    let body = b.lam(transformer, body)?;
    let body = b.lam(text_ty, body)?;
    let ty = b.pi(transformer, transformer)?;
    let ty = b.pi(transformer, ty)?;
    let ty = b.pi(text_ty, ty)?;
    b.define(&name("Compose"), ty, body)?;
    // Initial header has output count one; the byte buffer contains only quote.
    let zero = truth(b, false)?;
    let bits = (0..64).map(|i| truth(b, i == 32)).collect::<R<Vec<_>>>()?;
    let head = core_select(b, &bits, 6, 0, 6, zero)?;
    let head = b.wrap_selectors(6, head)?;
    let quote = word(b, 34, 3)?;
    let selectors = b.selectors(23)?;
    let mut bytes = b.app(quote, selectors[20..].to_vec())?;
    for &sel in &selectors[..20] {
        bytes = core_mux(b, sel, zero, bytes)?;
    }
    let bytes = b.wrap_selectors(23, bytes)?;
    let initial = invoke(b, "MakeState", vec![head, bytes])?;
    let text = b.var(0)?;
    let step = invoke(b, "Step", vec![text])?;
    let compose = invoke(b, "Compose", vec![text])?;
    let pair = b.compose_term(24, compose, &[step, step])?;
    let pair_body = b.lam(text_ty, pair)?;
    let pair_ty = b.pi(text_ty, transformer)?;
    b.define(&name("StepTwo"), pair_ty, pair_body)?;
    let pair = invoke(b, "StepTwo", vec![text])?;
    let pipeline = b.compose_term(24, compose, &vec![pair; 8192])?;
    let state = b.app(pipeline, vec![initial])?;
    // Append closing quote with zero input advance, then convert header layout.
    let packet = word(b, 1 | (34 << 8), 6)?;
    let state = invoke(b, "Update", vec![state, packet])?;
    // Share the completed scan between output length and byte storage.
    let completed_state = state;
    let state = b.var(0)?;
    let text = b.var(1)?;
    let head = invoke(b, "Header", vec![state])?;
    let count = invoke(b, "OutputCount", vec![head])?;
    let bytes = invoke(b, "Bytes", vec![state])?;
    let document = call(b, &d.document.make_definition, vec![count, bytes])?;
    let bounded = call(b, &d.bounded_definition, vec![text])?;
    let zero = truth(b, false)?;
    let empty = b.wrap_selectors(24, zero)?;
    let body = call(
        b,
        &format!("{PREFIX}.Cube.D24.Mux"),
        vec![bounded, document, empty],
    )?;
    let body = b.term(TermNode::Let {
        ty: state_ty,
        value: completed_state,
        body,
    })?;
    define(b, &d.format_definition, &[19], 24, body)?;
    d.static_transformers = b.static_transformers as u32;
    Ok(d)
}
pub fn generate_csharp_practical_ordinary_json_strings(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryJsonStringProgram> {
    let boundary = generate_boundary_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let mut b = Builder::new()?;
    let definition = if boundary.contracts().is_empty() {
        None
    } else {
        Some(emit(&mut b)?)
    };
    let certificate = b.finish()?;
    let p = OrdinaryJsonStringProgram {
        schema: "mpk.csharp.ordinary_json_strings.v1".into(),
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
pub fn import_csharp_practical_ordinary_json_strings(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryJsonStringProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_json_strings(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
