//! Native Go module-cache metadata shape, without root or VCS authority.
use super::super::super::GoTypeSourceBuffer;
use super::archive::folded;
use crate::canonical_json::{parse_strict_json, StrictJsonLimits, StrictJsonValue};
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::value::RawValue;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoModuleCacheInfoError {
    Source,
    Json,
    Fields,
    Time,
    Version,
}
impl fmt::Display for GoModuleCacheInfoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Go module cache info: {self:?}")
    }
}
impl std::error::Error for GoModuleCacheInfoError {}

// Retain source order and raw values: Go's time.Time.UnmarshalJSON receives the
// literal string without unescaping, even though ordinary JSON fields decode.
struct RawMembers(Vec<(String, Box<RawValue>)>);
impl<'de> Deserialize<'de> for RawMembers {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Members;
        impl<'de> Visitor<'de> for Members {
            type Value = RawMembers;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("an object")
            }
            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                let mut fields = Vec::new();
                while let Some(name) = map.next_key::<String>()? {
                    fields.push((name, map.next_value::<Box<RawValue>>()?));
                }
                Ok(RawMembers(fields))
            }
        }
        deserializer.deserialize_map(Members)
    }
}

// Go 1.25 first uses its RFC3339 fast path, then time.Parse's RFC3339
// fallback, which also accepts one-digit hours, comma fractions and zone
// offsets of exactly 24 hours or 60 minutes. The original literal is retained.
fn valid_go_time(raw: &[u8]) -> bool {
    fn number(raw: &[u8], at: &mut usize, count: usize) -> Option<u32> {
        let end = at.checked_add(count)?;
        let digits = raw.get(*at..end)?;
        if !digits.iter().all(u8::is_ascii_digit) {
            return None;
        }
        *at = end;
        Some(digits.iter().fold(0, |n, d| n * 10 + u32::from(d - b'0')))
    }
    fn literal(raw: &[u8], at: &mut usize, expected: u8) -> Option<()> {
        if raw.get(*at) != Some(&expected) {
            return None;
        }
        *at += 1;
        Some(())
    }
    fn parse(raw: &[u8]) -> Option<()> {
        let mut at = 0;
        let year = number(raw, &mut at, 4)?;
        literal(raw, &mut at, b'-')?;
        let month = number(raw, &mut at, 2)?;
        literal(raw, &mut at, b'-')?;
        let day = number(raw, &mut at, 2)?;
        literal(raw, &mut at, b'T')?;
        let count = if raw.get(at + 1).is_some_and(u8::is_ascii_digit) {
            2
        } else {
            1
        };
        let hour = number(raw, &mut at, count)?;
        literal(raw, &mut at, b':')?;
        let minute = number(raw, &mut at, 2)?;
        literal(raw, &mut at, b':')?;
        let second = number(raw, &mut at, 2)?;
        let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        let days = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if leap => 29,
            2 => 28,
            _ => return None,
        };
        if day == 0 || day > days || hour >= 24 || minute >= 60 || second >= 60 {
            return None;
        }
        if matches!(raw.get(at), Some(b'.' | b','))
            && raw.get(at + 1).is_some_and(u8::is_ascii_digit)
        {
            at += 1;
            while raw.get(at).is_some_and(u8::is_ascii_digit) {
                at += 1;
            }
        }
        match raw.get(at) {
            Some(b'Z') => at += 1,
            Some(b'+' | b'-') => {
                at += 1;
                let zone_hour = number(raw, &mut at, 2)?;
                literal(raw, &mut at, b':')?;
                let zone_minute = number(raw, &mut at, 2)?;
                if zone_hour > 24 || zone_minute > 60 {
                    return None;
                }
            }
            _ => return None,
        }
        (at == raw.len()).then_some(())
    }
    parse(raw).is_some()
}

/// Check the original metadata bytes using Go's strict JSON and struct rules.
/// Unknown fields are retained, not interpreted as VCS or source authority.
pub fn validate_go_module_cache_info(
    source: &GoTypeSourceBuffer,
    expected_version: &str,
) -> Result<(), GoModuleCacheInfoError> {
    use GoModuleCacheInfoError as E;
    if source.object().kind != "metadata" {
        return Err(E::Source);
    }
    let strict = parse_strict_json(
        source.bytes(),
        StrictJsonLimits::new(16_777_216, 16_777_216, 256, 1_048_576),
    )
    .map_err(|_| E::Json)?;
    let fields = strict.as_object().ok_or(E::Fields)?;
    let mut decoder = serde_json::Deserializer::from_slice(source.bytes());
    decoder.disable_recursion_limit();
    let raw = RawMembers::deserialize(&mut decoder).map_err(|_| E::Json)?;
    decoder.end().map_err(|_| E::Json)?;
    if fields.len() != raw.0.len() {
        return Err(E::Json);
    }
    let mut version = "";
    for ((name, value), (raw_name, literal)) in fields.iter().zip(&raw.0) {
        if name != raw_name {
            return Err(E::Json);
        }
        match folded(name).as_str() {
            "version" => match value {
                StrictJsonValue::Null => {}
                StrictJsonValue::String(text) => version = text,
                _ => return Err(E::Fields),
            },
            "time" => match value {
                StrictJsonValue::Null => {}
                StrictJsonValue::String(_) => {
                    let bytes = literal.get().as_bytes();
                    if bytes.len() < 2
                        || bytes.first() != Some(&b'"')
                        || bytes.last() != Some(&b'"')
                        || !valid_go_time(&bytes[1..bytes.len() - 1])
                    {
                        return Err(E::Time);
                    }
                }
                _ => return Err(E::Fields),
            },
            "origin" => match value {
                StrictJsonValue::Null => {}
                StrictJsonValue::Object(entries) => {
                    for (key, item) in entries {
                        if matches!(
                            folded(key).as_str(),
                            "vcs"
                                | "url"
                                | "subdir"
                                | "hash"
                                | "tagprefix"
                                | "tagsum"
                                | "ref"
                                | "reposum"
                        ) && !matches!(item, StrictJsonValue::String(_) | StrictJsonValue::Null)
                        {
                            return Err(E::Fields);
                        }
                    }
                }
                _ => return Err(E::Fields),
            },
            _ => {}
        }
    }
    if version != expected_version {
        return Err(E::Version);
    }
    Ok(())
}
