//! W03 calendar/time/Guid operation relations at original SSA use points.
//! Definitions and guarded predicates only; no native-body or application proof.
use super::*;
use crate::csharp_practical_vir_model::data_vc::{
    DataDefinitionFamily, DataOperationVc, DataSemanticDefinition,
};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryCalendarDataDefinition {
    pub source: DataSemanticDefinition,
    pub scalar: OrdinaryScalarDefinition,
    pub relation_definition: String,
    pub independent_failure_definitions: Vec<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryCalendarDataOperation {
    pub source: DataOperationVc,
    /// Original W03 formula role and corresponding closed ordinary definition.
    pub predicates: BTreeMap<String, String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryCalendarDataProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    data_vc_sha256: String,
    definitions: Vec<OrdinaryCalendarDataDefinition>,
    operations: Vec<OrdinaryCalendarDataOperation>,
    /// Other data families are explicitly outstanding, never silently discharged.
    pending_definition_ids: Vec<String>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryCalendarDataProgram {
    pub fn definitions(&self) -> &[OrdinaryCalendarDataDefinition] {
        &self.definitions
    }
    pub fn operations(&self) -> &[OrdinaryCalendarDataOperation] {
        &self.operations
    }
    pub fn pending_definition_ids(&self) -> &[String] {
        &self.pending_definition_ids
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary calendar data relations")
    }
}
fn width(id: &str) -> R<u32> {
    Ok(
        match id
            .strip_prefix("mpk.csharp.value.")
            .and_then(|s| s.strip_suffix(".v1"))
        {
            Some("bool") => 1,
            Some("guid") => 128,
            Some("i8" | "u8") => 8,
            Some("i16" | "u16" | "char") => 16,
            Some("i32" | "u32" | "date" | "day_of_week") => 32,
            Some("i64" | "u64" | "time" | "duration" | "instant") => 64,
            _ => return Err(OrdinaryCarrierError::Shape),
        },
    )
}
fn core_name(kind: &str, identity: &impl Serialize) -> String {
    let bytes = serde_json::to_vec(identity).expect("typed calendar data identity");
    format!("{PREFIX}.CalendarData.{kind}.H{:x}", Sha256::digest(bytes))
}
fn relation(b: &mut Builder, scalar: &OrdinaryScalarDefinition, name: &str) -> R<()> {
    let s = &scalar.operation;
    let count = s.argument_type_ids.len();
    let args = (0..count)
        .map(|i| b.var((count - i) as u32))
        .collect::<R<Vec<_>>>()?;
    let computed = call(b, &scalar.result_definition, args)?;
    let result_width = width(&s.normal_result_type_id)?;
    let depth = address_bits(result_width);
    // A repeated term-table ID is not a runtime let binding. Share the
    // computed product across all physical-bit comparisons, retaining lazy
    // demand and the original scalar body. Beneath this let, actual is Var(1).
    let value = b.var(0)?;
    let actual = b.var(1)?;
    let comparison = integer_data::result_equality(b, value, actual, result_width)?;
    let ty = b.cube(depth)?;
    let same = b.term(TermNode::Let {
        ty,
        value: computed,
        body: comparison,
    })?;
    let mut inputs = s
        .argument_type_ids
        .iter()
        .map(|t| width(t).map(address_bits))
        .collect::<R<Vec<_>>>()?;
    inputs.push(depth);
    define(b, name, &inputs, 0, same)
}
pub fn generate_csharp_practical_ordinary_calendar_data(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryCalendarDataProgram> {
    let data = crate::csharp_practical_vir_model::data_vc::generate_data_vcs(vir)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut c = compiler(vir, &layouts, Builder::new()?, &[])?;
    c.definedness_logic()?;
    let mut definitions = vec![];
    let mut pending = vec![];
    for d in data.definitions() {
        if d.family != DataDefinitionFamily::CalendarTimeGuidMoney
            || d.signature.id.starts_with("money.")
            || d.signature
                .ordered_checks
                .iter()
                .any(|check| check.tag != RequiredCheckTag::Exception)
        {
            pending.push(d.id.clone());
            continue;
        }
        let scalar = if ["date.", "guid.", "day_of_week."]
            .iter()
            .any(|p| d.signature.id.starts_with(p))
        {
            super::super::super::scalar_bits::emit_calendar(&mut c.b, &d.signature.id)?
        } else {
            super::super::super::scalar_bits::emit_temporal(&mut c.b, &d.signature.id)?
        };
        // These signatures have at most one exception. With no earlier
        // condition to mask it, the ordered predicate is also the raw one.
        if scalar.operation.ordered_checks.len() > 1 {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let independent_failure_definitions = scalar.ordered_failure_definitions.clone();
        if scalar.operation != d.signature
            || independent_failure_definitions.len() != d.failure_names.len()
            || scalar.ordered_failure_definitions.len() != d.failure_names.len()
            || d.failure_result_names.iter().any(Option::is_some)
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let relation_definition = core_name("Relation", &d.id);
        relation(&mut c.b, &scalar, &relation_definition)?;
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
            // W03 retains the original exception condition and source guard.
            if check.tag != RequiredCheckTag::Exception
                || !matches!(check.id.as_str(), "exception.overflow" | "exception.range")
            {
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
        definitions.push(OrdinaryCalendarDataDefinition {
            source: d.clone(),
            scalar,
            relation_definition,
            independent_failure_definitions,
        });
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
        operations.push(OrdinaryCalendarDataOperation {
            source: o.clone(),
            predicates,
        });
    }
    let certificate = c.b.finish()?;
    let p = OrdinaryCalendarDataProgram {
        schema: "mpk.csharp.ordinary_calendar_data.v1".into(),
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
pub fn import_csharp_practical_ordinary_calendar_data(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryCalendarDataProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_calendar_data(vir)?;
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
    fn ordinary_calendar_data_result_observes_all_storage_bits() {
        let mut b = Builder::new().unwrap();
        let computed = b.var(1).unwrap();
        let actual = b.var(0).unwrap();
        let body = integer_data::result_equality(&mut b, computed, actual, 128).unwrap();
        define(&mut b, "Test.CalendarData.Result", &[7, 7], 0, body).unwrap();
        assert!(integer_data::result_equality(&mut b, computed, actual, 3).is_err());
        let certificate = mpk_cert::decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        let ones = [0, 1, 2, 63, 64, 127].into_iter().collect::<BTreeSet<_>>();
        let input = sparse_cube(7, ones.clone());
        assert!(observed(run(
            &certificate,
            "Test.CalendarData.Result",
            vec![input.clone(), input.clone()]
        )));
        for index in 0..128 {
            let mut wrong = ones.clone();
            if !wrong.remove(&index) {
                wrong.insert(index);
            }
            assert!(
                !observed(run(
                    &certificate,
                    "Test.CalendarData.Result",
                    vec![input.clone(), sparse_cube(7, wrong)]
                )),
                "bit {index}"
            );
        }
    }
}
