//! W03 nullable-reference extraction with its exact NullReferenceException edge.
//! Definitions and guarded predicates only; no native-body or application proof.
use super::*;
use crate::csharp_practical_vir_model::data_vc::{
    DataDefinitionFamily, DataOperationVc, DataSemanticDefinition,
};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryReferenceDataDefinition {
    pub source: DataSemanticDefinition,
    pub operation: OrdinaryOutcomeOperation,
    pub failure_definitions: Vec<String>,
    pub relation_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryReferenceDataOperation {
    pub source: DataOperationVc,
    /// Original W03 formula role and corresponding closed ordinary definition.
    pub predicates: BTreeMap<String, String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryReferenceDataProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    data_vc_sha256: String,
    definitions: Vec<OrdinaryReferenceDataDefinition>,
    operations: Vec<OrdinaryReferenceDataOperation>,
    /// Other data families are explicitly outstanding, never silently discharged.
    pending_definition_ids: Vec<String>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryReferenceDataProgram {
    pub fn definitions(&self) -> &[OrdinaryReferenceDataDefinition] {
        &self.definitions
    }
    pub fn operations(&self) -> &[OrdinaryReferenceDataOperation] {
        &self.operations
    }
    pub fn pending_definition_ids(&self) -> &[String] {
        &self.pending_definition_ids
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary reference data relations")
    }
}
fn name(kind: &str, identity: &impl Serialize) -> String {
    format!(
        "{PREFIX}.ReferenceData.{kind}.H{:x}",
        Sha256::digest(serde_json::to_vec(identity).expect("typed reference data identity"))
    )
}
fn emit(
    c: &mut Clauses<'_>,
    d: &DataSemanticDefinition,
    operation: &OrdinaryOutcomeOperation,
) -> R<OrdinaryReferenceDataDefinition> {
    let vir = c.vir.ok_or(OrdinaryCarrierError::Linkage)?;
    let s = &d.signature;
    validate_closed_operation_signature(vir.construction_context().1, vir.data_closed(), s)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    if d.family != DataDefinitionFamily::NullableOutcome
        || s.tag != ClosedOperationTag::Data
        || s.argument_type_ids.len() != 1
        || s.id != format!("reference.value.{}", s.argument_type_ids[0])
        || operation.operation_id != format!("{}.value", s.argument_type_ids[0])
        || operation.failures.len() != 1
        || s.argument_type_ids != operation.argument_type_ids
        || s.normal_result_type_id != operation.result_type_id
        || s.ordered_checks.len() != operation.failures.len()
        || d.failure_names.len() != operation.failures.len()
        || d.failure_result_names.len() != operation.failures.len()
        || d.failure_result_names.iter().any(Option::is_some)
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
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
    let count = s.argument_type_ids.len();
    let args = (0..count)
        .map(|i| c.b.var((count - i) as u32))
        .collect::<R<Vec<_>>>()?;
    let computed = call(&mut c.b, &operation.normal_definition, args)?;
    let depth = inputs[count];
    let equal = source_value_data::storage_equal(&mut c.b, depth)?;
    let result = c.b.var(0)?;
    let actual = c.b.var(1)?;
    let body = call(&mut c.b, &equal, vec![result, actual])?;
    let ty = c.b.cube(depth)?;
    let body = c.b.term(TermNode::Let {
        ty,
        value: computed,
        body,
    })?;
    let relation_definition = name("Relation", &d.id);
    define(&mut c.b, &relation_definition, &inputs, 0, body)?;
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
    let mut failure_definitions = vec![];
    for (i, (check, failure)) in s.ordered_checks.iter().zip(&operation.failures).enumerate() {
        if failure.label != "invalid_operation"
            || check.tag != RequiredCheckTag::Exception
            || check.id != "exception.null_receiver"
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let args = failure
            .argument_indices
            .iter()
            .map(|&index| {
                if index >= count {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                c.b.var((count - 1 - index) as u32)
            })
            .collect::<R<Vec<_>>>()?;
        let body = call(&mut c.b, &failure.definition, args)?;
        let core = name("Failure", &(&d.id, i));
        define(&mut c.b, &core, &inputs[..count], 0, body)?;
        if c.constants
            .insert(
                d.failure_names[i].clone(),
                (signature(&s.argument_type_ids, SOURCE_BOOL), core.clone()),
            )
            .is_some()
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        failure_definitions.push(core);
    }
    Ok(OrdinaryReferenceDataDefinition {
        source: d.clone(),
        operation: operation.clone(),
        relation_definition,
        failure_definitions,
    })
}

pub fn generate_csharp_practical_ordinary_reference_data(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryReferenceDataProgram> {
    let data = crate::csharp_practical_vir_model::data_vc::generate_data_vcs(vir)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let (builder, foundations) =
        relations::emit_option_definitions(vir, &layouts, Builder::new()?)?;
    let available = foundations
        .iter()
        .flat_map(|d| &d.operations)
        .filter(|o| o.operation_id.ends_with(".value"))
        .map(|o| (o.argument_type_ids[0].as_str(), o))
        .collect::<BTreeMap<_, _>>();
    let mut c = compiler(vir, &layouts, builder, &[])?;
    c.definedness_logic()?;
    let mut definitions = vec![];
    let mut pending = vec![];
    for d in data.definitions() {
        let Some(operation) = available
            .get(
                d.signature
                    .id
                    .strip_prefix("reference.value.")
                    .unwrap_or(""),
            )
            .filter(|_| d.family == DataDefinitionFamily::NullableOutcome)
        else {
            pending.push(d.id.clone());
            continue;
        };
        definitions.push(emit(&mut c, d, operation)?);
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
            let name = name("Point", &(&o.id, &role));
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
        operations.push(OrdinaryReferenceDataOperation {
            source: o.clone(),
            predicates,
        });
    }
    let certificate = c.b.finish()?;
    let p = OrdinaryReferenceDataProgram {
        schema: "mpk.csharp.ordinary_reference_data.v1".into(),
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
pub fn import_csharp_practical_ordinary_reference_data(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryReferenceDataProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_reference_data(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
