//! Successful native transitions at exact invocation and normal-result points.
//! These open relations do not prove reachability or whole-body correctness.
use super::*;
use crate::csharp_practical_vir_model::data_vc::DataOperationVc;
use crate::csharp_practical_vir_model::ordinary_carriers::scalar_bits::bits_relation;
use crate::csharp_practical_vir_validation::PracticalControlAnchor;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlNativeDefinition {
    pub source: DataSemanticDefinition,
    pub relation_definition: String,
    /// None requires a source-scoped definition, such as symbolic ownership.
    pub failure_definitions: Vec<Option<String>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlNativeOperation {
    pub source: DataOperationVc,
    pub invocation: OperationInvocation,
    pub source_anchors: Vec<PracticalControlAnchor>,
    /// Operands at invocation; result at its exact normal successor.
    pub arguments: Vec<ControlBinding>,
    pub ownership: Option<OrdinaryConstructionOwnershipUse>,
    /// normal_execution is success_guard AND success_relation, never implication.
    pub predicates: BTreeMap<String, String>,
    pub pending_constant_names: Vec<String>,
}

fn entry(
    source: &DataSemanticDefinition,
    relation: &str,
    failures: impl Iterator<Item = Option<String>>,
) -> OrdinaryControlNativeDefinition {
    OrdinaryControlNativeDefinition {
        source: source.clone(),
        relation_definition: relation.into(),
        failure_definitions: failures.collect(),
    }
}

fn definitions(
    p: &OrdinaryControlEdgeProgram,
) -> BTreeMap<String, OrdinaryControlNativeDefinition> {
    let mut result = BTreeMap::new();
    for d in &p.integer_definitions {
        result.insert(
            d.source.id.clone(),
            entry(
                &d.source,
                &d.relation_definition,
                d.scalar
                    .ordered_failure_definitions
                    .iter()
                    .cloned()
                    .map(Some),
            ),
        );
    }
    for d in &p.string_definitions {
        result.insert(
            d.source.id.clone(),
            entry(
                &d.source,
                &d.relation_definition,
                d.independent_failure_definitions.iter().cloned().map(Some),
            ),
        );
    }
    for d in &p.construction_definitions {
        result.insert(
            d.source.id.clone(),
            entry(
                &d.source,
                &d.relation_definition,
                d.failure_definitions.iter().cloned(),
            ),
        );
    }
    for d in &p.sequence_definitions {
        result.insert(
            d.source.id.clone(),
            entry(
                &d.source,
                &d.relation_definition,
                d.failure_definitions.iter().cloned().map(Some),
            ),
        );
    }
    for d in &p.reference_definitions {
        result.insert(
            d.source.id.clone(),
            entry(
                &d.source,
                &d.relation_definition,
                d.failure_definitions.iter().cloned().map(Some),
            ),
        );
    }
    for d in &p.option_definitions {
        result.insert(
            d.source.id.clone(),
            entry(
                &d.source,
                &d.relation_definition,
                d.failure_definitions.iter().cloned().map(Some),
            ),
        );
    }
    result
}

fn emit_missing(
    c: &mut Clauses<'_>,
    p: &OrdinaryControlEdgeProgram,
    source: &DataSemanticDefinition,
) -> R<Option<OrdinaryControlNativeDefinition>> {
    let d = match source.family {
        DataDefinitionFamily::Structural => {
            let s = &source.signature;
            if s.argument_type_ids.len() != 2
                || s.argument_type_ids[0] != s.argument_type_ids[1]
                || !s.ordered_checks.is_empty()
                || !source.failure_names.is_empty()
            {
                return Err(OrdinaryCarrierError::Linkage);
            }
            let vir = c.vir.ok_or(OrdinaryCarrierError::Linkage)?;
            let nominal = &s.argument_type_ids[0];
            let token = nominal
                .strip_prefix("mpk.csharp.value.")
                .and_then(|t| t.strip_suffix(".v1"))
                .or_else(|| {
                    vir.construction_context()
                        .1
                        .source_types
                        .get(nominal)
                        .filter(|s| s.kind == SourceKind::Enum)
                        .and_then(|s| s.enum_underlying.as_deref())
                });
            let Some(
                token @ ("bool" | "char" | "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64"
                | "u64"),
            ) = token
            else {
                return Ok(None);
            };
            let width = scalar_width(token).ok_or(OrdinaryCarrierError::Shape)?;
            let compare = match s.tag {
                ClosedOperationTag::StructuralEqual => false,
                ClosedOperationTag::CanonicalCompare => true,
                _ => return Err(OrdinaryCarrierError::Linkage),
            };
            if s.normal_result_type_id
                != if compare {
                    "mpk.csharp.value.i32.v1"
                } else {
                    SOURCE_BOOL
                }
            {
                return Err(OrdinaryCarrierError::Linkage);
            }
            // A distinct key avoids re-emitting a cached foundation relation in
            // the predecessor builder. Enum comparison follows its underlying
            // signed integer, while scalar equality compares complete bits.
            let raw = bits_relation(
                &mut c.b,
                &format!("native.{}", s.id),
                width,
                token.starts_with('i'),
            )?;
            let left = c.b.var(2)?;
            let right = c.b.var(1)?;
            let actual = c.b.var(0)?;
            let computed = call(
                &mut c.b,
                if compare {
                    raw.compare.as_ref().ok_or(OrdinaryCarrierError::Shape)?
                } else {
                    &raw.equal
                },
                vec![left, right],
            )?;
            let same = integer_data::result_equality(
                &mut c.b,
                computed,
                actual,
                if compare { 32 } else { 1 },
            )?;
            let relation = name("NativeStructuralRelation", &source.id);
            define(
                &mut c.b,
                &relation,
                &[
                    address_bits(width),
                    address_bits(width),
                    if compare { 5 } else { 0 },
                ],
                0,
                same,
            )?;
            entry(source, &relation, std::iter::empty())
        }
        DataDefinitionFamily::IntegerBoolean => {
            let d = integer_data::emit_definition(c, source)?;
            entry(
                source,
                &d.relation_definition,
                d.scalar.ordered_failure_definitions.into_iter().map(Some),
            )
        }
        DataDefinitionFamily::String => {
            let d = string_data::emit(c, source)?;
            entry(
                source,
                &d.relation_definition,
                d.independent_failure_definitions.into_iter().map(Some),
            )
        }
        DataDefinitionFamily::SourceValue => {
            let d = source_value_data::emit(c, source)?;
            entry(source, &d.relation_definition, std::iter::empty())
        }
        DataDefinitionFamily::NullableOutcome if source.signature.id.starts_with("lifted.") => {
            let d = lifted_data::emit(c, source)?;
            entry(
                source,
                &d.relation_definition,
                d.independent_failure_definitions.into_iter().map(Some),
            )
        }
        DataDefinitionFamily::Foundation | DataDefinitionFamily::SequenceOwnership => {
            let id = source
                .signature
                .id
                .strip_prefix("construction.complete.")
                .or_else(|| source.signature.id.rsplit_once('.').map(|(id, _)| id));
            if let Some(f) = p
                .construction_definitions
                .iter()
                .find(|d| Some(d.construction.carrier.type_id.as_str()) == id)
            {
                let d = construction_data::emit(c, source, &f.construction)?;
                entry(
                    source,
                    &d.relation_definition,
                    d.failure_definitions.into_iter(),
                )
            } else if let Some(f) = p
                .sequence_definitions
                .iter()
                .find(|d| Some(d.sequence.carrier.type_id.as_str()) == id)
            {
                let d = sequence_data::emit(c, source, &f.sequence)?;
                entry(
                    source,
                    &d.relation_definition,
                    d.failure_definitions.into_iter().map(Some),
                )
            } else {
                return Ok(None);
            }
        }
        _ => return Ok(None),
    };
    Ok(Some(d))
}

fn aliases(c: &mut Clauses<'_>, d: &OrdinaryControlNativeDefinition) -> R<()> {
    let s = &d.source;
    if d.failure_definitions.len() != s.failure_names.len() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let mut args = s.signature.argument_type_ids.clone();
    args.push(s.signature.normal_result_type_id.clone());
    let pairs = std::iter::once((
        &s.relation_name,
        signature(&args, SOURCE_BOOL),
        &d.relation_definition,
    ))
    .chain(
        s.failure_names
            .iter()
            .zip(&d.failure_definitions)
            .filter_map(|(n, f)| {
                f.as_ref()
                    .map(|f| (n, signature(&s.signature.argument_type_ids, SOURCE_BOOL), f))
            }),
    );
    for (name, ty, core) in pairs {
        if !c.b.globals.contains_key(core) {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let value = (ty, core.clone());
        if c.constants.get(name).is_some_and(|old| old != &value) {
            return Err(OrdinaryCarrierError::Linkage);
        }
        c.constants.insert(name.clone(), value);
    }
    Ok(())
}

pub(super) fn append(
    c: &mut Clauses<'_>,
    p: &mut OrdinaryControlEdgeProgram,
    vir: &ValidatedPracticalVir,
) -> R<()> {
    let data = crate::csharp_practical_vir_model::data_vc::generate_data_vcs(vir)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    c.definedness_logic()?;
    let mut registry = definitions(p);
    let used = data
        .operations()
        .iter()
        .map(|o| o.definition_id.as_str())
        .collect::<BTreeSet<_>>();
    for source in data
        .definitions()
        .iter()
        .filter(|d| used.contains(d.id.as_str()))
    {
        if !registry.contains_key(&source.id) {
            if let Some(d) = emit_missing(c, p, source)? {
                registry.insert(source.id.clone(), d);
            }
        }
        if let Some(d) = registry.get(&source.id) {
            if d.source != *source {
                return Err(OrdinaryCarrierError::Linkage);
            }
            aliases(c, d)?;
        }
    }
    p.native_definitions = registry
        .values()
        .filter(|d| used.contains(d.source.id.as_str()))
        .cloned()
        .collect();
    for flow in &mut p.functions {
        let native = vir
            .functions()
            .iter()
            .find(|f| f.id == flow.source.function_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        for block in &native.blocks {
            let Some(invocation) = &block.invocation else {
                continue;
            };
            let Some(o) = data
                .operations()
                .iter()
                .find(|o| o.function_id == native.id && o.node_id == block.node.id)
            else {
                flow.pending_native_invocation_node_ids
                    .push(block.node.id.clone());
                continue;
            };
            let expected = invocation
                .operands
                .iter()
                .chain(std::iter::once(&invocation.result))
                .cloned()
                .collect::<Vec<_>>();
            let source = data
                .definitions()
                .iter()
                .find(|d| d.id == o.definition_id)
                .ok_or(OrdinaryCarrierError::Linkage)?;
            if o.subjects != expected
                || o.normal_successor_id != invocation.normal_successor_id
                || source.signature.id != invocation.operation_id
                || o.subjects.len() > 256
            {
                return Err(OrdinaryCarrierError::Linkage);
            }
            let arguments = o
                .subjects
                .iter()
                .enumerate()
                .map(|(i, s)| ControlBinding {
                    kind: if i < invocation.operands.len() {
                        "native_operand"
                    } else {
                        "native_result"
                    }
                    .into(),
                    edge_id: None,
                    node_id: if i < invocation.operands.len() {
                        block.node.id.clone()
                    } else {
                        invocation.normal_successor_id.clone()
                    },
                    value_id: s.id.clone(),
                    type_id: s.type_id.clone(),
                })
                .collect();
            let source_anchors = native
                .control_protocol
                .iter()
                .flat_map(|p| &p.anchors)
                .filter(|a| a.artifact_node_ids.contains(&block.node.id))
                .cloned()
                .collect();
            let mut operation = OrdinaryControlNativeOperation {
                source: o.clone(),
                invocation: invocation.clone(),
                source_anchors,
                arguments,
                ownership: None,
                predicates: BTreeMap::new(),
                pending_constant_names: vec![],
            };
            // Reuse only a binding proved at this exact function/node/receiver.
            let ownership = flow
                .edges
                .iter()
                .filter(|e| e.source.source_node_id == o.node_id)
                .find_map(|e| e.ownership.as_ref());
            if let Some(owner) = ownership {
                if owner.function_id != o.function_id
                    || owner.node_id != o.node_id
                    || Some(&owner.receiver_id) != invocation.operands.first().map(|a| &a.id)
                    || !source.failure_names.contains(&owner.source_failure_name)
                    || c.constants.contains_key(&owner.source_failure_name)
                {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                c.constants.insert(
                    owner.source_failure_name.clone(),
                    (
                        signature(&source.signature.argument_type_ids, SOURCE_BOOL),
                        owner.scoped_failure_definition.clone(),
                    ),
                );
                operation.ownership = Some(owner.clone());
            }
            let mut needed = BTreeSet::new();
            constants(&o.success_guard, &mut needed)?;
            constants(&o.success_relation, &mut needed)?;
            operation.pending_constant_names = needed
                .into_iter()
                .filter(|n| !c.constants.contains_key(n))
                .collect();
            if operation.pending_constant_names.is_empty() {
                for (role, term) in [
                    ("success_guard", &o.success_guard),
                    ("success_relation", &o.success_relation),
                ] {
                    let definition = name("NativePredicate", &(&o.id, role, term));
                    integer_data::predicate(c, &definition, &o.subjects, term)?;
                    operation.predicates.insert(role.into(), definition);
                }
                let terms = (0..o.subjects.len())
                    .map(|i| c.b.var((o.subjects.len() - 1 - i) as u32))
                    .collect::<R<Vec<_>>>()?;
                let guard = call(
                    &mut c.b,
                    &operation.predicates["success_guard"],
                    terms.clone(),
                )?;
                let relation = call(&mut c.b, &operation.predicates["success_relation"], terms)?;
                let mut body = call(&mut c.b, "Std.Bool.and", vec![guard, relation])?;
                for s in o.subjects.iter().rev() {
                    let ty = c.ty(&s.type_id, 0)?;
                    body = c.b.lam(ty, body)?;
                }
                let ty = c.ty(
                    &signature(
                        &o.subjects
                            .iter()
                            .map(|s| s.type_id.clone())
                            .collect::<Vec<_>>(),
                        SOURCE_BOOL,
                    ),
                    0,
                )?;
                let definition = name("NativeNormal", &(&o.id, &operation.arguments));
                c.b.define(&definition, ty, body)?;
                operation
                    .predicates
                    .insert("normal_execution".into(), definition);
            }
            if let Some(owner) = ownership {
                c.constants.remove(&owner.source_failure_name);
            }
            flow.native_operations.push(operation);
        }
    }
    Ok(())
}
