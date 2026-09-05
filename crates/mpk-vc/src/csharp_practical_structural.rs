//! CSHARP-03-T03-W06. One specialization entry for contracts, boundaries and
//! collection foundations. This is private candidate infrastructure, not a
//! source-callable API or proof discharge. Recipes share a DAG by concrete ID.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructuralRecipe {
    pub type_id: String,
    pub rule: String,
    /// Stored declaration order or template argument order; never CLR order.
    pub children: Vec<String>,
    pub total: bool,
}

#[derive(Debug)]
pub struct StructuralProgram<'a> {
    bundle: &'a ValidatedFoundationBundle,
    roots: &'a ValidatedClosedRootSet,
    closed_set: &'a ClosedInstanceSet,
    root: String,
    recipes: BTreeMap<String, StructuralRecipe>,
}

impl StructuralProgram<'_> {
    pub fn recipes(&self) -> &BTreeMap<String, StructuralRecipe> {
        &self.recipes
    }
    pub fn is_total(&self) -> bool {
        self.recipes[&self.root].total
    }
    pub fn structural_equal(
        &self,
        left: &MonomorphicValue,
        right: &MonomorphicValue,
    ) -> Result<bool, FoundationValidationError> {
        self.validate_pair(left, right)?;
        Ok(
            relate_monomorphic_values(self.bundle, self.roots, self.closed_set, true, left, right)?
                == Ordering::Equal,
        )
    }
    pub fn canonical_compare(
        &self,
        left: &MonomorphicValue,
        right: &MonomorphicValue,
    ) -> Result<Ordering, FoundationValidationError> {
        // Type eligibility precedes value inspection, including empty sequences
        // and absent options containing float. No data-dependent key admission.
        if !self.is_total() {
            return Err(value_failure(FoundationErrorCode::NonTotalKey));
        }
        self.validate_pair(left, right)?;
        relate_monomorphic_values(self.bundle, self.roots, self.closed_set, false, left, right)
    }
    fn validate_pair(
        &self,
        left: &MonomorphicValue,
        right: &MonomorphicValue,
    ) -> Result<(), FoundationValidationError> {
        if left.type_id() != self.root || right.type_id() != self.root {
            return Err(value_failure(FoundationErrorCode::ConcreteValueType));
        }
        validate_monomorphic_value(self.bundle, self.roots, self.closed_set, left)?;
        validate_monomorphic_value(self.bundle, self.roots, self.closed_set, right)
    }
}

pub fn generate_structural_program<'a>(
    bundle: &'a ValidatedFoundationBundle,
    roots: &'a ValidatedClosedRootSet,
    closed_set: &'a ClosedInstanceSet,
    type_id: &str,
) -> Result<StructuralProgram<'a>, FoundationValidationError> {
    let mut program = StructuralProgram {
        bundle,
        roots,
        closed_set,
        root: type_id.to_owned(),
        recipes: BTreeMap::new(),
    };
    program.specialize(type_id, &mut BTreeSet::new())?;
    Ok(program)
}

