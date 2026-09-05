//! W09 private ordered collection projections and concrete foundation relations.
//! No source helper is replaced by these operations. T04 lowers its captured
//! loops; T06 proves sortedness, uniqueness and operation commutation.
use super::*;

pub const ORDERED_COLLECTION_LOOP_OWNER: &str = "CSHARP-03-T04-W01/W02";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrderedCollectionError {
    Binding,
    KeyType,
    OperandType,
    InvalidRepresentation,
    DuplicateKey,
    DuplicateElement,
    MissingKey,
    Capacity,
}
impl OrderedCollectionError {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Binding => "binding",
            Self::KeyType => "key_type",
            Self::OperandType => "operand_type",
            Self::InvalidRepresentation => "invalid_representation",
            Self::DuplicateKey => "duplicate_key",
            Self::DuplicateElement => "duplicate_element",
            Self::MissingKey => "missing_key",
            Self::Capacity => "capacity",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderedEntryBinding {
    pub source_type_id: String,
    pub source_content_sha256: String,
    pub key_member_id: String,
    pub value_member_id: String,
}

/// Exact concrete signature and outcome order. T04 supplies the source error
/// projection for each outcome; these names never invent a source exception.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrderedCollectionOperation {
    pub id: String,
    pub name: String,
    pub argument_type_ids: Vec<String>,
    pub result_type_id: String,
    pub ordered_outcomes: Vec<String>,
}

pub struct OrderedCollectionModel<'a> {
    bundle: &'a ValidatedFoundationBundle,
    roots: &'a ValidatedClosedRootSet,
    closed: &'a ClosedInstanceSet,
    id: String,
    key: String,
    value: Option<String>,
    lookup: Option<String>,
    operations: Vec<OrderedCollectionOperation>,
}

impl<'a> OrderedCollectionModel<'a> {
    pub fn new(
        bundle: &'a ValidatedFoundationBundle,
        roots: &'a ValidatedClosedRootSet,
        closed: &'a ClosedInstanceSet,
        id: &str,
    ) -> Result<Self, OrderedCollectionError> {
        let metadata = closed
            .metadata
            .get(id)
            .ok_or(OrderedCollectionError::Binding)?;
        let map = match template_name(&metadata.template_id) {
            Some("ordered_map") if metadata.argument_ids.len() == 2 => true,
            Some("ordered_set") if metadata.argument_ids.len() == 1 => false,
            _ => return Err(OrderedCollectionError::Binding),
        };
        let key = metadata.argument_ids[0].clone();
        // Check the key TYPE even for empty representations. Value inspection
        // cannot accidentally admit an empty float-key map or set.
        let key_program = generate_structural_program(bundle, roots, closed, &key)
            .map_err(|_| OrderedCollectionError::KeyType)?;
        if !key_program.is_total() {
            return Err(OrderedCollectionError::KeyType);
        }
        let comparison = generate_structural_program(bundle, roots, closed, id)
            .map_err(|_| OrderedCollectionError::Binding)?;
        let entry = closed
            .entries()
            .iter()
            .find(|e| e["instance_id"] == id)
            .ok_or(OrderedCollectionError::Binding)?;
        let mut operations = Vec::new();
        for definition in entry["operation_definitions"]
            .as_array()
            .ok_or(OrderedCollectionError::Binding)?
        {
            let name = definition["id"]
                .as_str()
                .and_then(|operation| operation.strip_prefix(&format!("{id}.")))
                .ok_or(OrderedCollectionError::Binding)?;
            if name == "compare" && !comparison.is_total() {
                continue;
            }
            operations.push(OrderedCollectionOperation {
                id: definition["id"]
                    .as_str()
                    .ok_or(OrderedCollectionError::Binding)?
                    .to_owned(),
                name: name.to_owned(),
                argument_type_ids: json_string_array(
                    definition["argument_type_ids"]
                        .as_array()
                        .ok_or(OrderedCollectionError::Binding)?,
                )
                .ok_or(OrderedCollectionError::Binding)?,
                result_type_id: definition["normal_result_type_id"]
                    .as_str()
                    .ok_or(OrderedCollectionError::Binding)?
                    .to_owned(),
                ordered_outcomes: json_string_array(
                    definition["error_precedence"]
                        .as_array()
                        .ok_or(OrderedCollectionError::Binding)?,
                )
                .ok_or(OrderedCollectionError::Binding)?,
            });
        }
        operations.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(Self {
            bundle,
            roots,
            closed,
            id: id.to_owned(),
            key,
            value: map.then(|| metadata.argument_ids[1].clone()),
            lookup: operations
                .iter()
                .find(|operation| operation.name == "lookup")
                .map(|operation| operation.result_type_id.clone()),
            operations,
        })
    }

