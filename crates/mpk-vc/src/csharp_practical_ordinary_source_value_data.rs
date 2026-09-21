//! W03 source-product construction/field-read relations at original SSA points.
//! Constructor execution, public domains and native control proofs remain separate.
use super::*;
use crate::csharp_practical_vir_model::data_vc::{
    DataDefinitionFamily, DataOperationVc, DataSemanticDefinition,
};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinarySourceValueDataDefinition {
    pub source: DataSemanticDefinition,
    pub value_definition: String,
    pub relation_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinarySourceValueDataOperation {
    pub source: DataOperationVc,
    pub predicates: BTreeMap<String, String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinarySourceValueDataProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    data_vc_sha256: String,
    definitions: Vec<OrdinarySourceValueDataDefinition>,
    operations: Vec<OrdinarySourceValueDataOperation>,
    pending_definition_ids: Vec<String>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinarySourceValueDataProgram {
    pub fn definitions(&self) -> &[OrdinarySourceValueDataDefinition] {
        &self.definitions
    }
    pub fn operations(&self) -> &[OrdinarySourceValueDataOperation] {
        &self.operations
    }
    pub fn pending_definition_ids(&self) -> &[String] {
        &self.pending_definition_ids
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("source value data relations")
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
fn name(kind: &str, identity: &impl Serialize) -> String {
    format!(
        "{PREFIX}.SourceValueData.{kind}.H{:x}",
        Sha256::digest(serde_json::to_vec(identity).expect("typed source value data identity"))
    )
}
// Storage agreement is deliberately bitwise: semantic equality could identify
// different decimal encodings or signed zeros and is non-reflexive for NaNs.
pub(super) fn storage_equal(b: &mut Builder, depth: u32) -> R<String> {
    if u64::from(depth) >= crate::csharp_practical_vc_model::BINDER_DEPTH_MAX {
        return Err(OrdinaryCarrierError::Limit);
    }
    for d in 0..=depth {
        let name = format!("{PREFIX}.SourceValueData.StorageEqual.D{d}");
        if b.globals.contains_key(&name) {
            continue;
        }
        let left = b.var(1)?;
        let right = b.var(0)?;
        let same = if d == 0 {
            let different = call(b, "Std.Bool.not", vec![right])?;
            mux(b, left, right, different)?
        } else {
            let mut halves = vec![];
            for on in [false, true] {
                let selector = bit(b, on)?;
                let left = b.app(left, vec![selector])?;
                let right = b.app(right, vec![selector])?;
                halves.push(call(
                    b,
                    &format!("{PREFIX}.SourceValueData.StorageEqual.D{}", d - 1),
                    vec![left, right],
                )?);
            }
            call(b, "Std.Bool.and", halves)?
        };
        define(b, &name, &[d, d], 0, same)?;
    }
    Ok(format!("{PREFIX}.SourceValueData.StorageEqual.D{depth}"))
}
pub(super) fn emit(
    c: &mut Clauses<'_>,
    d: &DataSemanticDefinition,
) -> R<OrdinarySourceValueDataDefinition> {
    let s = &d.signature;
    let vir = c.vir.ok_or(OrdinaryCarrierError::Linkage)?;
    let roots = vir.construction_context().1;
    let expected = match s.tag {
        ClosedOperationTag::FieldRead => {
            let member =
                s.id.strip_prefix("field.read.")
                    .ok_or(OrdinaryCarrierError::Linkage)?;
            crate::csharp_practical_vir_model::source_field_operation(
                roots,
                vir.data_closed(),
                member,
            )
        }
        ClosedOperationTag::ValueConstruct => {
            crate::csharp_practical_vir_model::source_value_constructor_operation(
                roots,
                vir.data_closed(),
                &s.normal_result_type_id,
            )
        }
        _ => return Err(OrdinaryCarrierError::Linkage),
    }
    .map_err(|_| OrdinaryCarrierError::Linkage)?;
    if d.family != DataDefinitionFamily::SourceValue
        || expected != *s
        || !s.ordered_checks.is_empty()
        || !d.failure_names.is_empty()
        || !d.failure_result_names.is_empty()
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let owner = if s.tag == ClosedOperationTag::FieldRead {
        &s.argument_type_ids[0]
    } else {
        &s.normal_result_type_id
    };
    let carrier = c
        .carriers
        .get(owner.as_str())
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let structure = c
        .storage
        .get(&mut c.b, carrier, &c.carriers)?
        .ok_or(OrdinaryCarrierError::Shape)?;
    let OrdinaryStructuralOperations::Product { operations } = structure.operations else {
        return Err(OrdinaryCarrierError::Shape);
    };
    let value_definition = if s.tag == ClosedOperationTag::FieldRead {
        let member =
            s.id.strip_prefix("field.read.")
                .ok_or(OrdinaryCarrierError::Linkage)?;
        operations
            .fields
            .iter()
            .find(|f| f.field_id == member)
            .ok_or(OrdinaryCarrierError::Linkage)?
            .definition
            .clone()
    } else {
        operations.make_definition
    };
    let inputs = s
        .argument_type_ids
        .iter()
        .chain(std::iter::once(&s.normal_result_type_id))
        .map(|id| {
            c.carriers
                .get(id.as_str())
                .map(|c| c.depth)
                .ok_or(OrdinaryCarrierError::Linkage)
        })
        .collect::<R<Vec<_>>>()?;
    let depth = *inputs.last().ok_or(OrdinaryCarrierError::Shape)?;
    let count = s.argument_type_ids.len();
    let args = (0..count)
        .map(|i| c.b.var((count - i) as u32))
        .collect::<R<Vec<_>>>()?;
    let computed = call(&mut c.b, &value_definition, args)?;
    let result = c.b.var(0)?;
    let actual = c.b.var(1)?;
    let equal = storage_equal(&mut c.b, depth)?;
    let same = call(&mut c.b, &equal, vec![result, actual])?;
    let ty = c.b.cube(depth)?;
    let same = c.b.term(TermNode::Let {
        ty,
        value: computed,
        body: same,
    })?;
    let relation_definition = name("Relation", &d.id);
    define(&mut c.b, &relation_definition, &inputs, 0, same)?;
    let mut args = s.argument_type_ids.clone();
    args.push(s.normal_result_type_id.clone());
    if c.constants
        .insert(
            d.relation_name.clone(),
            (signature(&args, SOURCE_BOOL), relation_definition.clone()),
        )
        .is_some()
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(OrdinarySourceValueDataDefinition {
        source: d.clone(),
        value_definition,
        relation_definition,
    })
}
pub fn generate_csharp_practical_ordinary_source_value_data(
    vir: &ValidatedPracticalVir,
) -> R<OrdinarySourceValueDataProgram> {
    let data = crate::csharp_practical_vir_model::data_vc::generate_data_vcs(vir)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut c = compiler(vir, &layouts, Builder::new()?, &[])?;
    c.definedness_logic()?;
    let mut definitions = vec![];
    let mut pending = vec![];
    for d in data.definitions() {
        if d.family == DataDefinitionFamily::SourceValue {
            definitions.push(emit(&mut c, d)?);
        } else {
            pending.push(d.id.clone());
        }
    }
    let ids = definitions
        .iter()
        .map(|d| d.source.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut operations = vec![];
    for o in data
        .operations()
        .iter()
        .filter(|o| ids.contains(o.definition_id.as_str()))
    {
        if !o.checks.is_empty() {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let mut predicates = BTreeMap::new();
        for (role, term) in [
            ("success_guard", &o.success_guard),
            ("success_relation", &o.success_relation),
            ("success_goal", &o.success_goal),
        ] {
            let core = name("Point", &(&o.id, role));
            integer_data::predicate(&mut c, &core, &o.subjects, term)?;
            predicates.insert(role.into(), core);
        }
        operations.push(OrdinarySourceValueDataOperation {
            source: o.clone(),
            predicates,
        });
    }
    let certificate = c.b.finish()?;
    let program = OrdinarySourceValueDataProgram {
        schema: "mpk.csharp.ordinary_source_value_data.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        data_vc_sha256: data.hash(),
        definitions,
        operations,
        pending_definition_ids: pending,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if program.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(program)
}
pub fn import_csharp_practical_ordinary_source_value_data(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinarySourceValueDataProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_source_value_data(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::super::super::super::test_eval::{bit as observed, run, sparse_cube};
    use super::*;
    #[test]
    fn source_value_storage_preserves_nan_zero_and_padding_bits() {
        let mut b = Builder::new().unwrap();
        storage_equal(&mut b, 7).unwrap();
        assert!(storage_equal(&mut b, 256).is_err());
        let c = mpk_cert::decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        for (depth, pattern) in [
            (0, 0u128),
            (0, 1),
            (5, 0),
            (5, 0x80000000),
            (5, 0x7fc00000),
            (5, 0x7fc00001),
            (7, 1u128 << 127),
        ] {
            let ones = (0..(1usize << depth))
                .filter(|&i| pattern & (1u128 << i) != 0)
                .collect::<BTreeSet<_>>();
            let name = format!("{PREFIX}.SourceValueData.StorageEqual.D{depth}");
            let value = sparse_cube(depth, ones.clone());
            assert!(observed(run(&c, &name, vec![value.clone(), value.clone()])));
            for index in 0..(1usize << depth) {
                let mut altered = ones.clone();
                if !altered.remove(&index) {
                    altered.insert(index);
                }
                assert!(
                    !observed(run(
                        &c,
                        &name,
                        vec![value.clone(), sparse_cube(depth, altered)]
                    )),
                    "depth {depth}, pattern {pattern:x}, bit {index}"
                );
            }
        }
    }
}
