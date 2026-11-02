//! Closed practical-Go request selectors. This is a syntax/configuration
//! boundary: declaration existence, receiver aliases and source capture must
//! still be resolved against the original typed package universe.

use crate::canonical_json::{canonical_json_bytes, StrictJsonValue};
use crate::go_practical_type_keys::is_go_practical_identifier;
use crate::hash::{hash_canonical_json, HashDomain};
use std::fmt;

const SCHEMA: &str = "mpk.selection.go_functions.v1";
const DOMAIN: HashDomain = HashDomain::new("MPK-GO-PRACTICAL-SELECTION-1.0");
const ROOTS_MAX: usize = 8192;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoPracticalSelectionError {
    Json,
    Shape,
    Limit,
    Selector,
    Order,
}

impl fmt::Display for GoPracticalSelectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "practical Go selection: {self:?}")
    }
}

impl std::error::Error for GoPracticalSelectionError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoPracticalReceiverSelector {
    package: String,
    type_name: String,
}

impl GoPracticalReceiverSelector {
    pub fn package(&self) -> &str {
        &self.package
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoPracticalFunctionSelector {
    package: String,
    function: String,
    receiver: Option<GoPracticalReceiverSelector>,
}

impl GoPracticalFunctionSelector {
    pub fn package(&self) -> &str {
        &self.package
    }

    pub fn function(&self) -> &str {
        &self.function
    }

    pub fn receiver(&self) -> Option<&GoPracticalReceiverSelector> {
        self.receiver.as_ref()
    }
}

/// An owned, canonically ordered raw selection. It does not authorize a
/// practical profile or assert that a selector resolves to a source body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoPracticalSelection {
    roots: Vec<GoPracticalFunctionSelector>,
    canonical: Vec<u8>,
    sha256: String,
}

impl GoPracticalSelection {
    pub fn roots(&self) -> &[GoPracticalFunctionSelector] {
        &self.roots
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical
    }

    pub fn selection_sha256(&self) -> &str {
        &self.sha256
    }
}

fn exact_fields(value: &StrictJsonValue, fields: &[&str]) -> bool {
    value.as_object().is_some_and(|entries| {
        entries.len() == fields.len()
            && entries
                .iter()
                .all(|(name, _)| fields.contains(&name.as_str()))
    })
}

fn selector_identifier(value: &str) -> bool {
    value != "_" && is_go_practical_identifier(value)
}

fn selector(
    value: &StrictJsonValue,
) -> Result<GoPracticalFunctionSelector, GoPracticalSelectionError> {
    use GoPracticalSelectionError as E;
    if !exact_fields(value, &["package", "function", "receiver"]) {
        return Err(E::Shape);
    }
    let package = value
        .get("package")
        .and_then(StrictJsonValue::as_str)
        .ok_or(E::Selector)?;
    let function = value
        .get("function")
        .and_then(StrictJsonValue::as_str)
        .ok_or(E::Selector)?;
    if package.is_empty()
        || package.starts_with('/')
        || package.contains(['\\', '\0', '\n', '\r'])
        || !selector_identifier(function)
    {
        return Err(E::Selector);
    }
    let receiver = match value.get("receiver").ok_or(E::Shape)? {
        StrictJsonValue::Null => None,
        receiver if exact_fields(receiver, &["package", "type_name"]) => {
            let owner = receiver
                .get("package")
                .and_then(StrictJsonValue::as_str)
                .ok_or(E::Selector)?;
            let name = receiver
                .get("type_name")
                .and_then(StrictJsonValue::as_str)
                .ok_or(E::Selector)?;
            if owner != package || !selector_identifier(name) {
                return Err(E::Selector);
            }
            Some(GoPracticalReceiverSelector {
                package: owner.to_owned(),
                type_name: name.to_owned(),
            })
        }
        _ => return Err(E::Shape),
    };
    if function == "init" && receiver.is_none() {
        return Err(E::Selector);
    }
    Ok(GoPracticalFunctionSelector {
        package: package.to_owned(),
        function: function.to_owned(),
        receiver,
    })
}

/// Import the frozen Go selection envelope from a strict JSON value. The
/// caller's transport parser remains responsible for its own byte/depth caps.
pub fn import_go_practical_selection(
    value: &StrictJsonValue,
) -> Result<GoPracticalSelection, GoPracticalSelectionError> {
    use GoPracticalSelectionError as E;
    // This also rejects programmatically constructed duplicate names and
    // noncanonicalizable values before they can participate in identities.
    let canonical = canonical_json_bytes(value).map_err(|_| E::Json)?;
    if !exact_fields(value, &["schema", "value"])
        || value.get("schema").and_then(StrictJsonValue::as_str) != Some(SCHEMA)
    {
        return Err(E::Shape);
    }
    let selection = value.get("value").ok_or(E::Shape)?;
    if !exact_fields(selection, &["roots"]) {
        return Err(E::Shape);
    }
    let rows = selection
        .get("roots")
        .and_then(StrictJsonValue::as_array)
        .ok_or(E::Shape)?;
    if rows.is_empty() || rows.len() > ROOTS_MAX {
        return Err(E::Limit);
    }
    let mut roots = Vec::with_capacity(rows.len());
    let mut previous: Option<Vec<u8>> = None;
    for row in rows {
        let root = selector(row)?;
        let current = canonical_json_bytes(row).map_err(|_| E::Json)?;
        if previous.as_ref().is_some_and(|old| old >= &current) {
            return Err(E::Order);
        }
        previous = Some(current);
        roots.push(root);
    }
    let sha256 = hash_canonical_json(DOMAIN, value)
        .map_err(|_| E::Json)?
        .to_hex();
    Ok(GoPracticalSelection {
        roots,
        canonical,
        sha256,
    })
}
