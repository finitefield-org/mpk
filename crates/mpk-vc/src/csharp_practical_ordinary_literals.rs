//! Closed literal bodies reconstructed from validated VIR, without oracle calls.
//! A literal definition does not prove its source invocation or a boundary codec.
use super::*;
use sha2::{Digest, Sha256};
#[path = "csharp_practical_ordinary_boundary_literals.rs"]
mod boundary_literals;
pub use boundary_literals::{
    generate_csharp_practical_ordinary_boundary_literals,
    import_csharp_practical_ordinary_boundary_literals, OrdinaryBoundaryLiteralBinding,
    OrdinaryBoundaryLiteralProgram,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryLiteralDefinition {
    pub name: String,
    pub carrier: OrdinaryCarrier,
    pub value: MonomorphicValue,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OrdinaryLiteralOrigin {
    BlockValue {
        function_id: String,
        block_id: String,
        value_id: String,
    },
    FailedCheck {
        function_id: String,
        block_id: String,
        check_id: String,
    },
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryLiteralBinding {
    pub origin: OrdinaryLiteralOrigin,
    pub definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryLiteralProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryLiteralDefinition>,
    bindings: Vec<OrdinaryLiteralBinding>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryLiteralProgram {
    pub fn definitions(&self) -> &[OrdinaryLiteralDefinition] {
        &self.definitions
    }
    pub fn bindings(&self) -> &[OrdinaryLiteralBinding] {
        &self.bindings
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed ordinary literal program")
    }
}
#[derive(Clone, Copy)]
struct Encoded {
    term: u32,
    depth: u32,
    fingerprint: [u8; 32],
}
struct Literals<'a> {
    b: Builder,
    parts: BTreeMap<[u8; 32], u32>,
    carriers: BTreeMap<&'a str, &'a OrdinaryCarrier>,
}
fn fingerprint(value: &impl Serialize) -> [u8; 32] {
    Sha256::digest(serde_json::to_vec(value).expect("typed literal part")).into()
}
impl Literals<'_> {
    fn share(&mut self, child: Encoded) -> R<u32> {
        if let Some(term) = self.parts.get(&child.fingerprint) {
            return Ok(*term);
        }
        let name = format!(
            "{PREFIX}.LiteralPart.H{}",
            child
                .fingerprint
                .iter()
                .map(|v| format!("{v:02x}"))
                .collect::<String>()
        );
        let ty = self.b.cube(child.depth)?;
        self.b.define(&name, ty, child.term)?;
        let term = self.b.constant(&name)?;
        self.parts.insert(child.fingerprint, term);
        Ok(term)
    }
    fn zero(&mut self, depth: u32) -> R<Encoded> {
        let f = bit(&mut self.b, false)?;
        Ok(Encoded {
            term: self.b.wrap_selectors(depth, f)?,
            depth,
            fingerprint: fingerprint(&("zero", depth)),
        })
    }
    fn word(&mut self, n: u128, width: u32) -> R<Encoded> {
        if width > 128 {
            return Err(OrdinaryCarrierError::Shape);
        }
        let depth = address_bits(width);
        let mut out = bit(&mut self.b, false)?;
        for i in 0..width {
            if n & (1u128 << i) != 0 {
                let at = equal_address(&mut self.b, depth, 0, depth, i)?;
                let t = bit(&mut self.b, true)?;
                out = mux(&mut self.b, at, t, out)?;
            }
        }
        Ok(Encoded {
            term: self.b.wrap_selectors(depth, out)?,
            depth,
            fingerprint: fingerprint(&("bits", width, n.to_string())),
        })
    }
    // Closed children can be applied beneath new selectors without shifting.
    // Role/index selectors precede low zero padding and child selectors.
    fn slots(&mut self, roles: u32, child_depth: u32, children: &[Encoded]) -> R<Encoded> {
        let depth = roles
            .checked_add(child_depth)
            .ok_or(OrdinaryCarrierError::Limit)?;
        if depth > 253 || roles > 32 || (children.len() as u64) > (1u64 << roles) {
            return Err(OrdinaryCarrierError::Limit);
        }
        let mut out = bit(&mut self.b, false)?;
        if children.len() > 256 {
            // A linear selector for a full-capacity literal can exhaust the
            // unchanged checker's stack. Pair adjacent slots by each low-to-high
            // address bit, retaining zero for every unused slot and padding bit.
            let mut level = Vec::with_capacity(children.len());
            for child in children {
                let padding = child_depth
                    .checked_sub(child.depth)
                    .ok_or(OrdinaryCarrierError::Shape)?;
                let selectors = self.b.selectors(child.depth)?;
                let closed = self.share(*child)?;
                let leaf = self.b.app(closed, selectors)?;
                level.push(zero_padding(&mut self.b, depth, roles, padding, leaf)?);
            }
            for bit in 0..roles {
                let selector = self.b.var(depth - 1 - bit)?;
                let mut next = Vec::with_capacity(level.len().div_ceil(2));
                for pair in level.chunks(2) {
                    next.push(mux(
                        &mut self.b,
                        selector,
                        *pair.get(1).unwrap_or(&out),
                        pair[0],
                    )?);
                }
                level = next;
            }
            out = level[0];
        } else {
            for (i, child) in children.iter().enumerate().rev() {
                let padding = child_depth
                    .checked_sub(child.depth)
                    .ok_or(OrdinaryCarrierError::Shape)?;
                let selectors = self.b.selectors(child.depth)?;
                let closed = self.share(*child)?;
                let leaf = self.b.app(closed, selectors)?;
                let leaf = zero_padding(&mut self.b, depth, roles, padding, leaf)?;
                let at = equal_address(&mut self.b, depth, 0, roles, i as u32)?;
                out = mux(&mut self.b, at, leaf, out)?;
            }
        }
        Ok(Encoded {
            term: self.b.wrap_selectors(depth, out)?,
            depth,
            fingerprint: fingerprint(&(
                "slots",
                roles,
                child_depth,
                children.iter().map(|c| c.fingerprint).collect::<Vec<_>>(),
            )),
        })
    }
    fn product(&mut self, children: &[Encoded]) -> R<Encoded> {
        self.slots(
            address_bits(u32::try_from(children.len()).map_err(|_| OrdinaryCarrierError::Limit)?),
            children.iter().map(|c| c.depth).max().unwrap_or(0),
            children,
        )
    }
    fn sequence(&mut self, shape: &OrdinaryShape, children: &[Encoded]) -> R<Encoded> {
        let OrdinaryShape::Sequence { capacity, element } = shape else {
            return Err(OrdinaryCarrierError::Shape);
        };
        if children.len() > *capacity as usize {
            return Err(OrdinaryCarrierError::Limit);
        }
        let element_depth = shape_depth(element, &self.carriers)?;
        if children.iter().any(|c| c.depth != element_depth) {
            return Err(OrdinaryCarrierError::Shape);
        }
        let data = self.slots(address_bits(*capacity), element_depth, children)?;
        let length = self.word(children.len() as u128, 32)?;
        self.product(&[length, data])
    }
    fn sum(&mut self, carrier: &OrdinaryCarrier, tag: u32, children: &[Encoded]) -> R<Encoded> {
        let OrdinaryShape::Sum { arms } = &carrier.shape else {
            return Err(OrdinaryCarrierError::Shape);
        };
        let arm = arms
            .iter()
            .find(|a| a.tag == tag)
            .ok_or(OrdinaryCarrierError::Shape)?;
        if arm.fields.len() != children.len() {
            return Err(OrdinaryCarrierError::Shape);
        }
        for (f, c) in arm.fields.iter().zip(children) {
            if shape_depth(&f.shape, &self.carriers)? != c.depth {
                return Err(OrdinaryCarrierError::Shape);
            }
        }
        let payload = self.product(children)?;
        let payload = self.slots(0, carrier.depth - 1, &[payload])?;
        let tag = self.word(tag as u128, 32)?;
        self.product(&[tag, payload])
    }
    fn values(&mut self, values: &[MonomorphicValue]) -> R<Vec<Encoded>> {
        values.iter().map(|v| self.value(v)).collect()
    }
    fn value(&mut self, v: &MonomorphicValue) -> R<Encoded> {
        let c = *self
            .carriers
            .get(v.type_id())
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let shape = &c.shape;
        let number = |s: &str| {
            s.parse::<i128>()
                .map(|n| n as u128)
                .map_err(|_| OrdinaryCarrierError::Shape)
        };
        let hex = |s: &str| u128::from_str_radix(s, 16).map_err(|_| OrdinaryCarrierError::Shape);
        let width = match shape {
            OrdinaryShape::Bits { width } => *width,
            _ => 0,
        };
        let out = match v {
            MonomorphicValue::Unit { .. } => self.zero(0)?,
            MonomorphicValue::Bool { value, .. } => self.word(*value as u128, 1)?,
            MonomorphicValue::Signed { value, .. }
            | MonomorphicValue::Unsigned { value, .. }
            | MonomorphicValue::Enum { carrier: value, .. } => self.word(number(value)?, width)?,
            MonomorphicValue::Char { utf16, .. } => self.word(*utf16 as u128, width)?,
            MonomorphicValue::F32Bits { bits, .. } | MonomorphicValue::F64Bits { bits, .. } => {
                self.word(hex(bits)?, width)?
            }
            MonomorphicValue::Guid { n, .. } => self.word(hex(n)?, width)?,
            MonomorphicValue::Date { day_number, .. } => self.word(*day_number as u128, width)?,
            MonomorphicValue::Time { ticks, .. }
            | MonomorphicValue::Duration { ticks, .. }
            | MonomorphicValue::Instant {
                milliseconds: ticks,
                ..
            } => self.word(number(ticks)?, width)?,
            MonomorphicValue::DecimalBits {
                negative,
                scale,
                coefficient,
                ..
            } => {
                let children = [
                    self.word(*negative as u128, 1)?,
                    self.word(*scale as u128, 8)?,
                    self.word(number(coefficient)?, 96)?,
                ];
                self.product(&children)?
            }
            MonomorphicValue::Product { fields, .. } => {
                let children = fields
                    .iter()
                    .map(|f| self.value(&f.value))
                    .collect::<R<Vec<_>>>()?;
                self.product(&children)?
            }
            MonomorphicValue::OrderedEntry { key, value, .. } => {
                let children = [self.value(key)?, self.value(value)?];
                self.product(&children)?
            }
            MonomorphicValue::Money {
                amount, currency, ..
            } => {
                let children = [self.value(amount)?, self.value(currency)?];
                self.product(&children)?
            }
            MonomorphicValue::Transition {
                state,
                events,
                response,
                ..
            } => {
                let OrdinaryShape::Product { fields } = shape else {
                    return Err(OrdinaryCarrierError::Shape);
                };
                let event_id =
                    reference_id(&fields.get(1).ok_or(OrdinaryCarrierError::Shape)?.shape)?;
                let sequence = MonomorphicValue::Sequence {
                    type_id: event_id.into(),
                    elements: events.clone(),
                };
                let children = [
                    self.value(state)?,
                    self.value(&sequence)?,
                    self.value(response)?,
                ];
                self.product(&children)?
            }
            MonomorphicValue::Array { elements, .. }
            | MonomorphicValue::Sequence { elements, .. }
            | MonomorphicValue::OrderedSet { elements, .. } => {
                let children = self.values(elements)?;
                self.sequence(shape, &children)?
            }
            MonomorphicValue::String { utf16, .. } => {
                let children = utf16
                    .iter()
                    .map(|v| self.word(*v as u128, 16))
                    .collect::<R<Vec<_>>>()?;
                self.sequence(shape, &children)?
            }
            MonomorphicValue::OrderedMap { entries, .. } => {
                let mut children = vec![];
                for e in entries {
                    let pair = [self.value(&e.key)?, self.value(&e.value)?];
                    children.push(self.product(&pair)?);
                }
                self.sequence(shape, &children)?
            }
            MonomorphicValue::Option { arm, value, .. } => {
                let children = value.iter().map(|v| self.value(v)).collect::<R<Vec<_>>>()?;
                self.sum(c, if *arm == OptionArm::None { 0 } else { 1 }, &children)?
            }
            MonomorphicValue::BoundaryPresence { arm, value, .. } => {
                let tag = match arm {
                    BoundaryArm::Missing => 0,
                    BoundaryArm::Null => 1,
                    BoundaryArm::Value => 2,
                };
                let children = value.iter().map(|v| self.value(v)).collect::<R<Vec<_>>>()?;
                self.sum(c, tag, &children)?
            }
            MonomorphicValue::TaggedSum { arm, payload, .. } => {
                let OrdinaryShape::Sum { arms } = shape else {
                    return Err(OrdinaryCarrierError::Shape);
                };
                let tag = arms
                    .iter()
                    .find(|a| &a.id == arm)
                    .ok_or(OrdinaryCarrierError::Shape)?
                    .tag;
                let children = self.values(payload)?;
                self.sum(c, tag, &children)?
            }
            MonomorphicValue::ClosedException { tag, payload, .. } => {
                let children = if let Some(v) = payload {
                    let MonomorphicValue::Product { fields, .. } = v.as_ref() else {
                        return Err(OrdinaryCarrierError::Shape);
                    };
                    fields
                        .iter()
                        .map(|f| self.value(&f.value))
                        .collect::<R<Vec<_>>>()?
                } else {
                    vec![]
                };
                self.sum(c, *tag, &children)?
            }
            MonomorphicValue::ParseError { arm, .. } => self.word(
                match arm {
                    ParseErrorArm::InputBound => 0,
                    ParseErrorArm::Syntax => 1,
                    ParseErrorArm::Noncanonical => 2,
                    ParseErrorArm::ScalePrecision => 3,
                    ParseErrorArm::Range => 4,
                },
                width,
            )?,
        };
        if out.depth != c.depth {
            return Err(OrdinaryCarrierError::Shape);
        }
        Ok(out)
    }
}
fn reference_id(shape: &OrdinaryShape) -> R<&str> {
    match shape {
        OrdinaryShape::Reference { type_id } => Ok(type_id),
        OrdinaryShape::RoleBound { value, .. } => reference_id(value),
        _ => Err(OrdinaryCarrierError::Shape),
    }
}

/// Add validated constants to an existing core context without resetting costs.
pub(super) fn emit_named_values(
    vir: &ValidatedPracticalVir,
    layouts: &OrdinaryCarrierProgram,
    b: Builder,
    values: BTreeMap<String, MonomorphicValue>,
) -> R<(Builder, Vec<OrdinaryLiteralDefinition>)> {
    let mut emitter = Literals {
        b,
        parts: BTreeMap::new(),
        carriers: layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.as_str(), c))
            .collect(),
    };
    let mut definitions = vec![];
    for (name, value) in values {
        let (bundle, roots, _) = vir.construction_context();
        validate_monomorphic_value(bundle, roots, vir.data_closed(), &value)
            .map_err(|_| OrdinaryCarrierError::Shape)?;
        let encoded = emitter.value(&value)?;
        let ty = emitter.b.cube(encoded.depth)?;
        emitter.b.define(&name, ty, encoded.term)?;
        definitions.push(OrdinaryLiteralDefinition {
            name,
            carrier: emitter.carriers[value.type_id()].clone(),
            value,
        });
    }
    Ok((emitter.b, definitions))
}

