//! Attach only sidecar bytes retained in the selected source snapshot.
use super::*;
use crate::csharp_practical_source_artifacts::{self as a, PracticalJsonValue as J};
#[derive(Clone, Debug)]
pub struct DataSidecars {
    bindings: Vec<a::SemanticBindingInput>,
    contracts: Vec<a::ValidatedPracticalArtifact>,
}
impl DataSidecars {
    pub fn bindings(&self) -> &[a::SemanticBindingInput] {
        &self.bindings
    }
    pub fn contracts(&self) -> &[a::ValidatedPracticalArtifact] {
        &self.contracts
    }
    pub fn capture(
        context: &a::PracticalArtifactContext,
        captures: &a::CapturedInputSet,
    ) -> Result<Self, DataPhaseError> {
        let mut bindings = None;
        let mut contracts = vec![];
        let mut subjects = BTreeSet::new();
        for path in context.sidecar_paths() {
            let bytes = captures.entry(path).ok_or(DataPhaseError::Sidecar)?.bytes();
            let value =
                a::parse_canonical_practical_json(a::PracticalArtifactKind::MethodContract, bytes)
                    .map_err(|_| DataPhaseError::Sidecar)?;
            let schema = value
                .get("schema")
                .and_then(J::as_str)
                .ok_or(DataPhaseError::Sidecar)?;
            let (kind, subject) = match schema {
                "mpk.csharp.semantic_bindings.v1" => {
                    if bindings.is_some() {
                        return Err(DataPhaseError::Sidecar);
                    }
                    let artifact = a::validate_semantic_bindings_document(
                        Some(context),
                        Some(captures),
                        bytes,
                    )
                    .map_err(|_| DataPhaseError::Sidecar)?;
                    let rows = artifact
                        .value()
                        .get("bindings")
                        .and_then(J::as_array)
                        .ok_or(DataPhaseError::Sidecar)?;
                    let values = rows
                        .iter()
                        .map(binding_input)
                        .collect::<Result<Vec<_>, _>>()?;
                    let rebuilt = a::build_semantic_bindings(context, captures, values.clone())
                        .map_err(|_| DataPhaseError::Sidecar)?;
                    if rebuilt.value() != artifact.value() {
                        return Err(DataPhaseError::Sidecar);
                    }
                    bindings = Some(values);
                    continue;
                }
                "mpk.csharp.type_contract.v1" => {
                    (a::PracticalArtifactKind::TypeContract, "source_type_id")
                }
                "mpk.csharp.contract.v1" => {
                    (a::PracticalArtifactKind::MethodContract, "callable_id")
                }
                "mpk.csharp.boundary.v1"
                | "mpk.csharp.boundary_input.v1"
                | "mpk.csharp.boundary_output.v1" => {
                    return Err(DataPhaseError::LaterOwner("CSHARP-03-T05-W01"))
                }
                "mpk.csharp.transition.v1" => {
                    return Err(DataPhaseError::LaterOwner("CSHARP-03-T05-W04"))
                }
                _ => return Err(DataPhaseError::Sidecar),
            };
            let artifact = a::validate_contract_artifact(context, captures, kind, bytes)
                .map_err(|_| DataPhaseError::Sidecar)?;
            let id = artifact
                .value()
                .get(subject)
                .and_then(J::as_str)
                .ok_or(DataPhaseError::Sidecar)?;
            if !subjects.insert((schema.to_owned(), id.to_owned())) {
                return Err(DataPhaseError::Sidecar);
            }
            contracts.push(artifact);
        }
        contracts.sort_by(|left, right| {
            left.schema()
                .cmp(right.schema())
                .then_with(|| left.hash().cmp(right.hash()))
        });
        Ok(Self {
            bindings: bindings.unwrap_or_default(),
            contracts,
        })
    }
}
fn binding_input(row: &J) -> Result<a::SemanticBindingInput, DataPhaseError> {
    let string = |name: &str| {
        row.get(name)
            .and_then(J::as_str)
            .map(str::to_owned)
            .ok_or(DataPhaseError::Sidecar)
    };
    let map = |name: &str| -> Result<Vec<(String, String)>, DataPhaseError> {
        row.get(name)
            .and_then(J::as_object)
            .ok_or(DataPhaseError::Sidecar)?
            .iter()
            .map(|(key, value)| {
                Ok((
                    key.clone(),
                    value.as_str().ok_or(DataPhaseError::Sidecar)?.into(),
                ))
            })
            .collect()
    };
    Ok(a::SemanticBindingInput {
        enum_arms: row
            .get("enum_arms")
            .and_then(J::as_object)
            .ok_or(DataPhaseError::Sidecar)?
            .iter()
            .map(|(id, arms)| {
                let arms = arms
                    .as_object()
                    .ok_or(DataPhaseError::Sidecar)?
                    .iter()
                    .map(|(label, value)| {
                        Ok((
                            label.clone(),
                            value.as_str().ok_or(DataPhaseError::Sidecar)?.to_owned(),
                        ))
                    })
                    .collect::<Result<_, DataPhaseError>>()?;
                Ok((id.clone(), arms))
            })
            .collect::<Result<_, DataPhaseError>>()?,
        source_type_id: string("source_type_id")?,
        source_content_sha256: string("source_content_sha256")?,
        role: string("role")?,
        default_arm: string("default_arm")?,
        inferred_argument_ids: row
            .get("inferred_argument_ids")
            .and_then(J::as_array)
            .ok_or(DataPhaseError::Sidecar)?
            .iter()
            .map(|v| v.as_str().map(str::to_owned).ok_or(DataPhaseError::Sidecar))
            .collect::<Result<_, _>>()?,
        member_map: map("member_map")?
            .into_iter()
            .map(|(role, member_id)| a::SemanticBindingMember { role, member_id })
            .collect(),
        tag_arms: map("tag_arms")?
            .into_iter()
            .map(|(semantic_arm, source_tag)| a::SemanticArmMapping {
                source_tag,
                semantic_arm,
            })
            .collect(),
        operation_map: map("operation_map")?
            .into_iter()
            .map(|(operation, member_id)| a::SemanticOperationMapping {
                operation,
                member_id,
            })
            .collect(),
        bounds: row
            .get("bounds")
            .and_then(J::as_object)
            .ok_or(DataPhaseError::Sidecar)?
            .iter()
            .map(|(id, value)| {
                Ok(a::SemanticBound {
                    id: id.clone(),
                    maximum: value.as_u64().ok_or(DataPhaseError::Sidecar)?,
                })
            })
            .collect::<Result<_, DataPhaseError>>()?,
    })
}
