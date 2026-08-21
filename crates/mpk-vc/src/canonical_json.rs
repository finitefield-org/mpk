//! Strict JSON parsing and the narrowed RFC 8785 encoding shared by VIR-era
//! helper artifacts.
//!
//! This module is untrusted helper infrastructure. It does not encode or
//! change canonical `.mpcert` bytes.

use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

/// Largest interoperable integral JSON value admitted by VIR-era schemas.
pub const MAX_SAFE_JSON_INTEGER: i64 = 9_007_199_254_740_991;

/// Smallest interoperable integral JSON value admitted by VIR-era schemas.
pub const MIN_SAFE_JSON_INTEGER: i64 = -MAX_SAFE_JSON_INTEGER;

/// Largest JSON container depth supported by the shared recursive parser.
///
/// Every VIR-era specification limit is at or below this value. Rejecting a
/// larger configured limit keeps the call stack bounded even when a caller
/// accidentally supplies an unconstrained value.
pub const MAX_SUPPORTED_JSON_DEPTH: u64 = 768;

/// Explicit limits applied while parsing an untrusted JSON value.
///
/// `max_nodes` counts JSON values, including the root and container values;
/// object member names are strings but are not additional value nodes.
/// `max_depth` counts only open arrays and objects, with a container root at
/// depth one. `max_string_bytes` counts the UTF-8 bytes of each decoded object
/// name or string value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StrictJsonLimits {
    pub max_input_bytes: u64,
    pub max_nodes: u64,
    pub max_depth: u64,
    pub max_string_bytes: u64,
}

impl StrictJsonLimits {
    pub const fn new(
        max_input_bytes: u64,
        max_nodes: u64,
        max_depth: u64,
        max_string_bytes: u64,
    ) -> Self {
        Self {
            max_input_bytes,
            max_nodes,
            max_depth,
            max_string_bytes,
        }
    }
}

/// A JSON value that preserves array order and source object-member order.
///
/// Strict parsing rejects duplicate object names before constructing this
/// value. Programmatic objects are checked again by canonical encoding so a
/// caller cannot manufacture an ambiguous object and hash it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StrictJsonValue {
    Null,
    Bool(bool),
    Integer(i64),
    String(String),
    Array(Vec<StrictJsonValue>),
    Object(Vec<(String, StrictJsonValue)>),
}

impl StrictJsonValue {
    pub fn as_array(&self) -> Option<&[StrictJsonValue]> {
        match self {
            Self::Array(values) => Some(values),
            _ => None,
        }
    }

