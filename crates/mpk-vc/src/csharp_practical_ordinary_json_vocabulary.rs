//! Finite canonical JSON string vocabulary with exact ordinary carrier values.
use super::hex_codecs::call;
use super::integer_format::{circuit_with_block_bits, define};
use super::temporal::literal;
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonVocabularyDefinition {
    pub carrier: OrdinaryCarrier,
    /// Frozen ordinal order, matching the existing ParseError carrier encoding.
    pub names: Vec<String>,
    pub literals: Vec<OrdinaryJsonSyntaxLiteral>,
    /// C24 document, C5 start, C3 ending -> C8 header/value role packet.
    pub parse_definition: String,
    pub packet_depth: u32,
    pub header_definition: String,
    pub value_definition: String,
}

fn select(b: &mut Builder, id: &str, count: usize) -> R<String> {
    let mut c = Circuit::new(&vec![64; count]);
    let mut seen = F;
    let mut duplicate = F;
    let mut out = vec![F; 256];
    for ordinal in 0..count {
        let raw = c.inputs[ordinal].clone();
        let repeated = c.and(seen, raw[0]);
        duplicate = c.or(duplicate, repeated);
        seen = c.or(seen, raw[0]);
        let mut value = vec![F; 256];
        value[0] = T;
        value[2] = raw[1];
        value[34 << 1] = T;
        for i in 0..32 {
            value[(2 + i) << 1] = raw[2 + i];
        }
        for (i, bit) in literal(ordinal as u128, 32).into_iter().enumerate() {
            value[1 | (i << 1)] = bit;
        }
        out = c.select(raw[0], &value, &out);
    }
    let unique = c.not(duplicate);
    let valid = c.and(seen, unique);
    let out = c.select(valid, &out, &vec![F; 256]);
    circuit_with_block_bits(b, id, c, out, 7)
}

pub(super) fn emit(
    b: &mut Builder,
    tokens: &OrdinaryJsonTokenDefinition,
    grammar: &OrdinaryJsonGrammarDefinition,
    carriers: &[OrdinaryCarrier],
    syntax_helpers: (&str, &str),
    chunks: &mut BTreeMap<Vec<u8>, String>,
) -> R<Vec<OrdinaryJsonVocabularyDefinition>> {
    let Some(carrier) = carriers
        .iter()
        .find(|c| c.type_id == "mpk.csharp.value.parse_error.v1")
    else {
        return Ok(vec![]);
    };
    if carrier.depth != 5 || carrier.shape != (OrdinaryShape::Bits { width: 32 }) {
        return Err(OrdinaryCarrierError::Shape);
    }
    let id = "Mpk.CSharp.Ordinary.JsonVocabulary.ParseError";
    let names = [
        "input_bound",
        "syntax",
        "noncanonical",
        "scale_precision",
        "range",
    ];
    let mut literals = vec![];
    for name in names {
        literals.push(super::json_syntax::emit_literal(
            b,
            &tokens.strings.fragments,
            syntax_helpers.0,
            syntax_helpers.1,
            format!("\"{name}\"").into_bytes(),
            chunks,
        )?);
    }
    let selector = select(b, &format!("{id}.Select"), names.len())?;
    let doc = b.var(2)?;
    let start = b.var(1)?;
    let mut matched = vec![];
    for literal in &literals {
        matched.push(call(b, &literal.match_definition, vec![doc, start])?);
    }
    let parsed = call(b, &selector, matched)?;
    let packet = b.var(0)?;
    let doc = b.var(3)?;
    let ending = b.var(1)?;
    let header_definition = super::json_values::projection(b, 8, 7, false)?;
    let value_definition = super::json_values::projection(b, 8, 5, true)?;
    let head = call(b, &header_definition, vec![packet])?;
    let value = call(b, &value_definition, vec![packet])?;
    let head = call(b, &grammar.finish_definition, vec![doc, head, ending])?;
    let assemble = super::json_grammar::assemble(b, 5)?;
    let body = call(b, &assemble, vec![head, value])?;
    let ty = b.cube(8)?;
    let body = b.term(TermNode::Let {
        ty,
        value: parsed,
        body,
    })?;
    let parse_definition = format!("{id}.Parse");
    define(b, &parse_definition, &[24, 5, 3], 8, body)?;
    Ok(vec![OrdinaryJsonVocabularyDefinition {
        carrier: carrier.clone(),
        names: names.into_iter().map(str::to_owned).collect(),
        literals,
        parse_definition,
        packet_depth: 8,
        header_definition,
        value_definition,
    }])
}

#[cfg(test)]
mod tests {
    use super::super::super::test_eval::{apply, bit, run, V};
    use super::*;
    #[test]
    fn vocabulary_exactly_one_match_and_all_packet_bits() {
        let mut b = Builder::new().unwrap();
        let name = select(&mut b, "VocabularyTest", 5).unwrap();
        let cert = mpk_cert::decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        for mask in 0_u32..32 {
            let args = (0..5)
                .map(|ordinal| {
                    let mut bits = vec![false; 64];
                    bits[0] = mask & (1 << ordinal) != 0;
                    bits[1] = ordinal % 2 != 0;
                    for i in 0..32 {
                        bits[2 + i] = (0x80000_u32 + ordinal * 37 + 1) & (1 << i) != 0;
                    }
                    V::Cube(bits)
                })
                .collect();
            let result = run(&cert, &name, args);
            let mut expected = vec![false; 256];
            if mask.count_ones() == 1 {
                let ordinal = mask.trailing_zeros();
                expected[0] = true;
                expected[2] = ordinal % 2 != 0;
                expected[34 << 1] = true;
                for i in 0..32 {
                    expected[(2 + i) << 1] = (0x80000_u32 + ordinal * 37 + 1) & (1 << i) != 0;
                    expected[1 | (i << 1)] = ordinal & (1 << i) != 0;
                }
            }
            for (i, want) in expected.into_iter().enumerate() {
                let mut v = result.clone();
                for at in 0..8 {
                    v = apply(&cert, v, V::Bit(i & (1 << at) != 0));
                }
                assert_eq!(bit(v), want, "match mask{mask} bit{i}");
            }
        }
    }
}
