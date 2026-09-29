//! Exact closed nullable storage shared by control observations and relations.
use super::*;

pub(crate) fn option_payload_types(
    vir: &ValidatedPracticalVir,
) -> Result<BTreeMap<String, String>, ControlVcError> {
    vir.data_closed()
        .entries()
        .iter()
        .filter(|e| e["template_id"] == "mpk.csharp.semantic.option.v1")
        .map(|e| {
            let id = e["instance_id"].as_str().ok_or(ControlVcError::Contract)?;
            let metadata = vir
                .data_closed()
                .metadata
                .get(id)
                .ok_or(ControlVcError::Contract)?;
            let [payload] = metadata.argument_ids.as_slice() else {
                return Err(ControlVcError::Contract);
            };
            Ok((id.to_owned(), payload.clone()))
        })
        .collect()
}

// The source slot retains its nominal payload. Its storage is widened only to
// the exact validated Option<payload> occurring in that slot's native values.
pub(crate) fn represented_slot_types(
    vir: &ValidatedPracticalVir,
    flow: &ControlFunctionVc,
) -> Result<BTreeMap<String, String>, ControlVcError> {
    let payloads = option_payload_types(vir)?;
    let mut overrides = BTreeMap::new();
    for (slot, source) in &flow.slots {
        for value in flow
            .entry_values
            .iter()
            .filter(|(s, _)| s == slot)
            .map(|(_, v)| v)
            .chain(
                flow.transfers
                    .iter()
                    .filter(|t| &t.slot == slot)
                    .map(|t| &t.value),
            )
        {
            if payloads.get(&value.type_id) == Some(source)
                && overrides
                    .insert(slot.clone(), value.type_id.clone())
                    .is_some_and(|old| old != value.type_id)
            {
                return Err(ControlVcError::Contract);
            }
        }
    }
    Ok(overrides)
}