    pub fn as_array_mut(&mut self) -> Option<&mut Vec<StrictJsonValue>> {
        match self {
            Self::Array(values) => Some(values),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&[(String, StrictJsonValue)]> {
        match self {
            Self::Object(entries) => Some(entries),
            _ => None,
        }
    }

    pub fn as_object_mut(&mut self) -> Option<&mut Vec<(String, StrictJsonValue)>> {
        match self {
            Self::Object(entries) => Some(entries),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Integer(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    pub fn get(&self, name: &str) -> Option<&StrictJsonValue> {
        self.as_object()?
            .iter()
            .find_map(|(candidate, value)| (candidate == name).then_some(value))
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut StrictJsonValue> {
        self.as_object_mut()?
            .iter_mut()
            .find_map(|(candidate, value)| (candidate == name).then_some(value))
    }

    /// Clones an object while requiring and removing each named root field
    /// exactly once.
    ///
    /// Hash-owning schemas use this instead of silently ignoring an absent or
    /// misspelled self-hash member.
    pub fn clone_without_fields(&self, fields: &[&str]) -> Result<Self, ObjectFieldsError> {
        let entries = self.as_object().ok_or(ObjectFieldsError::NotObject)?;
        let mut requested = BTreeSet::new();
        for field in fields {
            if !requested.insert(*field) {
                return Err(ObjectFieldsError::DuplicateRequest {
                    field: (*field).to_owned(),
                });
            }
            match entries
                .iter()
                .filter(|(name, _)| name == field)
                .take(2)
                .count()
            {
                0 => {
                    return Err(ObjectFieldsError::MissingField {
                        field: (*field).to_owned(),
                    });
                }
                1 => {}
                _ => {
                    return Err(ObjectFieldsError::DuplicateField {
                        field: (*field).to_owned(),
                    });
                }
            }
        }

        Ok(Self::Object(
            entries
                .iter()
                .filter(|(name, _)| !requested.contains(name.as_str()))
                .cloned()
                .collect(),
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObjectFieldsError {
    NotObject,
    MissingField { field: String },
    DuplicateField { field: String },
    DuplicateRequest { field: String },
}

impl fmt::Display for ObjectFieldsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotObject => formatter.write_str("JSON value is not an object"),
            Self::MissingField { field } => {
                write!(formatter, "JSON object is missing required field {field:?}")
            }
            Self::DuplicateField { field } => {
                write!(formatter, "JSON object contains duplicate field {field:?}")
            }
            Self::DuplicateRequest { field } => {
                write!(formatter, "field {field:?} was requested more than once")
            }
        }
    }
}

impl Error for ObjectFieldsError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StrictJsonError {
    InputBytesExceeded { maximum: u64, actual: u64 },
    NodeLimitExceeded { maximum: u64 },
    UnsupportedDepthLimit { requested: u64, maximum: u64 },
    DepthLimitExceeded { maximum: u64 },
    StringBytesExceeded { maximum: u64 },
    Bom,
    InvalidUtf8 { valid_up_to: usize },
    DuplicateObjectName { name: String },
    FloatingPointNumber { offset: usize },
    IntegerOutOfRange { offset: usize },
    InvalidJson { offset: usize },
    TrailingBytes { offset: usize },
}

impl fmt::Display for StrictJsonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputBytesExceeded { maximum, actual } => write!(
                formatter,
                "JSON input has {actual} bytes, exceeding inclusive limit {maximum}"
            ),
            Self::NodeLimitExceeded { maximum } => {
                write!(formatter, "JSON value-node limit {maximum} exceeded")
            }
            Self::UnsupportedDepthLimit { requested, maximum } => write!(
                formatter,
                "configured JSON container-depth limit {requested} exceeds supported maximum {maximum}"
            ),
            Self::DepthLimitExceeded { maximum } => {
                write!(formatter, "JSON container-depth limit {maximum} exceeded")
            }
            Self::StringBytesExceeded { maximum } => {
                write!(
                    formatter,
                    "decoded JSON string-byte limit {maximum} exceeded"
                )
            }
            Self::Bom => formatter.write_str("JSON input begins with a forbidden UTF-8 BOM"),
            Self::InvalidUtf8 { valid_up_to } => {
                write!(
                    formatter,
                    "JSON input is invalid UTF-8 at byte {valid_up_to}"
                )
            }
            Self::DuplicateObjectName { name } => {
                write!(formatter, "duplicate JSON object name {name:?}")
            }
            Self::FloatingPointNumber { offset } => {
                write!(formatter, "floating-point JSON number at byte {offset}")
            }
            Self::IntegerOutOfRange { offset } => {
                write!(
                    formatter,
                    "JSON integer outside the safe range at byte {offset}"
                )
            }
            Self::InvalidJson { offset } => {
                write!(formatter, "invalid JSON syntax at byte {offset}")
            }
            Self::TrailingBytes { offset } => {
                write!(
                    formatter,
                    "bytes after the first JSON value at byte {offset}"
                )
            }
        }
    }
}

impl Error for StrictJsonError {}

/// Parses exactly one UTF-8 JSON value under the narrowed VIR-era number and
/// Unicode rules.
///
/// Leading and trailing JSON whitespace are accepted here. Artifact transports
/// that require byte-identical JCS (and possibly one final LF) compare their
/// framing separately after parsing.
pub fn parse_strict_json(
    input: &[u8],
    limits: StrictJsonLimits,
) -> Result<StrictJsonValue, StrictJsonError> {
    if limits.max_depth > MAX_SUPPORTED_JSON_DEPTH {
        return Err(StrictJsonError::UnsupportedDepthLimit {
            requested: limits.max_depth,
            maximum: MAX_SUPPORTED_JSON_DEPTH,
        });
    }
    let actual = u64::try_from(input.len()).unwrap_or(u64::MAX);
    if actual > limits.max_input_bytes {
        return Err(StrictJsonError::InputBytesExceeded {
            maximum: limits.max_input_bytes,
            actual,
        });
    }
    if input.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(StrictJsonError::Bom);
    }
    let text = std::str::from_utf8(input).map_err(|error| StrictJsonError::InvalidUtf8 {
        valid_up_to: error.valid_up_to(),
    })?;
    let mut parser = Parser {
        text,
        offset: 0,
        limits,
        nodes: 0,
    };
    parser.skip_whitespace();
    let value = parser.parse_value(0)?;
    parser.skip_whitespace();
    if parser.offset != parser.text.len() {
        return Err(StrictJsonError::TrailingBytes {
            offset: parser.offset,
        });
    }
    Ok(value)
}

struct Parser<'a> {
    text: &'a str,
    offset: usize,
    limits: StrictJsonLimits,
    nodes: u64,
}

impl Parser<'_> {
    fn parse_value(&mut self, parent_depth: u64) -> Result<StrictJsonValue, StrictJsonError> {
        self.nodes = self
            .nodes
            .checked_add(1)
            .ok_or(StrictJsonError::NodeLimitExceeded {
                maximum: self.limits.max_nodes,
            })?;
        if self.nodes > self.limits.max_nodes {
            return Err(StrictJsonError::NodeLimitExceeded {
                maximum: self.limits.max_nodes,
            });
        }

        match self.peek_byte() {
            Some(b'n') => {
                self.consume_literal(b"null")?;
                Ok(StrictJsonValue::Null)
            }
            Some(b't') => {
                self.consume_literal(b"true")?;
                Ok(StrictJsonValue::Bool(true))
            }
            Some(b'f') => {
                self.consume_literal(b"false")?;
                Ok(StrictJsonValue::Bool(false))
            }
            Some(b'"') => self.parse_string().map(StrictJsonValue::String),
            Some(b'[') => self.parse_array(parent_depth),
            Some(b'{') => self.parse_object(parent_depth),
            Some(b'-' | b'0'..=b'9') => self.parse_integer().map(StrictJsonValue::Integer),
            _ => Err(StrictJsonError::InvalidJson {
                offset: self.offset,
            }),
        }
    }

    fn parse_array(&mut self, parent_depth: u64) -> Result<StrictJsonValue, StrictJsonError> {
        let depth = self.enter_container(parent_depth)?;
        self.offset += 1;
        self.skip_whitespace();
        let mut values = Vec::new();
        if self.consume_if(b']') {
            return Ok(StrictJsonValue::Array(values));
        }

        loop {
            values.push(self.parse_value(depth)?);
            self.skip_whitespace();
            if self.consume_if(b']') {
                break;
            }
            if !self.consume_if(b',') {
                return Err(StrictJsonError::InvalidJson {
                    offset: self.offset,
                });
            }
            self.skip_whitespace();
        }
        Ok(StrictJsonValue::Array(values))
    }

    fn parse_object(&mut self, parent_depth: u64) -> Result<StrictJsonValue, StrictJsonError> {
        let depth = self.enter_container(parent_depth)?;
        self.offset += 1;
        self.skip_whitespace();
        let mut entries = Vec::new();
        let mut names = BTreeSet::new();
        if self.consume_if(b'}') {
            return Ok(StrictJsonValue::Object(entries));
        }

        loop {
            if self.peek_byte() != Some(b'"') {
                return Err(StrictJsonError::InvalidJson {
                    offset: self.offset,
                });
            }
            let name = self.parse_string()?;
            if !names.insert(name.clone()) {
                return Err(StrictJsonError::DuplicateObjectName { name });
            }
            self.skip_whitespace();
            if !self.consume_if(b':') {
                return Err(StrictJsonError::InvalidJson {
                    offset: self.offset,
                });
            }
            self.skip_whitespace();
            let value = self.parse_value(depth)?;
            entries.push((name, value));
            self.skip_whitespace();
            if self.consume_if(b'}') {
                break;
            }
            if !self.consume_if(b',') {
                return Err(StrictJsonError::InvalidJson {
                    offset: self.offset,
                });
            }
            self.skip_whitespace();
        }
        Ok(StrictJsonValue::Object(entries))
    }

    fn parse_integer(&mut self) -> Result<i64, StrictJsonError> {
        let start = self.offset;
        self.consume_if(b'-');
        match self.peek_byte() {
            Some(b'0') => {
                self.offset += 1;
                if matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                    return Err(StrictJsonError::InvalidJson {
                        offset: self.offset,
                    });
                }
            }
            Some(b'1'..=b'9') => {
                self.offset += 1;
                while matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                    self.offset += 1;
                }
            }
            _ => {
                return Err(StrictJsonError::InvalidJson {
                    offset: self.offset,
                });
            }
        }

