//! Raw canonical JSON tree limits. This scanner is conditional on separate JSON
//! syntax/UTF-8 validation; it does not establish grammar or source admission.
use super::boundary_document::emit as emit_document;
use super::hex_codecs::{call, truth, word};
use super::integer_format::{circuit_with_block_bits, define};
fn circuit(b: &mut Builder, id: &str, c: Circuit, out: Word) -> R<String> {
    circuit_with_block_bits(b, id, c, out, 7)
}
use super::temporal::literal;
use super::*;

const NAME: &str = "Mpk.CSharp.Ordinary.JsonRawLimits";
const STATE: u32 = 6;
const BLOCK_BYTES: u32 = 32;
const SCAN_STEPS: usize = 4096;
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonRawLimitsDefinition {
    pub document: OrdinaryBoundaryDocumentDefinition,
    pub state_depth: u32,
    pub block_bytes: u32,
    pub blocks_per_step: u32,
    pub scan_steps: u32,
    pub static_transformers: u32,
    pub scan_definition: String,
    /// Internal scan state -> Bool;not a document admission predicate.
    pub finished_definition: String,
    pub valid_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonRawLimitsProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    boundary_program_sha256: String,
    definition: Option<OrdinaryJsonRawLimitsDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryJsonRawLimitsProgram {
    pub fn definition(&self) -> Option<&OrdinaryJsonRawLimitsDefinition> {
        self.definition.as_ref()
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary raw JSON limits")
    }
}
fn name(suffix: &str) -> String {
    format!("{NAME}.{suffix}")
}
fn invoke(b: &mut Builder, suffix: &str, args: Vec<u32>) -> R<u32> {
    call(b, &name(suffix), args)
}
// State: cursor0..32,nodes32..51,depth51..57,in-string57,escape58,
// in-primitive59,pending-closed-string60,sticky failure61,reserved62..64.
// A closed string is counted only after inspecting the following byte: a
// canonical object key is immediately followed by colon and is not a value.
fn block_circuit() -> CircuitWithOutput {
    let mut c = Circuit::new(&[64, 256, 32]);
    let old = c.inputs[0].clone();
    let bytes = c.inputs[1].clone();
    let length = c.inputs[2].clone();
    let remaining = c.sub(&length, &old[..32]).0;
    let full = c.nonzero(&remaining[5..]);
    let mut nodes = old[32..51].to_vec();
    let mut depth = old[51..57].to_vec();
    let mut string = old[57];
    let mut escaped = old[58];
    let mut token = old[59];
    let mut pending = old[60];
    let mut bad = old[61];
    for i in 0..32 {
        let byte = (0..8).map(|k| bytes[i | (k << 5)]).collect::<Word>();
        let tail = c.lt(&literal(i as u128, 5), &remaining[..5], false);
        let active = c.or(full, tail);
        let quote = c.equal(&byte, &literal(34, 8));
        let slash = c.equal(&byte, &literal(92, 8));
        let colon = c.equal(&byte, &literal(58, 8));
        let open_obj = c.equal(&byte, &literal(123, 8));
        let open_arr = c.equal(&byte, &literal(91, 8));
        let open = c.or(open_obj, open_arr);
        let close_obj = c.equal(&byte, &literal(125, 8));
        let close_arr = c.equal(&byte, &literal(93, 8));
        let close = c.or(close_obj, close_arr);
        let mut delim = close;
        for ch in *b", \t\r\n" {
            let yes = c.equal(&byte, &literal(ch as u128, 8));
            delim = c.or(delim, yes);
        }
        let lower = c.lt(&byte, &literal(48, 8), false);
        let upper = c.lt(&literal(57, 8), &byte, false);
        let outside_digits = c.or(lower, upper);
        let mut primitive = c.not(outside_digits);
        for ch in *b"-tfn" {
            let yes = c.equal(&byte, &literal(ch as u128, 8));
            primitive = c.or(primitive, yes);
        }
        let outside = c.not(string);
        let not_token = c.not(token);
        let start = c.and(primitive, not_token);
        let start = c.and(start, outside);
        let opening = c.and(open, outside);
        let closing = c.and(close, outside);
        let not_colon = c.not(colon);
        let string_value = c.and(pending, not_colon);
        let container_or_atom = c.or(opening, start);
        let count = c.or(container_or_atom, string_value);
        let count = c.and(count, active);
        let too_deep = c.lt(&literal(32, 6), &depth, false);
        let depth_bad = c.and(too_deep, count);
        let empty = c.equal(&depth, &literal(0, 6));
        let underflow = c.and(closing, empty);
        let underflow = c.and(underflow, active);
        let inc = c.add(&nodes, &literal(0, 19), count).0;
        let count_over = c.lt(&literal(262144, 19), &inc, false);
        let count_invalid = c.or(count_over, old[61]);
        nodes = c.select(count_invalid, &literal(262145, 19), &inc);
        bad = c.or(bad, depth_bad);
        bad = c.or(bad, underflow);
        bad = c.or(bad, count_over);
        let plus = c.add(&depth, &literal(1, 6), F).0;
        let minus = c.sub(&depth, &literal(1, 6)).0;
        let next_depth = c.select(opening, &plus, &depth);
        let next_depth = c.select(closing, &minus, &next_depth);
        depth = c.select(active, &next_depth, &depth);
        let unescaped = c.not(escaped);
        let end_string = c.and(string, unescaped);
        let end_string = c.and(end_string, quote);
        let begin_string = c.and(outside, quote);
        let not_end = c.not(end_string);
        let keep_string = c.and(string, not_end);
        let next_string = c.or(begin_string, keep_string);
        let next_escape = c.and(string, unescaped);
        let next_escape = c.and(next_escape, slash);
        let not_delim = c.not(delim);
        let next_token = c.or(token, start);
        let next_token = c.and(next_token, not_delim);
        let next_token = c.and(next_token, outside);
        string = c.mux(active, next_string, string);
        escaped = c.mux(active, next_escape, escaped);
        token = c.mux(active, next_token, token);
        pending = c.mux(active, end_string, pending);
    }
    let mut out = c.add(&old[..32], &literal(32, 32), F).0;
    out.extend(nodes);
    out.extend(depth);
    out.extend([string, escaped, token, pending, bad]);
    out.resize(64, F);
    CircuitWithOutput {
        circuit: c,
        output: out,
    }
}
struct CircuitWithOutput {
    circuit: Circuit,
    output: Word,
}
// Preserve bounded dependency expansion and force the finite state between steps.
fn emit_state_pack(b: &mut Builder) -> R<()> {
    // Keep the duplicated Bool-rec branches behind a declaration boundary.
    // Inlining this recursor 64 times duplicates a shared subtree on each
    // level for the unchanged certificate dependency walker.
    let major = b.var(1)?;
    let value = b.var(0)?;
    let body = core_mux(b, major, value, value)?;
    define(b, &name("ForceBit"), &[0, 0], 0, body)?;
    let bits = (0..64)
        .map(|i| b.var(STATE + 63 - i))
        .collect::<R<Vec<_>>>()?;
    let zero = truth(b, false)?;
    let mut body = core_select(b, &bits, STATE, 0, STATE, zero)?;
    for &bit in bits.iter().rev() {
        body = invoke(b, "ForceBit", vec![bit, body])?;
    }
    let mut body = b.wrap_selectors(STATE, body)?;
    let mut ty = b.cube(STATE)?;
    for _ in 0..64 {
        body = b.lam(b.boolean, body)?;
        ty = b.pi(b.boolean, ty)?;
    }
    b.define(&name("PackState"), ty, body)?;
    let state = b.var(0)?;
    let bits = (0..64)
        .map(|i| core_read(b, state, i, STATE))
        .collect::<R<Vec<_>>>()?;
    let body = invoke(b, "PackState", bits)?;
    define(b, &name("SealState"), &[STATE], STATE, body)
}
fn seal_state(b: &mut Builder, state: u32) -> R<u32> {
    let seal = b.constant(&name("SealState"))?;
    // Account for this concrete S->S helper occurrence in the same budget as
    // the scan and its circuit helpers; a one-element composition is the leaf.
    let seal = b.compose(STATE, &[seal])?;
    b.app(seal, vec![state])
}
fn emit(b: &mut Builder) -> R<OrdinaryJsonRawLimitsDefinition> {
    let document = emit_document(b)?;
    emit_with_document(b, document)
}
pub(super) fn emit_with_document(
    b: &mut Builder,
    document: OrdinaryBoundaryDocumentDefinition,
) -> R<OrdinaryJsonRawLimitsDefinition> {
    if !b.globals.contains_key(&format!("{PREFIX}.Cube.D6.Mux")) {
        b.helpers(STATE)?;
    }
    emit_state_pack(b)?;
    let mut d = OrdinaryJsonRawLimitsDefinition {
        document,
        state_depth: STATE,
        block_bytes: BLOCK_BYTES,
        blocks_per_step: 8,
        scan_steps: SCAN_STEPS as u32,
        static_transformers: 0,
        scan_definition: name("Scan"),
        finished_definition: String::new(),
        valid_definition: name("Valid"),
    };
    let block = block_circuit();
    let step_circuit = circuit(b, &name("BlockCircuit"), block.circuit, block.output)?;
    let mut c = Circuit::new(&[32, 64]);
    let length = c.inputs[0].clone();
    let state = c.inputs[1].clone();
    let remaining = c.lt(&state[..32], &length, false);
    let valid = c.not(state[61]);
    let active = c.and(remaining, valid);
    let active = circuit(b, &name("ActiveCircuit"), c, vec![active])?;
    let length = b.var(1)?;
    let state = b.var(0)?;
    let body = call(b, &active, vec![length, state])?;
    define(b, &name("Active"), &[5, STATE], 0, body)?;
    // Internal chunk read relies on the scan's aligned cursor invariant.
    // Offset selectors are concatenated with cursor bits 5..20; no byte copy,
    // host decoding or truncated application-string representation is involved.
    let source = b.var(9)?;
    let state = b.var(8)?;
    let mut args = vec![truth(b, true)?];
    for i in 0..5 {
        args.push(b.var(7 - i)?);
    }
    for i in 5..20 {
        args.push(core_read(b, state, i, STATE)?);
    }
    for i in 5..8 {
        args.push(b.var(7 - i)?);
    }
    let body = b.app(source, args)?;
    let body = b.wrap_selectors(8, body)?;
    define(b, &name("ReadBlock"), &[24, STATE], 8, body)?;
    let source = b.var(2)?;
    let length = b.var(1)?;
    let state = b.var(0)?;
    let bytes = invoke(b, "ReadBlock", vec![source, state])?;
    let next = call(b, &step_circuit, vec![state, bytes, length])?;
    let active = invoke(b, "Active", vec![length, state])?;
    let selected = call(
        b,
        &format!("{PREFIX}.Cube.D6.Mux"),
        vec![active, next, state],
    )?;
    // Seal the selected result, including the guard's old-state branch. Sealing
    // only the true branch would leave the outer Mux closure retaining history.
    let body = seal_state(b, selected)?;
    define(b, &name("Step32"), &[24, 5, STATE], STATE, body)?;

    let state_ty = b.cube(STATE)?;
    let transformer = b.pi(state_ty, state_ty)?;
    let length_ty = b.cube(5)?;
    // length, f, g, state; share the first result before deciding to run g.
    let f = b.var(2)?;
    let state = b.var(0)?;
    let first = b.app(f, vec![state])?;
    let length = b.var(4)?;
    let g = b.var(2)?;
    let intermediate = b.var(0)?;
    let next = b.app(g, vec![intermediate])?;
    let active = invoke(b, "Active", vec![length, intermediate])?;
    let body = call(
        b,
        &format!("{PREFIX}.Cube.D6.Mux"),
        vec![active, next, intermediate],
    )?;
    let body = seal_state(b, body)?;
    let body = b.term(TermNode::Let {
        ty: state_ty,
        value: first,
        body,
    })?;
    let body = b.lam(state_ty, body)?;
    let body = b.lam(transformer, body)?;
    let body = b.lam(transformer, body)?;
    let body = b.lam(length_ty, body)?;
    let ty = b.pi(transformer, transformer)?;
    let ty = b.pi(transformer, ty)?;
    let ty = b.pi(length_ty, ty)?;
    b.define(&name("Compose"), ty, body)?;
    let source = b.var(1)?;
    let length = b.var(0)?;
    let step = invoke(b, "Step32", vec![source, length])?;
    let compose = invoke(b, "Compose", vec![length])?;
    // Count the eight concrete Step32 occurrences as well as every outer
    // invocation. Circuit helpers also consume the same Builder budget.
    let pair = b.compose_term(STATE, compose, &[step; 8])?;
    let body = bind_inputs(b, &[1 << 24, 32], pair)?;
    let ty = b.pi(length_ty, transformer)?;
    let document_ty = b.cube(24)?;
    let ty = b.pi(document_ty, ty)?;
    b.define(&name("Step256"), ty, body)?;
    let step = invoke(b, "Step256", vec![source, length])?;
    let pipeline = b.compose_term(STATE, compose, &vec![step; SCAN_STEPS])?;
    let zero = word(b, 0, STATE)?;
    let body = b.app(pipeline, vec![zero])?;
    define(b, &d.scan_definition, &[24, 5], STATE, body)?;
    let mut c = Circuit::new(&[64]);
    let state = c.inputs[0].clone();
    let nodes = c.add(&state[32..51], &literal(0, 19), state[60]).0;
    let bounded = c.lt(&nodes, &literal(262145, 19), false);
    let closed = c.equal(&state[51..57], &literal(0, 6));
    let invalid = c.or(state[57], state[58]);
    let invalid = c.or(invalid, state[61]);
    let valid = c.not(invalid);
    let valid = c.and(valid, closed);
    let valid = c.and(valid, bounded);
    let finish = circuit(b, &name("Finished"), c, vec![valid])?;
    d.finished_definition = finish.clone();
    let source = b.var(0)?;
    let length = call(b, &d.document.length_definition, vec![source])?;
    let state = call(b, &d.scan_definition, vec![source, length])?;
    let good = call(b, &finish, vec![state])?;
    let bounded = call(b, &d.document.bounded_definition, vec![source])?;
    let no = truth(b, false)?;
    let body = core_mux(b, bounded, good, no)?;
    define(b, &d.valid_definition, &[24], 0, body)?;
    d.static_transformers = b.static_transformers as u32;
    Ok(d)
}
pub fn generate_csharp_practical_ordinary_json_raw_limits(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryJsonRawLimitsProgram> {
    let boundary = generate_boundary_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let mut b = Builder::new()?;
    let definition = if boundary.contracts().is_empty() {
        None
    } else {
        Some(emit(&mut b)?)
    };
    let certificate = b.finish()?;
    let p = OrdinaryJsonRawLimitsProgram {
        schema: "mpk.csharp.ordinary_json_raw_limits.v1".into(),
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
pub fn import_csharp_practical_ordinary_json_raw_limits(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryJsonRawLimitsProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_json_raw_limits(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::super::test_eval::{apply, bit, run, V};
    use super::*;
    fn bits(n: u64, width: usize) -> V {
        V::Cube((0..width).map(|i| n & (1 << i) != 0).collect())
    }
    fn observed(c: &Certificate, value: V) -> u64 {
        (0..64).fold(0, |n, i| {
            let mut v = value.clone();
            for k in 0..6 {
                v = apply(c, v, V::Bit(i & (1 << k) != 0));
            }
            n | ((bit(v) as u64) << i)
        })
    }
    fn block(bytes: &[u8]) -> V {
        V::Cube(
            (0..256)
                .map(|i| bytes.get(i & 31).is_some_and(|b| b & (1 << (i >> 5)) != 0))
                .collect(),
        )
    }
    #[test]
    fn raw_json_limit_block_counts_keys_escapes_depth_and_cutoff() {
        let mut b = Builder::new().unwrap();
        let c = block_circuit();
        let f = circuit(&mut b, &name("BlockTest"), c.circuit, c.output).unwrap();
        let bytes = b.finish().unwrap();
        let cert = decode_canonical_certificate(&bytes).unwrap();
        let mut cases = 0;
        for (text, expected, pending) in [
            (r#"{}"#, 1, false),
            (r#"[]"#, 1, false),
            (r#"[1,-2,true,false,null]"#, 6, false),
            (r#"{"a":"x","b":{"k":[0]}}"#, 5, false),
            (r#""text""#, 0, true),
            (r#"{"a\"b":"[{}]\\\""}"#, 2, false),
        ] {
            let state = observed(
                &cert,
                run(
                    &cert,
                    &f,
                    vec![
                        bits(0, 64),
                        block(text.as_bytes()),
                        bits(text.len() as u64, 32),
                    ],
                ),
            );
            assert_eq!((state >> 32) & 0x7ffff, expected, "{text}");
            assert_eq!((state >> 60) & 1, u64::from(pending));
            assert_eq!(state >> 61, 0);
            assert_eq!((state >> 51) & 63, 0);
            cases += 1;
        }
        for count in [262143u64, 262144, 262145] {
            for text in [b"0".as_slice(), b" ".as_slice()] {
                let initial = count << 32;
                let state = observed(
                    &cert,
                    run(&cert, &f, vec![bits(initial, 64), block(text), bits(1, 32)]),
                );
                let expected = (count + u64::from(text == b"0")).min(262145);
                assert_eq!((state >> 32) & 0x7ffff, expected);
                assert_eq!((state >> 61) & 1, u64::from(expected > 262144));
                cases += 1;
            }
        }
        for depth in [31u64, 32, 33] {
            for text in [b"[".as_slice(), b"0".as_slice(), b"]".as_slice()] {
                let state = observed(
                    &cert,
                    run(
                        &cert,
                        &f,
                        vec![bits(depth << 51, 64), block(text), bits(1, 32)],
                    ),
                );
                assert_eq!((state >> 61) & 1, u64::from(depth > 32 && text != b"]"));
                cases += 1;
            }
        }
        // Close-string/colon decision and escaped quote span the32-byte boundary.
        for escaped in [false, true] {
            let mut prefix = vec![b'a'; 32];
            prefix[0] = b'"';
            prefix[31] = if escaped { b'\\' } else { b'"' };
            let suffix = if escaped {
                b"\"\"".as_slice()
            } else {
                b":0".as_slice()
            };
            let state = run(&cert, &f, vec![bits(0, 64), block(&prefix), bits(34, 32)]);
            let state = observed(
                &cert,
                run(&cert, &f, vec![state, block(suffix), bits(34, 32)]),
            );
            assert_eq!((state >> 32) & 0x7ffff, u64::from(!escaped));
            assert_eq!((state >> 60) & 1, u64::from(escaped));
            assert_eq!(state >> 61, 0);
            cases += 1;
        }
        eprintln!("Raw JSON block limits:{cases} full64-bit state observations");
        if let Some(out) = std::env::var_os("MPK_W09_JSON_RAW_LIMITS_HELPER_OUT") {
            let out = std::path::PathBuf::from(out);
            std::fs::create_dir_all(&out).unwrap();
            std::fs::write(
                out.join("raw-block.hex"),
                bytes.iter().map(|v| format!("{v:02x}")).collect::<String>() + "\n",
            )
            .unwrap();
        }
    }
    #[test]
    fn raw_json_limit_scan_generation_costs() {
        let mut b = Builder::new().unwrap();
        let d = emit(&mut b).unwrap();
        let bytes = b.finish().unwrap();
        let cert = decode_canonical_certificate(&bytes).unwrap();
        eprintln!(
            "Raw JSON limit scan:{} terms,{} declarations,{} static transformers",
            cert.term_table.len(),
            cert.declarations.len(),
            d.static_transformers
        );
        // Exercise the last physical32-byte block and finalization directly;
        // this complements full small-document scans without repeating1MiB
        // of unchanged prefix transitions for each high-address boundary case.
        for (length, active) in [(1_048_575u32, false), (1_048_576, true)] {
            let mut ones = BTreeSet::new();
            for i in 0..32 {
                if length & (1 << i) != 0 {
                    ones.insert(i << 19);
                }
            }
            for i in 0..8 {
                if b'0' & (1 << i) != 0 {
                    ones.insert(1 | (1_048_575usize << 1) | (i << 21));
                }
            }
            let document = super::super::test_eval::sparse_cube(24, ones);
            let state = run(
                &cert,
                &name("Step32"),
                vec![document, bits(u64::from(length), 32), bits(1_048_544, 64)],
            );
            let state = observed(&cert, state);
            assert_eq!(state & 0xffff_ffff, 1_048_576);
            assert_eq!((state >> 32) & 0x7ffff, u64::from(active));
            assert_eq!(state >> 61, 0);
        }
        for count in [262143u64, 262144, 262145] {
            for pending in [false, true] {
                let state = (count << 32) | (u64::from(pending) << 60);
                assert_eq!(
                    bit(run(&cert, &d.finished_definition, vec![bits(state, 64)])),
                    count + u64::from(pending) <= 262144
                );
            }
        }
        if let Some(out) = std::env::var_os("MPK_W09_JSON_RAW_LIMITS_HELPER_OUT") {
            let out = std::path::PathBuf::from(out);
            std::fs::create_dir_all(&out).unwrap();
            std::fs::write(
                out.join("raw-scan.hex"),
                bytes.iter().map(|v| format!("{v:02x}")).collect::<String>() + "\n",
            )
            .unwrap();
        }
    }
}
