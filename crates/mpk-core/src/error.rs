//! Structured core errors with stable codes and deterministic JSON output.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::{reduce::ReduceError, subst::SubstError};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum CoreErrorCode {
    InvalidName,
    InvalidLevelReference,
    InvalidTermReference,
    InvalidContextReference,
    InvalidDeclaration,
    UnboundVariable,
    UnknownGlobal,
    TypeMismatch,
    NotAFunction,
    FuelExhausted,
    SubstitutionError,
    UnsupportedFeature,
    InternalInvariant,
}

impl CoreErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidName => "CORE_INVALID_NAME",
            Self::InvalidLevelReference => "CORE_INVALID_LEVEL_REFERENCE",
            Self::InvalidTermReference => "CORE_INVALID_TERM_REFERENCE",
            Self::InvalidContextReference => "CORE_INVALID_CONTEXT_REFERENCE",
            Self::InvalidDeclaration => "CORE_INVALID_DECLARATION",
            Self::UnboundVariable => "CORE_UNBOUND_VARIABLE",
            Self::UnknownGlobal => "CORE_UNKNOWN_GLOBAL",
            Self::TypeMismatch => "CORE_TYPE_MISMATCH",
            Self::NotAFunction => "CORE_NOT_A_FUNCTION",
            Self::FuelExhausted => "CORE_FUEL_EXHAUSTED",
            Self::SubstitutionError => "CORE_SUBSTITUTION_ERROR",
            Self::UnsupportedFeature => "CORE_UNSUPPORTED_FEATURE",
            Self::InternalInvariant => "CORE_INTERNAL_INVARIANT",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum CoreLocationPart {
    Field(String),
    Index(u32),
}

impl CoreLocationPart {
    pub fn field(field: impl Into<String>) -> Self {
        Self::Field(field.into())
    }

    pub fn index(index: u32) -> Self {
        Self::Index(index)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct CoreLocation {
    parts: Vec<CoreLocationPart>,
}

impl CoreLocation {
    pub fn root() -> Self {
        Self::default()
    }

    pub fn new(parts: impl IntoIterator<Item = CoreLocationPart>) -> Self {
        Self {
            parts: parts.into_iter().collect(),
        }
    }

    pub fn is_root(&self) -> bool {
        self.parts.is_empty()
    }

    pub fn parts(&self) -> &[CoreLocationPart] {
        &self.parts
    }

    pub fn push_field(&mut self, field: impl Into<String>) {
        self.parts.push(CoreLocationPart::field(field));
    }

    pub fn push_index(&mut self, index: u32) {
        self.parts.push(CoreLocationPart::index(index));
    }

    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.push_field(field);
        self
    }

    pub fn with_index(mut self, index: u32) -> Self {
        self.push_index(index);
        self
    }

    fn write_json(&self, output: &mut String) {
        output.push('[');
        for (index, part) in self.parts.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            match part {
                CoreLocationPart::Field(field) => {
                    output.push_str("{\"field\":");
                    write_json_string(output, field);
                    output.push('}');
                }
                CoreLocationPart::Index(index) => {
                    output.push_str("{\"index\":");
                    write!(output, "{index}").expect("writing to string cannot fail");
                    output.push('}');
                }
            }
        }
        output.push(']');
    }
}

impl From<CoreLocationPart> for CoreLocation {
    fn from(part: CoreLocationPart) -> Self {
        Self { parts: vec![part] }
    }
}

impl From<Vec<CoreLocationPart>> for CoreLocation {
    fn from(parts: Vec<CoreLocationPart>) -> Self {
        Self { parts }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreError {
    code: CoreErrorCode,
    location: CoreLocation,
    details: BTreeMap<String, String>,
}

impl CoreError {
    pub fn new(code: CoreErrorCode, location: impl Into<CoreLocation>) -> Self {
        Self {
            code,
            location: location.into(),
            details: BTreeMap::new(),
        }
    }

    pub fn at_root(code: CoreErrorCode) -> Self {
        Self::new(code, CoreLocation::root())
    }

    pub fn code(&self) -> CoreErrorCode {
        self.code
    }

    pub fn location(&self) -> &CoreLocation {
        &self.location
    }

    pub fn details(&self) -> &BTreeMap<String, String> {
        &self.details
    }

    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details.insert(key.into(), value.into());
        self
    }

    pub fn from_reduce_error(location: impl Into<CoreLocation>, error: ReduceError) -> Self {
        match error {
            ReduceError::FuelExhausted => Self::new(CoreErrorCode::FuelExhausted, location),
            ReduceError::Substitution(error) => Self::from_subst_error(location, error),
        }
    }

    pub fn from_subst_error(location: impl Into<CoreLocation>, error: SubstError) -> Self {
        let location = location.into();
        match error {
            SubstError::VariableIndexOverflow { index, amount } => {
                Self::new(CoreErrorCode::SubstitutionError, location)
                    .with_detail("kind", "variable_index_overflow")
                    .with_detail("index", index.to_string())
                    .with_detail("amount", amount.to_string())
            }
            SubstError::BinderDepthOverflow { depth } => {
                Self::new(CoreErrorCode::SubstitutionError, location)
                    .with_detail("kind", "binder_depth_overflow")
                    .with_detail("depth", depth.to_string())
            }
            SubstError::TargetIndexOverflow { target, depth } => {
                Self::new(CoreErrorCode::SubstitutionError, location)
                    .with_detail("kind", "target_index_overflow")
                    .with_detail("target", target.to_string())
                    .with_detail("depth", depth.to_string())
            }
        }
    }

    pub fn to_deterministic_json(&self) -> String {
        let mut output = String::new();
        output.push_str("{\"code\":");
        write_json_string(&mut output, self.code.as_str());
        output.push_str(",\"location\":");
        self.location.write_json(&mut output);
        output.push_str(",\"details\":{");
        for (index, (key, value)) in self.details.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            write_json_string(&mut output, key);
            output.push(':');
            write_json_string(&mut output, value);
        }
        output.push_str("}}");
        output
    }
}

