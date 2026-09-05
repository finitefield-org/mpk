//! Concrete ordinal UTF-16 relations. Source evaluation order is retained in
//! the Roslyn handoff; resource/profile obligations never become exceptions.
use super::*;
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum StringOperand {
    Text { utf16: Option<Vec<u16>> },
    Char { utf16: u16 },
    Index { value: i32 },
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StringError {
    Signature,
    InputBound,
    OutputBound,
    NullReceiver,
    NullArgument,
    IndexOutOfRange,
    ArgumentOutOfRange,
}
impl StringError {
    pub fn exception_type(&self) -> Option<&'static str> {
        match self {
            Self::NullReceiver => Some("System.NullReferenceException"),
            Self::NullArgument => Some("System.ArgumentNullException"),
            Self::IndexOutOfRange => Some("System.IndexOutOfRangeException"),
            Self::ArgumentOutOfRange => Some("System.ArgumentOutOfRangeException"),
            _ => None,
        }
    }
}
fn text(value: &StringOperand) -> Result<Option<&[u16]>, StringError> {
    match value {
        StringOperand::Text { utf16 } => Ok(utf16.as_deref()),
        _ => Err(StringError::Signature),
    }
}
fn index(value: &StringOperand) -> Result<i32, StringError> {
    match value {
        StringOperand::Index { value } => Ok(*value),
        _ => Err(StringError::Signature),
    }
}
fn boolean(value: bool) -> MonomorphicValue {
    MonomorphicValue::Bool {
        type_id: "mpk.csharp.value.bool.v1".into(),
        value,
    }
}
fn integer(value: i32) -> MonomorphicValue {
    MonomorphicValue::Signed {
        type_id: "mpk.csharp.value.i32.v1".into(),
        value: value.to_string(),
    }
}
fn output(utf16: Vec<u16>) -> Result<MonomorphicValue, StringError> {
    if utf16.len() > STRING_VALUE_LENGTH_MAX as usize {
        Err(StringError::OutputBound)
    } else {
        Ok(MonomorphicValue::String {
            type_id: STRING_TYPE_ID.into(),
            utf16,
        })
    }
}
/// `instance_equals` distinguishes the exact null-receiver path of instance
/// String.Equals from the null-accepting static overload and equality operator.
pub fn evaluate_string_operation(
    id: &str,
    operands: &[StringOperand],
    instance_equals: bool,
) -> Result<MonomorphicValue, StringError> {
    if instance_equals && id != "string.equals.ordinal" {
        return Err(StringError::Signature);
    }
    for value in operands {
        if let StringOperand::Text { utf16: Some(units) } = value {
            if units.len() > STRING_VALUE_LENGTH_MAX as usize {
                return Err(StringError::InputBound);
            }
        }
    }
    if id.starts_with("string.concat.") || id == "string.interpolation.restricted" {
        let legal = match id {
            "string.concat.operator.string_string" | "string.concat.string2" => {
                operands.len() == 2
                    && operands
                        .iter()
                        .all(|x| matches!(x, StringOperand::Text { .. }))
            }
            "string.concat.string3" => {
                operands.len() == 3
                    && operands
                        .iter()
                        .all(|x| matches!(x, StringOperand::Text { .. }))
            }
            "string.concat.string4" => {
                operands.len() == 4
                    && operands
                        .iter()
                        .all(|x| matches!(x, StringOperand::Text { .. }))
            }
            "string.concat.operator.char_string" => matches!(
                operands,
                [StringOperand::Char { .. }, StringOperand::Text { .. }]
            ),
            "string.concat.operator.string_char" => matches!(
                operands,
                [StringOperand::Text { .. }, StringOperand::Char { .. }]
            ),
            "string.interpolation.restricted" => operands
                .iter()
                .all(|x| !matches!(x, StringOperand::Index { .. })),
            _ => false,
        };
        if !legal {
            return Err(StringError::Signature);
        }
        let mut result = Vec::new();
        for operand in operands {
            match operand {
                StringOperand::Text { utf16: Some(units) } => result.extend_from_slice(units),
                StringOperand::Char { utf16 } => result.push(*utf16),
                _ => {}
            }
            if result.len() > STRING_VALUE_LENGTH_MAX as usize {
                return Err(StringError::OutputBound);
            }
        }
        return output(result);
    }
    match (id, operands) {
        ("string.literal.decode", [value]) => {
            output(text(value)?.ok_or(StringError::Signature)?.to_vec())
        }
        ("string.is_null_or_empty", [value]) => {
            Ok(boolean(text(value)?.is_none_or(|s| s.is_empty())))
        }
        ("string.length", [value]) => Ok(integer(
            text(value)?.ok_or(StringError::NullReceiver)?.len() as i32,
        )),
        ("string.index", [value, offset]) => {
            let s = text(value)?.ok_or(StringError::NullReceiver)?;
            let i = index(offset)?;
            let unit = usize::try_from(i)
                .ok()
                .and_then(|i| s.get(i))
                .ok_or(StringError::IndexOutOfRange)?;
            Ok(MonomorphicValue::Char {
                type_id: "mpk.csharp.value.char.v1".into(),
                utf16: *unit,
            })
        }
        ("string.substring.start_length", [value, start, length]) => {
            let s = text(value)?.ok_or(StringError::NullReceiver)?;
            let start =
                usize::try_from(index(start)?).map_err(|_| StringError::ArgumentOutOfRange)?;
            let length =
                usize::try_from(index(length)?).map_err(|_| StringError::ArgumentOutOfRange)?;
            let end = start
                .checked_add(length)
                .ok_or(StringError::ArgumentOutOfRange)?;
            output(
                s.get(start..end)
                    .ok_or(StringError::ArgumentOutOfRange)?
                    .to_vec(),
            )
        }
        (
            "string.equality.operator"
            | "string.inequality.operator"
            | "string.equals.ordinal"
            | "string.compare.ordinal",
            [left, right],
        ) => {
            let a = text(left)?;
            let b = text(right)?;
            if instance_equals && a.is_none() {
                return Err(StringError::NullReceiver);
            }
            if id == "string.compare.ordinal" {
                return Ok(integer(match (a, b) {
                    (None, None) => 0,
                    (None, Some(_)) => -1,
                    (Some(_), None) => 1,
                    (Some(a), Some(b)) => a
                        .iter()
                        .zip(b)
                        .find_map(|(x, y)| (x != y).then_some(i32::from(*x) - i32::from(*y)))
                        .unwrap_or(a.len() as i32 - b.len() as i32),
                }));
            }
            Ok(boolean((a == b) ^ (id == "string.inequality.operator")))
        }
        (
            "string.contains.ordinal" | "string.starts_with.ordinal" | "string.ends_with.ordinal",
            [receiver, argument],
        ) => {
            let a = text(receiver)?.ok_or(StringError::NullReceiver)?;
            let b = text(argument)?.ok_or(StringError::NullArgument)?;
            Ok(boolean(match id {
                "string.starts_with.ordinal" => a.starts_with(b),
                "string.ends_with.ordinal" => a.ends_with(b),
                _ => b.is_empty() || a.windows(b.len()).any(|w| w == b),
            }))
        }
        _ => Err(StringError::Signature),
    }
}
