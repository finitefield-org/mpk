use std::collections::BTreeSet;
use std::fmt;

pub const MAX_NORMALIZED_PATH_BYTES: usize = 1_024;
pub const MAX_COMPONENT_BYTES: usize = 255;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PortablePath(String);

impl PortablePath {
    pub fn parse(value: &str) -> Result<Self, PortablePathError> {
        if value.len() > MAX_NORMALIZED_PATH_BYTES {
            return Err(PortablePathError::Limit);
        }
        if value.is_empty()
            || !value.is_ascii()
            || value.starts_with('/')
            || value.ends_with('/')
            || value.contains(['\\', ':'])
        {
            return Err(PortablePathError::Invalid);
        }

        for component in value.split('/') {
            if component.len() > MAX_COMPONENT_BYTES {
                return Err(PortablePathError::Limit);
            }
            if component.is_empty()
                || component == "."
                || component == ".."
                || component.ends_with('.')
                || !component
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
                || is_reserved_windows_component(component)
            {
                return Err(PortablePathError::Invalid);
            }
        }
        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn ascii_fold(&self) -> String {
        self.0.to_ascii_lowercase()
    }
}

impl fmt::Display for PortablePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PortablePathError {
    Limit,
    Invalid,
    Collision,
}

#[derive(Default)]
pub struct PortablePathSet {
    exact: BTreeSet<PortablePath>,
    folded: BTreeSet<String>,
}

impl PortablePathSet {
    pub fn insert(&mut self, path: PortablePath) -> Result<(), PortablePathError> {
        if self.exact.contains(&path) || self.folded.contains(&path.ascii_fold()) {
            return Err(PortablePathError::Collision);
        }
        self.folded.insert(path.ascii_fold());
        self.exact.insert(path);
        Ok(())
    }
}

fn is_reserved_windows_component(component: &str) -> bool {
    let stem = component
        .split_once('.')
        .map_or(component, |(stem, _extension)| stem);
    let folded = stem.to_ascii_uppercase();
    matches!(folded.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || reserved_numbered_name(&folded, "COM")
        || reserved_numbered_name(&folded, "LPT")
}

fn reserved_numbered_name(value: &str, prefix: &str) -> bool {
    value
        .strip_prefix(prefix)
        .is_some_and(|suffix| matches!(suffix.as_bytes(), [b'1'..=b'9']))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_path_accepts_the_closed_grammar() {
        let path = PortablePath::parse("contracts/math-v0_1.json").unwrap();
        assert_eq!(path.as_str(), "contracts/math-v0_1.json");
    }

    #[test]
    fn portable_path_rejects_nonportable_components() {
        for value in [
            "",
            "/x",
            "x/",
            "x//y",
            ".",
            "..",
            "x/../y",
            "x\\y",
            "x:y",
            "x y",
            "café",
            "name.",
            "CON",
            "nul.txt",
            "Com9.rs",
            "lpt1.anything",
        ] {
            assert_eq!(
                PortablePath::parse(value),
                Err(PortablePathError::Invalid),
                "{value}"
            );
        }
    }

    #[test]
    fn path_set_rejects_exact_and_ascii_fold_collisions() {
        let mut paths = PortablePathSet::default();
        paths
            .insert(PortablePath::parse("A/x.rs").unwrap())
            .unwrap();
        assert_eq!(
            paths.insert(PortablePath::parse("a/X.rs").unwrap()),
            Err(PortablePathError::Collision)
        );
    }

    #[test]
    fn path_limits_are_inclusive_and_numbered_device_prefixes_are_exact() {
        let component = "a".repeat(MAX_COMPONENT_BYTES);
        assert_eq!(PortablePath::parse(&component).unwrap().as_str(), component);
        assert_eq!(
            PortablePath::parse(&"a".repeat(MAX_COMPONENT_BYTES + 1)),
            Err(PortablePathError::Limit)
        );
        assert!(PortablePath::parse("COM10.json").is_ok());
        assert!(PortablePath::parse("LPT0.json").is_ok());
    }
}
