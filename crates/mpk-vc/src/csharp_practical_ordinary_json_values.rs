//! Typed primitive JSON results for subsequent compound grammar composition.
//! These ordinary definitions preserve the complete reconstructed value carrier.
use super::hex_codecs::{call, truth};
use super::integer_format::{circuit_with_block_bits, define};
use super::*;

const NAME: &str = "Mpk.CSharp.Ordinary.JsonValues";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonValueDefinition {
    pub carrier: OrdinaryCarrier,
    pub codec_id: String,
    pub scale: Option<u8>,
    pub rounding: Option<String>,
    pub token_definition: String,
    /// C24 document, C5 absolute start, C3 ending (0=EOF, 1=',', 2=']', 3='}').
    /// Result selector zero chooses a zero-padded C7 header: valid, at EOF,
    /// absolute u32 end, u32 logical cell count, then zero padding. Selector
    /// one chooses the complete carrier, also zero-padded. Invalid is all zero.
    /// Scalars contribute one cell; strings add their complete UTF-16 unit count.
    pub parse_definition: String,
    pub packet_depth: u32,
    pub header_definition: String,
    pub value_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonValueProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    boundary_program_sha256: String,
    definitions: Vec<OrdinaryJsonValueDefinition>,
    /// Complete remaining carrier IDs, rather than a claim that primitive
    /// adapters implement compound schemas, source enums or error vocabularies.
    deferred_type_ids: Vec<String>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryJsonValueProgram {
    pub fn definitions(&self) -> &[OrdinaryJsonValueDefinition] {
        &self.definitions
    }
    pub fn deferred_type_ids(&self) -> &[String] {
        &self.deferred_type_ids
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary typed JSON values")
    }
}

pub(super) fn projection(b: &mut Builder, depth: u32, value_depth: u32, role: bool) -> R<String> {
    let name = format!(
        "{NAME}.Packet.D{depth}.Project.D{value_depth}.R{}",
        u8::from(role)
    );
    if !b.globals.contains_key(&name) {
        let packet = b.var(value_depth)?;
        let mut selectors = vec![truth(b, role)?];
        selectors.extend(b.selectors(value_depth)?);
        selectors.resize(depth as usize, truth(b, false)?);
        let leaf = b.app(packet, selectors)?;
        let body = b.wrap_selectors(value_depth, leaf)?;
        define(b, &name, &[depth], value_depth, body)?;
    }
    Ok(name)
}

fn convert(
    b: &mut Builder,
    token_depth: u32,
    value_depth: u32,
    width: u32,
    end: usize,
) -> R<String> {
    let name = format!("{NAME}.Convert.T{token_depth}.D{value_depth}.W{width}.E{end}");
    if b.globals.contains_key(&name) {
        return Ok(name);
    }
    if width > 1 << value_depth || 2 + width as usize > end || end + 32 > 1 << token_depth {
        return Err(OrdinaryCarrierError::Shape);
    }
    let mut c = Circuit::new(&[1 << token_depth]);
    let raw = c.inputs[0].clone();
    let packet_depth = value_depth.max(7) + 1;
    let mut out = vec![F; 1 << packet_depth];
    out[0] = T;
    out[2] = raw[1];
    for i in 0..32 {
        out[(2 + i) << 1] = raw[end + i];
    }
    out[34 << 1] = T;
    for i in 0..width as usize {
        out[1 | (i << 1)] = raw[2 + i];
    }
    let out = c.select(raw[0], &out, &vec![F; out.len()]);
    let generated = circuit_with_block_bits(b, &name, c, out, 7)?;
    let raw = b.var(0)?;
    let body = call(b, &generated, vec![raw])?;
    define(b, &name, &[token_depth], packet_depth, body)?;
    Ok(name)
}

fn embedded_leaf(b: &mut Builder, value: u32, depth: u32, padded: u32) -> R<u32> {
    let selectors = (0..depth)
        .map(|i| b.var(padded - 1 - i))
        .collect::<R<Vec<_>>>()?;
    let mut leaf = b.app(value, selectors)?;
    for i in depth..padded {
        let selector = b.var(padded - 1 - i)?;
        let zero = truth(b, false)?;
        leaf = core_mux(b, selector, zero, leaf)?;
    }
    Ok(leaf)
}

fn string_assembly(b: &mut Builder) -> R<String> {
    let name = format!("{NAME}.String.Assemble");
    if b.globals.contains_key(&name) {
        return Ok(name);
    }
    let head = b.var(21)?;
    let text = b.var(20)?;
    let valid = core_read(b, head, 0, 7)?;
    let head_leaf = embedded_leaf(b, head, 7, 19)?;
    let text_leaf = embedded_leaf(b, text, 19, 19)?;
    let role = b.var(19)?;
    let leaf = core_mux(b, role, text_leaf, head_leaf)?;
    let zero = truth(b, false)?;
    let leaf = core_mux(b, valid, leaf, zero)?;
    let body = b.wrap_selectors(20, leaf)?;
    define(b, &name, &[7, 19], 20, body)?;
    Ok(name)
}

fn string_header(b: &mut Builder) -> R<String> {
    let mut c = Circuit::new(&[128, 8, 32]);
    let raw = c.inputs[0].clone();
    let ending = c.inputs[1].clone();
    let units = c.inputs[2].clone();
    let mut value_ending = T;
    for bit in &ending[2..] {
        let zero = c.not(*bit);
        value_ending = c.and(value_ending, zero);
    }
    let valid = c.and(raw[0], value_ending);
    let bounded = c.lt(&units, &super::temporal::literal(16_385, 32), false);
    let valid = c.and(valid, bounded);
    let cells = c.add(&units, &super::temporal::literal(1, 32), F).0;
    let mut header = vec![T, raw[1]];
    header.extend(&raw[2..34]);
    header.extend(cells);
    header.resize(128, F);
    let header = c.select(valid, &header, &vec![F; 128]);
    circuit_with_block_bits(b, &format!("{NAME}.String.Header"), c, header, 7)
}

fn emit_string(b: &mut Builder, tokens: &OrdinaryJsonTokenDefinition) -> R<String> {
    let finish = string_header(b)?;
    let assemble = string_assembly(b)?;
    let length = string_length(b)?;
    let doc = b.var(2)?;
    let start = b.var(1)?;
    let ending = b.var(0)?;
    let frame = call(b, &tokens.string_frame_definition, vec![doc, start, ending])?;
    let bound_frame = b.var(0)?;
    let ending = b.var(1)?;
    let head = call(b, &tokens.string_frame_header_definition, vec![bound_frame])?;
    let text = call(b, &tokens.string_frame_value_definition, vec![bound_frame])?;
    let units = call(b, &length, vec![text])?;
    let head = call(b, &finish, vec![head, ending, units])?;
    let body = call(b, &assemble, vec![head, text])?;
    let ty = b.cube(20)?;
    let body = b.term(TermNode::Let {
        ty,
        value: frame,
        body,
    })?;
    let name = format!("{NAME}.String.Parse");
    define(b, &name, &[24, 5, 3], 20, body)?;
    Ok(name)
}

fn string_length(b: &mut Builder) -> R<String> {
    let name = format!("{NAME}.String.Length");
    let text = b.var(5)?;
    let mut selectors = vec![truth(b, false)?; 14];
    selectors.extend(b.selectors(5)?);
    let leaf = b.app(text, selectors)?;
    let body = b.wrap_selectors(5, leaf)?;
    define(b, &name, &[19], 5, body)?;
    Ok(name)
}

#[allow(clippy::too_many_arguments)]
fn adapter(
    b: &mut Builder,
    carrier: &OrdinaryCarrier,
    codec_id: String,
    scale: Option<u8>,
    rounding: Option<String>,
    token: &str,
    token_depth: u32,
    width: u32,
    end: usize,
) -> R<OrdinaryJsonValueDefinition> {
    let converted = convert(b, token_depth, carrier.depth, width, end)?;
    let name = format!("{NAME}.From.{token}");
    let doc = b.var(2)?;
    let start = b.var(1)?;
    let ending = b.var(0)?;
    let raw = call(b, token, vec![doc, start, ending])?;
    let body = call(b, &converted, vec![raw])?;
    let depth = carrier.depth.max(7) + 1;
    define(b, &name, &[24, 5, 3], depth, body)?;
    Ok(OrdinaryJsonValueDefinition {
        carrier: carrier.clone(),
        codec_id,
        scale,
        rounding,
        token_definition: token.into(),
        parse_definition: name,
        packet_depth: depth,
        header_definition: projection(b, depth, 7, false)?,
        value_definition: projection(b, depth, carrier.depth, true)?,
    })
}

pub(super) fn emit(
    b: &mut Builder,
    tokens: &OrdinaryJsonTokenDefinition,
    carriers: &[OrdinaryCarrier],
) -> R<Vec<OrdinaryJsonValueDefinition>> {
    let mut definitions = vec![];
    for carrier in carriers {
        let Some(kind) = carrier
            .type_id
            .strip_prefix("mpk.csharp.value.")
            .and_then(|s| s.strip_suffix(".v1"))
        else {
            continue;
        };
        let raw_kind = if kind == "unit" { "null" } else { kind };
        if let Some(d) = tokens
            .scalars
            .iter()
            .chain(&tokens.quoted_scalars)
            .find(|d| d.kind == raw_kind)
        {
            let width = scalar_width(kind).ok_or(OrdinaryCarrierError::Shape)?;
            if carrier.shape != (OrdinaryShape::Bits { width })
                || carrier.depth != address_bits(width)
            {
                return Err(OrdinaryCarrierError::Shape);
            }
            let codec = match kind {
                "unit" | "bool" | "char" => format!("json.{raw_kind}"),
                "duration" => "duration_ticks".into(),
                "instant" => "unix_milliseconds".into(),
                _ => format!("integer.{kind}"),
            };
            definitions.push(adapter(
                b,
                carrier,
                codec,
                None,
                None,
                &d.parse_definition,
                7,
                width,
                66,
            )?);
        } else if kind == "string" {
            if carrier.depth != 19
                || carrier.shape
                    != (OrdinaryShape::Sequence {
                        capacity: 16_384,
                        element: Box::new(OrdinaryShape::Bits { width: 16 }),
                    })
            {
                return Err(OrdinaryCarrierError::Shape);
            }
            let parse = emit_string(b, tokens)?;
            definitions.push(OrdinaryJsonValueDefinition {
                carrier: carrier.clone(),
                codec_id: "json.string".into(),
                scale: None,
                rounding: None,
                token_definition: tokens.string_frame_definition.clone(),
                parse_definition: parse,
                packet_depth: 20,
                header_definition: projection(b, 20, 7, false)?,
                value_definition: projection(b, 20, 19, true)?,
            });
        } else {
            for d in tokens
                .quoted_codecs
                .iter()
                .filter(|d| d.codec.value_type_id == carrier.type_id)
            {
                let width = scalar_width(kind).ok_or(OrdinaryCarrierError::Shape)?;
                if carrier.depth != d.codec.value_depth
                    || carrier.shape != (OrdinaryShape::Bits { width })
                {
                    return Err(OrdinaryCarrierError::Shape);
                }
                definitions.push(adapter(
                    b,
                    carrier,
                    d.codec.codec_id.clone(),
                    None,
                    None,
                    &d.parse_definition,
                    8,
                    width,
                    130,
                )?);
            }
            for d in tokens
                .quoted_decimals
                .iter()
                .filter(|d| d.codec.value_type_id == carrier.type_id)
            {
                if carrier.depth != 9 || carrier.depth != d.codec.value_depth {
                    return Err(OrdinaryCarrierError::Shape);
                }
                definitions.push(adapter(
                    b,
                    carrier,
                    d.codec.codec_id.clone(),
                    d.codec.scale,
                    d.codec.rounding.clone(),
                    &d.parse_definition,
                    10,
                    512,
                    514,
                )?);
            }
            for d in tokens
                .quoted_calendars
                .iter()
                .filter(|d| d.codec.value_type_id == carrier.type_id)
            {
                let width = scalar_width(kind).ok_or(OrdinaryCarrierError::Shape)?;
                if carrier.depth != d.codec.value_depth
                    || carrier.shape != (OrdinaryShape::Bits { width })
                {
                    return Err(OrdinaryCarrierError::Shape);
                }
                definitions.push(adapter(
                    b,
                    carrier,
                    d.codec.codec_id.clone(),
                    None,
                    None,
                    &d.parse_definition,
                    7,
                    width,
                    66,
                )?);
            }
        }
    }
    Ok(definitions)
}

pub fn generate_csharp_practical_ordinary_json_values(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryJsonValueProgram> {
    let boundary = generate_boundary_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut b = Builder::new()?;
    let definitions = if boundary.contracts().is_empty() {
        vec![]
    } else {
        let tokens = super::emit_boundary_json_for_carriers(&mut b, layouts.carriers())?;
        emit(&mut b, &tokens, layouts.carriers())?
    };
    let defined = definitions
        .iter()
        .map(|d| d.carrier.type_id.as_str())
        .collect::<BTreeSet<_>>();
    let deferred_type_ids = layouts
        .carriers()
        .iter()
        .filter(|c| !defined.contains(c.type_id.as_str()))
        .map(|c| c.type_id.clone())
        .collect();
    let static_transformers = b.static_transformers;
    let certificate = b.finish()?;
    let p = OrdinaryJsonValueProgram {
        schema: "mpk.csharp.ordinary_json_values.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        boundary_program_sha256: boundary.hash(),
        definitions,
        deferred_type_ids,
        static_transformers,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_json_values(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryJsonValueProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_json_values(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::super::super::test_eval::{apply, bit, run, sparse_cube, V};
    use super::*;

    fn at(cert: &mpk_cert::encode::Certificate, v: &V, depth: u32, i: usize) -> bool {
        let mut value = v.clone();
        for k in 0..depth {
            value = apply(cert, value, V::Bit(i & (1 << k) != 0));
        }
        bit(value)
    }

    #[test]
    fn json_value_packets_preserve_complete_carriers_and_clear_invalid_storage() {
        let mut b = Builder::new().unwrap();
        let mut definitions = vec![];
        for (token, depth, width, end) in [
            (7, 0, 0, 66),
            (7, 0, 1, 66),
            (7, 3, 8, 66),
            (7, 4, 16, 66),
            (7, 5, 32, 66),
            (7, 6, 64, 66),
            (8, 5, 32, 130),
            (8, 6, 64, 130),
            (8, 7, 128, 130),
            (10, 9, 512, 514),
        ] {
            let name = convert(&mut b, token, depth, width, end).unwrap();
            let before = (b.c.declarations.len(), b.static_transformers);
            assert_eq!(convert(&mut b, token, depth, width, end).unwrap(), name);
            assert_eq!((b.c.declarations.len(), b.static_transformers), before);
            let packet_depth = depth.max(7) + 1;
            let head = projection(&mut b, packet_depth, 7, false).unwrap();
            let value = projection(&mut b, packet_depth, depth, true).unwrap();
            definitions.push((token, depth, width, end, name, head, value));
        }
        let bytes = b.finish().unwrap();
        let cert = decode_canonical_certificate(&bytes).unwrap();
        crate::csharp_practical_vc_model::validate_csharp_practical_certificate_structure(&cert)
            .unwrap();
        let mut packets = 0;
        for (token, depth, width, end, name, head_name, value_name) in definitions {
            for (valid, eof, cursor) in [
                (false, true, u32::MAX),
                (true, false, 0),
                (true, true, 1_048_576),
                (true, false, 0x8123_4567),
            ] {
                // Poison every unused token bit, including upper scalar payload bits.
                let mut raw = vec![true; 1 << token];
                raw[0] = valid;
                raw[1] = eof;
                for i in 0..width as usize {
                    raw[2 + i] = i % 3 != 1;
                }
                for i in 0..32 {
                    raw[end + i] = cursor & (1 << i) != 0;
                }
                let parsed = run(&cert, &name, vec![V::Cube(raw.clone())]);
                let packet_depth = depth.max(7) + 1;
                let mut expected = vec![false; 1 << packet_depth];
                if valid {
                    expected[0] = true;
                    expected[2] = eof;
                    expected[34 << 1] = true;
                    for i in 0..32 {
                        expected[(2 + i) << 1] = raw[end + i];
                    }
                    for i in 0..width as usize {
                        expected[1 | (i << 1)] = raw[2 + i];
                    }
                }
                for (i, want) in expected.iter().enumerate() {
                    assert_eq!(
                        at(&cert, &parsed, packet_depth, i),
                        *want,
                        "{name} valid={valid} bit={i}"
                    );
                }
                let head = run(&cert, &head_name, vec![parsed.clone()]);
                let value = run(&cert, &value_name, vec![parsed]);
                for i in 0..128 {
                    assert_eq!(at(&cert, &head, 7, i), expected[i << 1]);
                }
                for i in 0..1 << depth {
                    assert_eq!(at(&cert, &value, depth, i), expected[1 | (i << 1)]);
                }
                packets += 1;
            }
        }
        eprintln!("Typed JSON: {packets} complete scalar packets plus exact header/value projections, including complete C9 decimal storage");
    }

    #[test]
    fn json_string_semantic_cells_match_utf16_length() {
        let mut b = Builder::new().unwrap();
        let finish = string_header(&mut b).unwrap();
        let length = string_length(&mut b).unwrap();
        let bytes = b.finish().unwrap();
        if let Some(path) = std::env::var_os("MPK_W09_JSON_STRING_CELLS_CERT_OUT") {
            std::fs::write(path, &bytes).unwrap();
        }
        let cert = decode_canonical_certificate(&bytes).unwrap();
        crate::csharp_practical_vc_model::validate_csharp_practical_certificate_structure(&cert)
            .unwrap();
        let mut cases = 0;
        for units in [
            0_u32,
            1,
            2,
            3,
            16_383,
            16_384,
            16_385,
            65_536,
            1 << 31,
            u32::MAX,
        ] {
            let ones = (0..32)
                .filter(|i| units & (1 << i) != 0)
                .map(|i| i << 14)
                .collect();
            let count = run(&cert, &length, vec![sparse_cube(19, ones)]);
            for i in 0..32 {
                assert_eq!(at(&cert, &count, 5, i), units & (1 << i) != 0);
            }
            for valid in [false, true] {
                for ending in [0_u8, 1, 2, 3, 4, 128] {
                    let mut raw = vec![false; 128];
                    raw[0] = valid;
                    raw[1] = ending == 0;
                    for i in 0..32 {
                        raw[2 + i] = 73_u32 & (1 << i) != 0;
                    }
                    let ending_bits = V::Cube((0..8).map(|i| ending & (1 << i) != 0).collect());
                    let head = run(
                        &cert,
                        &finish,
                        vec![V::Cube(raw), ending_bits, count.clone()],
                    );
                    let good = valid && ending < 4 && units <= 16_384;
                    for i in 0..128 {
                        let expected = good
                            && match i {
                                0 => true,
                                1 => ending == 0,
                                2..=33 => 73_u32 & (1 << (i - 2)) != 0,
                                34..=65 => units.wrapping_add(1) & (1 << (i - 34)) != 0,
                                _ => false,
                            };
                        assert_eq!(
                            at(&cert, &head, 7, i),
                            expected,
                            "units{units} ending{ending} valid{valid} bit{i}"
                        );
                    }
                    cases += 1;
                }
            }
        }
        eprintln!("JSON string cells: {cases} complete headers plus all32 stored length bits");
    }

    #[test]
    fn json_string_value_packet_preserves_full_capacity_and_rejects_field_endings() {
        let mut b = Builder::new().unwrap();
        let finish = string_header(&mut b).unwrap();
        let assemble = string_assembly(&mut b).unwrap();
        let head_name = projection(&mut b, 20, 7, false).unwrap();
        let value_name = projection(&mut b, 20, 19, true).unwrap();
        let bytes = b.finish().unwrap();
        let cert = decode_canonical_certificate(&bytes).unwrap();
        crate::csharp_practical_vc_model::validate_csharp_practical_certificate_structure(&cert)
            .unwrap();
        let ones = BTreeSet::from([0, 1, 7, 31, 32, 16_383, 16_384, 262_144, 524_286, 524_287]);
        let value = sparse_cube(19, ones.clone());
        for ending in [0, 1, 2, 3, 4, 5, 8, 16, 32, 64, 128, 255] {
            for valid in [false, true] {
                let mut raw = vec![true; 128];
                raw[0] = valid;
                raw[1] = ending & 1 != 0;
                let cursor = 0x8123_4567_u32;
                for i in 0..32 {
                    raw[2 + i] = cursor & (1 << i) != 0;
                }
                let ending_bits = V::Cube((0..8).map(|i| ending & (1 << i) != 0).collect());
                let units = V::Cube((0..32).map(|i| 16_384_u32 & (1 << i) != 0).collect());
                let head = run(&cert, &finish, vec![V::Cube(raw), ending_bits, units]);
                let good = valid && ending < 4;
                let packet = run(&cert, &assemble, vec![head, value.clone()]);
                let projected_head = run(&cert, &head_name, vec![packet.clone()]);
                let projected_value = run(&cert, &value_name, vec![packet.clone()]);
                for i in 0..128 {
                    let expected = good
                        && match i {
                            0 | 34 | 48 => true,
                            1 => ending & 1 != 0,
                            2..=33 => cursor & (1 << (i - 2)) != 0,
                            _ => false,
                        };
                    assert_eq!(at(&cert, &packet, 20, i << 1), expected);
                    assert_eq!(at(&cert, &projected_head, 7, i), expected);
                }
                for i in [128, 129, 256, 257, 262_144, 524_287] {
                    assert!(!at(&cert, &packet, 20, i << 1));
                }
                for i in ones
                    .iter()
                    .copied()
                    .chain([2, 6, 30, 33, 16_382, 16_385, 524_285])
                {
                    let expected = good && ones.contains(&i);
                    assert_eq!(at(&cert, &packet, 20, 1 | (i << 1)), expected);
                    assert_eq!(at(&cert, &projected_value, 19, i), expected);
                }
            }
        }
        eprintln!("Typed JSON strings: 24 complete header and sparse full-capacity value cases, each invalid ending bit rejected");
    }

    #[test]
    fn json_values_all_primitive_codec_routes_fit_one_certificate() {
        // Source-free algorithm coverage complements the source fixtures, which
        // do not exercise every primitive route or both GUID formats.
        let kinds = [
            "unit", "bool", "char", "string", "i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64",
            "f32", "f64", "guid", "date", "time", "duration", "instant", "decimal",
        ];
        let carriers = kinds
            .iter()
            .map(|kind| {
                let (shape, depth) = match *kind {
                    "string" => (
                        OrdinaryShape::Sequence {
                            capacity: 16_384,
                            element: Box::new(OrdinaryShape::Bits { width: 16 }),
                        },
                        19,
                    ),
                    "decimal" => (
                        OrdinaryShape::Product {
                            fields: [("negative", 1), ("scale", 8), ("coefficient", 96)]
                                .into_iter()
                                .map(|(id, width)| OrdinaryField {
                                    id: id.into(),
                                    shape: OrdinaryShape::Bits { width },
                                })
                                .collect(),
                        },
                        9,
                    ),
                    _ => {
                        let width = scalar_width(kind).unwrap();
                        (OrdinaryShape::Bits { width }, address_bits(width))
                    }
                };
                OrdinaryCarrier {
                    type_id: format!("mpk.csharp.value.{kind}.v1"),
                    depth,
                    shape,
                }
            })
            .collect::<Vec<_>>();
        let mut b = Builder::new().unwrap();
        let tokens = super::super::emit_boundary_json_for_carriers(&mut b, &carriers).unwrap();
        let definitions = emit(&mut b, &tokens, &carriers).unwrap();
        assert_eq!(definitions.len(), 166);
        let ids = definitions
            .iter()
            .map(|d| d.carrier.type_id.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(ids, carriers.iter().map(|c| c.type_id.as_str()).collect());
        let codecs = definitions
            .iter()
            .map(|d| d.codec_id.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            codecs,
            BTreeSet::from([
                "json.null",
                "json.bool",
                "json.char",
                "json.string",
                "integer.i8",
                "integer.u8",
                "integer.i16",
                "integer.u16",
                "integer.i32",
                "integer.u32",
                "integer.i64",
                "integer.u64",
                "binary32",
                "binary64",
                "guid.n",
                "guid.d",
                "date",
                "time",
                "duration_ticks",
                "unix_milliseconds",
                "decimal.normalized",
                "decimal.fixed",
            ])
        );
        let static_transformers = b.static_transformers;
        let bytes = b.finish().unwrap();
        let cert = decode_canonical_certificate(&bytes).unwrap();
        crate::csharp_practical_vc_model::validate_csharp_practical_certificate_structure(&cert)
            .unwrap();
        let output = std::env::var_os("MPK_W09_JSON_VALUES_ALL_OUT").map(std::path::PathBuf::from);
        let dir = output.clone().unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
                "../../develop/migrations/csharp-03/ordinary-foundation/json-values-all-scalars",
            )
        });
        let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
        let metadata = serde_json::to_vec_pretty(&serde_json::json!({
            "scope":"Source-free complete primitive codec route matrix; compound grammar and application proofs remain pending",
            "definitions":definitions,"terms":cert.term_table.len(),"declarations":cert.declarations.len(),
            "static_transformers":static_transformers,"certificate_sha256":mpk_cert::hash_hex(&mpk_cert::certificate_hash(&bytes))
        })).unwrap();
        if output.is_some() {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("all-scalars.hex"), hex).unwrap();
            std::fs::write(dir.join("certificate.json"), metadata).unwrap();
        } else {
            assert_eq!(
                std::fs::read_to_string(dir.join("all-scalars.hex")).unwrap(),
                hex
            );
            assert_eq!(
                std::fs::read(dir.join("certificate.json")).unwrap(),
                metadata
            );
        }
        eprintln!("Typed JSON all primitive routes: 166 adapters, {} terms, {} declarations, {static_transformers} transformers", cert.term_table.len(), cert.declarations.len());
    }
}
