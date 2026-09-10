//! Exact lowercase JSON keyword decoding over the full boundary byte carrier.
//! Prefix packets are lexical results; the enclosing grammar owns delimiters.
use super::boundary_document::emit as emit_document;
use super::hex_codecs::{call, word};
use super::integer_format::{circuit, define};
use super::temporal::literal;
use super::*;

const NAME: &str = "Mpk.CSharp.Ordinary.JsonKeywords";
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonKeywordDefinition {
    pub document: OrdinaryBoundaryDocumentDefinition,
    pub fragments: OrdinaryBoundaryFragmentDefinition,
    /// C6 packet: prefix-valid, whole-valid, is-null, Boolean value,
    /// u32 consumed at bits 4..36, then zero padding. Invalid is all zero.
    /// A prefix does not validate the next delimiter or any following bytes.
    pub parse_definition: String,
    pub static_transformers: usize,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonKeywordProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    boundary_program_sha256: String,
    definition: Option<OrdinaryJsonKeywordDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryJsonKeywordProgram {
    pub fn definition(&self) -> Option<&OrdinaryJsonKeywordDefinition> {
        self.definition.as_ref()
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary JSON keywords")
    }
}

pub(super) fn emit(
    b: &mut Builder,
    fragments: OrdinaryBoundaryFragmentDefinition,
) -> R<OrdinaryJsonKeywordDefinition> {
    let document = fragments.document.clone();
    let mut c = Circuit::new(&[32, 8, 8, 8, 8, 8]);
    let length = c.inputs[0].clone();
    let bytes = c.inputs[1..].to_vec();
    let bounded = c.lt(&length, &literal(1_048_577, 32), false);
    let mut output = literal(0, 64);
    for (token, is_null, boolean) in [
        (b"null".as_slice(), T, F),
        (b"false".as_slice(), F, F),
        (b"true".as_slice(), F, T),
    ] {
        let n = token.len() as u128;
        let short = c.lt(&length, &literal(n, 32), false);
        let enough = c.not(short);
        let mut valid = c.and(bounded, enough);
        for (byte, &expected) in bytes.iter().zip(token) {
            let equal = c.equal(byte, &literal(expected as u128, 8));
            valid = c.and(valid, equal);
        }
        let whole = c.equal(&length, &literal(n, 32));
        let mut packet = vec![T, whole, is_null, boolean];
        packet.extend(literal(n, 32));
        packet.resize(64, F);
        output = c.select(valid, &packet, &output);
    }
    let parse = circuit(b, &format!("{NAME}.Packet"), c, output)?;
    let doc = b.var(0)?;
    let length = call(b, &document.length_definition, vec![doc])?;
    let mut args = vec![length];
    for i in 0..5 {
        let index = word(b, i, 5)?;
        args.push(call(b, &document.read_byte_definition, vec![doc, index])?);
    }
    let body = call(b, &parse, args)?;
    let parse_definition = format!("{NAME}.Parse");
    define(b, &parse_definition, &[24], 6, body)?;
    Ok(OrdinaryJsonKeywordDefinition {
        document,
        fragments,
        parse_definition,
        static_transformers: b.static_transformers,
    })
}
pub fn generate_csharp_practical_ordinary_json_keywords(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryJsonKeywordProgram> {
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
    let p = OrdinaryJsonKeywordProgram {
        schema: "mpk.csharp.ordinary_json_keywords.v1".into(),
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
pub fn import_csharp_practical_ordinary_json_keywords(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryJsonKeywordProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_json_keywords(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