pub fn generate_csharp_practical_ordinary_literals(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryLiteralProgram> {
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut emitter = Literals {
        b: Builder::new()?,
        parts: BTreeMap::new(),
        carriers: layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.as_str(), c))
            .collect(),
    };
    let mut values = BTreeMap::new();
    let mut bindings = vec![];
    let mut bind = |origin, value: &MonomorphicValue| {
        let bytes = serde_json::to_vec(value).expect("typed literal");
        let name = format!("{PREFIX}.Literal.H{:x}", Sha256::digest(&bytes));
        values.entry(name.clone()).or_insert_with(|| value.clone());
        bindings.push(OrdinaryLiteralBinding {
            origin,
            definition: name,
        });
    };
    for function in vir.functions() {
        for block in &function.blocks {
            for literal in &block.literal_values {
                bind(
                    OrdinaryLiteralOrigin::BlockValue {
                        function_id: function.id.clone(),
                        block_id: block.node.id.clone(),
                        value_id: literal.result.id.clone(),
                    },
                    &literal.value,
                );
            }
            for exception in &block.exception_values {
                bind(
                    OrdinaryLiteralOrigin::FailedCheck {
                        function_id: function.id.clone(),
                        block_id: block.node.id.clone(),
                        check_id: exception.check_id.clone(),
                    },
                    &exception.value,
                );
            }
        }
    }
    let mut definitions = vec![];
    for (name, value) in values {
        let (bundle, roots, _) = vir.construction_context();
        validate_monomorphic_value(bundle, roots, vir.data_closed(), &value)
            .map_err(|_| OrdinaryCarrierError::Shape)?;
        let encoded = emitter.value(&value)?;
        let ty = emitter.b.cube(encoded.depth)?;
        emitter.b.define(&name, ty, encoded.term)?;
        definitions.push(OrdinaryLiteralDefinition {
            name,
            carrier: emitter.carriers[value.type_id()].clone(),
            value,
        });
    }
    let certificate = emitter.b.finish()?;
    let result = OrdinaryLiteralProgram {
        schema: "mpk.csharp.ordinary_literals.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        definitions,
        bindings,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if result.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(result)
}
pub fn import_csharp_practical_ordinary_literals(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryLiteralProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let result = generate_csharp_practical_ordinary_literals(vir)?;
    if input != result.canonical_bytes() || certificate != result.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::super::super::test_eval::{bit as observe, run, V};
    use super::*;

    fn pin(name: &str, bytes: &[u8]) {
        let hex = bytes.iter().map(|v| format!("{v:02x}")).collect::<String>() + "\n";
        if let Some(dir) = std::env::var_os("MPK_W09_LITERAL_EDGES_OUT") {
            let dir = std::path::PathBuf::from(dir);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(format!("{name}.hex")), hex).unwrap();
        } else {
            let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../develop/migrations/csharp-03/ordinary-foundation/literal-edges")
                .join(format!("{name}.hex"));
            assert_eq!(std::fs::read_to_string(file).unwrap(), hex);
        }
    }
    #[test]
    fn ordinary_literal_scalar_storage_preserves_raw_representations() {
        let id = |s: &str| format!("mpk.csharp.value.{s}.v1");
        let carriers = [
            OrdinaryCarrier {
                type_id: id("f32"),
                depth: 5,
                shape: bits(32),
            },
            OrdinaryCarrier {
                type_id: id("f64"),
                depth: 6,
                shape: bits(64),
            },
            OrdinaryCarrier {
                type_id: id("decimal"),
                depth: 9,
                shape: product(vec![
                    field("negative", bits(1)),
                    field("scale", bits(8)),
                    field("coefficient", bits(96)),
                ]),
            },
        ];
        let mut e = Literals {
            b: Builder::new().unwrap(),
            parts: BTreeMap::new(),
            carriers: carriers.iter().map(|c| (c.type_id.as_str(), c)).collect(),
        };
        let mut cases = vec![];
        for (bits, n) in [
            ("80000000", 0x80000000u128),
            ("7fc01234", 0x7fc01234),
            ("7f800001", 0x7f800001),
        ] {
            cases.push((
                MonomorphicValue::F32Bits {
                    type_id: id("f32"),
                    bits: bits.into(),
                },
                (0..32).map(|i| n & (1 << i) != 0).collect::<Vec<_>>(),
            ));
        }
        for (bits, n) in [
            ("8000000000000000", 0x8000000000000000u128),
            ("fff8000000000123", 0xfff8000000000123),
            ("7ff0000000000001", 0x7ff0000000000001),
        ] {
            cases.push((
                MonomorphicValue::F64Bits {
                    type_id: id("f64"),
                    bits: bits.into(),
                },
                (0..64).map(|i| n & (1 << i) != 0).collect::<Vec<_>>(),
            ));
        }
        for (negative, scale, coefficient) in [(true, 28, 0u128), (false, 2, 12300)] {
            let mut expected = vec![false; 512];
            expected[0] = negative;
            for i in 0..8 {
                expected[1 | (i << 6)] = (scale & (1 << i)) != 0;
            }
            for i in 0..96 {
                expected[2 | (i << 2)] = (coefficient & (1 << i)) != 0;
            }
            cases.push((
                MonomorphicValue::DecimalBits {
                    type_id: id("decimal"),
                    negative,
                    scale,
                    coefficient: coefficient.to_string(),
                },
                expected,
            ));
        }
        for (i, (value, _)) in cases.iter().enumerate() {
            let encoded = e.value(value).unwrap();
            let ty = e.b.cube(encoded.depth).unwrap();
            e.b.define(&format!("Test.LiteralScalar.C{i}"), ty, encoded.term)
                .unwrap();
        }
        let bytes = e.b.finish().unwrap();
        pin("scalar-storage", &bytes);
        let certificate = decode_canonical_certificate(&bytes).unwrap();
        for (i, (value, expected)) in cases.iter().enumerate() {
            let depth = carriers
                .iter()
                .find(|c| c.type_id == value.type_id())
                .unwrap()
                .depth;
            for (index, bit) in expected.iter().enumerate() {
                let args = (0..depth).map(|s| V::Bit(index & (1 << s) != 0)).collect();
                assert_eq!(
                    observe(run(&certificate, &format!("Test.LiteralScalar.C{i}"), args)),
                    *bit,
                    "case {i}, bit {index}"
                );
            }
        }
    }
    #[test]
    fn ordinary_literal_large_slots_use_bounded_depth() {
        let mut e = Literals {
            b: Builder::new().unwrap(),
            parts: BTreeMap::new(),
            carriers: BTreeMap::new(),
        };
        let mut partial = Vec::new();
        for i in 0..257u128 {
            partial.push(e.word((i * 73) & 65535, 16).unwrap());
        }
        let value = e.slots(9, 5, &partial).unwrap();
        let ty = e.b.cube(value.depth).unwrap();
        e.b.define("Test.LargePartial", ty, value.term).unwrap();
        let full = (0..16384u128)
            .map(|i| e.word(u128::from(i.count_ones() % 2 == 1), 1).unwrap())
            .collect::<Vec<_>>();
        let value = e.slots(14, 0, &full).unwrap();
        let ty = e.b.cube(value.depth).unwrap();
        e.b.define("Test.LargeFull", ty, value.term).unwrap();
        let bytes = e.b.finish().unwrap();
        let cert = decode_canonical_certificate(&bytes).unwrap();
        // The observed address convention is independent of the new mux tree:
        // role bits first, then zero padding, then child bits, all low first.
        for i in 0..512usize {
            for bit in 0..32usize {
                let address = i | (bit << 9);
                let args = (0..14).map(|n| V::Bit(address & (1 << n) != 0)).collect();
                let expected = i < 257 && bit % 2 == 0 && ((i * 73) & (1 << (bit / 2))) != 0;
                assert_eq!(
                    observe(run(&cert, "Test.LargePartial", args)),
                    expected,
                    "partial slot {i}, padded child bit {bit}"
                );
            }
        }
        for i in 0..16384u32 {
            let args = (0..14).map(|n| V::Bit(i & (1 << n) != 0)).collect();
            assert_eq!(
                observe(run(&cert, "Test.LargeFull", args)),
                i.count_ones() % 2 == 1
            );
        }
    }

    #[test]
    fn ordinary_literal_parts_preserve_deep_padding_and_sharing() {
        let mut e = Literals {
            b: Builder::new().unwrap(),
            parts: BTreeMap::new(),
            carriers: BTreeMap::new(),
        };
        let low = e.word(1, 1).unwrap();
        let high = e.word(1 << 15, 16).unwrap();
        let product = e.product(&[low, high]).unwrap();
        assert_eq!(product.depth, 5);
        let ty = e.b.cube(product.depth).unwrap();
        e.b.define("Test.LiteralProduct", ty, product.term).unwrap();
        let child = e.word(128, 8).unwrap();
        let deep = e.slots(0, 253, &[child]).unwrap();
        let ty = e.b.cube(253).unwrap();
        e.b.define("Test.LiteralDeep", ty, deep.term).unwrap();
        let shared = e.share(child).unwrap();
        let count = e.b.c.declarations.len();
        assert_eq!(e.share(child).unwrap(), shared);
        assert_eq!(e.b.c.declarations.len(), count);
        assert!(e.slots(0, 254, &[child]).is_err());
        assert!(e.slots(0, 2, &[child]).is_err());
        let bytes = e.b.finish().unwrap();
        pin("deep-padding", &bytes);
        let certificate = decode_canonical_certificate(&bytes).unwrap();
        for i in 0..32 {
            let args = (0..5).map(|bit| V::Bit(i & (1 << bit) != 0)).collect();
            assert_eq!(
                observe(run(&certificate, "Test.LiteralProduct", args)),
                i == 0 || i == 31
            );
        }
        let mut address = vec![false; 253];
        address[250..].fill(true);
        let observe_address = |bits: &[bool]| {
            observe(run(
                &certificate,
                "Test.LiteralDeep",
                bits.iter().copied().map(V::Bit).collect(),
            ))
        };
        assert!(observe_address(&address));
        // Every padding selector is observed, including positions above 32/128.
        for i in 0..253 {
            address[i] = !address[i];
            assert!(!observe_address(&address), "selector {i}");
            address[i] = !address[i];
        }
    }
}
