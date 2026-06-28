//! Canonical global names and global ID resolution.

use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Name(String);

impl Name {
    pub fn parse(input: impl AsRef<str>) -> Result<Self, NameError> {
        let input = input.as_ref();
        validate_name(input)?;
        Ok(Self(input.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn components(&self) -> impl Iterator<Item = &str> {
        self.0.split('.')
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NameError {
    EmptyName,
    EmptyComponent {
        component_index: usize,
        byte_index: usize,
    },
    NonAscii {
        byte_index: usize,
    },
    InvalidComponentStart {
        component_index: usize,
        byte_index: usize,
        byte: u8,
    },
    InvalidComponentChar {
        component_index: usize,
        byte_index: usize,
        byte: u8,
    },
}

impl NameError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::EmptyName => "EMPTY_NAME",
            Self::EmptyComponent { .. } => "EMPTY_COMPONENT",
            Self::NonAscii { .. } => "NON_ASCII",
            Self::InvalidComponentStart { .. } => "INVALID_COMPONENT_START",
            Self::InvalidComponentChar { .. } => "INVALID_COMPONENT_CHAR",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct GlobalId(u32);

impl GlobalId {
    pub fn index(self) -> usize {
        self.0 as usize
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Default)]
pub struct NameResolver {
    names: Vec<Name>,
    ids_by_name: HashMap<Name, GlobalId>,
}

impl NameResolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    pub fn register(&mut self, raw: impl AsRef<str>) -> Result<GlobalId, NameError> {
        let name = Name::parse(raw)?;
        Ok(self.register_name(name))
    }

    pub fn register_name(&mut self, name: Name) -> GlobalId {
        if let Some(id) = self.ids_by_name.get(&name) {
            return *id;
        }

        let index = u32::try_from(self.names.len()).expect("global name table exceeded u32 ids");
        let id = GlobalId(index);
        self.names.push(name.clone());
        self.ids_by_name.insert(name, id);
        id
    }

    pub fn resolve(&self, raw: impl AsRef<str>) -> Result<Option<GlobalId>, NameError> {
        let name = Name::parse(raw)?;
        Ok(self.resolve_name(&name))
    }

    pub fn resolve_name(&self, name: &Name) -> Option<GlobalId> {
        self.ids_by_name.get(name).copied()
    }

    pub fn name(&self, id: GlobalId) -> Option<&Name> {
        self.names.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (GlobalId, &Name)> {
        self.names
            .iter()
            .enumerate()
            .map(|(index, name)| (GlobalId(index as u32), name))
    }
}

fn validate_name(input: &str) -> Result<(), NameError> {
    if input.is_empty() {
        return Err(NameError::EmptyName);
    }

    let bytes = input.as_bytes();
    let mut component_index = 0;
    let mut component_start = 0;

    for (byte_index, byte) in bytes.iter().copied().enumerate() {
        if byte == b'.' {
            validate_component(bytes, component_index, component_start, byte_index)?;
            component_index += 1;
            component_start = byte_index + 1;
        } else if !byte.is_ascii() {
            return Err(NameError::NonAscii { byte_index });
        }
    }

    validate_component(bytes, component_index, component_start, bytes.len())
}

fn validate_component(
    bytes: &[u8],
    component_index: usize,
    start: usize,
    end: usize,
) -> Result<(), NameError> {
    if start == end {
        return Err(NameError::EmptyComponent {
            component_index,
            byte_index: start,
        });
    }

    let first = bytes[start];
    if !is_component_start(first) {
        return Err(NameError::InvalidComponentStart {
            component_index,
            byte_index: start,
            byte: first,
        });
    }

    for (byte_index, byte) in bytes.iter().copied().enumerate().take(end).skip(start + 1) {
        if !is_component_continue(byte) {
            return Err(NameError::InvalidComponentChar {
                component_index,
                byte_index,
                byte,
            });
        }
    }

    Ok(())
}

fn is_component_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_component_continue(byte: u8) -> bool {
    is_component_start(byte) || byte.is_ascii_digit() || byte == b'\''
}

#[cfg(test)]
mod tests {
    use crate::{Name, NameError, NameResolver};

    #[test]
    fn parses_canonical_dotted_names() {
        let name = Name::parse("Std.Bool.true'").expect("valid name");
        let components: Vec<_> = name.components().collect();

        assert_eq!(name.as_str(), "Std.Bool.true'");
        assert_eq!(components, ["Std", "Bool", "true'"]);
    }

    #[test]
    fn rejects_empty_names_and_components() {
        assert_eq!(Name::parse("").unwrap_err(), NameError::EmptyName);
        assert_eq!(
            Name::parse(".Core").unwrap_err(),
            NameError::EmptyComponent {
                component_index: 0,
                byte_index: 0,
            }
        );
        assert_eq!(
            Name::parse("Core.").unwrap_err(),
            NameError::EmptyComponent {
                component_index: 1,
                byte_index: 5,
            }
        );
        assert_eq!(
            Name::parse("Core..id").unwrap_err(),
            NameError::EmptyComponent {
                component_index: 1,
                byte_index: 5,
            }
        );
    }

    #[test]
    fn rejects_invalid_component_starts_and_chars() {
        assert_eq!(
            Name::parse("9Core").unwrap_err(),
            NameError::InvalidComponentStart {
                component_index: 0,
                byte_index: 0,
                byte: b'9',
            }
        );
        assert_eq!(
            Name::parse("Core.+").unwrap_err(),
            NameError::InvalidComponentStart {
                component_index: 1,
                byte_index: 5,
                byte: b'+',
            }
        );
        assert_eq!(
            Name::parse("Core.id-name").unwrap_err(),
            NameError::InvalidComponentChar {
                component_index: 1,
                byte_index: 7,
                byte: b'-',
            }
        );
    }

    #[test]
    fn rejects_non_ascii_names() {
        assert_eq!(
            Name::parse("Core\u{2019}id").unwrap_err(),
            NameError::NonAscii { byte_index: 4 }
        );
        assert_eq!(
            Name::parse("Core\u{2019}id").unwrap_err().code(),
            "NON_ASCII"
        );
    }

    #[test]
    fn resolver_reuses_existing_global_ids() {
        let mut resolver = NameResolver::new();

        let first = resolver.register("Core.Id").expect("valid name");
        let second = resolver.register("Core.Id").expect("valid name");
        let other = resolver.register("Core.Const").expect("valid name");

        assert_eq!(first, second);
        assert_ne!(first, other);
        assert_eq!(resolver.len(), 2);
        assert_eq!(resolver.name(first).unwrap().as_str(), "Core.Id");
        assert_eq!(resolver.resolve("Core.Id").unwrap(), Some(first));
    }

    #[test]
    fn resolver_rejects_invalid_lookup_names() {
        let resolver = NameResolver::new();

        assert_eq!(
            resolver.resolve("Core.").unwrap_err(),
            NameError::EmptyComponent {
                component_index: 1,
                byte_index: 5,
            }
        );
    }
}
