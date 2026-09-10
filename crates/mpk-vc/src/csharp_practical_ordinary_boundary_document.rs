//! Private UTF-8 byte storage for complete boundary documents, not C# strings.
use super::super::super::boundary_vc::BOUNDARY_DOCUMENT_TYPE;
use super::hex_codecs::{call, truth};
use super::integer_format::{circuit, define};
use super::temporal::literal;
use super::*;

const DEPTH: u32 = 24;
const CAPACITY: u32 = 1_048_576;
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryBoundaryDocumentDefinition {
    /// Actual ordinary core type declaration; never a registered source value ID.
    pub type_name: String,
    pub depth: u32,
    pub shape: OrdinaryShape,
    pub length_definition: String,
    pub bounded_definition: String,
    pub read_byte_definition: String,
    pub make_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryBoundaryDocumentProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    boundary_program_sha256: String,
    contract_ids: Vec<String>,
    definition: Option<OrdinaryBoundaryDocumentDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryBoundaryDocumentProgram {
    pub fn definition(&self) -> Option<&OrdinaryBoundaryDocumentDefinition> {
        self.definition.as_ref()
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary boundary documents")
    }
}
pub(super) fn emit(b: &mut Builder) -> R<OrdinaryBoundaryDocumentDefinition> {
    let shape = OrdinaryShape::Sequence {
        capacity: CAPACITY,
        element: Box::new(OrdinaryShape::Bits { width: 8 }),
    };
    let d = OrdinaryBoundaryDocumentDefinition {
        type_name: BOUNDARY_DOCUMENT_TYPE.into(),
        depth: DEPTH,
        shape,
        length_definition: format!("{BOUNDARY_DOCUMENT_TYPE}.Length"),
        bounded_definition: format!("{BOUNDARY_DOCUMENT_TYPE}.Bounded"),
        read_byte_definition: format!("{BOUNDARY_DOCUMENT_TYPE}.ReadByte"),
        make_definition: format!("{BOUNDARY_DOCUMENT_TYPE}.Make"),
    };
    let ty = b.cube(DEPTH)?;
    b.define(BOUNDARY_DOCUMENT_TYPE, b.sort, ty)?;
    let document = b.var(5)?;
    let zero = truth(b, false)?;
    let mut args = vec![zero; 19];
    args.extend(b.selectors(5)?);
    let body = b.app(document, args)?;
    let body = b.wrap_selectors(5, body)?;
    define(b, &d.length_definition, &[DEPTH], 5, body)?;
    let mut c = Circuit::new(&[32]);
    let len = c.inputs[0].clone();
    let good = c.lt(&len, &literal(CAPACITY as u128 + 1, 32), false);
    let bounded = circuit(
        b,
        &format!("{BOUNDARY_DOCUMENT_TYPE}.LengthBound"),
        c,
        vec![good],
    )?;
    let document = b.var(0)?;
    let length = call(b, &d.length_definition, vec![document])?;
    let body = call(b, &bounded, vec![length])?;
    define(b, &d.bounded_definition, &[DEPTH], 0, body)?;
    let mut c = Circuit::new(&[32, 32]);
    let index = c.inputs[0].clone();
    let length = c.inputs[1].clone();
    let within_storage = c.lt(&index, &literal(CAPACITY as u128, 32), false);
    let active = c.lt(&index, &length, false);
    let active = c.and(within_storage, active);
    let active = circuit(
        b,
        &format!("{BOUNDARY_DOCUMENT_TYPE}.Active"),
        c,
        vec![active],
    )?;
    let document = b.var(4)?;
    let index = b.var(3)?;
    let length = call(b, &d.length_definition, vec![document])?;
    let visible = call(b, &active, vec![index, length])?;
    let mut args = vec![truth(b, true)?];
    for i in 0..20 {
        args.push(core_read(b, index, i, 5)?);
    }
    args.extend(b.selectors(3)?);
    let byte = b.app(document, args)?;
    let body = core_mux(b, visible, byte, zero)?;
    let body = b.wrap_selectors(3, body)?;
    define(b, &d.read_byte_definition, &[DEPTH, 5], 3, body)?;
    // Make preserves all 32 length bits, including an over-bound sentinel.
    // Bounded is a separate required predicate. It zeroes every inactive byte
    // and all header padding; byte storage has twenty index and three bit slots.
    let length = b.var(25)?;
    let bytes = b.var(24)?;
    let mut args = b.selectors(24)?;
    args.remove(0);
    let raw = b.app(bytes, args)?;
    let index_bits = (0..20).map(|i| b.var(27 - i)).collect::<R<Vec<_>>>()?;
    let index = core_select(b, &index_bits, 5, 0, 5, zero)?;
    let index = b.wrap_selectors(5, index)?;
    let visible = call(b, &active, vec![index, length])?;
    let data = core_mux(b, visible, raw, zero)?;
    let args = (0..5).rev().map(|i| b.var(i)).collect::<R<Vec<_>>>()?;
    let mut header = b.app(length, args)?;
    for i in 1..19 {
        let selector = b.var(23 - i)?;
        header = core_mux(b, selector, zero, header)?;
    }
    let role = b.var(23)?;
    let body = core_mux(b, role, data, header)?;
    let body = b.wrap_selectors(DEPTH, body)?;
    define(b, &d.make_definition, &[5, 23], DEPTH, body)?;
    Ok(d)
}
pub fn generate_csharp_practical_ordinary_boundary_documents(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryBoundaryDocumentProgram> {
    let boundary = generate_boundary_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let mut b = Builder::new()?;
    let definition = if boundary.contracts().is_empty() {
        None
    } else {
        Some(emit(&mut b)?)
    };
    let contract_ids = boundary
        .sequents()
        .iter()
        .filter(|s| s.kind == "input_acceptance")
        .map(|s| {
            s.id.strip_prefix("boundary.input_acceptance.")
                .map(str::to_owned)
                .ok_or(OrdinaryCarrierError::Linkage)
        })
        .collect::<R<Vec<_>>>()?;
    let certificate = b.finish()?;
    let p = OrdinaryBoundaryDocumentProgram {
        schema: "mpk.csharp.ordinary_boundary_documents.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        boundary_program_sha256: boundary.hash(),
        contract_ids,
        definition,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_boundary_documents(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryBoundaryDocumentProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_boundary_documents(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