impl StructuralProgram<'_> {
    fn specialize(
        &mut self,
        id: &str,
        active: &mut BTreeSet<String>,
    ) -> Result<bool, FoundationValidationError> {
        if let Some(recipe) = self.recipes.get(id) {
            return Ok(recipe.total);
        }
        if !active.insert(id.to_owned()) {
            return Err(source_failure(FoundationErrorCode::SourceCycle));
        }
        let (rule, children, mut total) = if let Some(source) = self.roots.source_types.get(id) {
            (
                if source.kind == SourceKind::Enum {
                    "enum_carrier"
                } else {
                    "stored_product"
                }
                .to_owned(),
                source
                    .members
                    .iter()
                    .map(|member| closed_type_id(self.bundle, &member.ty))
                    .collect::<Result<Vec<_>, _>>()?,
                true,
            )
        } else if let Some(metadata) = self.closed_set.metadata.get(id) {
            let name = template_name(&metadata.template_id)
                .ok_or_else(|| value_failure(FoundationErrorCode::ConcreteValueType))?;
            let rule = match name {
                "bounded_sequence" => "element_then_length",
                "ordered_entry" => "key_then_value",
                "ordered_map" => "canonical_entries_then_length",
                "ordered_set" => "canonical_elements_then_length",
                "option" => "null_first_active_payload",
                "boundary_field" | "lookup" | "result" | "validation" => "tag_then_active_payload",
                "money" => "currency_then_decimal",
                "transition" => "state_events_response",
                _ => return Err(value_failure(FoundationErrorCode::ConcreteValueType)),
            };
            let mut children = metadata.argument_ids.clone();
            if name == "money" {
                children.push("mpk.csharp.value.decimal.v1".to_owned());
            }
            (rule.to_owned(), children, true)
        } else {
            let primitive = id
                .strip_prefix("mpk.csharp.value.")
                .and_then(|s| s.strip_suffix(".v1"))
                .filter(|p| PRIMITIVES.contains(p))
                .ok_or_else(|| value_failure(FoundationErrorCode::ConcreteValueType))?;
            (
                match primitive {
                    "f32" | "f64" => "ieee_equal",
                    "decimal" => "decimal_numeric",
                    "guid" => "unsigned_n_fields",
                    "string" => "ordinal_utf16",
                    "exception" => "exception_tag_payload",
                    _ => "scalar_value",
                }
                .to_owned(),
                vec![],
                primitive_total(primitive),
            )
        };
        for child in &children {
            total &= self.specialize(child, active)?;
        }
        active.remove(id);
        self.recipes.insert(
            id.to_owned(),
            StructuralRecipe {
                type_id: id.to_owned(),
                rule,
                children,
                total,
            },
        );
        Ok(total)
    }
}

// Signature validation uses the same scalar matrix. Memoization preserves
// the bounds of a shared source DAG instead of expanding repeated fields.
pub(super) fn primitive_total(id: &str) -> bool {
    PRIMITIVES.contains(&id) && !matches!(id, "f32" | "f64" | "exception")
}
pub(super) fn concrete_total(
    roots: &ValidatedClosedRootSet,
    set: &ClosedInstanceSet,
    id: &str,
) -> bool {
    fn closed(
        roots: &ValidatedClosedRootSet,
        ty: &ClosedType,
        memo: &mut BTreeMap<ClosedType, bool>,
    ) -> bool {
        if let Some(result) = memo.get(ty) {
            return *result;
        }
        let result = match ty {
            ClosedType::Primitive(id) => primitive_total(id),
            ClosedType::Source(id) => roots
                .source_types
                .get(id)
                .is_some_and(|s| s.members.iter().all(|m| closed(roots, &m.ty, memo))),
            ClosedType::Instance {
                template,
                arguments,
            } => {
                template != "sequence_construction"
                    && arguments.iter().all(|a| closed(roots, a, memo))
            }
        };
        memo.insert(ty.clone(), result);
        result
    }
    fn concrete(
        roots: &ValidatedClosedRootSet,
        set: &ClosedInstanceSet,
        id: &str,
        types: &mut BTreeMap<ClosedType, bool>,
        ids: &mut BTreeMap<String, bool>,
    ) -> bool {
        if let Some(result) = ids.get(id) {
            return *result;
        }
        let result = if roots.source_types.contains_key(id) {
            closed(roots, &ClosedType::Source(id.to_owned()), types)
        } else if let Some(metadata) = set.metadata.get(id) {
            template_name(&metadata.template_id) != Some("sequence_construction")
                && metadata
                    .argument_ids
                    .iter()
                    .all(|a| concrete(roots, set, a, types, ids))
        } else {
            id.strip_prefix("mpk.csharp.value.")
                .and_then(|s| s.strip_suffix(".v1"))
                .is_some_and(primitive_total)
        };
        ids.insert(id.to_owned(), result);
        result
    }
    concrete(roots, set, id, &mut BTreeMap::new(), &mut BTreeMap::new())
}
