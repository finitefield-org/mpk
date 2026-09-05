//! W08 concrete construction elimination. Symbolic predicates stay in VIR for
//! T06; this interpreter never treats an undischarged obligation as a value.
use super::*;

pub const SEQUENCE_STATES_PER_METHOD_MAX: usize = 32;
pub const SEQUENCE_LIVE_STATES_MAX: usize = 8;

struct ConstructionCells {
    state: SequenceConstructionState,
    cells: Vec<Option<MonomorphicValue>>,
}

/// One method's linear construction substrate. No caller-supplied initialized
/// set, default-eligibility flag or mutable state can be imported through it.
/// Recursive defaults are explicit typed fills, just like source initializers.
pub struct SequenceConstructionBatch<'a> {
    bundle: &'a ValidatedFoundationBundle,
    roots: &'a ValidatedClosedRootSet,
    closed: &'a ClosedInstanceSet,
    states: BTreeMap<String, ConstructionCells>,
    published: BTreeMap<String, MonomorphicValue>,
}

fn failure(code: PracticalVirErrorCode) -> PracticalVirValidationError {
    vir_failure(PracticalVirValidationPhase::Construction, code)
}

impl<'a> SequenceConstructionBatch<'a> {
    pub fn new(
        bundle: &'a ValidatedFoundationBundle,
        roots: &'a ValidatedClosedRootSet,
        closed: &'a ClosedInstanceSet,
    ) -> Self {
        Self {
            bundle,
            roots,
            closed,
            states: BTreeMap::new(),
            published: BTreeMap::new(),
        }
    }

    pub fn allocate(
        &mut self,
        id: &str,
        instance: &str,
        owner: &str,
        length: i64,
    ) -> Result<(), PracticalVirValidationError> {
        if self.states.contains_key(id) {
            return Err(failure(PracticalVirErrorCode::ConstructionOwnership));
        }
        if self.states.len() >= SEQUENCE_STATES_PER_METHOD_MAX
            || self
                .states
                .values()
                .filter(|s| s.state.status == ConstructionStatus::Active)
                .count()
                >= SEQUENCE_LIVE_STATES_MAX
        {
            return Err(failure(PracticalVirErrorCode::ConstructionBound));
        }
        // Array/wrapper publication retains the 4096 bound, even though the
        // internal construction capacity is 16384. No public caller can raise it.
        let state = SequenceConstructionState::allocate(
            self.closed,
            id,
            instance,
            owner,
            length,
            false,
            ARRAY_VALUE_LENGTH_MAX as u32,
        )?;
        let cells = vec![None; state.length as usize];
        self.states
            .insert(id.to_owned(), ConstructionCells { state, cells });
        Ok(())
    }

    /// Validate the transition and concrete element before changing any state.
    /// Freeze returns only an immutable value; transfer keeps a unique owner.
    pub fn apply(
        &mut self,
        id: &str,
        action: &SequenceConstructionAction,
        value: Option<MonomorphicValue>,
    ) -> Result<Option<MonomorphicValue>, PracticalVirValidationError> {
        let current = self
            .states
            .get(id)
            .ok_or_else(|| failure(PracticalVirErrorCode::ConstructionOwnership))?;
        let effect = current.state.apply(self.closed, action)?;
        let write_index = match action {
            SequenceConstructionAction::Fill { index, .. }
            | SequenceConstructionAction::Rewrite { index, .. } => Some(*index as usize),
            _ => None,
        };
        if write_index.is_some() != value.is_some() {
            return Err(failure(PracticalVirErrorCode::OperandType));
        }
        if let Some(value) = &value {
            if value.type_id() != current.state.element_type_id {
                return Err(failure(PracticalVirErrorCode::OperandType));
            }
            validate_monomorphic_value(self.bundle, self.roots, self.closed, value)
                .map_err(|_| failure(PracticalVirErrorCode::OperandType))?;
        }
        let result = match action {
            SequenceConstructionAction::Read { index, .. } => {
                current.cells[*index as usize].clone()
            }
            SequenceConstructionAction::Freeze { .. } => {
                let elements = current
                    .cells
                    .iter()
                    .cloned()
                    .collect::<Option<Vec<_>>>()
                    .ok_or_else(|| failure(PracticalVirErrorCode::ConstructionInitialization))?;
                let published = MonomorphicValue::Sequence {
                    type_id: current.state.published_type_id.clone(),
                    elements,
                };
                validate_monomorphic_value(self.bundle, self.roots, self.closed, &published)
                    .map_err(|_| failure(PracticalVirErrorCode::ConstructionBound))?;
                Some(published)
            }
            _ => None,
        };
        let current = self
            .states
            .get_mut(id)
            .expect("validated construction identity");
        if let Some(index) = write_index {
            current.cells[index] = value;
        }
        current.state = effect.state;
        if let SequenceConstructionAction::Freeze { .. } = action {
            self.published
                .insert(id.to_owned(), result.clone().expect("freeze result"));
            current.cells.clear();
        } else if let SequenceConstructionAction::Discard { .. } = action {
            current.cells.clear();
        }
        Ok(result)
    }

    pub fn state(&self, id: &str) -> Option<&SequenceConstructionState> {
        self.states.get(id).map(|s| &s.state)
    }

