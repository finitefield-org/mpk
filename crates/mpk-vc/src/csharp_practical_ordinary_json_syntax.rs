//! Exact closed-schema JSON lexemes. Compound value parsing/proofs remain separate.
use super::hex_codecs::{call, truth, word};
use super::integer_format::{circuit_with_block_bits, define};
use super::temporal::literal;
use super::*;
use crate::csharp_practical_source_artifacts::{
    canonical_practical_json_bytes, parse_canonical_practical_json, PracticalArtifactKind,
    PracticalJsonValue as J,
};
use sha2::{Digest, Sha256};
const NAME: &str = "Mpk.CSharp.Ordinary.JsonSyntax";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonSyntaxLiteral {
    pub utf8: Vec<u8>,
    /// C24 document, C5 absolute start -> C6 packet: valid, at-document-end, u32 end,
    /// then zero padding. Exact match consumes the literal; invalid is all zero.
    /// The end flag does not establish validity of a complete JSON value.
    pub match_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonFieldSyntax {
    pub owner: String,
    pub group: String,
    pub field_id: String,
    pub name_utf16: Vec<u16>,
    /// Matches the exact canonical quoted name followed immediately by colon.
    /// Order in this list follows the source/sidecar; presence is a later rule.
    pub match_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonSyntaxProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    boundary_program_sha256: String,
    literals: Vec<OrdinaryJsonSyntaxLiteral>,
    fields: Vec<OrdinaryJsonFieldSyntax>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryJsonSyntaxProgram {
    pub fn literals(&self) -> &[OrdinaryJsonSyntaxLiteral] {
        &self.literals
    }
    pub fn fields(&self) -> &[OrdinaryJsonFieldSyntax] {
        &self.fields
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary JSON syntax")
    }
}
fn literal_name(bytes: &[u8]) -> String {
    format!("{NAME}.Literal.H{:x}", Sha256::digest(bytes))
}
fn circuit(b: &mut Builder, id: &str, c: Circuit, out: Word) -> R<String> {
    circuit_with_block_bits(b, id, c, out, 7)
}
pub(super) fn helpers(b: &mut Builder) -> R<(String, String)> {
    // Capture the first 32 byte cells from a sliced C24 document as C8.
    let doc = b.var(8)?;
    let mut args = vec![truth(b, true)?];
    args.extend((0..5).map(|i| b.var(7 - i)).collect::<R<Vec<_>>>()?);
    args.extend(vec![truth(b, false)?; 15]);
    args.extend((5..8).map(|i| b.var(7 - i)).collect::<R<Vec<_>>>()?);
    let body = b.app(doc, args)?;
    let body = b.wrap_selectors(8, body)?;
    define(b, &format!("{NAME}.Capture32"), &[24], 8, body)?;
    let mut c = Circuit::new(&[32, 32]);
    let a = c.inputs[0].clone();
    let offset = c.inputs[1].clone();
    let end = c.add(&a, &offset, F).0;
    let add = circuit(b, &format!("{NAME}.Offset"), c, end)?;
    let mut c = Circuit::new(&[1, 32, 32, 32]);
    let matched = c.inputs[0][0];
    let length = c.inputs[1].clone();
    let start = c.inputs[2].clone();
    let count = c.inputs[3].clone();
    let bounded = c.lt(&length, &literal(1_048_577, 32), false);
    let (remaining, no_borrow) = c.sub(&length, &start);
    let excess = c.lt(&remaining, &count, false);
    let fits = c.not(excess);
    let mut good = c.and(matched, bounded);
    good = c.and(good, no_borrow);
    good = c.and(good, fits);
    let end = c.add(&start, &count, F).0;
    let whole = c.equal(&end, &length);
    let mut packet = vec![T, whole];
    packet.extend(end);
    packet.resize(64, F);
    let packet = c.select(good, &packet, &vec![F; 64]);
    let finish = circuit(b, &format!("{NAME}.Finish"), c, packet)?;
    Ok((add, finish))
}
fn conjunction(b: &mut Builder, values: &[u32]) -> R<u32> {
    match values {
        [] => truth(b, true),
        [value] => Ok(*value),
        _ => {
            let mid = values.len() / 2;
            let left = conjunction(b, &values[..mid])?;
            let right = conjunction(b, &values[mid..])?;
            call(b, "Std.Bool.and", vec![left, right])
        }
    }
}
pub(super) fn emit_literal(
    b: &mut Builder,
    fragments: &OrdinaryBoundaryFragmentDefinition,
    add: &str,
    finish: &str,
    bytes: Vec<u8>,
    chunks: &mut BTreeMap<Vec<u8>, String>,
) -> R<OrdinaryJsonSyntaxLiteral> {
    if bytes.is_empty() || bytes.len() > 1_048_576 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let mut matches = vec![];
    for (i, chunk) in bytes.chunks(32).enumerate() {
        let compare = if let Some(name) = chunks.get(chunk) {
            name.clone()
        } else {
            let mut c = Circuit::new(&[256]);
            let raw = c.inputs[0].clone();
            let mut good = T;
            for (at, byte) in chunk.iter().enumerate() {
                for bit in 0..8 {
                    let actual = raw[at | (bit << 5)];
                    let equal = if byte & (1 << bit) != 0 {
                        actual
                    } else {
                        c.not(actual)
                    };
                    good = c.and(good, equal);
                }
            }
            let name = format!("{NAME}.Chunk.H{:x}", Sha256::digest(chunk));
            let name = circuit(b, &name, c, vec![good])?;
            chunks.insert(chunk.to_vec(), name.clone());
            name
        };
        let doc = b.var(1)?;
        let start = b.var(0)?;
        let offset = word(b, (i * 32) as u32, 5)?;
        let at = call(b, add, vec![start, offset])?;
        let count = word(b, chunk.len() as u32, 5)?;
        let slice = call(b, &fragments.slice_definition, vec![doc, at, count])?;
        let captured = call(b, &format!("{NAME}.Capture32"), vec![slice])?;
        matches.push(call(b, &compare, vec![captured])?);
    }
    let matched = conjunction(b, &matches)?;
    let doc = b.var(1)?;
    let start = b.var(0)?;
    let length = call(b, &fragments.document.length_definition, vec![doc])?;
    let count = word(b, bytes.len() as u32, 5)?;
    let body = call(b, finish, vec![matched, length, start, count])?;
    let match_definition = literal_name(&bytes);
    define(b, &match_definition, &[24, 5], 6, body)?;
    Ok(OrdinaryJsonSyntaxLiteral {
        utf8: bytes,
        match_definition,
    })
}
fn field(
    owner: &str,
    group: &str,
    id: &str,
    name: &J,
    fields: &mut Vec<OrdinaryJsonFieldSyntax>,
    literals: &mut BTreeSet<Vec<u8>>,
) -> R<()> {
    let name_utf16 = match name {
        J::String(s) => s.encode_utf16().collect(),
        J::Utf16String(s) => s.clone(),
        _ => return Err(OrdinaryCarrierError::Linkage),
    };
    let mut bytes =
        canonical_practical_json_bytes(name).map_err(|_| OrdinaryCarrierError::Linkage)?;
    bytes.push(b':');
    fields.push(OrdinaryJsonFieldSyntax {
        owner: owner.into(),
        group: group.into(),
        field_id: id.into(),
        name_utf16,
        match_definition: literal_name(&bytes),
    });
    literals.insert(bytes);
    Ok(())
}
pub fn generate_csharp_practical_ordinary_json_syntax(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryJsonSyntaxProgram> {
    let boundary = generate_boundary_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let mut b = Builder::new()?;
    let mut fields = vec![];
    let mut bytes = BTreeSet::new();
    let mut literals = vec![];
    if !boundary.contracts().is_empty() {
        for punctuation in *b"{}[],:" {
            bytes.insert(vec![punctuation]);
        }
        for doc in boundary.contracts() {
            let doc = parse_canonical_practical_json(
                PracticalArtifactKind::BoundaryContract,
                doc.as_bytes(),
            )
            .map_err(|_| OrdinaryCarrierError::Linkage)?;
            let owner = doc
                .get("contract_sha256")
                .and_then(J::as_str)
                .ok_or(OrdinaryCarrierError::Linkage)?;
            for group in ["input_fields", "output_fields"] {
                for f in doc
                    .get(group)
                    .and_then(J::as_array)
                    .ok_or(OrdinaryCarrierError::Linkage)?
                {
                    field(
                        owner,
                        group,
                        f.get("field_id")
                            .and_then(J::as_str)
                            .ok_or(OrdinaryCarrierError::Linkage)?,
                        f.get("json_name").ok_or(OrdinaryCarrierError::Linkage)?,
                        &mut fields,
                        &mut bytes,
                    )?;
                }
            }
        }
        let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
        let (_, roots, _) = vir.construction_context();
        for carrier in layouts.carriers() {
            if let Some(source) = roots.source_types.get(&carrier.type_id) {
                if source.kind != SourceKind::Enum {
                    for member in &source.members {
                        field(
                            &carrier.type_id,
                            "source_members",
                            &member.name,
                            &J::string(&member.name),
                            &mut fields,
                            &mut bytes,
                        )?;
                    }
                }
            }
        }
        for entry in vir.data_closed().entries() {
            let template = entry
                .get("template_id")
                .and_then(serde_json::Value::as_str)
                .ok_or(OrdinaryCarrierError::Linkage)?;
            let owner = entry
                .get("instance_id")
                .and_then(serde_json::Value::as_str)
                .ok_or(OrdinaryCarrierError::Linkage)?;
            let names: &[&str] = match template {
                "mpk.csharp.semantic.ordered_entry.v1" | "mpk.csharp.semantic.ordered_map.v1" => {
                    &["key", "value"]
                }
                "mpk.csharp.semantic.money.v1" => &["amount", "currency"],
                "mpk.csharp.semantic.transition.v1" => &["state", "events", "response"],
                "mpk.csharp.semantic.option.v1"
                | "mpk.csharp.semantic.lookup.v1"
                | "mpk.csharp.semantic.result.v1"
                | "mpk.csharp.semantic.validation.v1"
                | "mpk.csharp.semantic.boundary_field.v1" => &["tag", "payload"],
                _ => &[],
            };
            // This is the ordered field-name vocabulary, not a claim that an
            // optional active payload is always present in an encoded value.
            for name in names {
                field(
                    owner,
                    "semantic_field_names",
                    name,
                    &J::string(*name),
                    &mut fields,
                    &mut bytes,
                )?;
            }
        }
        let document = super::boundary_document::emit(&mut b)?;
        let fragments = super::boundary_fragments::emit(&mut b, document)?;
        let (add, finish) = helpers(&mut b)?;
        let mut chunks = BTreeMap::new();
        for bytes in bytes {
            literals.push(emit_literal(
                &mut b,
                &fragments,
                &add,
                &finish,
                bytes,
                &mut chunks,
            )?);
        }
    }
    let static_transformers = b.static_transformers;
    let certificate = b.finish()?;
    let p = OrdinaryJsonSyntaxProgram {
        schema: "mpk.csharp.ordinary_json_syntax.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        boundary_program_sha256: boundary.hash(),
        literals,
        fields,
        static_transformers,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_json_syntax(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryJsonSyntaxProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_json_syntax(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::super::super::test_eval::{apply, bit, run, sparse_cube, V};
    use super::*;
    fn doc(length: u32, bytes: &[u8], offset: usize) -> V {
        let mut ones: BTreeSet<_> = (0..32)
            .filter(|i| length & (1 << i) != 0)
            .map(|i| i << 19)
            .collect();
        for (i, byte) in bytes.iter().enumerate() {
            for k in 0..8 {
                if byte & (1 << k) != 0 {
                    ones.insert(1 | ((offset + i) << 1) | (k << 21));
                }
            }
        }
        sparse_cube(24, ones)
    }
    fn number(n: u32) -> V {
        V::Cube((0..32).map(|i| n & (1 << i) != 0).collect())
    }
    #[test]
    fn json_syntax_literal_boundaries_and_complete_packets() {
        let mut b = Builder::new().unwrap();
        let document = super::super::boundary_document::emit(&mut b).unwrap();
        let fragments = super::super::boundary_fragments::emit(&mut b, document).unwrap();
        let (add, finish) = helpers(&mut b).unwrap();
        let mut chunks = BTreeMap::new();
        let mut texts = vec![
            b"{".to_vec(),
            b"\"field0\":".to_vec(),
            b"\"\\ud800\":".to_vec(),
            "\"日😀\":".as_bytes().to_vec(),
        ];
        for n in [28, 29, 30, 60, 61, 62, 254] {
            texts.push(format!("\"{}\":", "a".repeat(n)).into_bytes());
        }
        let defs = texts
            .into_iter()
            .map(|text| emit_literal(&mut b, &fragments, &add, &finish, text, &mut chunks).unwrap())
            .collect::<Vec<_>>();
        let bytes = b.finish().unwrap();
        let cert = decode_canonical_certificate(&bytes).unwrap();
        crate::csharp_practical_vc_model::validate_csharp_practical_certificate_structure(&cert)
            .unwrap();
        let mut cases = 0;
        let mut check =
            |d: &OrdinaryJsonSyntaxLiteral, input: V, start: u32, expected: Option<(bool, u32)>| {
                let value = run(&cert, &d.match_definition, vec![input, number(start)]);
                let mut wanted = [false; 64];
                if let Some((whole, end)) = expected {
                    wanted[0] = true;
                    wanted[1] = whole;
                    for i in 0..32 {
                        wanted[2 + i] = end & (1 << i) != 0;
                    }
                }
                for (i, want) in wanted.into_iter().enumerate() {
                    let mut actual = value.clone();
                    for k in 0..6 {
                        actual = apply(&cert, actual, V::Bit(i & (1 << k) != 0));
                    }
                    assert_eq!(
                        bit(actual),
                        want,
                        "case {cases}, length {}, start {start}, bit {i}",
                        d.utf8.len()
                    );
                }
                cases += 1;
            };
        for d in &defs {
            let n = d.utf8.len() as u32;
            check(d, doc(n, &d.utf8, 0), 0, Some((true, n)));
            check(d, doc(n + 2, &d.utf8, 1), 1, Some((false, n + 1)));
            let start = 1_048_576 - n;
            check(
                d,
                doc(1_048_576, &d.utf8, start as usize),
                start,
                Some((true, 1_048_576)),
            );
            check(d, doc(n - 1, &d.utf8, 0), 0, None);
            check(d, doc(n, &d.utf8, 0), u32::MAX, None);
            check(d, doc(n, &d.utf8, 0), 1_048_576, None);
            check(d, doc(u32::MAX, &d.utf8, 0), 0, None);
            for at in BTreeSet::from([0, 31, 32, 63, 64, 256, d.utf8.len() - 1]) {
                if at >= d.utf8.len() {
                    continue;
                }
                let mut changed = d.utf8.clone();
                changed[at] ^= 1;
                check(d, doc(n, &changed, 0), 0, None);
            }
        }
        let surrogate = &defs[2];
        check(surrogate, doc(9, b"\"\\uD800\":", 0), 0, None);
        eprintln!("JSON syntax: {cases} complete 64-bit packets, including chunk edges, Unicode, full cursor/document bounds and poisoned inactive bytes");
    }
}
