//! W03 integer/Boolean operation relations at original SSA use points.
//! Definitions and guarded predicates only; no native-body or application proof.
use super::*;
use crate::csharp_practical_vir_model::data_vc::{
    DataDefinitionFamily, DataOperationVc, DataSemanticDefinition,
};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryIntegerDataDefinition {
    pub source: DataSemanticDefinition,
    pub scalar: OrdinaryScalarDefinition,
    pub relation_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryIntegerDataOperation {
    pub source: DataOperationVc,
    /// Original W03 formula role and corresponding closed ordinary definition.
    pub predicates: BTreeMap<String, String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryIntegerDataProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    data_vc_sha256: String,
    definitions: Vec<OrdinaryIntegerDataDefinition>,
    operations: Vec<OrdinaryIntegerDataOperation>,
    /// Other data families are explicitly outstanding, never silently discharged.
    pending_definition_ids: Vec<String>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryIntegerDataProgram {
    pub fn definitions(&self) -> &[OrdinaryIntegerDataDefinition] {
        &self.definitions
    }
    pub fn operations(&self) -> &[OrdinaryIntegerDataOperation] {
        &self.operations
    }
    pub fn pending_definition_ids(&self) -> &[String] {
        &self.pending_definition_ids
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary integer data relations")
    }
}
fn width(id: &str) -> R<u32> {
    Ok(
        match id
            .strip_prefix("mpk.csharp.value.")
            .and_then(|s| s.strip_suffix(".v1"))
        {
            Some("bool") => 1,
            Some("i8" | "u8") => 8,
            Some("i16" | "u16" | "char") => 16,
            Some("i32" | "u32") => 32,
            Some("i64" | "u64") => 64,
            _ => return Err(OrdinaryCarrierError::Shape),
        },
    )
}
fn core_name(kind: &str, identity: &impl Serialize) -> String {
    let bytes = serde_json::to_vec(identity).expect("typed integer data identity");
    format!("{PREFIX}.IntegerData.{kind}.H{:x}", Sha256::digest(bytes))
}
fn relation(b: &mut Builder, scalar: &OrdinaryScalarDefinition, name: &str) -> R<()> {
    let s = &scalar.operation;
    let count = s.argument_type_ids.len();
    let args = (0..count)
        .map(|i| b.var((count - i) as u32))
        .collect::<R<Vec<_>>>()?;
    let computed = call(b, &scalar.result_definition, args)?;
    let actual = b.var(0)?;
    let result_width = width(&s.normal_result_type_id)?;
    let depth = address_bits(result_width);
    let same = result_equality(b, computed, actual, result_width)?;
    let mut inputs = s
        .argument_type_ids
        .iter()
        .map(|t| width(t).map(address_bits))
        .collect::<R<Vec<_>>>()?;
    inputs.push(depth);
    define(b, name, &inputs, 0, same)
}
pub(super) fn result_equality(
    b: &mut Builder,
    computed: u32,
    actual: u32,
    result_width: u32,
) -> R<u32> {
    if !matches!(result_width, 1 | 8 | 16 | 32 | 64 | 512) {
        return Err(OrdinaryCarrierError::Shape);
    }
    let depth = address_bits(result_width);
    let mut same = bit(b, true)?;
    for index in 0..result_width {
        let selectors = (0..depth)
            .map(|i| bit(b, index & (1 << i) != 0))
            .collect::<R<Vec<_>>>()?;
        let left = b.app(computed, selectors.clone())?;
        let right = b.app(actual, selectors)?;
        let different = call(b, "Std.Bool.not", vec![right])?;
        let equal = mux(b, left, right, different)?;
        same = call(b, "Std.Bool.and", vec![same, equal])?;
    }
    Ok(same)
}
fn point_term(term: &ContractTerm, subjects: &[TypedValueRef]) -> R<ContractTerm> {
    // W03 operation sequents number free variables by subjects[i]. The
    // contract compiler instead consumes de Bruijn indices beneath the
    // ordered subject lambdas. Do not reverse the public argument order.
    Ok(match term {
        ContractTerm::Var { index, type_id } => {
            if subjects.get(*index).map(|s| &s.type_id) != Some(type_id) {
                return Err(OrdinaryCarrierError::Linkage);
            }
            ContractTerm::Var {
                index: subjects.len() - 1 - index,
                type_id: type_id.clone(),
            }
        }
        ContractTerm::Const { .. } => term.clone(),
        ContractTerm::App {
            function,
            argument,
            type_id,
        } => ContractTerm::App {
            function: Box::new(point_term(function, subjects)?),
            argument: Box::new(point_term(argument, subjects)?),
            type_id: type_id.clone(),
        },
        // W03 operation guards/relations have no local binders. A future
        // extension must specify its free/local index convention explicitly.
        _ => return Err(OrdinaryCarrierError::Shape),
    })
}
pub(super) fn predicate(
    c: &mut Clauses<'_>,
    name: &str,
    subjects: &[TypedValueRef],
    term: &ContractTerm,
) -> R<()> {
    if term.type_id() != SOURCE_BOOL {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let args = subjects
        .iter()
        .map(|s| s.type_id.clone())
        .collect::<Vec<_>>();
    let term = point_term(term, subjects)?;
    let mut body = c.lower(&term, &mut args.clone(), 0)?;
    for t in args.iter().rev() {
        let ty = c.ty(t, 0)?;
        body = c.b.lam(ty, body)?;
    }
    let ty = c.ty(&signature(&args, SOURCE_BOOL), 0)?;
    c.b.define(name, ty, body)
}
pub fn generate_csharp_practical_ordinary_integer_data(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryIntegerDataProgram> {
    let data = crate::csharp_practical_vir_model::data_vc::generate_data_vcs(vir)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut c = compiler(vir, &layouts, Builder::new()?, &[])?;
    c.definedness_logic()?;
    let mut definitions = vec![];
    let mut pending = vec![];
    for d in data.definitions() {
        if d.family != DataDefinitionFamily::IntegerBoolean {
            pending.push(d.id.clone());
            continue;
        }
        let scalar = super::super::super::scalar_bits::emit_integer(&mut c.b, &d.signature.id)?;
        if scalar.operation != d.signature
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
            .zip(&scalar.ordered_failure_definitions)
        {
            // Integer divide/remainder zero and min/-1 failures are disjoint;
            // their frozen ordered predicates equal individual failed checks.
            if check.tag != RequiredCheckTag::Exception {
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
        definitions.push(OrdinaryIntegerDataDefinition {
            source: d.clone(),
            scalar,
            relation_definition,
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
            predicate(&mut c, &name, &o.subjects, term)?;
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
        operations.push(OrdinaryIntegerDataOperation {
            source: o.clone(),
            predicates,
        });
    }
    let certificate = c.b.finish()?;
    let p = OrdinaryIntegerDataProgram {
        schema: "mpk.csharp.ordinary_integer_data.v1".into(),
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
pub fn import_csharp_practical_ordinary_integer_data(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryIntegerDataProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_integer_data(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::super::super::super::test_eval::{bit as observed, run, V};
    use super::*;
    #[test]
    fn ordinary_integer_data_relation_observes_every_result_bit() {
        let mut b = Builder::new().unwrap();
        let mut cases = vec![];
        for token in ["i32", "u32", "i64", "u64"] {
            let id = format!("integer.{token}.add.unchecked");
            let scalar =
                super::super::super::super::scalar_bits::emit_integer(&mut b, &id).unwrap();
            let name = format!("Test.IntegerData.{token}");
            relation(&mut b, &scalar, &name).unwrap();
            cases.push((
                name,
                width(&scalar.operation.normal_result_type_id).unwrap(),
                false,
            ));
        }
        for token in ["i8", "u8", "i16", "u16", "char"] {
            let id = format!("integer.convert.i64.{token}.unchecked");
            let scalar =
                super::super::super::super::scalar_bits::emit_integer(&mut b, &id).unwrap();
            let name = format!("Test.IntegerData.Convert.{token}");
            relation(&mut b, &scalar, &name).unwrap();
            cases.push((
                name,
                width(&scalar.operation.normal_result_type_id).unwrap(),
                true,
            ));
        }
        let scalar =
            super::super::super::super::scalar_bits::emit_integer(&mut b, "boolean.xor").unwrap();
        relation(&mut b, &scalar, "Test.IntegerData.Bool").unwrap();
        let cert = mpk_cert::decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        let word = |n: u64, bits: u32| V::Cube((0..bits).map(|i| n & (1 << i) != 0).collect());
        let mut observations = 0;
        for (name, bits, unary) in cases {
            let mask = u64::MAX >> (64 - bits);
            for (left, right) in [(0, 0), (mask, 1), (mask / 2, 1), (mask, mask)] {
                let expected = if unary {
                    left & mask
                } else {
                    left.wrapping_add(right) & mask
                };
                for changed in
                    std::iter::once(expected).chain((0..bits).map(|i| expected ^ (1 << i)))
                {
                    let args = if unary {
                        vec![word(left, 64), word(changed, bits)]
                    } else {
                        vec![word(left, bits), word(right, bits), word(changed, bits)]
                    };
                    assert_eq!(
                        observed(run(&cert, &name, args)),
                        changed == expected,
                        "{name}: {left}+{right}, result {changed}"
                    );
                    observations += 1;
                }
            }
        }
        for left in [false, true] {
            for right in [false, true] {
                for result in [false, true] {
                    assert_eq!(
                        observed(run(
                            &cert,
                            "Test.IntegerData.Bool",
                            vec![V::Bit(left), V::Bit(right), V::Bit(result)]
                        )),
                        result == (left ^ right)
                    );
                    observations += 1;
                }
            }
        }
        assert_eq!(observations, 1068);
    }
}