fn write_json_string(output: &mut String, value: &str) {
    output.push('"');
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{08}' => output.push_str("\\b"),
            '\u{0c}' => output.push_str("\\f"),
            ch if ch <= '\u{1f}' => {
                write!(output, "\\u{:04x}", ch as u32).expect("writing to string cannot fail");
            }
            ch => output.push(ch),
        }
    }
    output.push('"');
}

#[cfg(test)]
mod tests {
    use crate::{
        CoreError, CoreErrorCode, CoreLocation, CoreLocationPart, ReduceError, SubstError,
    };

    #[test]
    fn serializes_error_json_with_stable_key_order() {
        let location = CoreLocation::root()
            .with_field("term_table")
            .with_index(7)
            .with_field("body");
        let error =
            CoreError::new(CoreErrorCode::UnboundVariable, location).with_detail("index", "2");

        assert_eq!(
            error.to_deterministic_json(),
            "{\"code\":\"CORE_UNBOUND_VARIABLE\",\"location\":[{\"field\":\"term_table\"},{\"index\":7},{\"field\":\"body\"}],\"details\":{\"index\":\"2\"}}"
        );
    }

    #[test]
    fn sorts_details_by_key_for_deterministic_output() {
        let error = CoreError::at_root(CoreErrorCode::TypeMismatch)
            .with_detail("expected", "Sort")
            .with_detail("actual", "Var");

        assert_eq!(
            error.to_deterministic_json(),
            "{\"code\":\"CORE_TYPE_MISMATCH\",\"location\":[],\"details\":{\"actual\":\"Var\",\"expected\":\"Sort\"}}"
        );
    }

    #[test]
    fn escapes_json_strings_deterministically() {
        let location = CoreLocation::root().with_field("field\"\\\n");
        let error = CoreError::new(CoreErrorCode::InvalidName, location)
            .with_detail("bad\tkey", "Core\u{2019}\r\n");

        assert_eq!(
            error.to_deterministic_json(),
            "{\"code\":\"CORE_INVALID_NAME\",\"location\":[{\"field\":\"field\\\"\\\\\\n\"}],\"details\":{\"bad\\tkey\":\"Core\u{2019}\\r\\n\"}}"
        );
    }

    #[test]
    fn location_parts_preserve_path_order() {
        let location = CoreLocation::new([
            CoreLocationPart::field("declarations"),
            CoreLocationPart::index(3),
            CoreLocationPart::field("type"),
        ]);

        assert!(!location.is_root());
        assert_eq!(location.parts().len(), 3);
    }

    #[test]
    fn stable_error_codes_are_exposed_as_strings() {
        assert_eq!(CoreErrorCode::FuelExhausted.as_str(), "CORE_FUEL_EXHAUSTED");
        assert_eq!(
            CoreErrorCode::SubstitutionError.as_str(),
            "CORE_SUBSTITUTION_ERROR"
        );
    }

    #[test]
    fn reduce_errors_convert_to_core_errors() {
        let error = CoreError::from_reduce_error(
            CoreLocationPart::field("whnf"),
            ReduceError::FuelExhausted,
        );

        assert_eq!(
            error.to_deterministic_json(),
            "{\"code\":\"CORE_FUEL_EXHAUSTED\",\"location\":[{\"field\":\"whnf\"}],\"details\":{}}"
        );
    }

    #[test]
    fn subst_errors_convert_to_sorted_core_error_details() {
        let error = CoreError::from_subst_error(
            CoreLocationPart::field("subst"),
            SubstError::TargetIndexOverflow {
                target: 3,
                depth: 5,
            },
        );

        assert_eq!(
            error.to_deterministic_json(),
            "{\"code\":\"CORE_SUBSTITUTION_ERROR\",\"location\":[{\"field\":\"subst\"}],\"details\":{\"depth\":\"5\",\"kind\":\"target_index_overflow\",\"target\":\"3\"}}"
        );
    }
}
