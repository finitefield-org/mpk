//! Source and day-of-week enums use quoted canonical integers in their exact domain.
use super::hex_codecs::call;
use super::integer_format::{circuit_with_block_bits, define};
use super::temporal::literal;
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonEnumDefinition {
    pub carrier: OrdinaryCarrier,
    pub underlying: String,
    pub declared_values: Vec<String>,
    pub token_definition: String,
    /// C24 document, C5 start, C3 ending -> ordinary header/value role packet.
    /// The quoted carrier contributes one cell; undeclared integers reject.
    pub parse_definition: String,
    pub packet_depth: u32,
    pub header_definition: String,
    pub value_definition: String,
}

fn convert(b: &mut Builder, id: &str, width: u32, values: &[String]) -> R<String> {
    if !matches!(width, 8 | 16 | 32 | 64) || values.is_empty() {
        return Err(OrdinaryCarrierError::Shape);
    }
    let mut c = Circuit::new(&[128]);
    let raw = c.inputs[0].clone();
    let mut declared = F;
    for value in values {
        let value = value
            .parse::<i128>()
            .map_err(|_| OrdinaryCarrierError::Shape)?;
        // Compare all 64 token bits before narrowing to the enum carrier.
        // This also preserves sign extension for smaller signed underlying types.
        let equal = c.equal(&raw[2..66], &literal(value as u128, 64));
        declared = c.or(declared, equal);
    }
    let valid = c.and(raw[0], declared);
    let mut out = vec![F; 256];
    out[0] = T;
    out[2] = raw[1];
    for i in 0..32 {
        out[(2 + i) << 1] = raw[66 + i];
    }
    out[34 << 1] = T;
    for i in 0..width as usize {
        out[1 | (i << 1)] = raw[2 + i];
    }
    let out = c.select(valid, &out, &vec![F; 256]);
    circuit_with_block_bits(b, &format!("{id}.Convert"), c, out, 7)
}

pub(super) fn emit(
    b: &mut Builder,
    tokens: &OrdinaryJsonTokenDefinition,
    carriers: &[OrdinaryCarrier],
    vir: &ValidatedPracticalVir,
) -> R<Vec<OrdinaryJsonEnumDefinition>> {
    let (_, roots, _) = vir.construction_context();
    let mut definitions = vec![];
    for carrier in carriers {
        let source = roots
            .source_types
            .get(&carrier.type_id)
            .filter(|s| s.kind == SourceKind::Enum);
        let (underlying, declared_values) = if let Some(source) = source {
            (
                source
                    .enum_underlying
                    .as_deref()
                    .ok_or(OrdinaryCarrierError::Shape)?,
                source.enum_values.clone(),
            )
        } else if carrier.type_id == "mpk.csharp.value.day_of_week.v1" {
            ("i32", (0..=6).map(|i| i.to_string()).collect())
        } else {
            continue;
        };
        let width = match underlying {
            "i8" | "u8" => 8,
            "i16" | "u16" => 16,
            "i32" | "u32" => 32,
            "i64" | "u64" => 64,
            _ => return Err(OrdinaryCarrierError::Shape),
        };
        if carrier.shape != (OrdinaryShape::Bits { width }) || carrier.depth != width.ilog2() {
            return Err(OrdinaryCarrierError::Shape);
        }
        let token = tokens
            .quoted_scalars
            .iter()
            .find(|t| {
                t.kind
                    == if underlying.starts_with('i') {
                        "i64"
                    } else {
                        "u64"
                    }
            })
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let hex = carrier
            .type_id
            .as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        let id = format!("Mpk.CSharp.Ordinary.JsonEnums.T{hex}");
        let converter = convert(b, &id, width, &declared_values)?;
        let args = (0..3).rev().map(|i| b.var(i)).collect::<R<Vec<_>>>()?;
        let raw = call(b, &token.parse_definition, args)?;
        let body = call(b, &converter, vec![raw])?;
        let parse_definition = format!("{id}.Parse");
        define(b, &parse_definition, &[24, 5, 3], 8, body)?;
        let header_definition = super::json_values::projection(b, 8, 7, false)?;
        let value_definition = super::json_values::projection(b, 8, carrier.depth, true)?;
        definitions.push(OrdinaryJsonEnumDefinition {
            carrier: carrier.clone(),
            underlying: underlying.into(),
            declared_values,
            token_definition: token.parse_definition.clone(),
            parse_definition,
            packet_depth: 8,
            header_definition,
            value_definition,
        });
    }
    Ok(definitions)
}

#[cfg(test)]
mod tests {
    use super::super::super::test_eval::{apply, bit, run, V};
    use super::*;
    #[test]
    fn enum_membership_before_narrowing_and_complete_packet() {
        let mut b = Builder::new().unwrap();
        let mut defs = vec![];
        for width in [8, 16, 32, 64] {
            for signed in [false, true] {
                let minimum = if signed { -(1_i128 << (width - 1)) } else { 0 };
                let maximum = (1_i128 << (width - u32::from(signed))) - 1;
                let values = vec![minimum.to_string(), "0".into(), maximum.to_string()];
                let name = convert(
                    &mut b,
                    &format!("EnumTest.W{width}.S{signed}"),
                    width,
                    &values,
                )
                .unwrap();
                defs.push((name, width, minimum, maximum));
            }
        }
        let cert = mpk_cert::decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        let mut cases = 0;
        for (name, width, minimum, maximum) in defs {
            let mut inputs = vec![
                (minimum, true, true),
                (maximum, true, true),
                (0, true, true),
                (1, true, false),
                (0, false, false),
            ];
            // A C7 token has only 64 payload bits. Values outside that width
            // must be rejected by lexical parsing, before this converter.
            if width < 64 {
                inputs.extend([(maximum + 1, true, false), (minimum - 1, true, false)]);
            }
            for (value, raw_valid, good) in inputs {
                let mut raw = vec![false; 128];
                raw[0] = raw_valid;
                raw[1] = true;
                for i in 0..64 {
                    raw[2 + i] = (value as u128) & (1 << i) != 0;
                }
                let end = 0x0008_1234_u32;
                for i in 0..32 {
                    raw[66 + i] = end & (1 << i) != 0;
                }
                let result = run(&cert, &name, vec![V::Cube(raw)]);
                let mut expected = vec![false; 256];
                if good {
                    expected[0] = true;
                    expected[2] = true;
                    expected[34 << 1] = true;
                    for i in 0..32 {
                        expected[(2 + i) << 1] = end & (1 << i) != 0;
                    }
                    for i in 0..width {
                        expected[1 | (i << 1) as usize] = (value as u128) & (1 << i) != 0;
                    }
                }
                for (i, wanted) in expected.into_iter().enumerate() {
                    let mut v = result.clone();
                    for at in 0..8 {
                        v = apply(&cert, v, V::Bit(i & (1 << at) != 0));
                    }
                    assert_eq!(bit(v), wanted, "width{width} value{value} bit{i}");
                }
                cases += 1;
            }
        }
        assert_eq!(cases, 52);
    }
}
