//! UTF-8 byte validity and leading-BOM rejection, before full JSON grammar.
use super::boundary_document::emit as emit_document;
use super::hex_codecs::{call, truth, word};
use super::integer_format::{circuit, define};
use super::temporal::literal;
use super::*;

const NAME: &str = "Mpk.CSharp.Ordinary.BoundaryUtf8";
const STATE: u32 = 6;
const BLOCK_BYTES: u32 = 64;
const SCAN_STEPS: usize = 8192;
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryBoundaryUtf8Definition {
    pub document: OrdinaryBoundaryDocumentDefinition,
    pub state_depth: u32,
    pub block_bytes: u32,
    pub blocks_per_step: u32,
    pub scan_steps: u32,
    pub static_transformers: u32,
    pub scan_definition: String,
    pub valid_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryBoundaryUtf8Program {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    boundary_program_sha256: String,
    definition: Option<OrdinaryBoundaryUtf8Definition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryBoundaryUtf8Program {
    pub fn definition(&self) -> Option<&OrdinaryBoundaryUtf8Definition> {
        self.definition.as_ref()
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary boundary UTF-8")
    }
}
fn name(suffix: &str) -> String {
    format!("{NAME}.{suffix}")
}
fn invoke(b: &mut Builder, suffix: &str, args: Vec<u32>) -> R<u32> {
    call(b, &name(suffix), args)
}
fn between(c: &mut Circuit, byte: &[Bit], low: u128, high: u128) -> Bit {
    let below = c.lt(byte, &literal(low, 8), false);
    let above = c.lt(&literal(high, 8), byte, false);
    let bad = c.or(below, above);
    c.not(bad)
}
// State: next block byte index [0..32), continuation count [32..34),
// next-byte lower/upper bounds [34..42)/[42..50), sticky failure bit 50.
fn block_circuit() -> CircuitWithOutput {
    let mut c = Circuit::new(&[64, 512, 32]);
    let original = c.inputs[0].clone();
    let bytes = c.inputs[1].clone();
    let length = c.inputs[2].clone();
    let remaining_bytes = c.sub(&length, &original[..32]).0;
    let full_block = c.nonzero(&remaining_bytes[6..]);
    let mut remaining = original[32..34].to_vec();
    let mut lower = original[34..42].to_vec();
    let mut upper = original[42..50].to_vec();
    let mut bad = original[50];
    let mut any_nonascii = F;
    for i in 0..64 {
        let byte = (0..8).map(|k| bytes[i | (k << 6)]).collect::<Word>();
        let within_tail = c.lt(&literal(i as u128, 6), &remaining_bytes[..6], false);
        let active = c.or(full_block, within_tail);
        let nonascii = c.and(active, byte[7]);
        any_nonascii = c.or(any_nonascii, nonascii);
        let continuation = c.nonzero(&remaining);
        let below = c.lt(&byte, &lower, false);
        let above = c.lt(&upper, &byte, false);
        let cont_bad = c.or(below, above);
        let ascii = c.not(byte[7]);
        let two = between(&mut c, &byte, 0xc2, 0xdf);
        let three = between(&mut c, &byte, 0xe0, 0xef);
        let four = between(&mut c, &byte, 0xf0, 0xf4);
        let lead = c.or(ascii, two);
        let lead = c.or(lead, three);
        let lead = c.or(lead, four);
        let lead_bad = c.not(lead);
        let invalid = c.mux(continuation, cont_bad, lead_bad);
        let invalid = c.and(active, invalid);
        bad = c.or(bad, invalid);
        let mut lead_count = c.select(two, &literal(1, 2), &literal(0, 2));
        lead_count = c.select(three, &literal(2, 2), &lead_count);
        lead_count = c.select(four, &literal(3, 2), &lead_count);
        let decreased = c.sub(&remaining, &literal(1, 2)).0;
        let count = c.select(continuation, &decreased, &lead_count);
        remaining = c.select(active, &count, &remaining);
        let e0 = c.equal(&byte, &literal(0xe0, 8));
        let ed = c.equal(&byte, &literal(0xed, 8));
        let f0 = c.equal(&byte, &literal(0xf0, 8));
        let f4 = c.equal(&byte, &literal(0xf4, 8));
        let lead_low = c.select(e0, &literal(0xa0, 8), &literal(0x80, 8));
        let lead_low = c.select(f0, &literal(0x90, 8), &lead_low);
        let lead_high = c.select(ed, &literal(0x9f, 8), &literal(0xbf, 8));
        let lead_high = c.select(f4, &literal(0x8f, 8), &lead_high);
        let next_low = c.select(continuation, &literal(0x80, 8), &lead_low);
        let next_high = c.select(continuation, &literal(0xbf, 8), &lead_high);
        lower = c.select(active, &next_low, &lower);
        upper = c.select(active, &next_high, &upper);
    }
    let next_index = c
        .add(&original[..32], &literal(BLOCK_BYTES as u128, 32), F)
        .0;
    let mut normal = next_index.clone();
    normal.extend(remaining);
    normal.extend(lower);
    normal.extend(upper);
    normal.push(bad);
    normal.resize(64, F);
    // A whole active ASCII prefix cannot alter a complete decoder state except
    // its cursor and normal next-byte bounds. This shortcut is itself ordinary
    // Boolean circuitry and still inspects every active byte's high bit.
    let complete = c.equal(&original[32..34], &literal(0, 2));
    let all_ascii = c.not(any_nonascii);
    let fast = c.and(complete, all_ascii);
    let mut ascii_state = next_index;
    ascii_state.extend(literal(0, 2));
    ascii_state.extend(literal(0x80, 8));
    ascii_state.extend(literal(0xbf, 8));
    ascii_state.push(original[50]);
    ascii_state.resize(64, F);
    let output = c.select(fast, &ascii_state, &normal);
    CircuitWithOutput { circuit: c, output }
}
struct CircuitWithOutput {
    circuit: Circuit,
    output: Word,
}
// Return a closed cube whose environment contains only its 64 Boolean
// arguments. Every observed leaf demands all arguments through ordinary Bool
// recursors, allowing evaluated bit suspensions to release the prior circuit
// state. No observer shortcut, function-valued recursor or new rule is used.
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
fn emit(b: &mut Builder) -> R<OrdinaryBoundaryUtf8Definition> {
    let document = emit_document(b)?;
    b.helpers(STATE)?;
    emit_state_pack(b)?;
    let mut d = OrdinaryBoundaryUtf8Definition {
        document,
        state_depth: STATE,
        block_bytes: BLOCK_BYTES,
        blocks_per_step: 2,
        scan_steps: SCAN_STEPS as u32,
        static_transformers: 0,
        scan_definition: name("Scan"),
        valid_definition: name("Valid"),
    };
    let block = block_circuit();
    let step_circuit = circuit(b, &name("BlockCircuit"), block.circuit, block.output)?;
    let mut c = Circuit::new(&[32, 64]);
    let length = c.inputs[0].clone();
    let state = c.inputs[1].clone();
    let remaining = c.lt(&state[..32], &length, false);
    let valid = c.not(state[50]);
    let active = c.and(remaining, valid);
    let active = circuit(b, &name("ActiveCircuit"), c, vec![active])?;
    let length = b.var(1)?;
    let state = b.var(0)?;
    let body = call(b, &active, vec![length, state])?;
    define(b, &name("Active"), &[5, STATE], 0, body)?;
    // Internal chunk read relies on the scan's aligned cursor invariant.
    // Offset selectors are concatenated with cursor bits 6..20; no byte copy,
    // host decoding or truncated application-string representation is involved.
    let source = b.var(10)?;
    let state = b.var(9)?;
    let mut args = vec![truth(b, true)?];
    for i in 0..6 {
        args.push(b.var(8 - i)?);
    }
    for i in 6..20 {
        args.push(core_read(b, state, i, STATE)?);
    }
    for i in 6..9 {
        args.push(b.var(8 - i)?);
    }
    let body = b.app(source, args)?;
    let body = b.wrap_selectors(9, body)?;
    define(b, &name("ReadBlock"), &[24, STATE], 9, body)?;
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
    define(b, &name("Step64"), &[24, 5, STATE], STATE, body)?;

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
    let step = invoke(b, "Step64", vec![source, length])?;
    let compose = invoke(b, "Compose", vec![length])?;
    // Count the two concrete Step64 occurrences as well as every outer
    // invocation. Circuit helpers also consume the same Builder budget.
    let pair = b.compose_term(STATE, compose, &[step, step])?;
    let body = bind_inputs(b, &[1 << 24, 32], pair)?;
    let ty = b.pi(length_ty, transformer)?;
    let document_ty = b.cube(24)?;
    let ty = b.pi(document_ty, ty)?;
    b.define(&name("Step128"), ty, body)?;
    let step = invoke(b, "Step128", vec![source, length])?;
    let pipeline = b.compose_term(STATE, compose, &vec![step; SCAN_STEPS])?;
    let zero = word(b, 0, STATE)?;
    let body = b.app(pipeline, vec![zero])?;
    define(b, &d.scan_definition, &[24, 5], STATE, body)?;
    let mut c = Circuit::new(&[64]);
    let state = c.inputs[0].clone();
    let pending = c.nonzero(&state[32..34]);
    let invalid = c.or(pending, state[50]);
    let finished = c.not(invalid);
    let finish = circuit(b, &name("Finished"), c, vec![finished])?;
    let mut c = Circuit::new(&[8, 8, 8]);
    let a = c.inputs[0].clone();
    let a = c.equal(&a, &literal(0xef, 8));
    let v = c.inputs[1].clone();
    let v = c.equal(&v, &literal(0xbb, 8));
    let z = c.inputs[2].clone();
    let z = c.equal(&z, &literal(0xbf, 8));
    let bom = c.and(a, v);
    let bom = c.and(bom, z);
    let no_bom = c.not(bom);
    let no_bom = circuit(b, &name("NoBom"), c, vec![no_bom])?;
    let source = b.var(0)?;
    let length = call(b, &d.document.length_definition, vec![source])?;
    let lets = vec![(b.cube(5)?, length)];
    let source = b.var(1)?;
    let length = b.var(0)?;
    let state = call(b, &d.scan_definition, vec![source, length])?;
    let good = call(b, &finish, vec![state])?;
    let mut prefix = vec![];
    for i in 0..3 {
        let index = word(b, i, 5)?;
        prefix.push(call(
            b,
            &d.document.read_byte_definition,
            vec![source, index],
        )?);
    }
    let no_bom = call(b, &no_bom, prefix)?;
    let no = truth(b, false)?;
    let good = core_mux(b, no_bom, good, no)?;
    let bounded = call(b, &d.document.bounded_definition, vec![source])?;
    let mut body = core_mux(b, bounded, good, no)?;
    for (ty, value) in lets.into_iter().rev() {
        body = b.term(TermNode::Let { ty, value, body })?;
    }
    define(b, &d.valid_definition, &[24], 0, body)?;
    d.static_transformers = b.static_transformers as u32;
    Ok(d)
}
pub fn generate_csharp_practical_ordinary_boundary_utf8(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryBoundaryUtf8Program> {
    let boundary = generate_boundary_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let mut b = Builder::new()?;
    let definition = if boundary.contracts().is_empty() {
        None
    } else {
        Some(emit(&mut b)?)
    };
    let certificate = b.finish()?;
    let p = OrdinaryBoundaryUtf8Program {
        schema: "mpk.csharp.ordinary_boundary_utf8.v1".into(),
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
pub fn import_csharp_practical_ordinary_boundary_utf8(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryBoundaryUtf8Program> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_boundary_utf8(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}

#[cfg(test)]
mod packing_tests {
    use super::*;

    #[test]
    fn csharp_03_t06_w09_utf8_packing_has_bounded_dependency_walk() {
        let mut b = Builder::new().unwrap();
        emit_state_pack(&mut b).unwrap();
        // Count expanded term visits without actually expanding the tree.
        // The certificate dependency collector revisits a shared argument;
        // deeply nested duplicated branches therefore must fail this check.
        let mut costs = Vec::<u128>::new();
        for term in &b.c.term_table {
            let children = match term {
                TermNode::Sort(_) | TermNode::Var(_) | TermNode::Const { .. } => vec![],
                TermNode::App {
                    function,
                    arguments,
                } => std::iter::once(*function)
                    .chain(arguments.iter().copied())
                    .collect(),
                TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => vec![*ty, *body],
                TermNode::Let { ty, value, body } => vec![*ty, *value, *body],
            };
            let cost = children.into_iter().fold(1u128, |total, child| {
                total.saturating_add(costs[child as usize])
            });
            costs.push(cost);
        }
        let index = b.globals[&name("PackState")];
        let mpk_cert::encode::DeclarationKind::Def { value, .. } =
            b.c.declarations[index as usize].kind
        else {
            panic!()
        };
        assert!(
            costs[value as usize] < 4096,
            "expanded dependency visits: {}",
            costs[value as usize]
        );
        eprintln!(
            "UTF-8 packing expanded dependency visits {}",
            costs[value as usize]
        );
    }
}
