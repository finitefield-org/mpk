//! Private, owned object construction. The shared foundation supplies every
//! slot/product carrier and validates payloads; this module supplies lifetime
//! and definite-assignment rules, never a second value expansion algorithm.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObjectConstructionError {
    Owner,
    Origin,
    Member,
    Unassigned,
    DuplicateAssignment,
    Payload,
    Incomplete,
    Carrier,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObjectConstructionOperation {
    Begin {
        owner: String,
    },
    Read {
        owner: String,
        member: String,
        ordinal: usize,
    },
    Write {
        owner: String,
        member: String,
        ordinal: usize,
    },
    Finalize {
        owner: String,
    },
}

impl ObjectConstructionOperation {
    pub fn from_id(
        roots: &ValidatedClosedRootSet,
        id: &str,
    ) -> Result<Self, ObjectConstructionError> {
        if let Some(owner) = id.strip_prefix("object.begin.") {
            object_construction_root(roots, owner)?;
            return Ok(Self::Begin {
                owner: owner.into(),
            });
        }
        if let Some(owner) = id.strip_prefix("object.finalize.") {
            object_construction_root(roots, owner)?;
            return Ok(Self::Finalize {
                owner: owner.into(),
            });
        }
        let (write, member) = if let Some(member) = id.strip_prefix("object.write.") {
            (true, member)
        } else if let Some(member) = id.strip_prefix("object.read.") {
            (false, member)
        } else {
            return Err(ObjectConstructionError::Member);
        };
        let source = roots
            .source_types
            .values()
            .find(|s| s.members.iter().any(|m| m.id == member))
            .ok_or(ObjectConstructionError::Member)?;
        object_construction_root(roots, &source.id)?;
        let ordinal = source
            .members
            .iter()
            .position(|m| m.id == member)
            .ok_or(ObjectConstructionError::Member)?;
        Ok(if write {
            Self::Write {
                owner: source.id.clone(),
                member: member.into(),
                ordinal,
            }
        } else {
            Self::Read {
                owner: source.id.clone(),
                member: member.into(),
                ordinal,
            }
        })
    }
    pub fn owner(&self) -> &str {
        match self {
            Self::Begin { owner }
            | Self::Read { owner, .. }
            | Self::Write { owner, .. }
            | Self::Finalize { owner } => owner,
        }
    }
}

/// Preserve the source identity and logical signature while threading one
/// owned transaction through the private constructor execution boundary.
pub fn object_constructor_execution_signature(
    bundle: &ValidatedFoundationBundle,
    roots: &ValidatedClosedRootSet,
    closed: &ClosedInstanceSet,
    callable: &DataSourceCallable,
) -> Result<ClosedOperationSignature, ObjectConstructionError> {
    if callable.identity()["kind"] != "constructor" {
        return Err(ObjectConstructionError::Owner);
    }
    let logical = callable
        .logical_signature(bundle)
        .map_err(|_| ObjectConstructionError::Owner)?;
    let begin = object_construction_signature(
        roots,
        closed,
        &format!("object.begin.{}", logical.normal_result_type_id),
    )?;
    let mut arguments = vec![begin.normal_result_type_id.clone()];
    arguments.extend(logical.argument_type_ids);
    Ok(ClosedOperationSignature {
        id: logical.id,
        tag: ClosedOperationTag::ConstructorExecute,
        argument_type_ids: arguments,
        normal_result_type_id: begin.normal_result_type_id,
        ordered_checks: vec![],
    })
}

pub fn object_construction_can_finalize(
    bundle: &ValidatedFoundationBundle,
    roots: &ValidatedClosedRootSet,
    closed: &ClosedInstanceSet,
    owner: &str,
    definitely_assigned: u32,
) -> Result<(), ObjectConstructionError> {
    object_construction_root(roots, owner)?;
    for (ordinal, member) in roots.source_types[owner].members.iter().enumerate() {
        if definitely_assigned & (1u32 << ordinal) != 0 {
            continue;
        }
        if member.required {
            return Err(ObjectConstructionError::Incomplete);
        }
        let ty =
            closed_type_id(bundle, &member.ty).map_err(|_| ObjectConstructionError::Payload)?;
        domain::default_with_obligations(bundle, roots, closed, &ty, false)
            .map_err(|_| ObjectConstructionError::Incomplete)?;
    }
    Ok(())
}

