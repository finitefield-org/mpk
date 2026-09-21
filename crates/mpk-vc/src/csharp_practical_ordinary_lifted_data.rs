//! W03 nullable lifted operation relations at original SSA use points.
//! Definitions and guarded predicates only; no native-body or application proof.
use super::*;
use crate::csharp_practical_vir_model::data_vc::{
    DataDefinitionFamily, DataOperationVc, DataSemanticDefinition,
};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryLiftedDataDefinition {
    pub source: DataSemanticDefinition,
    pub scalar: OrdinaryScalarDefinition,
    pub value_definition: String,
    pub relation_definition: String,
    pub independent_failure_definitions: Vec<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryLiftedDataOperation {
    pub source: DataOperationVc,
    /// Original W03 formula role and corresponding closed ordinary definition.
    pub predicates: BTreeMap<String, String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryLiftedDataProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    data_vc_sha256: String,
    definitions: Vec<OrdinaryLiftedDataDefinition>,
    operations: Vec<OrdinaryLiftedDataOperation>,
    /// Other data families are explicitly outstanding, never silently discharged.
    pending_definition_ids: Vec<String>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryLiftedDataProgram {
    pub fn definitions(&self) -> &[OrdinaryLiftedDataDefinition] {
        &self.definitions
    }
    pub fn operations(&self) -> &[OrdinaryLiftedDataOperation] {
        &self.operations
    }
    pub fn pending_definition_ids(&self) -> &[String] {
        &self.pending_definition_ids
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary lifted data relations")
    }
}
fn core_name(kind: &str, identity: &impl Serialize) -> String {
    let bytes = serde_json::to_vec(identity).expect("typed lifted data identity");
    format!("{PREFIX}.LiftedData.{kind}.H{:x}", Sha256::digest(bytes))
}
pub(super) fn emit(
    c: &mut Clauses<'_>,
    d: &DataSemanticDefinition,
) -> R<OrdinaryLiftedDataDefinition> {
    let vir = c.vir.ok_or(OrdinaryCarrierError::Linkage)?;
    let s = &d.signature;
    let expected = crate::csharp_practical_vir_model::domain::lifted_operation_signature(
        vir.construction_context().1,
        vir.data_closed(),
        &s.id,
        &s.argument_type_ids,
        &s.normal_result_type_id,
    )
    .map_err(|_| OrdinaryCarrierError::Linkage)?;
    if expected != *s
        || d.failure_names.len() != s.ordered_checks.len()
        || d.failure_result_names.iter().any(Option::is_some)
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let parts = s.id.split('.').collect::<Vec<_>>();
    let (token, operation, mode) = (parts[1], parts[2], parts[3]);
    let scalar_id = match token {
        "bool" => format!("boolean.{operation}"),
        "i32" | "i64" => format!("integer.{token}.{operation}.{mode}"),
        "f32" => format!("floating.single.{operation}"),
        "f64" => format!("floating.double.{operation}"),
        "decimal" => format!("decimal.{operation}"),
        _ => return Err(OrdinaryCarrierError::Shape),
    };
    if !c.scalars.contains_key(&scalar_id) {
        use super::super::super::scalar_bits;
        let value = if token == "decimal" {
            scalar_bits::emit_decimal_contract(&mut c.b, &scalar_id)?
        } else {
            let scalar = if matches!(token, "f32" | "f64") {
                scalar_bits::emit_floating(&mut c.b, &scalar_id)?
            } else {
                scalar_bits::emit_integer(&mut c.b, &scalar_id)?
            };
            let failures = scalar.ordered_failure_definitions.clone();
            (scalar, failures)
        };
        c.scalars.insert(scalar_id.clone(), value);
    }
    let (scalar, raw) = c.scalars[&scalar_id].clone();
    let comparison = s.normal_result_type_id == SOURCE_BOOL;
    let primitive = format!("mpk.csharp.value.{token}.v1");
    if scalar.operation.argument_type_ids != vec![primitive.clone(); s.argument_type_ids.len()]
        || scalar.operation.normal_result_type_id
            != if comparison { SOURCE_BOOL } else { &primitive }
        || scalar.operation.ordered_checks != s.ordered_checks
        || raw.len() != d.failure_names.len()
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let carrier = c
        .carriers
        .get(s.argument_type_ids[0].as_str())
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let structure = c
        .storage
        .get(&mut c.b, carrier, &c.carriers)?
        .ok_or(OrdinaryCarrierError::Shape)?;
    let OrdinaryStructuralOperations::Sum { arms, .. } = structure.operations else {
        return Err(OrdinaryCarrierError::Shape);
    };
    let some = arms
        .iter()
        .find(|a| a.arm_id == "some" && a.tag == 1 && a.fields.len() == 1)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let none = arms
        .iter()
        .find(|a| a.arm_id == "none" && a.tag == 0 && a.fields.is_empty())
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let input_depths = vec![carrier.depth; s.argument_type_ids.len()];
    let result_depth = c
        .carriers
        .get(s.normal_result_type_id.as_str())
        .ok_or(OrdinaryCarrierError::Linkage)?
        .depth;
    let mut present = vec![];
    let mut values = vec![];
    for i in 0..input_depths.len() {
        let arg = c.b.var((input_depths.len() - 1 - i) as u32)?;
        present.push(call(&mut c.b, &some.is_active_definition, vec![arg])?);
        values.push(call(&mut c.b, &some.fields[0].definition, vec![arg])?);
    }
    let mut all = bit(&mut c.b, true)?;
    for &p in &present {
        all = call(&mut c.b, "Std.Bool.and", vec![all, p])?;
    }
    let computed = call(&mut c.b, &scalar.result_definition, values.clone())?;
    let result = if comparison {
        let absent_value = if matches!(operation, "equal" | "not_equal") {
            let any = call(&mut c.b, "Std.Bool.or", present.clone())?;
            if operation == "equal" {
                call(&mut c.b, "Std.Bool.not", vec![any])?
            } else {
                any
            }
        } else {
            bit(&mut c.b, false)?
        };
        mux(&mut c.b, all, computed, absent_value)?
    } else {
        let is_some = if token == "bool" && matches!(operation, "and" | "or") {
            // Nullable bool has three-valued AND/OR: false & null is false,
            // true | null is true, even when the other operand is absent.
            let mut determined = bit(&mut c.b, false)?;
            for (&p, &v) in present.iter().zip(&values) {
                let decisive = if operation == "and" {
                    call(&mut c.b, "Std.Bool.not", vec![v])?
                } else {
                    v
                };
                let decisive = call(&mut c.b, "Std.Bool.and", vec![p, decisive])?;
                determined = call(&mut c.b, "Std.Bool.or", vec![determined, decisive])?;
            }
            call(&mut c.b, "Std.Bool.or", vec![all, determined])?
        } else {
            all
        };
        let some_value = call(&mut c.b, &some.make_definition, vec![computed])?;
        let none_value = call(&mut c.b, &none.make_definition, vec![])?;
        let mux_name = format!("{PREFIX}.Cube.D{result_depth}.Mux");
        if !c.b.globals.contains_key(&mux_name) {
            c.b.helpers(result_depth)?;
        }
        call(&mut c.b, &mux_name, vec![is_some, some_value, none_value])?
    };
    let value_definition = core_name("Value", &d.id);
    define(
        &mut c.b,
        &value_definition,
        &input_depths,
        result_depth,
        result,
    )?;
    let mut independent_failure_definitions = vec![];
    for (i, failure) in raw.iter().enumerate() {
        // Null propagation precedes scalar demand. Absent operands must not
        // trigger divide-by-zero/overflow from inactive zero payloads.
        let failure = call(&mut c.b, failure, values.clone())?;
        let failure = call(&mut c.b, "Std.Bool.and", vec![all, failure])?;
        let name = core_name("Failure", &(&d.id, i));
        define(&mut c.b, &name, &input_depths, 0, failure)?;
        independent_failure_definitions.push(name);
    }
    let relation_definition = core_name("Relation", &d.id);
    let count = input_depths.len();
    let args = (0..count)
        .map(|i| c.b.var((count - i) as u32))
        .collect::<R<Vec<_>>>()?;
    let computed = call(&mut c.b, &value_definition, args)?;
    let value = c.b.var(0)?;
    let actual = c.b.var(1)?;
    let equal = string_data::storage_equal(&mut c.b, result_depth)?;
    let comparison = call(&mut c.b, &equal, vec![value, actual])?;
    let ty = c.b.cube(result_depth)?;
    let same = c.b.term(TermNode::Let {
        ty,
        value: computed,
        body: comparison,
    })?;
    let mut inputs = input_depths;
    inputs.push(result_depth);
    define(&mut c.b, &relation_definition, &inputs, 0, same)?;
    let mut args = s.argument_type_ids.clone();
    args.push(s.normal_result_type_id.clone());
    c.constants.insert(
        d.relation_name.clone(),
        (signature(&args, SOURCE_BOOL), relation_definition.clone()),
    );
    for (symbol, body) in d.failure_names.iter().zip(&independent_failure_definitions) {
        c.constants.insert(
            symbol.clone(),
            (signature(&s.argument_type_ids, SOURCE_BOOL), body.clone()),
        );
    }
    Ok(OrdinaryLiftedDataDefinition {
        source: d.clone(),
        scalar,
        value_definition,
        relation_definition,
        independent_failure_definitions,
    })
}
pub fn generate_csharp_practical_ordinary_lifted_data(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryLiftedDataProgram> {
    let data = crate::csharp_practical_vir_model::data_vc::generate_data_vcs(vir)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut c = compiler(vir, &layouts, Builder::new()?, &[])?;
    c.definedness_logic()?;
    let mut definitions = vec![];
    let mut pending = vec![];
    for d in data.definitions() {
        if d.family == DataDefinitionFamily::NullableOutcome
            && d.signature.id.starts_with("lifted.")
        {
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
        operations.push(OrdinaryLiftedDataOperation {
            source: o.clone(),
            predicates,
        });
    }
    let certificate = c.b.finish()?;
    let p = OrdinaryLiftedDataProgram {
        schema: "mpk.csharp.ordinary_lifted_data.v1".into(),
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
pub fn import_csharp_practical_ordinary_lifted_data(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryLiftedDataProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_lifted_data(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
