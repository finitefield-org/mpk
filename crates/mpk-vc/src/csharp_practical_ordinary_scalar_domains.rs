//! Scalar representation domains in ordinary core. Public source invariants,
//! recursive domains and default eligibility are assembled separately.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OrdinaryScalarDomainRule {
    Bits { width: u32 },
    Range { maximum: u64 },
    DeclaredEnum { values: Vec<String> },
    Decimal,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryScalarDomainDefinition {
    pub carrier: OrdinaryCarrier,
    pub rule: OrdinaryScalarDomainRule,
    pub definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryScalarDomainProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryScalarDomainDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryScalarDomainProgram {
    pub fn definitions(&self) -> &[OrdinaryScalarDomainDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed scalar domain program")
    }
}
fn read(b: &mut Builder, depth: u32, address: u32) -> R<u32> {
    let x = b.var(0)?;
    let a = prefix(depth, address)
        .into_iter()
        .map(|v| bit(b, v))
        .collect::<R<Vec<_>>>()?;
    b.app(x, a)
}
fn and(b: &mut Builder, x: u32, y: u32) -> R<u32> {
    let f = bit(b, false)?;
    mux(b, x, y, f)
}
fn or(b: &mut Builder, x: u32, y: u32) -> R<u32> {
    let t = bit(b, true)?;
    mux(b, x, t, y)
}
fn zero(b: &mut Builder, x: u32) -> R<u32> {
    let t = bit(b, true)?;
    let f = bit(b, false)?;
    mux(b, x, f, t)
}
fn at_most(b: &mut Builder, bits: &[u32], maximum: u64) -> R<u32> {
    if bits.len() > 64 {
        return Err(OrdinaryCarrierError::Shape);
    }
    let f = bit(b, false)?;
    let t = bit(b, true)?;
    let mut le = t;
    // Higher bits override the accumulated low-bit comparison.
    for (i, &x) in bits.iter().enumerate() {
        le = if maximum & (1 << i) == 0 {
            mux(b, x, f, le)?
        } else {
            mux(b, x, le, t)?
        };
    }
    Ok(le)
}
pub(super) fn selected_rule(
    carrier: &OrdinaryCarrier,
    vir: &ValidatedPracticalVir,
) -> R<Option<OrdinaryScalarDomainRule>> {
    let (_, roots, _) = vir.construction_context();
    if let Some(source) = roots.source_types.get(&carrier.type_id) {
        return Ok((source.kind == SourceKind::Enum).then(|| {
            OrdinaryScalarDomainRule::DeclaredEnum {
                values: source.enum_values.clone(),
            }
        }));
    }
    let Some(token) = carrier
        .type_id
        .strip_prefix("mpk.csharp.value.")
        .and_then(|s| s.strip_suffix(".v1"))
    else {
        return Ok(None);
    };
    primitive_rule(token)
}
fn primitive_rule(token: &str) -> R<Option<OrdinaryScalarDomainRule>> {
    Ok(Some(match token {
        "date" => OrdinaryScalarDomainRule::Range { maximum: 3_652_058 },
        "time" => OrdinaryScalarDomainRule::Range {
            maximum: 863_999_999_999,
        },
        "day_of_week" => OrdinaryScalarDomainRule::Range { maximum: 6 },
        "parse_error" => OrdinaryScalarDomainRule::Range { maximum: 4 },
        "decimal" => OrdinaryScalarDomainRule::Decimal,
        "string" | "exception" => return Ok(None),
        _ => OrdinaryScalarDomainRule::Bits {
            width: scalar_width(token).ok_or(OrdinaryCarrierError::Shape)?,
        },
    }))
}
pub(super) fn emit_domain(
    b: &mut Builder,
    carrier: &OrdinaryCarrier,
    rule: OrdinaryScalarDomainRule,
) -> R<OrdinaryScalarDomainDefinition> {
    // Scalars have at most 512 physical leaves (decimal); fail before shifts.
    if carrier.depth > 9 {
        return Err(OrdinaryCarrierError::Shape);
    }
    let width = match &carrier.shape {
        OrdinaryShape::Bits { width } if address_bits(*width) == carrier.depth => Some(*width),
        _ => None,
    };
    let body = match &rule {
        OrdinaryScalarDomainRule::Bits { width: expected } if width == Some(*expected) => {
            let mut valid = bit(b, true)?;
            for i in *expected..1 << carrier.depth {
                let x = read(b, carrier.depth, i)?;
                let z = zero(b, x)?;
                valid = and(b, valid, z)?;
            }
            valid
        }
        OrdinaryScalarDomainRule::Range { maximum } => {
            let width = width
                .filter(|w| matches!(w, 32 | 64))
                .ok_or(OrdinaryCarrierError::Shape)?;
            if width < 64 && *maximum >= 1u64 << width {
                return Err(OrdinaryCarrierError::Shape);
            }
            let bits = (0..width)
                .map(|i| read(b, carrier.depth, i))
                .collect::<R<Vec<_>>>()?;
            at_most(b, &bits, *maximum)?
        }
        OrdinaryScalarDomainRule::DeclaredEnum { values } => {
            let width = width
                .filter(|w| matches!(w, 8 | 16 | 32 | 64))
                .ok_or(OrdinaryCarrierError::Shape)?;
            let bits = (0..width)
                .map(|i| read(b, carrier.depth, i))
                .collect::<R<Vec<_>>>()?;
            let mut member = bit(b, false)?;
            for value in values {
                // The validated source table has already checked canonical spelling,
                // signedness and representability. Preserve the low two's-complement bits.
                let value =
                    parse_canonical_integer(value).ok_or(OrdinaryCarrierError::Shape)? as u128;
                let mut equal = bit(b, true)?;
                for (i, &x) in bits.iter().enumerate() {
                    let equal_bit = if value & (1 << i) != 0 {
                        x
                    } else {
                        zero(b, x)?
                    };
                    equal = and(b, equal, equal_bit)?;
                }
                member = or(b, member, equal)?;
            }
            member
        }
        OrdinaryScalarDomainRule::Decimal => {
            if carrier.depth != 9
                || carrier.shape
                    != product(vec![
                        field("negative", bits(1)),
                        field("scale", bits(8)),
                        field("coefficient", bits(96)),
                    ])
            {
                return Err(OrdinaryCarrierError::Shape);
            }
            let scale = (0..8)
                .map(|i| read(b, 9, 1 + i * 64))
                .collect::<R<Vec<_>>>()?;
            let mut valid = at_most(b, &scale, 28)?;
            for i in 0..512 {
                // Two field selectors, then leading-zero padding and child bits.
                // Coefficient values use exactly 96 bits; all other addresses are zero.
                let occupied = i == 0 || i % 64 == 1 || (i % 4 == 2 && i / 4 < 96);
                if !occupied {
                    let x = read(b, 9, i)?;
                    let z = zero(b, x)?;
                    valid = and(b, valid, z)?;
                }
            }
            valid
        }
        _ => return Err(OrdinaryCarrierError::Shape),
    };
    let hex = carrier
        .type_id
        .as_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let definition = format!("{PREFIX}.ScalarDomain.T{hex}.Valid");
    define(b, &definition, &[carrier.depth], 0, body)?;
    Ok(OrdinaryScalarDomainDefinition {
        carrier: carrier.clone(),
        rule,
        definition,
    })
}
/// Reconstruct all reachable primitive scalar and source-enum representation
/// domains. This does not discharge application invariants or recursive domains.
pub fn generate_csharp_practical_ordinary_scalar_domains(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryScalarDomainProgram> {
    let carriers = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut b = Builder::new()?;
    let mut definitions = vec![];
    for c in carriers.carriers() {
        if let Some(rule) = selected_rule(c, vir)? {
            definitions.push(emit_domain(&mut b, c, rule)?);
        }
    }
    let certificate = b.finish()?;
    let p = OrdinaryScalarDomainProgram {
        schema: "mpk.csharp.ordinary_scalar_domains.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        definitions,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_scalar_domains(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryScalarDomainProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_scalar_domains(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}

#[cfg(test)]
mod tests {
    use super::super::super::tests::{bit as observed_bit, run, V};
    use super::*;
    fn fixture() -> (Certificate, Vec<OrdinaryScalarDomainDefinition>, Vec<u8>) {
        let mut b = Builder::new().unwrap();
        let mut defs = vec![];
        for token in [
            "unit",
            "bool",
            "i8",
            "u8",
            "i16",
            "u16",
            "char",
            "i32",
            "u32",
            "f32",
            "i64",
            "u64",
            "f64",
            "guid",
            "duration",
            "instant",
            "date",
            "time",
            "day_of_week",
            "parse_error",
            "decimal",
        ] {
            let shape = if token == "decimal" {
                product(vec![
                    field("negative", bits(1)),
                    field("scale", bits(8)),
                    field("coefficient", bits(96)),
                ])
            } else {
                bits(scalar_width(token).unwrap())
            };
            let depth = if token == "decimal" {
                9
            } else {
                address_bits(scalar_width(token).unwrap())
            };
            let carrier = OrdinaryCarrier {
                type_id: format!("mpk.csharp.value.{token}.v1"),
                shape,
                depth,
            };
            defs.push(
                emit_domain(&mut b, &carrier, primitive_rule(token).unwrap().unwrap()).unwrap(),
            );
        }
        for (name, width, values) in [
            ("Test.Signed", 8, vec!["-128", "0", "7"]),
            (
                "Test.Unsigned",
                64,
                vec!["9223372036854775808", "18446744073709551615"],
            ),
        ] {
            let carrier = OrdinaryCarrier {
                type_id: name.into(),
                depth: address_bits(width),
                shape: bits(width),
            };
            defs.push(
                emit_domain(
                    &mut b,
                    &carrier,
                    OrdinaryScalarDomainRule::DeclaredEnum {
                        values: values.into_iter().map(str::to_owned).collect(),
                    },
                )
                .unwrap(),
            );
        }
        let carrier = OrdinaryCarrier {
            type_id: "Test.ThreeBits".into(),
            depth: 2,
            shape: bits(3),
        };
        defs.push(
            emit_domain(
                &mut b,
                &carrier,
                OrdinaryScalarDomainRule::Bits { width: 3 },
            )
            .unwrap(),
        );
        let bytes = b.finish().unwrap();
        (decode_canonical_certificate(&bytes).unwrap(), defs, bytes)
    }
    fn definition<'a>(
        defs: &'a [OrdinaryScalarDomainDefinition],
        token: &str,
    ) -> &'a OrdinaryScalarDomainDefinition {
        defs.iter()
            .find(|d| {
                d.carrier.type_id == format!("mpk.csharp.value.{token}.v1")
                    || d.carrier.type_id == token
            })
            .unwrap()
    }
    fn raw(n: u128, depth: u32) -> V {
        if depth == 0 {
            V::Bit(n & 1 != 0)
        } else {
            V::Cube((0..1 << depth).map(|i| n & (1u128 << i) != 0).collect())
        }
    }
    fn valid(c: &Certificate, d: &OrdinaryScalarDomainDefinition, v: V) -> bool {
        observed_bit(run(c, &d.definition, vec![v]))
    }
    #[test]
    fn scalar_domains_ranges_enum_and_unrestricted_bits() {
        let (c, defs, _) = fixture();
        for (token, maximum) in [
            ("date", 3_652_058u128),
            ("time", 863_999_999_999),
            ("day_of_week", 6),
            ("parse_error", 4),
        ] {
            let d = definition(&defs, token);
            for (n, expected) in [
                (0, true),
                (1, true),
                (maximum - 1, true),
                (maximum, true),
                (maximum + 1, false),
                (1u128 << ((1 << d.carrier.depth) - 1), false),
                ((1u128 << (1 << d.carrier.depth)) - 1, false),
            ] {
                assert_eq!(
                    valid(&c, d, raw(n, d.carrier.depth)),
                    expected,
                    "{token} {n}"
                );
            }
        }
        let d = definition(&defs, "Test.Signed");
        for n in 0..256 {
            assert_eq!(valid(&c, d, raw(n, 3)), [128, 0, 7].contains(&n));
        }
        let d = definition(&defs, "Test.Unsigned");
        for n in [
            0,
            1,
            (1u128 << 63) - 1,
            1u128 << 63,
            (1u128 << 63) + 1,
            u64::MAX as u128,
        ] {
            assert_eq!(
                valid(&c, d, raw(n, 6)),
                [1u128 << 63, u64::MAX as u128].contains(&n)
            );
        }
        for token in [
            "bool", "i8", "u8", "i16", "u16", "char", "i32", "u32", "i64", "u64", "f32", "f64",
            "guid", "duration", "instant",
        ] {
            let d = definition(&defs, token);
            // IEEE NaNs, infinities and signed zero are all legitimate representations.
            for n in [
                0,
                1,
                u128::MAX,
                0x7fc0_0000,
                0x7ff8_0000_0000_0000,
                0x8000_0000_0000_0000,
            ] {
                assert!(valid(&c, d, raw(n, d.carrier.depth)), "{token}");
            }
        }
        let d = definition(&defs, "unit");
        assert!(valid(&c, d, V::Bit(false)));
        assert!(!valid(&c, d, V::Bit(true)));
        let d = definition(&defs, "Test.ThreeBits");
        for n in 0..16 {
            assert_eq!(valid(&c, d, raw(n, 2)), n < 8);
        }
        assert_eq!(primitive_rule("unknown"), Err(OrdinaryCarrierError::Shape));
        assert_eq!(primitive_rule("string"), Ok(None));
        assert_eq!(primitive_rule("exception"), Ok(None));
    }
    fn decimal(negative: bool, scale: u8, coefficient: u128) -> Vec<bool> {
        let mut raw = vec![false; 512];
        raw[0] = negative;
        for i in 0..8 {
            raw[1 + 64 * i] = scale & (1 << i) != 0;
        }
        for i in 0..96 {
            raw[2 + 4 * i] = coefficient & (1u128 << i) != 0;
        }
        raw
    }
    #[test]
    fn scalar_domains_decimal_scale_and_every_padding_address() {
        let (c, defs, _) = fixture();
        let d = definition(&defs, "decimal");
        for scale in 0..=255 {
            for negative in [false, true] {
                assert_eq!(
                    valid(&c, d, V::Cube(decimal(negative, scale, 0))),
                    scale <= 28
                );
                assert_eq!(
                    valid(&c, d, V::Cube(decimal(negative, scale, (1u128 << 96) - 1))),
                    scale <= 28
                );
            }
        }
        // Scale is not normalized away: trailing zeros and signed zero are valid.
        assert!(valid(&c, d, V::Cube(decimal(true, 28, 0))));
        assert!(valid(&c, d, V::Cube(decimal(false, 2, 100))));
        let occupied = decimal(true, 255, (1u128 << 96) - 1);
        for (i, used) in occupied.iter().enumerate() {
            if !used {
                let mut raw = decimal(false, 0, 1);
                raw[i] = true;
                assert!(!valid(&c, d, V::Cube(raw)), "nonzero padding address {i}");
            }
        }
    }
    #[test]
    fn scalar_domains_certificate_replays() {
        let (c, defs, bytes) = fixture();
        let (_, again, second) = fixture();
        assert_eq!(defs, again);
        assert_eq!(bytes, second);
        let metadata = serde_json::json!({"definitions":defs,"terms":c.term_table.len(),"declarations":c.declarations.len(),"certificate_sha256":mpk_cert::hash_hex(&mpk_cert::certificate_hash(&bytes))});
        let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
        if let Some(out) = std::env::var_os("MPK_W09_SCALAR_DOMAINS_OUT") {
            let out = std::path::PathBuf::from(out);
            std::fs::create_dir_all(&out).unwrap();
            std::fs::write(out.join("scalar-domains.hex"), hex).unwrap();
            std::fs::write(
                out.join("core-metrics.json"),
                serde_json::to_vec_pretty(&metadata).unwrap(),
            )
            .unwrap();
        } else {
            let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../develop/migrations/csharp-03/ordinary-foundation/scalar-domains");
            assert_eq!(
                std::fs::read_to_string(out.join("scalar-domains.hex")).unwrap(),
                hex
            );
            assert_eq!(
                serde_json::from_slice::<Value>(
                    &std::fs::read(out.join("core-metrics.json")).unwrap()
                )
                .unwrap(),
                metadata
            );
        }
    }
}