pub fn object_construction_constructor_member(
    roots: &ValidatedClosedRootSet,
    owner: &str,
    ordinal: usize,
) -> Result<(), ObjectConstructionError> {
    object_construction_root(roots, owner)?;
    let member = roots.source_types[owner]
        .members
        .get(ordinal)
        .ok_or(ObjectConstructionError::Member)?;
    if member.required {
        return Err(ObjectConstructionError::Member);
    }
    Ok(())
}
pub fn object_construction_signature(
    roots: &ValidatedClosedRootSet,
    closed: &ClosedInstanceSet,
    id: &str,
) -> Result<ClosedOperationSignature, ObjectConstructionError> {
    let operation = ObjectConstructionOperation::from_id(roots, id)?;
    let source = &roots.source_types[operation.owner()];
    let ty = object_construction_type(
        &source
            .members
            .iter()
            .map(|m| m.ty.clone())
            .collect::<Vec<_>>(),
    );
    let carrier = closed_type_id_for_operation(closed, roots, &ty)
        .map_err(|_| ObjectConstructionError::Carrier)?;
    let (arguments, result) = match operation {
        ObjectConstructionOperation::Begin { .. } => (vec![], carrier),
        ObjectConstructionOperation::Finalize { .. } => (vec![carrier], source.id.clone()),
        ObjectConstructionOperation::Read { ordinal, .. } => (
            vec![carrier],
            closed_type_id_for_operation(closed, roots, &source.members[ordinal].ty)
                .map_err(|_| ObjectConstructionError::Payload)?,
        ),
        ObjectConstructionOperation::Write { ordinal, .. } => (
            vec![
                carrier.clone(),
                closed_type_id_for_operation(closed, roots, &source.members[ordinal].ty)
                    .map_err(|_| ObjectConstructionError::Payload)?,
            ],
            carrier,
        ),
    };
    Ok(ClosedOperationSignature {
        id: id.into(),
        tag: ClosedOperationTag::Data,
        argument_type_ids: arguments,
        normal_result_type_id: result,
        ordered_checks: vec![],
    })
}

/// This type deliberately has no Clone or Deserialize implementation. A
/// serialized carrier alone cannot create a construction capability.
#[derive(Debug)]
pub struct ObjectConstructionState {
    origin: String,
    owner: String,
    source_context_sha256: String,
    slots: Vec<Option<MonomorphicValue>>,
}

/// The balanced tree is just a closed type expression consumed by T02.
pub(super) fn object_construction_type(members: &[ClosedType]) -> ClosedType {
    match members.len() {
        0 => ClosedType::Primitive("unit".into()),
        1 => ClosedType::Instance {
            template: "boundary_field".into(),
            arguments: vec![members[0].clone()],
        },
        n => ClosedType::Instance {
            template: "ordered_entry".into(),
            arguments: vec![
                object_construction_type(&members[..n / 2]),
                object_construction_type(&members[n / 2..]),
            ],
        },
    }
}

/// Root provenance must come from an actual owner declaration, not a supplied
/// semantic binding. The native source importer supplies that declaration.
pub fn object_construction_root(
    roots: &ValidatedClosedRootSet,
    owner: &str,
) -> Result<Value, ObjectConstructionError> {
    let source = roots
        .source_types
        .get(owner)
        .filter(|s| {
            s.kind != SourceKind::Enum && s.members.iter().any(|m| m.storage == "init_auto")
        })
        .ok_or(ObjectConstructionError::Owner)?;
    let members = source
        .members
        .iter()
        .map(|m| m.ty.clone())
        .collect::<Vec<_>>();
    Ok(
        json!({"origin":"source_construction", "provenance_id":format!("{owner}.object_construction"),
        "type":object_construction_type(&members).to_value()}),
    )
}

