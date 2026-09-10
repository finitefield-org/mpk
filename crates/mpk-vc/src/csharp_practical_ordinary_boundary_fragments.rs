//! Bounded byte-document slicing and concatenation for typed JSON composition.
//! These preserve bytes; JSON syntax, UTF-8 and typed-value validity are separate.
use super::boundary_document::emit as emit_document;
use super::hex_codecs::{call, truth};
use super::integer_format::{circuit, define};
use super::temporal::literal;
use super::*;

const NAME: &str = "Mpk.CSharp.Ordinary.BoundaryFragments";
const DEPTH: u32 = 24;
const CAPACITY: u128 = 1_048_576;
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryBoundaryFragmentDefinition {
    pub document: OrdinaryBoundaryDocumentDefinition,
    pub slice_valid_definition: String,
    pub slice_definition: String,
    pub concat_valid_definition: String,
    pub concat_definition: String,
    pub static_transformers: usize,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryBoundaryFragmentProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    boundary_program_sha256: String,
    definition: Option<OrdinaryBoundaryFragmentDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryBoundaryFragmentProgram {
    pub fn definition(&self) -> Option<&OrdinaryBoundaryFragmentDefinition> {
        self.definition.as_ref()
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary boundary fragments")
    }
}
fn name(suffix: &str) -> String {
    format!("{NAME}.{suffix}")
}
// Twenty byte-address selectors followed by three bit selectors are in scope.
// Reify the full u32 index, adding five local selectors for its result cube.
fn byte_index(b: &mut Builder) -> R<u32> {
    let bits = (0..20).map(|i| b.var(27 - i)).collect::<R<Vec<_>>>()?;
    let zero = truth(b, false)?;
    let value = core_select(b, &bits, 5, 0, 5, zero)?;
    b.wrap_selectors(5, value)
}
fn byte_leaf(b: &mut Builder, read: &str, doc: u32, index: u32) -> R<u32> {
    let byte = call(b, read, vec![doc, index])?;
    let selectors = b.selectors(3)?;
    b.app(byte, selectors)
}
pub(super) fn emit(
    b: &mut Builder,
    document: OrdinaryBoundaryDocumentDefinition,
) -> R<OrdinaryBoundaryFragmentDefinition> {
    if !b
        .globals
        .contains_key(&format!("{PREFIX}.Cube.D{DEPTH}.Compose"))
    {
        b.helpers(DEPTH)?;
    }
    let mut d = OrdinaryBoundaryFragmentDefinition {
        document,
        slice_valid_definition: name("SliceValid"),
        slice_definition: name("Slice"),
        concat_valid_definition: name("ConcatValid"),
        concat_definition: name("Concat"),
        static_transformers: 0,
    };
    let mut c = Circuit::new(&[32, 32, 32]);
    let length = c.inputs[0].clone();
    let start = c.inputs[1].clone();
    let count = c.inputs[2].clone();
    let bounded = c.lt(&length, &literal(CAPACITY + 1, 32), false);
    let past_end = c.lt(&length, &start, false);
    let start_valid = c.not(past_end);
    let remaining = c.sub(&length, &start).0;
    let too_long = c.lt(&remaining, &count, false);
    let count_valid = c.not(too_long);
    let valid = c.and(bounded, start_valid);
    let valid = c.and(valid, count_valid);
    let slice_bound = circuit(b, &name("SliceBound"), c, vec![valid])?;
    let doc = b.var(2)?;
    let start = b.var(1)?;
    let count = b.var(0)?;
    let length = call(b, &d.document.length_definition, vec![doc])?;
    let body = call(b, &slice_bound, vec![length, start, count])?;
    define(b, &d.slice_valid_definition, &[DEPTH, 5, 5], 0, body)?;

    let mut c = Circuit::new(&[32, 32]);
    let left = c.inputs[0].clone();
    let right = c.inputs[1].clone();
    let sum = c.add(&left, &right, F).0;
    let add = circuit(b, &name("AddIndex"), c, sum)?;
    let mut c = Circuit::new(&[32, 32]);
    let left = c.inputs[0].clone();
    let right = c.inputs[1].clone();
    let difference = c.sub(&left, &right).0;
    let subtract = circuit(b, &name("SubtractIndex"), c, difference)?;
    let mut c = Circuit::new(&[32, 32]);
    let left = c.inputs[0].clone();
    let right = c.inputs[1].clone();
    let less = c.lt(&left, &right, false);
    let less = circuit(b, &name("LessIndex"), c, vec![less])?;

    // Construct an address function, not a host copy or a scan with smaller bounds.
    let doc = b.var(25)?;
    let start = b.var(24)?;
    let index = byte_index(b)?;
    let absolute = call(b, &add, vec![start, index])?;
    let leaf = byte_leaf(b, &d.document.read_byte_definition, doc, absolute)?;
    let bytes = b.wrap_selectors(23, leaf)?;
    let doc = b.var(2)?;
    let start = b.var(1)?;
    let count = b.var(0)?;
    let valid = call(b, &d.slice_valid_definition, vec![doc, start, count])?;
    let value = call(b, &d.document.make_definition, vec![count, bytes])?;
    let zero = b.constant(&format!("{PREFIX}.Cube.D24.Zero"))?;
    let body = call(
        b,
        &format!("{PREFIX}.Cube.D24.Mux"),
        vec![valid, value, zero],
    )?;
    define(b, &d.slice_definition, &[DEPTH, 5, 5], DEPTH, body)?;

    let mut c = Circuit::new(&[32, 32]);
    let left = c.inputs[0].clone();
    let right = c.inputs[1].clone();
    let left_valid = c.lt(&left, &literal(CAPACITY + 1, 32), false);
    let right_valid = c.lt(&right, &literal(CAPACITY + 1, 32), false);
    let (sum, carry) = c.add(&left, &right, F);
    let sum_valid = c.lt(&sum, &literal(CAPACITY + 1, 32), false);
    let no_carry = c.not(carry);
    let valid = c.and(left_valid, right_valid);
    let valid = c.and(valid, sum_valid);
    let valid = c.and(valid, no_carry);
    let concat_bound = circuit(b, &name("ConcatBound"), c, vec![valid])?;
    let left = b.var(1)?;
    let right = b.var(0)?;
    let left_len = call(b, &d.document.length_definition, vec![left])?;
    let right_len = call(b, &d.document.length_definition, vec![right])?;
    let body = call(b, &concat_bound, vec![left_len, right_len])?;
    define(b, &d.concat_valid_definition, &[DEPTH, DEPTH], 0, body)?;

    let left = b.var(24)?;
    let right = b.var(23)?;
    let left_len = call(b, &d.document.length_definition, vec![left])?;
    let index = byte_index(b)?;
    let in_left = call(b, &less, vec![index, left_len])?;
    let right_index = call(b, &subtract, vec![index, left_len])?;
    let left_leaf = byte_leaf(b, &d.document.read_byte_definition, left, index)?;
    let right_leaf = byte_leaf(b, &d.document.read_byte_definition, right, right_index)?;
    let leaf = core_mux(b, in_left, left_leaf, right_leaf)?;
    let bytes = b.wrap_selectors(23, leaf)?;
    let left = b.var(1)?;
    let right = b.var(0)?;
    let left_len = call(b, &d.document.length_definition, vec![left])?;
    let right_len = call(b, &d.document.length_definition, vec![right])?;
    let length = call(b, &add, vec![left_len, right_len])?;
    let valid = call(b, &d.concat_valid_definition, vec![left, right])?;
    let value = call(b, &d.document.make_definition, vec![length, bytes])?;
    let body = call(
        b,
        &format!("{PREFIX}.Cube.D24.Mux"),
        vec![valid, value, zero],
    )?;
    define(b, &d.concat_definition, &[DEPTH, DEPTH], DEPTH, body)?;
    d.static_transformers = b.static_transformers;
    Ok(d)
}
pub fn generate_csharp_practical_ordinary_boundary_fragments(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryBoundaryFragmentProgram> {
    let boundary = generate_boundary_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let mut b = Builder::new()?;
    let definition = if boundary.contracts().is_empty() {
        None
    } else {
        let document = emit_document(&mut b)?;
        Some(emit(&mut b, document)?)
    };
    let certificate = b.finish()?;
    let p = OrdinaryBoundaryFragmentProgram {
        schema: "mpk.csharp.ordinary_boundary_fragments.v1".into(),
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
pub fn import_csharp_practical_ordinary_boundary_fragments(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryBoundaryFragmentProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_boundary_fragments(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
