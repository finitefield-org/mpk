//! Private original-command records retain the same canonical bytes/domains.
//! Only root metadata/diagnostics hex fields use the already fixed native byte
//! budgets; ordinary JSON limits and every kernel/certificate limit stay put.
use super::*;
use crate::canonical_json::StrictJsonValue;

const STDOUT_BYTES: usize = 16_777_216;
const STDERR_BYTES: usize = 1_048_576;
fn known_schema(value: &StrictJsonValue) -> bool {
    matches!(
        value.get("schema").and_then(StrictJsonValue::as_str),
        Some(
            "mpk.go.practical.source_export_inputs.v1"
                | "mpk.go.practical.source_module_observations.v1"
                | "mpk.go.practical.source_module_graph_inputs.v2"
        )
    )
}
fn hex_limit(field: &str) -> Option<usize> {
    match field {
        "metadata_hex" => Some(2 * STDOUT_BYTES),
        "diagnostics_hex" => Some(2 * STDERR_BYTES),
        _ => None,
    }
}
fn ordinary(value: &StrictJsonValue) -> bool {
    match value {
        StrictJsonValue::String(s) => s.len() as u64 <= limits().max_string_bytes,
        StrictJsonValue::Array(values) => values.iter().all(ordinary),
        StrictJsonValue::Object(values) => values
            .iter()
            .all(|(k, v)| k.len() as u64 <= limits().max_string_bytes && ordinary(v)),
        _ => true,
    }
}
fn parse(record: &[u8]) -> Result<StrictJsonValue> {
    let mut bounds = limits();
    bounds.max_string_bytes = (2 * STDOUT_BYTES) as u64;
    let value = parse_strict_json(record, bounds).map_err(|_| GoTypeKeyError::Json)?;
    if !known_schema(&value) {
        return Err(GoTypeKeyError::Shape);
    }
    let fields = value.as_object().ok_or(GoTypeKeyError::Shape)?;
    for (key, item) in fields {
        if key.len() as u64 > limits().max_string_bytes {
            return Err(GoTypeKeyError::Json);
        }
        if let Some(limit) = hex_limit(key) {
            let s = item.as_str().ok_or(GoTypeKeyError::Shape)?;
            if s.len() > limit
                || !s.len().is_multiple_of(2)
                || !s
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err(GoTypeKeyError::Limit);
            }
        } else if !ordinary(item) {
            return Err(GoTypeKeyError::Json);
        }
    }
    if value.get("metadata_hex").is_none() || value.get("diagnostics_hex").is_none() {
        return Err(GoTypeKeyError::Shape);
    }
    Ok(value)
}
pub(super) fn decode<T: serde::de::DeserializeOwned + Serialize>(record: &[u8]) -> Result<T> {
    let original = parse(record)?;
    let value: T = serde_json::from_slice(record).map_err(|_| GoTypeKeyError::Shape)?;
    let encoded = serde_json::to_vec(&value).map_err(|_| GoTypeKeyError::Shape)?;
    let expected = parse(&encoded)?;
    if canonical_json_bytes(&original).map_err(|_| GoTypeKeyError::Json)?
        != canonical_json_bytes(&expected).map_err(|_| GoTypeKeyError::Json)?
    {
        return Err(GoTypeKeyError::Shape);
    }
    Ok(value)
}
pub(super) fn identity<T: Serialize>(record: &T, domain: HashDomain) -> Result<String> {
    let raw = serde_json::to_vec(record).map_err(|_| GoTypeKeyError::Shape)?;
    let value = parse(&raw)?;
    let schema = value.get("schema").and_then(StrictJsonValue::as_str);
    let expected = match schema {
        Some("mpk.go.practical.source_export_inputs.v1") => {
            HashDomain::new("MPK-PRIVATE-GO-SOURCE-EXPORT-INPUTS-1.0")
        }
        Some("mpk.go.practical.source_module_observations.v1") => {
            HashDomain::new("MPK-PRIVATE-GO-SOURCE-MODULE-OBSERVATIONS-1.0")
        }
        Some("mpk.go.practical.source_module_graph_inputs.v2") => {
            HashDomain::new("MPK-PRIVATE-GO-SOURCE-MODULE-GRAPH-INPUTS-2.0")
        }
        _ => return Err(GoTypeKeyError::Shape),
    };
    if domain != expected {
        return Err(GoTypeKeyError::Identity);
    }
    let body = value
        .clone_without_fields(&["content_sha256"])
        .map_err(|_| GoTypeKeyError::Shape)?;
    Ok(hash_canonical_json(domain, &body)
        .map_err(|_| GoTypeKeyError::Identity)?
        .to_hex())
}

#[cfg(test)]
mod tests;
