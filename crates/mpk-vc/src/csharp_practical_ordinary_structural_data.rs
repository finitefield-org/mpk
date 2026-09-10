//! W03 structural equality and canonical-comparison SSA result relations.
//! Input domains and complete native-body proofs remain independent obligations.
use super::*;
use crate::csharp_practical_vir_model::data_vc::{
    DataDefinitionFamily, DataOperationVc, DataSemanticDefinition,
};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryStructuralDataDefinition {
    pub source: DataSemanticDefinition,
    pub value_definition: String,
    pub relation_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryStructuralDataOperation {
    pub source: DataOperationVc,
    pub predicates: BTreeMap<String, String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryStructuralDataProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    data_vc_sha256: String,
    definitions: Vec<OrdinaryStructuralDataDefinition>,
    operations: Vec<OrdinaryStructuralDataOperation>,
    pending_definition_ids: Vec<String>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryStructuralDataProgram {
    pub fn definitions(&self) -> &[OrdinaryStructuralDataDefinition] {
        &self.definitions
    }
    pub fn operations(&self) -> &[OrdinaryStructuralDataOperation] {
        &self.operations
    }
    pub fn pending_definition_ids(&self) -> &[String] {
        &self.pending_definition_ids
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("structural data relations")
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
fn name(kind: &str, identity: &impl Serialize) -> String {
    format!(
        "{PREFIX}.StructuralData.{kind}.H{:x}",
        Sha256::digest(serde_json::to_vec(identity).expect("typed structural data identity"))
    )
}
fn emit(c: &mut Clauses<'_>, d: &DataSemanticDefinition) -> R<OrdinaryStructuralDataDefinition> {
    let s = &d.signature;
    let compare = match s.tag {
        ClosedOperationTag::StructuralEqual => false,
        ClosedOperationTag::CanonicalCompare => true,
        _ => return Err(OrdinaryCarrierError::Linkage),
    };
    let result_type = if compare {
        "mpk.csharp.value.i32.v1"
    } else {
        SOURCE_BOOL
    };
    if d.family != DataDefinitionFamily::Structural
        || s.argument_type_ids.len() != 2
        || s.argument_type_ids[0] != s.argument_type_ids[1]
        || s.normal_result_type_id != result_type
        || !s.ordered_checks.is_empty()
        || !d.failure_names.is_empty()
        || !d.failure_result_names.is_empty()
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let input_depth = c
        .carriers
        .get(s.argument_type_ids[0].as_str())
        .ok_or(OrdinaryCarrierError::Linkage)?
        .depth;
    let function = c.relation_function(
        &s.argument_type_ids[0],
        if compare {
            relations::ContractRelationOperation::Compare
        } else {
            relations::ContractRelationOperation::Equal
        },
    )?;
    let TermNode::Const { global, .. } = c.b.c.term_table[function as usize] else {
        return Err(OrdinaryCarrierError::Shape);
    };
    let value_definition =
        c.b.c.name_table[c.b.c.declarations[global as usize].name as usize].clone();
    let left = c.b.var(2)?;
    let right = c.b.var(1)?;
    let actual = c.b.var(0)?;
    let computed = c.b.app(function, vec![left, right])?;
    let same =
        integer_data::result_equality(&mut c.b, computed, actual, if compare { 32 } else { 1 })?;
    let relation_definition = name("Relation", &d.id);
    define(
        &mut c.b,
        &relation_definition,
        &[input_depth, input_depth, if compare { 5 } else { 0 }],
        0,
        same,
    )?;
    let mut args = s.argument_type_ids.clone();
    args.push(result_type.into());
    c.constants.insert(
        d.relation_name.clone(),
        (signature(&args, SOURCE_BOOL), relation_definition.clone()),
    );
    Ok(OrdinaryStructuralDataDefinition {
        source: d.clone(),
        value_definition,
        relation_definition,
    })
}
pub fn generate_csharp_practical_ordinary_structural_data(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryStructuralDataProgram> {
    let data = crate::csharp_practical_vir_model::data_vc::generate_data_vcs(vir)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut c = compiler(vir, &layouts, Builder::new()?, &[])?;
    c.definedness_logic()?;
    let mut definitions = vec![];
    let mut pending = vec![];
    for d in data.definitions() {
        if d.family == DataDefinitionFamily::Structural {
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
        operations.push(OrdinaryStructuralDataOperation {
            source: o.clone(),
            predicates,
        });
    }
    let certificate = c.b.finish()?;
    let program = OrdinaryStructuralDataProgram {
        schema: "mpk.csharp.ordinary_structural_data.v1".into(),
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
pub fn import_csharp_practical_ordinary_structural_data(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryStructuralDataProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_structural_data(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