        if matches!(self.peek_byte(), Some(b'.' | b'e' | b'E')) {
            return Err(StrictJsonError::FloatingPointNumber { offset: start });
        }
        let value = self.text[start..self.offset]
            .parse::<i64>()
            .map_err(|_| StrictJsonError::IntegerOutOfRange { offset: start })?;
        if !(MIN_SAFE_JSON_INTEGER..=MAX_SAFE_JSON_INTEGER).contains(&value) {
            return Err(StrictJsonError::IntegerOutOfRange { offset: start });
        }
        Ok(value)
    }

    fn parse_string(&mut self) -> Result<String, StrictJsonError> {
        let opening = self.offset;
        if !self.consume_if(b'"') {
            return Err(StrictJsonError::InvalidJson { offset: opening });
        }
        let mut output = String::new();
        let mut output_bytes = 0_u64;

        loop {
            let Some(byte) = self.peek_byte() else {
                return Err(StrictJsonError::InvalidJson {
                    offset: self.offset,
                });
            };
            match byte {
                b'"' => {
                    self.offset += 1;
                    return Ok(output);
                }
                b'\\' => {
                    self.offset += 1;
                    let escape_offset = self.offset;
                    let Some(escape) = self.peek_byte() else {
                        return Err(StrictJsonError::InvalidJson {
                            offset: self.offset,
                        });
                    };
                    self.offset += 1;
                    let character = match escape {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'/' => '/',
                        b'b' => '\u{0008}',
                        b'f' => '\u{000c}',
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        b'u' => self.parse_unicode_escape(escape_offset)?,
                        _ => {
                            return Err(StrictJsonError::InvalidJson {
                                offset: escape_offset,
                            });
                        }
                    };
                    self.push_string_character(&mut output, &mut output_bytes, character)?;
                }
                0x00..=0x1f => {
                    return Err(StrictJsonError::InvalidJson {
                        offset: self.offset,
                    });
                }
                _ => {
                    let character = self.text[self.offset..].chars().next().ok_or(
                        StrictJsonError::InvalidJson {
                            offset: self.offset,
                        },
                    )?;
                    self.offset += character.len_utf8();
                    self.push_string_character(&mut output, &mut output_bytes, character)?;
                }
            }
        }
    }

    fn parse_unicode_escape(&mut self, escape_offset: usize) -> Result<char, StrictJsonError> {
        let first = self.parse_hex_quad(escape_offset)?;
        let scalar = if (0xd800..=0xdbff).contains(&first) {
            if self.peek_byte() != Some(b'\\')
                || self.text.as_bytes().get(self.offset + 1) != Some(&b'u')
            {
                return Err(StrictJsonError::InvalidJson {
                    offset: escape_offset,
                });
            }
            self.offset += 2;
            let second = self.parse_hex_quad(escape_offset)?;
            if !(0xdc00..=0xdfff).contains(&second) {
                return Err(StrictJsonError::InvalidJson {
                    offset: escape_offset,
                });
            }
            0x1_0000 + (((u32::from(first) - 0xd800) << 10) | (u32::from(second) - 0xdc00))
        } else if (0xdc00..=0xdfff).contains(&first) {
            return Err(StrictJsonError::InvalidJson {
                offset: escape_offset,
            });
        } else {
            u32::from(first)
        };
        char::from_u32(scalar).ok_or(StrictJsonError::InvalidJson {
            offset: escape_offset,
        })
    }

    fn parse_hex_quad(&mut self, error_offset: usize) -> Result<u16, StrictJsonError> {
        let end = self
            .offset
            .checked_add(4)
            .ok_or(StrictJsonError::InvalidJson {
                offset: error_offset,
            })?;
        let digits =
            self.text
                .as_bytes()
                .get(self.offset..end)
                .ok_or(StrictJsonError::InvalidJson {
                    offset: error_offset,
                })?;
        let mut value = 0_u16;
        for digit in digits {
            let nibble = match digit {
                b'0'..=b'9' => u16::from(*digit - b'0'),
                b'a'..=b'f' => u16::from(*digit - b'a' + 10),
                b'A'..=b'F' => u16::from(*digit - b'A' + 10),
                _ => {
                    return Err(StrictJsonError::InvalidJson {
                        offset: error_offset,
                    });
                }
            };
            value = (value << 4) | nibble;
        }
        self.offset = end;
        Ok(value)
    }

    fn push_string_character(
        &self,
        output: &mut String,
        output_bytes: &mut u64,
        character: char,
    ) -> Result<(), StrictJsonError> {
        *output_bytes = output_bytes
            .checked_add(character.len_utf8() as u64)
            .ok_or(StrictJsonError::StringBytesExceeded {
                maximum: self.limits.max_string_bytes,
            })?;
        if *output_bytes > self.limits.max_string_bytes {
            return Err(StrictJsonError::StringBytesExceeded {
                maximum: self.limits.max_string_bytes,
            });
        }
        output.push(character);
        Ok(())
    }

    fn consume_literal(&mut self, literal: &[u8]) -> Result<(), StrictJsonError> {
        let end = self
            .offset
            .checked_add(literal.len())
            .ok_or(StrictJsonError::InvalidJson {
                offset: self.offset,
            })?;
        if self.text.as_bytes().get(self.offset..end) != Some(literal) {
            return Err(StrictJsonError::InvalidJson {
                offset: self.offset,
            });
        }
        self.offset = end;
        Ok(())
    }

    fn enter_container(&self, parent_depth: u64) -> Result<u64, StrictJsonError> {
        let depth = parent_depth
            .checked_add(1)
            .ok_or(StrictJsonError::DepthLimitExceeded {
                maximum: self.limits.max_depth,
            })?;
        if depth > self.limits.max_depth {
            return Err(StrictJsonError::DepthLimitExceeded {
                maximum: self.limits.max_depth,
            });
        }
        Ok(depth)
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek_byte(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.offset += 1;
        }
    }

    fn consume_if(&mut self, expected: u8) -> bool {
        if self.peek_byte() == Some(expected) {
            self.offset += 1;
            true
        } else {
            false
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.text.as_bytes().get(self.offset).copied()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CanonicalJsonError {
    IntegerOutOfRange { value: i64 },
    DuplicateObjectName { name: String },
}

impl fmt::Display for CanonicalJsonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IntegerOutOfRange { value } => {
                write!(formatter, "JSON integer {value} is outside the safe range")
            }
            Self::DuplicateObjectName { name } => {
                write!(formatter, "duplicate JSON object name {name:?}")
            }
        }
    }
}