    /// The public/certificate handoff contains no construction state. A forgotten
    /// freeze/discard rejects the entire method, including earlier publications.
    pub fn finish(self) -> Result<BTreeMap<String, MonomorphicValue>, PracticalVirValidationError> {
        if self
            .states
            .values()
            .any(|s| s.state.status == ConstructionStatus::Active)
        {
            return Err(failure(PracticalVirErrorCode::ConstructionState));
        }
        Ok(self.published)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SequenceWrapperBinding {
    pub source_type_id: String,
    pub source_content_sha256: String,
    pub elements_member_id: String,
    pub sequence_type_id: String,
}

fn scalar_metadata(roots: &ValidatedClosedRootSet, ty: &ClosedType) -> bool {
    match ty {
        ClosedType::Primitive(_) => true,
        ClosedType::Source(id) => roots
            .source_types
            .get(id)
            .is_some_and(|s| s.kind == SourceKind::Enum),
        ClosedType::Instance {
            template,
            arguments,
        } => template == "option" && arguments.len() == 1 && scalar_metadata(roots, &arguments[0]),
    }
}

/// Project exactly one stored non-null array with scalar metadata. The caller
/// must already have validated the content-bound semantic binding document;
/// projection does not discharge constructor/getter/commutation obligations.
pub fn project_bounded_sequence_wrapper(
    bundle: &ValidatedFoundationBundle,
    roots: &ValidatedClosedRootSet,
    closed: &ClosedInstanceSet,
    value: &MonomorphicValue,
    binding: &SequenceWrapperBinding,
) -> Result<MonomorphicValue, PracticalVirValidationError> {
    validate_monomorphic_value(bundle, roots, closed, value)
        .map_err(|_| failure(PracticalVirErrorCode::OperandType))?;
    let MonomorphicValue::Product { type_id, fields } = value else {
        return Err(failure(PracticalVirErrorCode::OperandType));
    };
    let source = roots
        .source_types
        .get(type_id)
        .ok_or_else(|| failure(PracticalVirErrorCode::OperandType))?;
    if source.id != binding.source_type_id || source.source_sha256 != binding.source_content_sha256
    {
        return Err(failure(PracticalVirErrorCode::OperandType));
    }
    let mut array_member = None;
    for member in &source.members {
        match &member.ty {
            ClosedType::Instance { template, .. } if template == "bounded_sequence" => {
                if array_member.replace(member).is_some() {
                    return Err(failure(PracticalVirErrorCode::OperandType));
                }
            }
            ty if scalar_metadata(roots, ty) => {}
            _ => return Err(failure(PracticalVirErrorCode::OperandType)),
        }
    }
    let member = array_member
        .filter(|m| m.id == binding.elements_member_id)
        .ok_or_else(|| failure(PracticalVirErrorCode::OperandType))?;
    let field = fields
        .iter()
        .find(|f| f.name == member.name)
        .ok_or_else(|| failure(PracticalVirErrorCode::OperandType))?;
    if field.value.type_id() != binding.sequence_type_id {
        return Err(failure(PracticalVirErrorCode::OperandType));
    }
    project_bounded_sequence_array(bundle, roots, closed, &field.value)
}

/// Both admitted source arrays and wrapper storage use the same immutable
/// representation before shared equality or lexicographic comparison.
pub fn project_bounded_sequence_array(
    bundle: &ValidatedFoundationBundle,
    roots: &ValidatedClosedRootSet,
    closed: &ClosedInstanceSet,
    value: &MonomorphicValue,
) -> Result<MonomorphicValue, PracticalVirValidationError> {
    validate_monomorphic_value(bundle, roots, closed, value)
        .map_err(|_| failure(PracticalVirErrorCode::OperandType))?;
    match value {
        MonomorphicValue::Array { type_id, elements }
        | MonomorphicValue::Sequence { type_id, elements }
            if elements.len() <= ARRAY_VALUE_LENGTH_MAX as usize =>
        {
            Ok(MonomorphicValue::Sequence {
                type_id: type_id.clone(),
                elements: elements.clone(),
            })
        }
        _ => Err(failure(PracticalVirErrorCode::OperandType)),
    }
}

pub fn bounded_sequence_length(
    bundle: &ValidatedFoundationBundle,
    roots: &ValidatedClosedRootSet,
    closed: &ClosedInstanceSet,
    value: &MonomorphicValue,
) -> Result<usize, PracticalVirValidationError> {
    validate_monomorphic_value(bundle, roots, closed, value)
        .map_err(|_| failure(PracticalVirErrorCode::OperandType))?;
    match value {
        MonomorphicValue::Array { elements, .. } | MonomorphicValue::Sequence { elements, .. } => {
            Ok(elements.len())
        }
        _ => Err(failure(PracticalVirErrorCode::OperandType)),
    }
}

pub fn bounded_sequence_read<'a>(
    bundle: &ValidatedFoundationBundle,
    roots: &ValidatedClosedRootSet,
    closed: &ClosedInstanceSet,
    value: &'a MonomorphicValue,
    index: i32,
) -> Result<&'a MonomorphicValue, PracticalVirValidationError> {
    let length = bounded_sequence_length(bundle, roots, closed, value)?;
    let index = checked_construction_index(index, length as u32)? as usize;
    match value {
        MonomorphicValue::Array { elements, .. } | MonomorphicValue::Sequence { elements, .. } => {
            Ok(&elements[index])
        }
        _ => Err(failure(PracticalVirErrorCode::OperandType)),
    }
}
