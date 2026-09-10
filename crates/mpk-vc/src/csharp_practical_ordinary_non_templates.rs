//! Finite unit/parse-error and closed-exception operations.
//! Input representation/public domains remain mandatory caller obligations.
use super::super::scalar_bits::bits_relation;
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryFiniteOperation {
    pub operation_id: String,
    /// Closed exception type for construct/is_type, or source member for payload.
    pub specialization: Option<String>,
    pub argument_type_ids: Vec<String>,
    pub result_type_id: String,
    pub definition: String,
    /// Payload reads require this exact active tag in addition to input domains.
    /// The generated zero on other tags does not establish a normal outcome.
    pub active_tag_requirement: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryFiniteOperationProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    operations: Vec<OrdinaryFiniteOperation>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryFiniteOperationProgram {
    pub fn operations(&self) -> &[OrdinaryFiniteOperation] {
        &self.operations
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed finite-operation program")
    }
}

fn operation_name(id: &str, specialization: &str) -> String {
    // Separating fields avoids ambiguous concatenation of source identities.
    let hex = |s: &str| {
        s.as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    };
    format!("{PREFIX}.Finite.O{}.S{}", hex(id), hex(specialization))
}

fn check_operations(bundle: &ValidatedFoundationBundle, id: &str, expected: &[&str]) -> R<()> {
    let definition = bundle
        .non_template_definitions()
        .iter()
        .find(|d| d["id"] == id)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    if definition["operations"] != json!(expected) {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(())
}

fn scalar_operations(
    b: &mut Builder,
    bundle: &ValidatedFoundationBundle,
    carrier: &OrdinaryCarrier,
    token: &str,
) -> R<Vec<OrdinaryFiniteOperation>> {
    let (width, first) = match token {
        "unit" => (0, "make"),
        "parse_error" => (32, "tag"),
        _ => return Err(OrdinaryCarrierError::Shape),
    };
    let id = format!("mpk.csharp.value.{token}.v1");
    let mut operations = vec![];
    if carrier.shape != (OrdinaryShape::Bits { width }) || carrier.depth != address_bits(width) {
        return Err(OrdinaryCarrierError::Shape);
    }
    check_operations(bundle, &id, &[first, "equal", "compare"])?;
    let operation_id = format!("{id}.{first}");
    let definition = operation_name(&operation_id, "");
    let arguments = if width == 0 { vec![] } else { vec![id.clone()] };
    let body = if width == 0 {
        bit(b, false)?
    } else {
        b.var(0)?
    };
    define(
        b,
        &definition,
        &if width == 0 { vec![] } else { vec![5] },
        carrier.depth,
        body,
    )?;
    operations.push(OrdinaryFiniteOperation {
        operation_id,
        specialization: None,
        argument_type_ids: arguments,
        result_type_id: if width == 0 {
            id.clone()
        } else {
            "mpk.csharp.value.u32.v1".into()
        },
        definition,
        active_tag_requirement: None,
    });
    let relation = bits_relation(b, &format!("finite.{token}"), width, false)?;
    for (operation, result, definition) in [
        ("equal", BOOL_TYPE_ID, relation.equal),
        (
            "compare",
            "mpk.csharp.value.i32.v1",
            relation.compare.ok_or(OrdinaryCarrierError::Shape)?,
        ),
    ] {
        operations.push(OrdinaryFiniteOperation {
            operation_id: format!("{id}.{operation}"),
            specialization: None,
            argument_type_ids: vec![id.clone(), id.clone()],
            result_type_id: result.into(),
            definition,
            active_tag_requirement: None,
        });
    }
    Ok(operations)
}

pub fn generate_csharp_practical_ordinary_finite_operations(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryFiniteOperationProgram> {
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let carriers = layouts
        .carriers()
        .iter()
        .map(|c| (c.type_id.as_str(), c))
        .collect::<BTreeMap<_, _>>();
    let mut b = Builder::new()?;
    let mut storage_cache = StorageCache::default();
    let operations = emit_finite(vir, &mut b, &carriers, &mut storage_cache)?;
    let (bundle, _, _) = vir.construction_context();
    let certificate = b.finish()?;
    let program = OrdinaryFiniteOperationProgram {
        schema: "mpk.csharp.ordinary_finite_operations.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: bundle.content_sha256().into(),
        operations,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if program.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(program)
}

pub fn import_csharp_practical_ordinary_finite_operations(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryFiniteOperationProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_finite_operations(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}

pub(super) fn emit_finite(
    vir: &ValidatedPracticalVir,
    b: &mut Builder,
    carriers: &BTreeMap<&str, &OrdinaryCarrier>,
    storage_cache: &mut StorageCache,
) -> R<Vec<OrdinaryFiniteOperation>> {
    let (bundle, roots, _) = vir.construction_context();
    let mut operations = vec![];
    for token in ["unit", "parse_error"] {
        let id = format!("mpk.csharp.value.{token}.v1");
        if let Some(carrier) = carriers.get(id.as_str()) {
            operations.extend(scalar_operations(b, bundle, carrier, token)?);
        }
    }
    if let Some(carrier) = carriers.get(EXCEPTION_TYPE_ID) {
        check_operations(
            bundle,
            EXCEPTION_TYPE_ID,
            &["construct", "is_type", "payload"],
        )?;
        let universe =
            derive_closed_exception_universe(roots, vir.data_closed(), vir.source_exceptions())
                .map_err(|_| OrdinaryCarrierError::Linkage)?;
        let storage = storage_cache
            .get(b, carrier, carriers)?
            .ok_or(OrdinaryCarrierError::Shape)?;
        let OrdinaryStructuralOperations::Sum { arms, .. } = storage.operations else {
            return Err(OrdinaryCarrierError::Shape);
        };
        for arm in universe.arms() {
            let storage_arm = arms
                .iter()
                .find(|a| a.tag == arm.tag && a.arm_id == arm.type_id)
                .ok_or(OrdinaryCarrierError::Linkage)?;
            let operation_id = format!("{EXCEPTION_TYPE_ID}.construct");
            let definition = operation_name(&operation_id, &arm.type_id);
            let (arguments, depths, fields) = if arm.tag < 9 {
                (vec![], vec![], vec![])
            } else {
                let source = carriers
                    .get(arm.type_id.as_str())
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let product = storage_cache
                    .get(b, source, carriers)?
                    .ok_or(OrdinaryCarrierError::Shape)?;
                let OrdinaryStructuralOperations::Product {
                    operations: product,
                } = product.operations
                else {
                    return Err(OrdinaryCarrierError::Shape);
                };
                if product
                    .fields
                    .iter()
                    .map(|f| &f.field_id)
                    .ne(arm.payload_member_ids.iter())
                {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let value = b.var(0)?;
                let fields = product
                    .fields
                    .iter()
                    .map(|f| call(b, &f.definition, vec![value]))
                    .collect::<R<Vec<_>>>()?;
                (vec![arm.type_id.clone()], vec![source.depth], fields)
            };
            let body = call(b, &storage_arm.make_definition, fields)?;
            define(b, &definition, &depths, carrier.depth, body)?;
            operations.push(OrdinaryFiniteOperation {
                operation_id,
                specialization: Some(arm.type_id.clone()),
                argument_type_ids: arguments,
                result_type_id: EXCEPTION_TYPE_ID.into(),
                definition,
                active_tag_requirement: None,
            });

            let operation_id = format!("{EXCEPTION_TYPE_ID}.is_type");
            let definition = operation_name(&operation_id, &arm.type_id);
            let value = b.var(0)?;
            let mut matches = bit(b, false)?;
            for descendant in universe
                .arms()
                .iter()
                .filter(|a| a.ancestry.contains(&arm.type_id))
            {
                let predicate = &arms
                    .iter()
                    .find(|a| a.tag == descendant.tag)
                    .ok_or(OrdinaryCarrierError::Linkage)?
                    .is_active_definition;
                let active = call(b, predicate, vec![value])?;
                matches = call(b, "Std.Bool.or", vec![matches, active])?;
            }
            define(b, &definition, &[carrier.depth], 0, matches)?;
            operations.push(OrdinaryFiniteOperation {
                operation_id,
                specialization: Some(arm.type_id.clone()),
                argument_type_ids: vec![EXCEPTION_TYPE_ID.into()],
                result_type_id: BOOL_TYPE_ID.into(),
                definition,
                active_tag_requirement: None,
            });
            for (index, projection) in storage_arm.fields.iter().enumerate() {
                if arm.payload_member_ids.get(index) != Some(&projection.field_id) {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                operations.push(OrdinaryFiniteOperation {
                    operation_id: format!("{EXCEPTION_TYPE_ID}.payload"),
                    specialization: Some(projection.field_id.clone()),
                    argument_type_ids: vec![EXCEPTION_TYPE_ID.into()],
                    result_type_id: arm
                        .payload_type_ids
                        .get(index)
                        .ok_or(OrdinaryCarrierError::Linkage)?
                        .clone(),
                    definition: projection.definition.clone(),
                    active_tag_requirement: Some(arm.tag),
                });
            }
        }
    }
    operations.sort_by(|a, b| {
        (&a.operation_id, &a.specialization).cmp(&(&b.operation_id, &b.specialization))
    });
    Ok(operations)
}

#[cfg(test)]
mod tests {
    use super::super::super::test_eval::{apply, bit as observed_bit, run, V};
    use super::*;

    #[test]
    fn finite_scalar_operations_observe_every_admitted_value() {
        let bundle = validate_registered_foundation_bundle(
            registered_foundation_descriptor_transport(),
            registered_foundation_definitions_transport(),
        )
        .unwrap();
        let mut b = Builder::new().unwrap();
        let mut operations = vec![];
        for (token, width) in [("unit", 0), ("parse_error", 32)] {
            let carrier = OrdinaryCarrier {
                type_id: format!("mpk.csharp.value.{token}.v1"),
                depth: address_bits(width),
                shape: OrdinaryShape::Bits { width },
            };
            operations.extend(scalar_operations(&mut b, &bundle, &carrier, token).unwrap());
        }
        let bytes = b.finish().unwrap();
        let certificate = decode_canonical_certificate(&bytes).unwrap();
        let find = |token, operation| {
            operations
                .iter()
                .find(|o| o.operation_id == format!("mpk.csharp.value.{token}.v1.{operation}"))
                .unwrap()
                .definition
                .as_str()
        };
        let word = |n: u32| V::Cube((0..32).map(|i| n & (1 << i) != 0).collect());
        let number = |value: V| {
            (0..32).fold(0u32, |n, i| {
                let mut leaf = value.clone();
                for bit in 0..5 {
                    leaf = apply(&certificate, leaf, V::Bit(i & (1 << bit) != 0));
                }
                n | (u32::from(observed_bit(leaf)) << i)
            })
        };
        assert!(!observed_bit(run(
            &certificate,
            find("unit", "make"),
            vec![]
        )));
        assert!(observed_bit(run(
            &certificate,
            find("unit", "equal"),
            vec![V::Bit(false); 2]
        )));
        assert_eq!(
            number(run(
                &certificate,
                find("unit", "compare"),
                vec![V::Bit(false); 2]
            )),
            0
        );
        for left in 0..5 {
            assert_eq!(
                number(run(
                    &certificate,
                    find("parse_error", "tag"),
                    vec![word(left)]
                )),
                left
            );
            for right in 0..5 {
                assert_eq!(
                    observed_bit(run(
                        &certificate,
                        find("parse_error", "equal"),
                        vec![word(left), word(right)]
                    )),
                    left == right
                );
                let expected = if left < right {
                    -1i32
                } else {
                    i32::from(left != right)
                };
                assert_eq!(
                    number(run(
                        &certificate,
                        find("parse_error", "compare"),
                        vec![word(left), word(right)]
                    )) as i32,
                    expected
                );
            }
        }
        assert_eq!(operations.len(), 6);
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/finite-operations");
        let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
        let metadata = json!({"operations":operations,"terms":certificate.term_table.len(),"declarations":certificate.declarations.len(),"scope":"scalar helper certificate; parse-error original-source integration remains separate"});
        if let Some(output) = std::env::var_os("MPK_W09_FINITE_OUT") {
            let output = std::path::PathBuf::from(output);
            std::fs::create_dir_all(&output).unwrap();
            std::fs::write(output.join("core.hex"), hex).unwrap();
            std::fs::write(
                output.join("core.json"),
                serde_json::to_vec_pretty(&metadata).unwrap(),
            )
            .unwrap();
        } else {
            assert_eq!(
                std::fs::read_to_string(fixture.join("core.hex")).unwrap(),
                hex
            );
            assert_eq!(
                serde_json::from_slice::<Value>(&std::fs::read(fixture.join("core.json")).unwrap())
                    .unwrap(),
                metadata
            );
        }
    }
}
