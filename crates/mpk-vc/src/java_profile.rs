//! Closed Java names and selection grammar. No filesystem or compiler lookup.

use crate::source_map::is_portable_normalized_path;
use crate::vir::{BitVectorWidth, VirType};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

const EXCLUDED_WORDS: &str = "abstract assert boolean break byte case catch char class const continue default do double else enum extends final finally float for goto if implements import instanceof int interface long native new package private protected public return short static strictfp super switch synchronized this throw throws transient try void volatile while _ true false null exports module non-sealed open opens permits provides record requires sealed to transitive uses var when with yield";

pub(crate) fn valid_identifier(name: &str) -> bool {
    name.as_bytes()
        .first()
        .is_some_and(|c| c.is_ascii_alphabetic() || *c == b'_')
        && name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
        && !EXCLUDED_WORDS
            .split_ascii_whitespace()
            .any(|word| word == name)
}

pub(crate) fn valid_compilation(name: &str) -> bool {
    (1..=64).contains(&name.len())
        && name.as_bytes()[0].is_ascii_lowercase()
        && name.split(['.', '_', '-']).all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
}

fn valid_package(name: &str) -> bool {
    name.split('.').all(valid_identifier)
        && !["java", "javax", "jdk", "sun", "com.sun"]
            .iter()
            .any(|prefix| {
                name == *prefix
                    || name
                        .strip_prefix(prefix)
                        .is_some_and(|rest| rest.starts_with('.'))
            })
}

pub(crate) fn valid_source_path(path: &str) -> bool {
    if !is_portable_normalized_path(path) {
        return false;
    }
    let Some(relative) = path
        .strip_prefix("src/")
        .and_then(|p| p.strip_suffix(".java"))
    else {
        return false;
    };
    let Some((package, interface)) = relative.rsplit_once('/') else {
        return false;
    };
    valid_package(&package.replace('/', "."))
        && package.split('/').all(valid_identifier)
        && valid_identifier(interface)
}

pub(crate) struct MethodId<'a> {
    pub owner: &'a str,
    pub name: &'a str,
    pub parameters: Vec<VirType>,
    pub result: VirType,
}

impl MethodId<'_> {
    pub fn source_path(&self) -> String {
        format!("src/{}.java", self.owner.replace('.', "/"))
    }
}

fn scalar(token: &str) -> Option<VirType> {
    match token {
        "boolean" => Some(VirType::Bool {}),
        "int" => Some(VirType::Bv {
            width: BitVectorWidth::Bits32,
            signed: true,
        }),
        "long" => Some(VirType::Bv {
            width: BitVectorWidth::Bits64,
            signed: true,
        }),
        _ => None,
    }
}

pub(crate) fn is_integer(ty: &VirType) -> bool {
    matches!(
        ty,
        VirType::Bv {
            width: BitVectorWidth::Bits32 | BitVectorWidth::Bits64,
            signed: true
        }
    )
}

pub(crate) fn is_scalar(ty: &VirType) -> bool {
    matches!(ty, VirType::Bool {}) || is_integer(ty)
}

pub(crate) fn method_id(id: &str) -> Option<MethodId<'_>> {
    if id.is_empty() || id.len() > 1024 || !id.is_ascii() {
        return None;
    }
    let (owner, signature) = id.split_once("::")?;
    let (package, interface) = owner.rsplit_once('.')?;
    let (name, signature) = signature.split_once('(')?;
    let (parameters, result) = signature.split_once(")->")?;
    if !valid_package(package) || !valid_identifier(interface) || !valid_identifier(name) {
        return None;
    }
    let parameters = if parameters.is_empty() {
        Vec::new()
    } else {
        parameters
            .split(',')
            .map(scalar)
            .collect::<Option<Vec<_>>>()?
    };
    let slots: usize = parameters
        .iter()
        .map(|ty| match ty {
            VirType::Bv {
                width: BitVectorWidth::Bits64,
                ..
            } => 2,
            _ => 1,
        })
        .sum();
    if slots > 255 {
        return None;
    }
    Some(MethodId {
        owner,
        name,
        parameters,
        result: scalar(result)?,
    })
}

pub(crate) fn valid_selection(value: &Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    if object.len() != 4
        || !value
            .get("compilation")
            .and_then(Value::as_str)
            .is_some_and(valid_compilation)
    {
        return false;
    }
    let mut paths = Vec::new();
    for (field, maximum) in [("sources", 256), ("contracts", 128), ("methods", 32)] {
        let Some(items) = value.get(field).and_then(Value::as_array) else {
            return false;
        };
        if items.is_empty() || items.len() > maximum {
            return false;
        }
        let mut previous = "";
        for item in items {
            let Some(item) = item.as_str() else {
                return false;
            };
            let valid = match field {
                "sources" => valid_source_path(item),
                "contracts" => {
                    is_portable_normalized_path(item)
                        && item.starts_with("contracts/")
                        && item.ends_with(".json")
                }
                "methods" => method_id(item).is_some(),
                _ => false,
            };
            if !valid || previous >= item {
                return false;
            }
            previous = item;
            if field != "methods" {
                paths.push(item);
            }
        }
    }
    valid_path_inventory(paths)
}

/// Files and their implied directories share one portable, case-sensitive namespace.
pub(crate) fn valid_path_inventory<'a>(paths: impl IntoIterator<Item = &'a str>) -> bool {
    let mut entries = BTreeMap::new();
    let mut files = BTreeSet::new();
    let mut directories = BTreeSet::new();
    for path in paths {
        if !is_portable_normalized_path(path) || !files.insert(path) {
            return false;
        }
        let mut current = path;
        loop {
            if entries
                .insert(current.to_ascii_lowercase(), current)
                .is_some_and(|old| old != current)
            {
                return false;
            }
            let Some((parent, _)) = current.rsplit_once('/') else {
                break;
            };
            directories.insert(parent);
            current = parent;
        }
    }
    files.is_disjoint(&directories)
}