impl Error for CanonicalJsonError {}

/// Encodes a value as compact RFC 8785 JCS narrowed to safe integers.
///
/// Arrays are emitted exactly as supplied. Schema-owned unordered collections
/// must be normalized explicitly before calling this function.
pub fn canonical_json_bytes(value: &StrictJsonValue) -> Result<Vec<u8>, CanonicalJsonError> {
    let mut output = Vec::new();
    write_canonical_value(value, &mut output)?;
    Ok(output)
}

fn write_canonical_value(
    value: &StrictJsonValue,
    output: &mut Vec<u8>,
) -> Result<(), CanonicalJsonError> {
    match value {
        StrictJsonValue::Null => output.extend_from_slice(b"null"),
        StrictJsonValue::Bool(false) => output.extend_from_slice(b"false"),
        StrictJsonValue::Bool(true) => output.extend_from_slice(b"true"),
        StrictJsonValue::Integer(integer) => {
            if !(MIN_SAFE_JSON_INTEGER..=MAX_SAFE_JSON_INTEGER).contains(integer) {
                return Err(CanonicalJsonError::IntegerOutOfRange { value: *integer });
            }
            output.extend_from_slice(integer.to_string().as_bytes());
        }
        StrictJsonValue::String(string) => write_canonical_string(string, output),
        StrictJsonValue::Array(values) => {
            output.push(b'[');
            for (index, item) in values.iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                write_canonical_value(item, output)?;
            }
            output.push(b']');
        }
        StrictJsonValue::Object(entries) => {
            let mut ordered: Vec<_> = entries.iter().collect();
            ordered.sort_by(|left, right| compare_utf16_code_units(&left.0, &right.0));
            for pair in ordered.windows(2) {
                if pair[0].0 == pair[1].0 {
                    return Err(CanonicalJsonError::DuplicateObjectName {
                        name: pair[0].0.clone(),
                    });
                }
            }

            output.push(b'{');
            for (index, (name, item)) in ordered.into_iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                write_canonical_string(name, output);
                output.push(b':');
                write_canonical_value(item, output)?;
            }
            output.push(b'}');
        }
    }
    Ok(())
}

