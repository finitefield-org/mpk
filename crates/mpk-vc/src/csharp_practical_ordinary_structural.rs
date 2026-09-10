//! W09 unit 3 storage constructors/projections for closed products and sums.
//! These functions require valid child values. They do not discharge domains,
//! source constructor invariants, validation bounds or application VCs.
use super::*;
#[path = "csharp_practical_ordinary_binding_projections.rs"]
mod binding_projections;
pub use binding_projections::{
    generate_csharp_practical_ordinary_binding_projections,
    generate_csharp_practical_ordinary_binding_rebuilds,
    import_csharp_practical_ordinary_binding_projections,
    import_csharp_practical_ordinary_binding_rebuilds, OrdinaryBindingProjectionDefinition,
    OrdinaryBindingProjectionProgram, OrdinaryBindingRebuildDefinition,
    OrdinaryBindingRebuildProgram,
};
#[path = "csharp_practical_ordinary_literals.rs"]
mod literals;
pub use literals::{
    generate_csharp_practical_ordinary_literals, import_csharp_practical_ordinary_literals,
    OrdinaryLiteralBinding, OrdinaryLiteralDefinition, OrdinaryLiteralOrigin,
    OrdinaryLiteralProgram,
};

pub(super) fn emit_literal_values(
    vir: &ValidatedPracticalVir,
    layouts: &OrdinaryCarrierProgram,
    b: Builder,
    values: BTreeMap<String, MonomorphicValue>,
) -> R<(Builder, Vec<OrdinaryLiteralDefinition>)> {
    literals::emit_named_values(vir, layouts, b, values)
}

#[path = "csharp_practical_ordinary_construction_ops.rs"]
mod construction_ops;
pub use construction_ops::{
    generate_csharp_practical_ordinary_constructions,
    import_csharp_practical_ordinary_constructions, OrdinaryConstructionDefinition,
    OrdinaryConstructionFailure, OrdinaryConstructionOperation, OrdinaryConstructionProgram,
};