    pub fn operations(&self) -> &[OrderedCollectionOperation] {
        &self.operations
    }
    pub fn validate_handoff(
        &self,
        candidate: &[OrderedCollectionOperation],
    ) -> Result<(), OrderedCollectionError> {
        if candidate == self.operations {
            Ok(())
        } else {
            Err(OrderedCollectionError::Binding)
        }
    }
    pub fn validate(&self, value: &MonomorphicValue) -> Result<(), OrderedCollectionError> {
        if value.type_id() != self.id {
            return Err(OrderedCollectionError::OperandType);
        }
        validate_monomorphic_value(self.bundle, self.roots, self.closed, value)
            .map_err(|_| OrderedCollectionError::InvalidRepresentation)
    }
    fn entry_members(
        &self,
        binding: &OrderedEntryBinding,
    ) -> Result<(&StoredMember, &StoredMember), OrderedCollectionError> {
        let source = self
            .roots
            .source_types
            .get(&binding.source_type_id)
            .ok_or(OrderedCollectionError::Binding)?;
        if source.kind == SourceKind::Enum
            || source.source_sha256 != binding.source_content_sha256
            || binding.key_member_id == binding.value_member_id
        {
            return Err(OrderedCollectionError::Binding);
        }
        let key = source
            .members
            .iter()
            .find(|m| m.id == binding.key_member_id)
            .ok_or(OrderedCollectionError::Binding)?;
        let value = source
            .members
            .iter()
            .find(|m| m.id == binding.value_member_id)
            .ok_or(OrderedCollectionError::Binding)?;
        if closed_type_id(self.bundle, &key.ty).map_err(|_| OrderedCollectionError::Binding)?
            != self.key
            || Some(
                closed_type_id(self.bundle, &value.ty)
                    .map_err(|_| OrderedCollectionError::Binding)?,
            ) != self.value
        {
            return Err(OrderedCollectionError::Binding);
        }
        Ok((key, value))
    }