fn write_canonical_string(value: &str, output: &mut Vec<u8>) {
    const LOWER_HEX: &[u8; 16] = b"0123456789abcdef";

    output.push(b'"');
    for character in value.chars() {
        match character {
            '"' => output.extend_from_slice(b"\\\""),
            '\\' => output.extend_from_slice(b"\\\\"),
            '\u{0008}' => output.extend_from_slice(b"\\b"),
            '\t' => output.extend_from_slice(b"\\t"),
            '\n' => output.extend_from_slice(b"\\n"),
            '\u{000c}' => output.extend_from_slice(b"\\f"),
            '\r' => output.extend_from_slice(b"\\r"),
            '\u{0000}'..='\u{001f}' => {
                let code = character as u8;
                output.extend_from_slice(b"\\u00");
                output.push(LOWER_HEX[usize::from(code >> 4)]);
                output.push(LOWER_HEX[usize::from(code & 0x0f)]);
            }
            _ => {
                let mut buffer = [0_u8; 4];
                output.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
            }
        }
    }
    output.push(b'"');
}

/// RFC 8785 object-name ordering over UTF-16 code units.
pub fn compare_utf16_code_units(left: &str, right: &str) -> Ordering {
    left.encode_utf16().cmp(right.encode_utf16())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnorderedSetError {
    pub duplicate_index: usize,
}

impl fmt::Display for UnorderedSetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "unordered set contains equal entries at sorted index {}",
            self.duplicate_index
        )
    }
}

impl Error for UnorderedSetError {}

/// Explicitly normalizes a specification-owned unordered set and rejects an
/// equal adjacent pair rather than silently deduplicating it.
pub fn normalize_unordered_set_by<T, F>(
    values: &mut [T],
    mut compare: F,
) -> Result<(), UnorderedSetError>
where
    F: FnMut(&T, &T) -> Ordering,
{
    values.sort_by(|left, right| compare(left, right));
    for index in 1..values.len() {
        if compare(&values[index - 1], &values[index]) == Ordering::Equal {
            return Err(UnorderedSetError {
                duplicate_index: index,
            });
        }
    }
    Ok(())
}

/// Normalizes a specification-owned string set by raw UTF-8 bytes.
pub fn normalize_unordered_utf8_strings(values: &mut [String]) -> Result<(), UnorderedSetError> {
    normalize_unordered_set_by(values, |left, right| left.as_bytes().cmp(right.as_bytes()))
}
