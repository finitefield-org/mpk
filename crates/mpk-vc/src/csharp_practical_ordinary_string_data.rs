//! W03 UTF-16 string operation relations at original SSA use points.
//! Definitions and guarded predicates only; no native-body or application proof.
use super::*;
use crate::csharp_practical_vir_model::data_vc::{
    DataDefinitionFamily, DataOperationVc, DataSemanticDefinition,
};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryStringDataDefinition {
    pub source: DataSemanticDefinition,
    pub scalar: OrdinaryStringDefinition,
    pub relation_definition: String,
    pub independent_failure_definitions: Vec<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryStringDataOperation {
    pub source: DataOperationVc,
    /// Original W03 formula role and corresponding closed ordinary definition.
    pub predicates: BTreeMap<String, String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryStringDataProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    data_vc_sha256: String,
    definitions: Vec<OrdinaryStringDataDefinition>,
    operations: Vec<OrdinaryStringDataOperation>,
    /// Other data families are explicitly outstanding, never silently discharged.
    pending_definition_ids: Vec<String>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryStringDataProgram {
    pub fn definitions(&self) -> &[OrdinaryStringDataDefinition] {
        &self.definitions
    }
    pub fn operations(&self) -> &[OrdinaryStringDataOperation] {
        &self.operations
    }
    pub fn pending_definition_ids(&self) -> &[String] {
        &self.pending_definition_ids
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary string data relations")
    }
}
fn core_name(kind: &str, identity: &impl Serialize) -> String {
    let bytes = serde_json::to_vec(identity).expect("typed string data identity");
    format!("{PREFIX}.StringData.{kind}.H{:x}", Sha256::digest(bytes))
}
pub(super) fn storage_equal(b: &mut Builder, depth: u32) -> R<String> {
    if depth > 19 {
        return Err(OrdinaryCarrierError::Shape);
    }
    for d in 0..=depth {
        let name = format!("{PREFIX}.StringData.StorageEqual.D{d}");
        if b.globals.contains_key(&name) {
            continue;
        }
        let left = b.var(1)?;
        let right = b.var(0)?;
        let same = if d == 0 {
            let not_right = call(b, "Std.Bool.not", vec![right])?;
            mux(b, left, right, not_right)?
        } else {
            let mut halves = vec![];
            for on in [false, true] {
                let selector = bit(b, on)?;
                let l = b.app(left, vec![selector])?;
                let r = b.app(right, vec![selector])?;
                halves.push(call(
                    b,
                    &format!("{PREFIX}.StringData.StorageEqual.D{}", d - 1),
                    vec![l, r],
                )?);
            }
            call(b, "Std.Bool.and", halves)?
        };
        define(b, &name, &[d, d], 0, same)?;
    }
    Ok(format!("{PREFIX}.StringData.StorageEqual.D{depth}"))
}
fn relation(c: &mut Clauses<'_>, scalar: &OrdinaryStringDefinition, name: &str) -> R<()> {
    let s = &scalar.operation;
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
    let computed = call(&mut c.b, &scalar.result_definition, args)?;
    let value = c.b.var(0)?;
    let actual = c.b.var(1)?;
    let equality = storage_equal(&mut c.b, depth)?;
    let comparison = call(&mut c.b, &equality, vec![value, actual])?;
    let ty = c.b.cube(depth)?;
    let same = c.b.term(TermNode::Let {
        ty,
        value: computed,
        body: comparison,
    })?;
    define(&mut c.b, name, &inputs, 0, same)
}
pub(super) fn emit(
    c: &mut Clauses<'_>,
    d: &DataSemanticDefinition,
) -> R<OrdinaryStringDataDefinition> {
    if d.family != DataDefinitionFamily::String {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let vir = c.vir.ok_or(OrdinaryCarrierError::Linkage)?;
    let expected = strings::operation_signature(vir.data_closed(), &d.signature.id)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    if expected != d.signature {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let nullable = strings::operation_signature(vir.data_closed(), "string.length")
        .map_err(|_| OrdinaryCarrierError::Linkage)?
        .argument_type_ids[0]
        != STRING_TYPE_ID;
    let (scalar, independent_failure_definitions) =
        c.strings.emit_native(&mut c.b, expected, nullable)?;
    if scalar.operation != d.signature
        || independent_failure_definitions.len() != d.failure_names.len()
        || scalar.ordered_failure_definitions.len() != d.failure_names.len()
        || d.failure_result_names.iter().any(Option::is_some)
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let relation_definition = core_name("Relation", &d.id);
    relation(c, &scalar, &relation_definition)?;
    let mut args = d.signature.argument_type_ids.clone();
    args.push(d.signature.normal_result_type_id.clone());
    c.constants.insert(
        d.relation_name.clone(),
        (signature(&args, SOURCE_BOOL), relation_definition.clone()),
    );
    for ((check, symbol), body) in d
        .signature
        .ordered_checks
        .iter()
        .zip(&d.failure_names)
        .zip(&independent_failure_definitions)
    {
        // Resource/profile failures stay static obligations, never exceptions.
        if !matches!(
            check.tag,
            RequiredCheckTag::Exception | RequiredCheckTag::StaticObligation
        ) {
            return Err(OrdinaryCarrierError::Linkage);
        }
        c.constants.insert(
            symbol.clone(),
            (
                signature(&d.signature.argument_type_ids, SOURCE_BOOL),
                body.clone(),
            ),
        );
    }
    Ok(OrdinaryStringDataDefinition {
        source: d.clone(),
        scalar,
        relation_definition,
        independent_failure_definitions,
    })
}
pub fn generate_csharp_practical_ordinary_string_data(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryStringDataProgram> {
    let data = crate::csharp_practical_vir_model::data_vc::generate_data_vcs(vir)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut c = compiler(vir, &layouts, Builder::new()?, &[])?;
    c.definedness_logic()?;
    let mut definitions = vec![];
    let mut pending = vec![];
    for d in data.definitions() {
        if d.family != DataDefinitionFamily::String {
            pending.push(d.id.clone());
            continue;
        }
        definitions.push(emit(&mut c, d)?);
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
        let mut predicates = BTreeMap::new();
        let mut add = |role: String, term: &ContractTerm| -> R<()> {
            let name = core_name("Point", &(&o.id, &role));
            integer_data::predicate(&mut c, &name, &o.subjects, term)?;
            if predicates.insert(role, name).is_some() {
                return Err(OrdinaryCarrierError::Linkage);
            }
            Ok(())
        };
        add("success_guard".into(), &o.success_guard)?;
        add("success_relation".into(), &o.success_relation)?;
        add("success_goal".into(), &o.success_goal)?;
        for (i, check) in o.checks.iter().enumerate() {
            add(format!("check.{i}.prefix"), &check.prefix_guard)?;
            add(format!("check.{i}.failed"), &check.failure_predicate)?;
            add(format!("check.{i}.guard"), &check.failure_guard)?;
            if let Some(goal) = &check.static_goal {
                add(format!("check.{i}.static_goal"), goal)?;
            }
            if check.tagged_result_goal.is_some() {
                return Err(OrdinaryCarrierError::Linkage);
            }
        }
        operations.push(OrdinaryStringDataOperation {
            source: o.clone(),
            predicates,
        });
    }
    let certificate = c.b.finish()?;
    let p = OrdinaryStringDataProgram {
        schema: "mpk.csharp.ordinary_string_data.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        data_vc_sha256: data.hash(),
        definitions,
        operations,
        pending_definition_ids: pending,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_string_data(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryStringDataProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_string_data(vir)?;
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
    fn ordinary_string_data_storage_equality_observes_complete_carrier() {
        let mut b = Builder::new().unwrap();
        let large = storage_equal(&mut b, 19).unwrap();
        assert!(storage_equal(&mut b, 20).is_err());
        let cert = mpk_cert::decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        for depth in [0, 4, 5, 19] {
            let name = format!("{PREFIX}.StringData.StorageEqual.D{depth}");
            let ones = [0, (1usize << depth) - 1]
                .into_iter()
                .collect::<BTreeSet<_>>();
            let input = sparse_cube(depth, ones.clone());
            assert!(observed(run(
                &cert,
                &name,
                vec![input.clone(), input.clone()]
            )));
            let indexes = if depth == 19 {
                vec![0, 1, 2, 32768, 65536, 1 << 18, (1 << 19) - 1]
            } else {
                (0..1 << depth).collect()
            };
            for index in indexes {
                let mut changed = ones.clone();
                if !changed.remove(&index) {
                    changed.insert(index);
                }
                assert!(
                    !observed(run(
                        &cert,
                        &name,
                        vec![input.clone(), sparse_cube(depth, changed)]
                    )),
                    "D{depth} bit {index}"
                );
            }
        }
        assert_eq!(large, format!("{PREFIX}.StringData.StorageEqual.D19"));
    }
}
