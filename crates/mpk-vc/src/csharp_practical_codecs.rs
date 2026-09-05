//! W10: one closed scalar codec relation, independent of boundary documents.
//! Formatting bounds and source commutation are obligations, not exceptions.
use super::*;
const DECIMAL_MAX: u128 = (1_u128 << 96) - 1;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodecError {
    UnknownCodec,
    UnknownRounding,
    Configuration,
    OperandType,
    OutputBound,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodecRounding {
    ToEven,
    AwayFromZero,
    ToZero,
    ToNegativeInfinity,
    ToPositiveInfinity,
}
impl CodecRounding {
    pub fn from_id(id: &str) -> Result<Self, CodecError> {
        match id {
            "ToEven" => Ok(Self::ToEven),
            "AwayFromZero" => Ok(Self::AwayFromZero),
            "ToZero" => Ok(Self::ToZero),
            "ToNegativeInfinity" => Ok(Self::ToNegativeInfinity),
            "ToPositiveInfinity" => Ok(Self::ToPositiveInfinity),
            _ => Err(CodecError::UnknownRounding),
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundaryCodec {
    id: String,
    type_id: String,
    scale: Option<u8>,
    rounding: Option<CodecRounding>,
}
impl BoundaryCodec {
    pub fn new(
        id: &str,
        type_id: &str,
        scale: Option<u8>,
        rounding: Option<&str>,
    ) -> Result<Self, CodecError> {
        let token = codec_token(id)?;
        let rounding = rounding.map(CodecRounding::from_id).transpose()?;
        if type_id != format!("mpk.csharp.value.{token}.v1") {
            return Err(CodecError::OperandType);
        }
        if id == "decimal.fixed" {
            if scale.is_none() || rounding.is_none() {
                return Err(CodecError::Configuration);
            }
        } else if scale.is_some() || rounding.is_some() {
            return Err(CodecError::Configuration);
        }
        Ok(Self {
            id: id.into(),
            type_id: type_id.into(),
            scale,
            rounding,
        })
    }
    pub fn type_id(&self) -> &str {
        &self.type_id
    }
    pub fn parse(&self, input: &[u16]) -> Result<MonomorphicValue, ParseErrorArm> {
        if input.len() > STRING_VALUE_LENGTH_MAX as usize {
            return Err(ParseErrorArm::InputBound);
        }
        if input.iter().any(|c| *c > 127) {
            return Err(ParseErrorArm::Syntax);
        }
        let bytes: Vec<u8> = input.iter().map(|c| *c as u8).collect();
        let text = std::str::from_utf8(&bytes).map_err(|_| ParseErrorArm::Syntax)?;
        let ty = self.type_id.clone();
        if let Some(token) = self.id.strip_prefix("integer.") {
            let number = parse_integer(text, token)?;
            return Ok(if token.starts_with('i') {
                MonomorphicValue::Signed {
                    type_id: ty,
                    value: number,
                }
            } else {
                MonomorphicValue::Unsigned {
                    type_id: ty,
                    value: number,
                }
            });
        }
        Ok(match self.id.as_str() {
            "duration_ticks" => MonomorphicValue::Duration {
                type_id: ty,
                ticks: parse_integer(text, "i64")?,
            },
            "unix_milliseconds" => MonomorphicValue::Instant {
                type_id: ty,
                milliseconds: parse_integer(text, "i64")?,
            },
            "decimal.normalized" | "decimal.fixed" => {
                let (negative, scale, coefficient) = parse_decimal(text, self.scale)?;
                MonomorphicValue::DecimalBits {
                    type_id: ty,
                    negative,
                    scale,
                    coefficient: coefficient.to_string(),
                }
            }
            "binary32" => {
                hex(text, 8, &[])?;
                MonomorphicValue::F32Bits {
                    type_id: ty,
                    bits: text.into(),
                }
            }
            "binary64" => {
                hex(text, 16, &[])?;
                MonomorphicValue::F64Bits {
                    type_id: ty,
                    bits: text.into(),
                }
            }
            "guid.n" | "guid.d" => {
                let d = self.id == "guid.d";
                hex(
                    text,
                    if d { 36 } else { 32 },
                    if d { &[8, 13, 18, 23] } else { &[] },
                )?;
                MonomorphicValue::Guid {
                    type_id: ty,
                    n: text.replace('-', ""),
                }
            }
            "date" => {
                shape(&bytes, 10, &[(4, b'-'), (7, b'-')])?;
                let y = digits(&bytes[0..4]);
                let m = digits(&bytes[5..7]);
                let d = digits(&bytes[8..10]);
                if y == 0 || !(1..=12).contains(&m) || d == 0 || d > month_days(y, m) {
                    return Err(ParseErrorArm::Range);
                }
                MonomorphicValue::Date {
                    type_id: ty,
                    day_number: year_days(y)
                        + (1..m).map(|month| month_days(y, month)).sum::<u32>()
                        + d
                        - 1,
                }
            }
            "time" => {
                shape(&bytes, 16, &[(2, b':'), (5, b':'), (8, b'.')])?;
                let h = digits(&bytes[..2]);
                let m = digits(&bytes[3..5]);
                let s = digits(&bytes[6..8]);
                if h > 23 || m > 59 || s > 59 {
                    return Err(ParseErrorArm::Range);
                }
                MonomorphicValue::Time {
                    type_id: ty,
                    ticks: ((u64::from(h) * 3600 + u64::from(m) * 60 + u64::from(s)) * 10_000_000
                        + u64::from(digits(&bytes[9..])))
                    .to_string(),
                }
            }
            _ => unreachable!("closed codec"),
        })
    }
    /// Produces the registered result<T,parse_error> handoff without collapsing
    /// parsing failures into source exceptions or accepting a document mapping.
    pub fn parse_typed(
        &self,
        b: &ValidatedFoundationBundle,
        r: &ValidatedClosedRootSet,
        c: &ClosedInstanceSet,
        result_id: &str,
        input: &[u16],
    ) -> Result<MonomorphicValue, CodecError> {
        let args = require_instance(c, result_id, "result").map_err(|_| CodecError::OperandType)?;
        if args != [self.type_id.as_str(), PARSE_ERROR_TYPE_ID] {
            return Err(CodecError::OperandType);
        }
        let (arm, payload) = match self.parse(input) {
            Ok(v) => ("ok", v),
            Err(error) => (
                "error",
                MonomorphicValue::ParseError {
                    type_id: PARSE_ERROR_TYPE_ID.into(),
                    arm: error,
                },
            ),
        };
        let value = MonomorphicValue::TaggedSum {
            type_id: result_id.into(),
            arm: arm.into(),
            payload: vec![payload],
        };
        validate_monomorphic_value(b, r, c, &value).map_err(|_| CodecError::OperandType)?;
        Ok(value)
    }
    pub fn format(
        &self,
        b: &ValidatedFoundationBundle,
        r: &ValidatedClosedRootSet,
        c: &ClosedInstanceSet,
        value: &MonomorphicValue,
    ) -> Result<Vec<u16>, CodecError> {
        if self.scale.is_some_and(|scale| scale > 28) {
            return Err(CodecError::Configuration);
        }
        if value.type_id() != self.type_id {
            return Err(CodecError::OperandType);
        }
        validate_monomorphic_value(b, r, c, value).map_err(|_| CodecError::OperandType)?;
        let text = match value {
            MonomorphicValue::Signed { value, .. } | MonomorphicValue::Unsigned { value, .. } => {
                value.clone()
            }
            MonomorphicValue::Duration { ticks, .. } | MonomorphicValue::Time { ticks, .. }
                if self.id != "time" =>
            {
                ticks.clone()
            }
            MonomorphicValue::Instant { milliseconds, .. } => milliseconds.clone(),
            MonomorphicValue::F32Bits { bits, .. } | MonomorphicValue::F64Bits { bits, .. } => {
                bits.clone()
            }
            MonomorphicValue::Guid { n, .. } => {
                if self.id == "guid.n" {
                    n.clone()
                } else {
                    format!(
                        "{}-{}-{}-{}-{}",
                        &n[..8],
                        &n[8..12],
                        &n[12..16],
                        &n[16..20],
                        &n[20..]
                    )
                }
            }
            MonomorphicValue::DecimalBits {
                negative,
                scale,
                coefficient,
                ..
            } => format_decimal(
                *negative,
                *scale,
                coefficient.parse().map_err(|_| CodecError::OperandType)?,
                self.scale,
                self.rounding,
            ),
            MonomorphicValue::Date { day_number, .. } => {
                let mut y = day_number / 366 + 1;
                while y < 9999 && year_days(y + 1) <= *day_number {
                    y += 1;
                }
                let mut d = day_number - year_days(y);
                let mut m = 1;
                while d >= month_days(y, m) {
                    d -= month_days(y, m);
                    m += 1;
                }
                format!("{y:04}-{m:02}-{:02}", d + 1)
            }
            MonomorphicValue::Time { ticks, .. } => {
                let n: u64 = ticks.parse().map_err(|_| CodecError::OperandType)?;
                format!(
                    "{:02}:{:02}:{:02}.{:07}",
                    n / 36_000_000_000,
                    n / 600_000_000 % 60,
                    n / 10_000_000 % 60,
                    n % 10_000_000
                )
            }
            _ => return Err(CodecError::OperandType),
        };
        if text.len() > STRING_VALUE_LENGTH_MAX as usize {
            return Err(CodecError::OutputBound);
        }
        Ok(text.encode_utf16().collect())
    }
}
fn parse_integer(text: &str, token: &str) -> Result<String, ParseErrorArm> {
    let negative = text.starts_with('-');
    let plus = text.starts_with('+');
    let body = text.strip_prefix(['-', '+']).unwrap_or(text);
    if body.is_empty()
        || !body.bytes().all(|c| c.is_ascii_digit())
        || (negative && token.starts_with('u'))
    {
        return Err(ParseErrorArm::Syntax);
    }
    if plus || (body.len() > 1 && body.starts_with('0')) || (negative && body == "0") {
        return Err(ParseErrorArm::Noncanonical);
    }
    let width: u32 = token[1..].parse().map_err(|_| ParseErrorArm::Syntax)?;
    let n: u128 = body.parse().map_err(|_| ParseErrorArm::Range)?;
    let max = if token.starts_with('u') {
        (1_u128 << width) - 1
    } else {
        (1_u128 << (width - 1)) - u128::from(!negative)
    };
    if n > max {
        return Err(ParseErrorArm::Range);
    }
    Ok(text.into())
}
fn parse_decimal(text: &str, fixed: Option<u8>) -> Result<(bool, u8, u128), ParseErrorArm> {
    let negative = text.starts_with('-');
    let plus = text.starts_with('+');
    let body = text.strip_prefix(['-', '+']).unwrap_or(text);
    let parts: Vec<_> = body.split('.').collect();
    if parts.len() > 2
        || parts
            .iter()
            .any(|s| s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(ParseErrorArm::Syntax);
    }
    let mut scale = parts.get(1).map_or(0, |s| s.len());
    if plus
        || (parts[0].len() > 1 && parts[0].starts_with('0'))
        || (fixed.is_none() && scale > 0 && body.ends_with('0'))
        || (negative && body.bytes().all(|b| b == b'0' || b == b'.'))
    {
        return Err(ParseErrorArm::Noncanonical);
    }
    if scale > 28 || fixed.is_some_and(|s| usize::from(s) != scale) {
        return Err(ParseErrorArm::ScalePrecision);
    }
    let joined = parts.concat();
    let mut coefficient = joined.trim_start_matches('0');
    if coefficient.is_empty() {
        coefficient = "0";
    }
    let max = DECIMAL_MAX.to_string();
    let excessive = |s: &str| s.len() > max.len() || (s.len() == max.len() && s > max.as_str());
    while excessive(coefficient) && scale > 0 && coefficient.ends_with('0') {
        coefficient = &coefficient[..coefficient.len() - 1];
        scale -= 1;
    }
    if excessive(coefficient) {
        return Err(ParseErrorArm::Range);
    }
    Ok((
        negative,
        scale as u8,
        coefficient.parse().map_err(|_| ParseErrorArm::Range)?,
    ))
}
fn format_decimal(
    negative: bool,
    mut scale: u8,
    mut n: u128,
    fixed: Option<u8>,
    rounding: Option<CodecRounding>,
) -> String {
    if let Some(target) = fixed {
        if target < scale {
            let divisor = 10_u128.pow(u32::from(scale - target));
            let q = n / divisor;
            let rem = n % divisor;
            let up = match rounding.expect("validated fixed codec") {
                CodecRounding::ToEven => {
                    rem > divisor / 2 || (rem == divisor / 2 && !q.is_multiple_of(2))
                }
                CodecRounding::AwayFromZero => rem >= divisor / 2,
                CodecRounding::ToZero => false,
                CodecRounding::ToNegativeInfinity => negative && rem != 0,
                CodecRounding::ToPositiveInfinity => !negative && rem != 0,
            };
            n = q + u128::from(up);
            scale = target;
        }
    } else {
        while scale > 0 && n.is_multiple_of(10) {
            n /= 10;
            scale -= 1;
        }
    }
    let mut digits = n.to_string();
    let actual = usize::from(scale);
    if actual > 0 {
        if digits.len() <= actual {
            digits = format!("{}{}", "0".repeat(actual + 1 - digits.len()), digits);
        }
        digits.insert(digits.len() - actual, '.');
    }
    if let Some(target) = fixed {
        if target > scale {
            if scale == 0 {
                digits.push('.');
            }
            digits.push_str(&"0".repeat(usize::from(target - scale)));
        }
    }
    if negative && n != 0 {
        digits.insert(0, '-');
    }
    digits
}
fn hex(text: &str, length: usize, hyphens: &[usize]) -> Result<(), ParseErrorArm> {
    if text.len() != length
        || text.bytes().enumerate().any(|(i, b)| {
            if hyphens.contains(&i) {
                b != b'-'
            } else {
                !b.is_ascii_hexdigit()
            }
        })
    {
        return Err(ParseErrorArm::Syntax);
    }
    if text.bytes().any(|b| b.is_ascii_uppercase()) {
        return Err(ParseErrorArm::Noncanonical);
    }
    Ok(())
}
fn shape(bytes: &[u8], length: usize, punctuation: &[(usize, u8)]) -> Result<(), ParseErrorArm> {
    if bytes.len() != length
        || bytes.iter().enumerate().any(|(i, b)| {
            if let Some((_, mark)) = punctuation.iter().find(|(j, _)| *j == i) {
                b != mark
            } else {
                !b.is_ascii_digit()
            }
        })
    {
        Err(ParseErrorArm::Syntax)
    } else {
        Ok(())
    }
}
fn digits(bytes: &[u8]) -> u32 {
    bytes.iter().fold(0, |n, b| n * 10 + u32::from(b - b'0'))
}
fn year_days(y: u32) -> u32 {
    let n = y - 1;
    n * 365 + n / 4 - n / 100 + n / 400
}
fn month_days(y: u32, m: u32) -> u32 {
    match m {
        2 => {
            if y.is_multiple_of(4) && (!y.is_multiple_of(100) || y.is_multiple_of(400)) {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

pub(super) fn codec_token(id: &str) -> Result<&'static str, CodecError> {
    Ok(match id {
        "binary32" => "f32",
        "binary64" => "f64",
        "date" => "date",
        "time" => "time",
        "duration_ticks" => "duration",
        "unix_milliseconds" => "instant",
        "guid.n" | "guid.d" => "guid",
        "decimal.normalized" | "decimal.fixed" => "decimal",
        "integer.i8" => "i8",
        "integer.u8" => "u8",
        "integer.i16" => "i16",
        "integer.u16" => "u16",
        "integer.i32" => "i32",
        "integer.u32" => "u32",
        "integer.i64" => "i64",
        "integer.u64" => "u64",
        _ => return Err(CodecError::UnknownCodec),
    })
}