impl ObjectConstructionState {
    pub fn begin(
        roots: &ValidatedClosedRootSet,
        owner: &str,
        origin: &str,
    ) -> Result<Self, ObjectConstructionError> {
        object_construction_root(roots, owner)?;
        if origin.is_empty() || origin.len() > 1024 {
            return Err(ObjectConstructionError::Origin);
        }
        Ok(Self {
            origin: origin.into(),
            owner: owner.into(),
            source_context_sha256: sha256_raw_file_bytes(roots.canonical_json()).to_hex(),
            slots: vec![None; roots.source_types[owner].members.len()],
        })
    }
    pub fn origin(&self) -> &str {
        &self.origin
    }
    pub fn owner(&self) -> &str {
        &self.owner
    }
    pub fn assigned_members(&self) -> u32 {
        self.slots.iter().enumerate().fold(0, |mask, (i, slot)| {
            mask | if slot.is_some() { 1u32 << i } else { 0 }
        })
    }
    fn member(
        &self,
        roots: &ValidatedClosedRootSet,
        member_id: &str,
    ) -> Result<usize, ObjectConstructionError> {
        self.check_context(roots)?;
        roots
            .source_types
            .get(&self.owner)
            .and_then(|s| s.members.iter().position(|m| m.id == member_id))
            .ok_or(ObjectConstructionError::Member)
    }
    fn check_context(&self, roots: &ValidatedClosedRootSet) -> Result<(), ObjectConstructionError> {
        if self.source_context_sha256 != sha256_raw_file_bytes(roots.canonical_json()).to_hex() {
            return Err(ObjectConstructionError::Owner);
        }
        Ok(())
    }
    pub fn read(
        &self,
        roots: &ValidatedClosedRootSet,
        member_id: &str,
    ) -> Result<&MonomorphicValue, ObjectConstructionError> {
        self.slots[self.member(roots, member_id)?]
            .as_ref()
            .ok_or(ObjectConstructionError::Unassigned)
    }
    pub fn write(
        &mut self,
        bundle: &ValidatedFoundationBundle,
        roots: &ValidatedClosedRootSet,
        closed: &ClosedInstanceSet,
        member_id: &str,
        value: MonomorphicValue,
    ) -> Result<(), ObjectConstructionError> {
        let ordinal = self.member(roots, member_id)?;
        if self.slots[ordinal].is_some() {
            return Err(ObjectConstructionError::DuplicateAssignment);
        }
        let expected = closed_type_id(bundle, &roots.source_types[&self.owner].members[ordinal].ty)
            .map_err(|_| ObjectConstructionError::Payload)?;
        if value.type_id() != expected
            || validate_monomorphic_value(bundle, roots, closed, &value).is_err()
        {
            return Err(ObjectConstructionError::Payload);
        }
        self.slots[ordinal] = Some(value);
        Ok(())
    }
    /// The carrier is useful to check the sole T02 representation. It confers
    /// no ownership and is never accepted as input to begin/write/finalize.
    pub fn carrier(
        &self,
        bundle: &ValidatedFoundationBundle,
        roots: &ValidatedClosedRootSet,
        closed: &ClosedInstanceSet,
    ) -> Result<MonomorphicValue, ObjectConstructionError> {
        self.check_context(roots)?;
        fn build(
            bundle: &ValidatedFoundationBundle,
            members: &[ClosedType],
            slots: &[Option<MonomorphicValue>],
        ) -> Result<MonomorphicValue, ObjectConstructionError> {
            let type_id = closed_type_id(bundle, &object_construction_type(members))
                .map_err(|_| ObjectConstructionError::Carrier)?;
            Ok(match members.len() {
                0 => MonomorphicValue::Unit { type_id },
                1 => MonomorphicValue::BoundaryPresence {
                    type_id,
                    arm: if slots[0].is_some() {
                        BoundaryArm::Value
                    } else {
                        BoundaryArm::Missing
                    },
                    value: slots[0].clone().map(Box::new),
                },
                n => MonomorphicValue::OrderedEntry {
                    type_id,
                    key: Box::new(build(bundle, &members[..n / 2], &slots[..n / 2])?),
                    value: Box::new(build(bundle, &members[n / 2..], &slots[n / 2..])?),
                },
            })
        }
        let members = roots.source_types[&self.owner]
            .members
            .iter()
            .map(|m| m.ty.clone())
            .collect::<Vec<_>>();
        let carrier = build(bundle, &members, &self.slots)?;
        validate_monomorphic_value(bundle, roots, closed, &carrier)
            .map_err(|_| ObjectConstructionError::Carrier)?;
        Ok(carrier)
    }
    /// Consuming finalization is the only publication point. Existing source
    /// handoff obligations remain pending; value validity is not invariant proof.
    pub fn finalize(
        self,
        bundle: &ValidatedFoundationBundle,
        roots: &ValidatedClosedRootSet,
        closed: &ClosedInstanceSet,
    ) -> Result<MonomorphicValue, ObjectConstructionError> {
        self.check_context(roots)?;
        let source = roots
            .source_types
            .get(&self.owner)
            .ok_or(ObjectConstructionError::Owner)?;
        let fields = source
            .members
            .iter()
            .zip(self.slots)
            .map(|(member, slot)| {
                let value = match slot {
                    Some(value) => value,
                    None if member.required => return Err(ObjectConstructionError::Incomplete),
                    None => {
                        let type_id = closed_type_id(bundle, &member.ty)
                            .map_err(|_| ObjectConstructionError::Payload)?;
                        // Finalization's CLR default is structural. Source public
                        // invariants remain the W05 obligations retained in VIR.
                        domain::default_with_obligations(bundle, roots, closed, &type_id, false)
                            .map_err(|_| ObjectConstructionError::Incomplete)?
                    }
                };
                Ok(NamedMonomorphicValue {
                    name: member.name.clone(),
                    value: Box::new(value),
                })
            })
            .collect::<Result<Vec<_>, ObjectConstructionError>>()?;
        let value = MonomorphicValue::Product {
            type_id: self.owner,
            fields,
        };
        validate_monomorphic_value(bundle, roots, closed, &value)
            .map_err(|_| ObjectConstructionError::Payload)?;
        Ok(value)
    }
    /// Exception/abandon paths consume the capability without a source value.
    pub fn discard(self) {}
}