#[path = "csharp_practical_ordinary_non_templates.rs"]
mod non_templates;
pub use non_templates::{
    generate_csharp_practical_ordinary_finite_operations,
    import_csharp_practical_ordinary_finite_operations, OrdinaryFiniteOperation,
    OrdinaryFiniteOperationProgram,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryProjection {
    pub field_id: String,
    pub depth: u32,
    pub definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryProductOperations {
    pub make_definition: String,
    pub fields: Vec<OrdinaryProjection>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryArmOperations {
    pub tag: u32,
    pub arm_id: String,
    pub make_definition: String,
    pub is_active_definition: String,
    pub invalid_operation_definition: String,
    pub fields: Vec<OrdinaryProjection>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OrdinaryStructuralOperations {
    Product {
        operations: OrdinaryProductOperations,
    },
    Sum {
        tag_definition: String,
        arms: Vec<OrdinaryArmOperations>,
    },
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryStructuralDefinition {
    pub carrier: OrdinaryCarrier,
    pub operations: OrdinaryStructuralOperations,
}

/// One concrete stored shape has one constructor/projection family per VIR.
/// Sharing is explicit here; Builder still rejects duplicate global definitions.
#[derive(Default)]
struct StorageCache(BTreeMap<String, OrdinaryStructuralDefinition>);
impl StorageCache {
    fn get(
        &mut self,
        b: &mut Builder,
        carrier: &OrdinaryCarrier,
        carriers: &BTreeMap<&str, &OrdinaryCarrier>,
    ) -> R<Option<OrdinaryStructuralDefinition>> {
        if let Some(value) = self.0.get(&carrier.type_id) {
            if value.carrier != *carrier {
                return Err(OrdinaryCarrierError::Linkage);
            }
            return Ok(Some(value.clone()));
        }
        let value = emit(b, carrier, carriers)?;
        if let Some(value) = &value {
            self.0.insert(carrier.type_id.clone(), value.clone());
        }
        Ok(value)
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryStructuralProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryStructuralDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryStructuralProgram {
    pub fn definitions(&self) -> &[OrdinaryStructuralDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed structural program")
    }
}
fn call(b: &mut Builder, name: &str, args: Vec<u32>) -> R<u32> {
    let f = b.constant(name)?;
    b.app(f, args)
}
fn bit(b: &mut Builder, value: bool) -> R<u32> {
    b.constant(if value {
        "Std.Bool.true"
    } else {
        "Std.Bool.false"
    })
}
fn mux(b: &mut Builder, c: u32, yes: u32, no: u32) -> R<u32> {
    call(b, "Std.Bool.rec", vec![no, yes, c])
}
fn define(b: &mut Builder, name: &str, inputs: &[u32], result: u32, mut body: u32) -> R<()> {
    let mut ty = b.cube(result)?;
    for d in inputs.iter().rev() {
        let input = b.cube(*d)?;
        body = b.lam(input, body)?;
        ty = b.pi(input, ty)?;
    }
    b.define(name, ty, body)
}
fn shape_depth(s: &OrdinaryShape, carriers: &BTreeMap<&str, &OrdinaryCarrier>) -> R<u32> {
    let d = match s {
        OrdinaryShape::Bits { width } => address_bits(*width),
        OrdinaryShape::Reference { type_id } => {
            carriers
                .get(type_id.as_str())
                .ok_or(OrdinaryCarrierError::Linkage)?
                .depth
        }
        OrdinaryShape::RoleBound { value, .. } => shape_depth(value, carriers)?,
        OrdinaryShape::Product { fields } => product_depths(fields, carriers)?.1,
        OrdinaryShape::Array { capacity, element } => {
            address_bits(*capacity) + shape_depth(element, carriers)?
        }
        OrdinaryShape::Sequence { capacity, element } => {
            1 + 5.max(address_bits(*capacity) + shape_depth(element, carriers)?)
        }
        OrdinaryShape::Sum { arms } => {
            let mut payload = 0;
            for a in arms {
                payload = payload.max(product_depths(&a.fields, carriers)?.1);
            }
            1 + 5.max(payload)
        }
    };
    if d > 253 {
        Err(OrdinaryCarrierError::Limit)
    } else {
        Ok(d)
    }
}
fn product_depths(
    fields: &[OrdinaryField],
    carriers: &BTreeMap<&str, &OrdinaryCarrier>,
) -> R<(Vec<u32>, u32)> {
    let ds = fields
        .iter()
        .map(|f| shape_depth(&f.shape, carriers))
        .collect::<R<Vec<_>>>()?;
    let count = u32::try_from(ds.len()).map_err(|_| OrdinaryCarrierError::Limit)?;
    let depth = address_bits(count) + ds.iter().copied().max().unwrap_or(0);
    if depth + count > 256 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok((ds, depth))
}
// Selectors occupy the last `outer` binders. Address groups and fields are
// least-significant-first, exactly as in the independently rebuilt carrier.
fn equal_address(b: &mut Builder, outer: u32, start: u32, width: u32, value: u32) -> R<u32> {
    if start + width > outer || width > 32 {
        return Err(OrdinaryCarrierError::Shape);
    }
    let mut yes = bit(b, true)?;
    for i in 0..width {
        let mut v = b.var(outer - 1 - start - i)?;
        if value & (1 << i) == 0 {
            v = call(b, "Std.Bool.not", vec![v])?;
        }
        yes = call(b, "Std.Bool.and", vec![yes, v])?;
    }
    Ok(yes)
}
fn zero_padding(b: &mut Builder, outer: u32, start: u32, count: u32, value: u32) -> R<u32> {
    // Padding may exceed 32 selectors; every padding selector must be false.
    if start + count > outer {
        return Err(OrdinaryCarrierError::Shape);
    }
    let mut padded = bit(b, false)?;
    for i in start..start + count {
        let v = b.var(outer - 1 - i)?;
        padded = call(b, "Std.Bool.or", vec![padded, v])?;
    }
    let f = bit(b, false)?;
    mux(b, padded, f, value)
}
fn product_leaf(b: &mut Builder, ds: &[u32], outer: u32, start: u32) -> R<u32> {
    let count = u32::try_from(ds.len()).map_err(|_| OrdinaryCarrierError::Limit)?;
    if outer + count > 256 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let roles = address_bits(count);
    let max = ds.iter().copied().max().unwrap_or(0);
    if start + roles + max != outer {
        return Err(OrdinaryCarrierError::Shape);
    }
    let mut result = bit(b, false)?;
    for (i, d) in ds.iter().copied().enumerate().rev() {
        let arg = b.var(outer + count - 1 - i as u32)?;
        let address = b.selectors(d)?;
        let child = b.app(arg, address)?;
        let child = zero_padding(b, outer, start + roles, max - d, child)?;
        let selected = equal_address(b, outer, start, roles, i as u32)?;
        result = mux(b, selected, child, result)?;
    }
    Ok(result)
}
fn prefix(bits: u32, value: u32) -> Vec<bool> {
    (0..bits).map(|i| value & (1 << i) != 0).collect()
}
fn project(
    b: &mut Builder,
    name: &str,
    source_depth: u32,
    target_depth: u32,
    prefix: &[bool],
    active: Option<&str>,
) -> R<()> {
    if prefix.len() as u32 + target_depth != source_depth {
        return Err(OrdinaryCarrierError::Shape);
    }
    let source = b.var(target_depth)?;
    let mut address = prefix.iter().map(|v| bit(b, *v)).collect::<R<Vec<_>>>()?;
    address.extend(b.selectors(target_depth)?);
    let mut body = b.app(source, address)?;
    if let Some(active) = active {
        let valid = call(b, active, vec![source])?;
        let f = bit(b, false)?;
        body = mux(b, valid, body, f)?;
    }
    let body = b.wrap_selectors(target_depth, body)?;
    define(b, name, &[source_depth], target_depth, body)
}
fn product_fields(
    b: &mut Builder,
    name: &str,
    fields: &[OrdinaryField],
    ds: &[u32],
    source_depth: u32,
    leading: &[bool],
    active: Option<&str>,
) -> R<Vec<OrdinaryProjection>> {
    let roles = address_bits(fields.len() as u32);
    let max = ds.iter().copied().max().unwrap_or(0);
    fields
        .iter()
        .zip(ds)
        .enumerate()
        .map(|(i, (f, d))| {
            let name = format!("{name}.Field.F{i}");
            let mut address = leading.to_vec();
            address.extend(prefix(roles, i as u32));
            address.extend(vec![false; (max - d) as usize]);
            project(b, &name, source_depth, *d, &address, active)?;
            Ok(OrdinaryProjection {
                field_id: f.id.clone(),
                depth: *d,
                definition: name,
            })
        })
        .collect()
}
pub(super) fn emit(
    b: &mut Builder,
    carrier: &OrdinaryCarrier,
    carriers: &BTreeMap<&str, &OrdinaryCarrier>,
) -> R<Option<OrdinaryStructuralDefinition>> {
    if shape_depth(&carrier.shape, carriers)? != carrier.depth {
        return Err(OrdinaryCarrierError::Shape);
    }
    let hex = carrier
        .type_id
        .as_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let name = format!("{PREFIX}.Structural.T{hex}");
    let operations = match &carrier.shape {
        OrdinaryShape::Product { fields } => {
            let (ds, d) = product_depths(fields, carriers)?;
            let body = product_leaf(b, &ds, d, 0)?;
            let body = b.wrap_selectors(d, body)?;
            let make_definition = format!("{name}.MakeStorage");
            define(b, &make_definition, &ds, d, body)?;
            let fields = product_fields(b, &name, fields, &ds, d, &[], None)?;
            OrdinaryStructuralOperations::Product {
                operations: OrdinaryProductOperations {
                    make_definition,
                    fields,
                },
            }
        }
        OrdinaryShape::Sum { arms } => {
            let d = carrier.depth;
            let payload = d - 1;
            let tag_definition = format!("{name}.Tag");
            let mut tag_prefix = vec![false];
            tag_prefix.extend(vec![false; (payload - 5) as usize]);
            project(b, &tag_definition, d, 5, &tag_prefix, None)?;
            let mut operations = vec![];
            let mut tags = BTreeSet::new();
            for a in arms {
                if !tags.insert(a.tag) {
                    return Err(OrdinaryCarrierError::Shape);
                }
                let arm_name = format!("{name}.Arm.A{}", a.tag);
                let (ds, arm_depth) = product_depths(&a.fields, carriers)?;
                let body = product_leaf(b, &ds, d, 1 + payload - arm_depth)?;
                let value = zero_padding(b, d, 1, payload - arm_depth, body)?;
                let mut tag = bit(b, false)?;
                for i in 0..32 {
                    if a.tag & (1 << i) != 0 {
                        let at = equal_address(b, d, d - 5, 5, i)?;
                        tag = call(b, "Std.Bool.or", vec![tag, at])?;
                    }
                }
                let tag = zero_padding(b, d, 1, payload - 5, tag)?;
                let role = b.var(d - 1)?;
                let body = mux(b, role, value, tag)?;
                let body = b.wrap_selectors(d, body)?;
                let make_definition = format!("{arm_name}.MakeStorage");
                define(b, &make_definition, &ds, d, body)?;
                let is_active_definition = format!("{arm_name}.IsActive");
                let source = b.var(0)?;
                let tag = call(b, &tag_definition, vec![source])?;
                let mut active = bit(b, true)?;
                for i in 0..32 {
                    let address = prefix(5, i)
                        .into_iter()
                        .map(|v| bit(b, v))
                        .collect::<R<Vec<_>>>()?;
                    let mut v = b.app(tag, address)?;
                    if a.tag & (1 << i) == 0 {
                        v = call(b, "Std.Bool.not", vec![v])?;
                    }
                    active = call(b, "Std.Bool.and", vec![active, v])?;
                }
                define(b, &is_active_definition, &[d], 0, active)?;
                let invalid_operation_definition = format!("{arm_name}.InvalidOperation");
                let inactive = call(b, "Std.Bool.not", vec![active])?;
                define(b, &invalid_operation_definition, &[d], 0, inactive)?;
                let mut leading = vec![true];
                leading.extend(vec![false; (payload - arm_depth) as usize]);
                let fields = product_fields(
                    b,
                    &arm_name,
                    &a.fields,
                    &ds,
                    d,
                    &leading,
                    Some(&is_active_definition),
                )?;
                operations.push(OrdinaryArmOperations {
                    tag: a.tag,
                    arm_id: a.id.clone(),
                    make_definition,
                    is_active_definition,
                    invalid_operation_definition,
                    fields,
                });
            }
            OrdinaryStructuralOperations::Sum {
                tag_definition,
                arms: operations,
            }
        }
        _ => return Ok(None),
    };
    Ok(Some(OrdinaryStructuralDefinition {
        carrier: carrier.clone(),
        operations,
    }))
}
/// Emit storage-level product/sum constructors and projections for every
/// reachable carrier of those shapes, including uninvoked concrete instances.
/// Semantic domains and public constructor/check obligations remain separate.
pub fn generate_csharp_practical_ordinary_structural(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryStructuralProgram> {
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let carriers = layouts
        .carriers()
        .iter()
        .map(|c| (c.type_id.as_str(), c))
        .collect::<BTreeMap<_, _>>();
    let mut b = Builder::new()?;
    let mut definitions = vec![];
    for c in carriers.values() {
        if let Some(d) = emit(&mut b, c, &carriers)? {
            definitions.push(d);
        }
    }
    let certificate = b.finish()?;
    let p = OrdinaryStructuralProgram {
        schema: "mpk.csharp.ordinary_structural_storage.v1".into(),
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
pub fn import_csharp_practical_ordinary_structural(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryStructuralProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_structural(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}

#[cfg(test)]
mod tests {
    use super::super::tests::{apply, bit as observed_bit, run, V};
    use super::*;

    fn fixture() -> (Certificate, Vec<OrdinaryStructuralDefinition>, Vec<u8>) {
        let fields = vec![
            field("flag", bits(1)),
            field("small", bits(3)),
            field("byte", bits(8)),
        ];
        let carriers = [
            OrdinaryCarrier {
                type_id: "Test.Product".into(),
                depth: 5,
                shape: product(fields.clone()),
            },
            OrdinaryCarrier {
                type_id: "Test.Empty".into(),
                depth: 0,
                shape: product(vec![]),
            },
            OrdinaryCarrier {
                type_id: "Test.Sum".into(),
                depth: 6,
                shape: OrdinaryShape::Sum {
                    arms: vec![
                        OrdinaryArm {
                            tag: 0,
                            id: "none".into(),
                            fields: vec![],
                        },
                        OrdinaryArm {
                            tag: 7,
                            id: "fields".into(),
                            fields,
                        },
                        OrdinaryArm {
                            tag: 0x80000000,
                            id: "high".into(),
                            fields: vec![field("bit", bits(1))],
                        },
                    ],
                },
            },
        ];
        let map = carriers.iter().map(|c| (c.type_id.as_str(), c)).collect();
        let mut b = Builder::new().unwrap();
        let defs = carriers
            .iter()
            .map(|c| emit(&mut b, c, &map).unwrap().unwrap())
            .collect();
        let bytes = b.finish().unwrap();
        let cert = decode_canonical_certificate(&bytes).unwrap();
        (cert, defs, bytes)
    }
    fn leaves(cert: &Certificate, value: V, depth: u32) -> Vec<bool> {
        (0..1 << depth)
            .map(|address| {
                let mut v = value.clone();
                for bit in 0..depth {
                    v = apply(cert, v, V::Bit(address & (1 << bit) != 0));
                }
                observed_bit(v)
            })
            .collect()
    }
    fn inputs() -> Vec<V> {
        vec![
            V::Bit(true),
            V::Cube(vec![true, false, true, false]),
            V::Cube(vec![true, false, true, true, false, true, false, true]),
        ]
    }
    fn expected_product() -> Vec<bool> {
        (0..32)
            .map(|i| match i & 3 {
                0 => i >> 2 == 0,
                1 => i & 4 == 0 && [true, false, true, false][i >> 3],
                2 => [true, false, true, true, false, true, false, true][i >> 2],
                _ => false,
            })
            .collect()
    }
    #[test]
    fn structural_storage_products_preserve_fields_and_padding() {
        let (cert, defs, _) = fixture();
        let OrdinaryStructuralOperations::Product { operations } = &defs[0].operations else {
            panic!()
        };
        let value = run(&cert, &operations.make_definition, inputs());
        assert_eq!(leaves(&cert, value.clone(), 5), expected_product());
        for (f, input) in operations.fields.iter().zip(inputs()) {
            assert_eq!(
                leaves(
                    &cert,
                    run(&cert, &f.definition, vec![value.clone()]),
                    f.depth
                ),
                leaves(&cert, input, f.depth)
            );
        }
        let OrdinaryStructuralOperations::Product { operations } = &defs[1].operations else {
            panic!()
        };
        assert!(!observed_bit(run(
            &cert,
            &operations.make_definition,
            vec![]
        )));
    }
    #[test]
    fn structural_storage_sum_tags_active_fields_and_failure() {
        let (cert, defs, _) = fixture();
        let OrdinaryStructuralOperations::Sum {
            tag_definition,
            arms,
        } = &defs[2].operations
        else {
            panic!()
        };
        for (which, args, payload) in [
            (0, vec![], vec![false; 32]),
            (1, inputs(), expected_product()),
            (2, vec![V::Bit(true)], (0..32).map(|i| i == 0).collect()),
        ] {
            let arm = &arms[which];
            let value = run(&cert, &arm.make_definition, args);
            let expected = (0..64)
                .map(|i| {
                    if i & 1 == 0 {
                        arm.tag & (1 << (i >> 1)) != 0
                    } else {
                        payload[i >> 1]
                    }
                })
                .collect::<Vec<_>>();
            assert_eq!(leaves(&cert, value.clone(), 6), expected);
            assert_eq!(
                leaves(&cert, run(&cert, tag_definition, vec![value.clone()]), 5),
                (0..32).map(|i| arm.tag & (1 << i) != 0).collect::<Vec<_>>()
            );
            for (i, other) in arms.iter().enumerate() {
                assert_eq!(
                    observed_bit(run(&cert, &other.is_active_definition, vec![value.clone()])),
                    i == which
                );
                assert_eq!(
                    observed_bit(run(
                        &cert,
                        &other.invalid_operation_definition,
                        vec![value.clone()]
                    )),
                    i != which
                );
                for f in &other.fields {
                    let out = leaves(
                        &cert,
                        run(&cert, &f.definition, vec![value.clone()]),
                        f.depth,
                    );
                    if i != which {
                        assert_eq!(out, vec![false; 1 << f.depth]);
                    } else {
                        let input = if i == 1 {
                            inputs().remove(
                                other
                                    .fields
                                    .iter()
                                    .position(|x| x.field_id == f.field_id)
                                    .unwrap(),
                            )
                        } else {
                            V::Bit(true)
                        };
                        assert_eq!(out, leaves(&cert, input, f.depth));
                    }
                }
            }
        }
        // Unknown tag, with nonzero payload garbage, must not be mistaken for
        // any active arm and cannot expose that payload through a field getter.
        let raw = (0..64)
            .map(|i| i & 1 == 1 || 42u32 & (1 << (i >> 1)) != 0)
            .collect::<Vec<_>>();
        for arm in arms {
            assert!(!observed_bit(run(
                &cert,
                &arm.is_active_definition,
                vec![V::Cube(raw.clone())]
            )));
            assert!(observed_bit(run(
                &cert,
                &arm.invalid_operation_definition,
                vec![V::Cube(raw.clone())]
            )));
            for f in &arm.fields {
                assert_eq!(
                    leaves(
                        &cert,
                        run(&cert, &f.definition, vec![V::Cube(raw.clone())]),
                        f.depth
                    ),
                    vec![false; 1 << f.depth]
                );
            }
        }
    }
    #[test]
    fn structural_storage_binder_boundary_and_wide_padding() {
        let mut shape = bits(1);
        for _ in 0..251 {
            shape = OrdinaryShape::Array {
                capacity: 2,
                element: Box::new(shape),
            };
        }
        let deep = OrdinaryCarrier {
            type_id: "Test.Deep".into(),
            depth: 251,
            shape,
        };
        let carrier = OrdinaryCarrier {
            type_id: "Test.Boundary".into(),
            depth: 253,
            shape: product(vec![
                field("first", bits(1)),
                field("deep", reference(&deep.type_id)),
                field("last", bits(1)),
            ]),
        };
        let carriers = BTreeMap::from([(deep.type_id.as_str(), &deep)]);
        assert_eq!(shape_depth(&deep.shape, &carriers), Ok(251));
        let mut b = Builder::new().unwrap();
        let zero = bit(&mut b, false).unwrap();
        let zero = b.wrap_selectors(251, zero).unwrap();
        define(&mut b, "Test.Deep.Zero", &[], 251, zero).unwrap();
        let definition = emit(&mut b, &carrier, &carriers).unwrap().unwrap();
        let bytes = b.finish().unwrap();
        let cert = decode_canonical_certificate(&bytes).unwrap();
        let OrdinaryStructuralOperations::Product { operations } = &definition.operations else {
            panic!()
        };
        let value = run(
            &cert,
            &operations.make_definition,
            vec![
                V::Bit(true),
                run(&cert, "Test.Deep.Zero", vec![]),
                V::Bit(true),
            ],
        );
        for (role, padding, expected) in [
            (0u32, None, true),
            (0, Some(200), false),
            (2, None, true),
            (2, Some(35), false),
            (3, None, false),
            (1, None, false),
        ] {
            let mut selected = value.clone();
            for i in 0..253 {
                let bit = if i < 2 {
                    role & (1 << i) != 0
                } else {
                    padding == Some(i)
                };
                selected = apply(&cert, selected, V::Bit(bit));
            }
            assert_eq!(observed_bit(selected), expected);
        }
        let too_wide = OrdinaryCarrier {
            type_id: "Test.Excess".into(),
            depth: 253,
            shape: product(
                (0..4)
                    .map(|i| field(&i.to_string(), reference(&deep.type_id)))
                    .collect(),
            ),
        };
        assert_eq!(
            emit(&mut Builder::new().unwrap(), &too_wide, &carriers),
            Err(OrdinaryCarrierError::Limit)
        );
        let missing = OrdinaryCarrier {
            type_id: "Test.Missing".into(),
            depth: 0,
            shape: product(vec![field("missing", reference("Absent"))]),
        };
        assert_eq!(
            emit(&mut Builder::new().unwrap(), &missing, &carriers),
            Err(OrdinaryCarrierError::Linkage)
        );
        let duplicate = OrdinaryCarrier {
            type_id: "Test.Duplicate".into(),
            depth: 6,
            shape: OrdinaryShape::Sum {
                arms: vec![
                    OrdinaryArm {
                        tag: 7,
                        id: "one".into(),
                        fields: vec![],
                    },
                    OrdinaryArm {
                        tag: 7,
                        id: "two".into(),
                        fields: vec![],
                    },
                ],
            },
        };
        assert_eq!(
            emit(&mut Builder::new().unwrap(), &duplicate, &carriers),
            Err(OrdinaryCarrierError::Shape)
        );
        let metrics = serde_json::json!({"definition":definition,"terms":cert.term_table.len(),"declarations":cert.declarations.len(),"accepted_binder_depth":256,"rejected_binder_depth":257,"certificate_sha256":mpk_cert::hash_hex(&mpk_cert::certificate_hash(&bytes))});
        let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
        if let Some(dir) = std::env::var_os("MPK_W09_STRUCTURAL_OUT") {
            let dir = std::path::PathBuf::from(dir);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("binder-boundary.hex"), hex).unwrap();
            std::fs::write(
                dir.join("binder-metrics.json"),
                serde_json::to_vec_pretty(&metrics).unwrap(),
            )
            .unwrap();
        } else {
            let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../develop/migrations/csharp-03/ordinary-foundation/structural-storage");
            assert_eq!(
                std::fs::read_to_string(dir.join("binder-boundary.hex")).unwrap(),
                hex
            );
            assert_eq!(
                serde_json::from_slice::<Value>(
                    &std::fs::read(dir.join("binder-metrics.json")).unwrap()
                )
                .unwrap(),
                metrics
            );
        }
    }
    #[test]
    fn structural_storage_certificates_reproduce() {
        let (cert, defs, bytes) = fixture();
        let (_, again, bytes_again) = fixture();
        assert_eq!(defs, again);
        assert_eq!(bytes, bytes_again);
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/structural-storage");
        let metrics = serde_json::json!({"definitions":defs,"terms":cert.term_table.len(),"declarations":cert.declarations.len(),"certificate_sha256":mpk_cert::hash_hex(&mpk_cert::certificate_hash(&bytes))});
        let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
        if let Some(out) = std::env::var_os("MPK_W09_STRUCTURAL_OUT") {
            let out = std::path::PathBuf::from(out);
            std::fs::create_dir_all(&out).unwrap();
            std::fs::write(out.join("constructors.hex"), hex).unwrap();
            std::fs::write(
                out.join("core-metrics.json"),
                serde_json::to_vec_pretty(&metrics).unwrap(),
            )
            .unwrap();
        } else {
            assert_eq!(
                std::fs::read_to_string(dir.join("constructors.hex")).unwrap(),
                hex
            );
            assert_eq!(
                serde_json::from_slice::<Value>(
                    &std::fs::read(dir.join("core-metrics.json")).unwrap()
                )
                .unwrap(),
                metrics
            );
        }
    }
}

#[path = "csharp_practical_ordinary_scalar_domains.rs"]
mod scalar_domains;
pub use scalar_domains::{
    generate_csharp_practical_ordinary_scalar_domains,
    import_csharp_practical_ordinary_scalar_domains, OrdinaryScalarDomainDefinition,
    OrdinaryScalarDomainProgram, OrdinaryScalarDomainRule,
};

#[path = "csharp_practical_ordinary_ordered_fold.rs"]
mod ordered_fold;
pub use ordered_fold::{
    generate_csharp_practical_ordinary_ordered_folds,
    import_csharp_practical_ordinary_ordered_folds, OrdinaryOrderedFoldDefinition,
    OrdinaryOrderedFoldProgram,
};

#[path = "csharp_practical_ordinary_relations.rs"]
mod relations;
pub(super) use relations::emit_ordered_key_relations;
pub use relations::{
    generate_csharp_practical_ordinary_domains, generate_csharp_practical_ordinary_relations,
    import_csharp_practical_ordinary_domains, import_csharp_practical_ordinary_relations,
    OrdinaryDomainDefinition, OrdinaryDomainProgram, OrdinaryRelationDefinition,
    OrdinaryRelationProgram,
};
pub use relations::{
    generate_csharp_practical_ordinary_entries, import_csharp_practical_ordinary_entries,
    OrdinaryEntryDefinition, OrdinaryEntryProgram,
};
pub use relations::{
    generate_csharp_practical_ordinary_money, import_csharp_practical_ordinary_money,
    OrdinaryMoneyDefinition, OrdinaryMoneyFailure, OrdinaryMoneyOperation, OrdinaryMoneyProgram,
};
pub use relations::{
    generate_csharp_practical_ordinary_observations, import_csharp_practical_ordinary_observations,
    OrdinaryObservationDefinition, OrdinaryObservationProgram,
};
pub use relations::{
    generate_csharp_practical_ordinary_outcomes, import_csharp_practical_ordinary_outcomes,
    OrdinaryOutcomeDefinition, OrdinaryOutcomeFailure, OrdinaryOutcomeOperation,
    OrdinaryOutcomeProgram,
};
pub use relations::{
    generate_csharp_practical_ordinary_sequences, import_csharp_practical_ordinary_sequences,
    OrdinarySequenceOperations, OrdinarySequenceProgram,
};

#[path = "csharp_practical_ordinary_aggregate_fold.rs"]
mod aggregate_fold;
pub(super) use aggregate_fold::emit_fold as emit_aggregate_fold;

pub use aggregate_fold::{
    generate_csharp_practical_ordinary_aggregate_folds,
    import_csharp_practical_ordinary_aggregate_folds, OrdinaryAggregateFoldDefinition,
    OrdinaryAggregateFoldProgram,
};

pub use relations::{
    generate_csharp_practical_ordinary_collections, import_csharp_practical_ordinary_collections,
    OrdinaryCollectionDefinition, OrdinaryCollectionFailure, OrdinaryCollectionOperation,
    OrdinaryCollectionProgram,
};

pub use relations::{
    generate_csharp_practical_ordinary_structural_boundary,
    generate_csharp_practical_ordinary_structural_foundations,
    generate_csharp_practical_ordinary_structural_public,
    import_csharp_practical_ordinary_structural_boundary,
    import_csharp_practical_ordinary_structural_foundations,
    import_csharp_practical_ordinary_structural_public, OrdinaryDeferredFoundationInstance,
    OrdinaryStructuralFoundationProgram,
};

pub use literals::{
    generate_csharp_practical_ordinary_boundary_literals,
    import_csharp_practical_ordinary_boundary_literals, OrdinaryBoundaryLiteralBinding,
    OrdinaryBoundaryLiteralProgram,
};

pub use relations::{
    generate_csharp_practical_ordinary_binding_guards,
    generate_csharp_practical_ordinary_binding_orders,
    generate_csharp_practical_ordinary_binding_relations,
    generate_csharp_practical_ordinary_boundary_rules,
    import_csharp_practical_ordinary_binding_guards,
    import_csharp_practical_ordinary_binding_orders,
    import_csharp_practical_ordinary_binding_relations,
    import_csharp_practical_ordinary_boundary_rules, OrdinaryBindingAgreement,
    OrdinaryBindingPredicate, OrdinaryBindingRelationProgram, OrdinaryBoundaryRuleProgram,
};

#[path = "csharp_practical_ordinary_source_clauses.rs"]
mod source_clauses;
pub use source_clauses::{
    generate_csharp_practical_ordinary_contract_expressions,
    generate_csharp_practical_ordinary_source_clauses,
    import_csharp_practical_ordinary_contract_expressions,
    import_csharp_practical_ordinary_source_clauses, OrdinaryContractExpressionDefinition,
    OrdinaryContractExpressionProgram, OrdinarySourceClauseDefinition, OrdinarySourceClauseProgram,
};

pub use relations::{
    generate_csharp_practical_ordinary_public_defaults,
    generate_csharp_practical_ordinary_public_domains,
    import_csharp_practical_ordinary_public_defaults,
    import_csharp_practical_ordinary_public_domains, OrdinaryPublicDefaultDefinition,
    OrdinaryPublicDomainDefinition, OrdinaryPublicDomainProgram,
};

pub use source_clauses::{
    generate_csharp_practical_ordinary_integer_data, import_csharp_practical_ordinary_integer_data,
    OrdinaryIntegerDataDefinition, OrdinaryIntegerDataOperation, OrdinaryIntegerDataProgram,
};

pub use source_clauses::{
    generate_csharp_practical_ordinary_structural_data,
    import_csharp_practical_ordinary_structural_data, OrdinaryStructuralDataDefinition,
    OrdinaryStructuralDataOperation, OrdinaryStructuralDataProgram,
};

pub use source_clauses::{
    generate_csharp_practical_ordinary_floating_data,
    import_csharp_practical_ordinary_floating_data, OrdinaryFloatingDataDefinition,
    OrdinaryFloatingDataOperation, OrdinaryFloatingDataProgram,
};

pub use source_clauses::{
    generate_csharp_practical_ordinary_decimal_data, import_csharp_practical_ordinary_decimal_data,
    OrdinaryDecimalDataDefinition, OrdinaryDecimalDataOperation, OrdinaryDecimalDataProgram,
};