    /// Projection never sorts or removes duplicates. All unmapped entry fields
    /// remain subject to field-complete reconstruction obligations in the source
    /// handoff; this concrete relation is not a round-trip proof.
    pub fn project(
        &self,
        source: &MonomorphicValue,
        wrapper: Option<&SequenceWrapperBinding>,
        entry_binding: Option<&OrderedEntryBinding>,
    ) -> Result<MonomorphicValue, OrderedCollectionError> {
        let members = match (&self.value, entry_binding) {
            (Some(_), Some(binding)) => Some(self.entry_members(binding)?),
            (None, None) => None,
            _ => return Err(OrderedCollectionError::Binding),
        };
        let sequence = if let Some(binding) = wrapper {
            project_bounded_sequence_wrapper(self.bundle, self.roots, self.closed, source, binding)
        } else {
            project_bounded_sequence_array(self.bundle, self.roots, self.closed, source)
        }
        .map_err(|_| OrderedCollectionError::Binding)?;
        let MonomorphicValue::Sequence { type_id, elements } = sequence else {
            return Err(OrderedCollectionError::Binding);
        };
        let arguments = require_instance(self.closed, &type_id, "bounded_sequence")
            .map_err(|_| OrderedCollectionError::Binding)?;
        let expected = entry_binding
            .map(|binding| binding.source_type_id.as_str())
            .unwrap_or(&self.key);
        if arguments[0] != expected {
            return Err(OrderedCollectionError::Binding);
        }
        let result = if let Some((key, value)) = members {
            let entries = elements
                .into_iter()
                .map(|element| {
                    let MonomorphicValue::Product { fields, .. } = element else {
                        return Err(OrderedCollectionError::Binding);
                    };
                    let k = fields
                        .iter()
                        .find(|f| f.name == key.name)
                        .ok_or(OrderedCollectionError::Binding)?;
                    let v = fields
                        .iter()
                        .find(|f| f.name == value.name)
                        .ok_or(OrderedCollectionError::Binding)?;
                    Ok(MonomorphicMapEntry {
                        key: k.value.clone(),
                        value: v.value.clone(),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            MonomorphicValue::OrderedMap {
                type_id: self.id.clone(),
                entries,
            }
        } else {
            MonomorphicValue::OrderedSet {
                type_id: self.id.clone(),
                elements,
            }
        };
        self.validate(&result)?;
        Ok(result)
    }
    pub fn count(&self, collection: &MonomorphicValue) -> Result<usize, OrderedCollectionError> {
        self.validate(collection)?;
        match collection {
            MonomorphicValue::OrderedMap { entries, .. } => Ok(entries.len()),
            MonomorphicValue::OrderedSet { elements, .. } => Ok(elements.len()),
            _ => Err(OrderedCollectionError::OperandType),
        }
    }
    fn position(
        &self,
        collection: &MonomorphicValue,
        key: &MonomorphicValue,
    ) -> Result<(usize, bool), OrderedCollectionError> {
        self.validate(collection)?;
        if key.type_id() != self.key {
            return Err(OrderedCollectionError::OperandType);
        }
        validate_monomorphic_value(self.bundle, self.roots, self.closed, key)
            .map_err(|_| OrderedCollectionError::OperandType)?;
        let keys: Vec<_> = match collection {
            MonomorphicValue::OrderedMap { entries, .. } => {
                entries.iter().map(|e| e.key.as_ref()).collect()
            }
            MonomorphicValue::OrderedSet { elements, .. } => elements.iter().collect(),
            _ => return Err(OrderedCollectionError::OperandType),
        };
        for (i, current) in keys.iter().enumerate() {
            let order = relate_monomorphic_values(
                self.bundle,
                self.roots,
                self.closed,
                false,
                current,
                key,
            )
            .map_err(|_| OrderedCollectionError::KeyType)?;
            if order != Ordering::Less {
                return Ok((i, order == Ordering::Equal));
            }
        }
        Ok((keys.len(), false))
    }
    pub fn contains(
        &self,
        collection: &MonomorphicValue,
        key: &MonomorphicValue,
    ) -> Result<bool, OrderedCollectionError> {
        Ok(self.position(collection, key)?.1)
    }
    pub fn lookup(
        &self,
        collection: &MonomorphicValue,
        key: &MonomorphicValue,
    ) -> Result<MonomorphicValue, OrderedCollectionError> {
        let lookup = self
            .lookup
            .as_ref()
            .ok_or(OrderedCollectionError::OperandType)?;
        let (i, found) = self.position(collection, key)?;
        let MonomorphicValue::OrderedMap { entries, .. } = collection else {
            return Err(OrderedCollectionError::OperandType);
        };
        let result = MonomorphicValue::TaggedSum {
            type_id: lookup.clone(),
            arm: if found { "found" } else { "missing_key" }.into(),
            payload: if found {
                vec![*entries[i].value.clone()]
            } else {
                vec![]
            },
        };
        validate_monomorphic_value(self.bundle, self.roots, self.closed, &result)
            .map_err(|_| OrderedCollectionError::Binding)?;
        Ok(result)
    }
    /// Concrete specification relation only. Source growth, sorting and helper
    /// acceptance remain T04 work. `capacity` is a stricter application bound,
    /// never an override that may raise the frozen 4096 maximum.
    pub fn update(
        &self,
        collection: &MonomorphicValue,
        key: MonomorphicValue,
        value: Option<MonomorphicValue>,
        replace: bool,
        capacity: usize,
    ) -> Result<MonomorphicValue, OrderedCollectionError> {
        let (position, found) = self.position(collection, &key)?;
        let effective_capacity = capacity.min(MAP_VALUE_LENGTH_MAX as usize);
        match (&self.value, &value) {
            (Some(expected), Some(value)) if value.type_id() == expected => {
                validate_monomorphic_value(self.bundle, self.roots, self.closed, value)
                    .map_err(|_| OrderedCollectionError::OperandType)?
            }
            (None, None) if !replace => {}
            _ => return Err(OrderedCollectionError::OperandType),
        }
        if replace && !found {
            return Err(OrderedCollectionError::MissingKey);
        }
        if !replace && found {
            return Err(if self.value.is_some() {
                OrderedCollectionError::DuplicateKey
            } else {
                OrderedCollectionError::DuplicateElement
            });
        }
        if !replace && self.count(collection)? >= effective_capacity {
            return Err(OrderedCollectionError::Capacity);
        }
        let mut result = collection.clone();
        match &mut result {
            MonomorphicValue::OrderedMap { entries, .. } => {
                let entry = MonomorphicMapEntry {
                    key: Box::new(key),
                    value: Box::new(value.ok_or(OrderedCollectionError::OperandType)?),
                };
                if replace {
                    entries[position] = entry;
                } else {
                    entries.insert(position, entry);
                }
            }
            MonomorphicValue::OrderedSet { elements, .. } => elements.insert(position, key),
            _ => return Err(OrderedCollectionError::OperandType),
        }
        self.validate(&result)?;
        Ok(result)
    }
}
