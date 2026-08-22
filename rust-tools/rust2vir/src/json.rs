use std::collections::BTreeMap;

const JSON_DEPTH_MAX: usize = 64;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum JsonValue {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
}

impl JsonValue {
    pub(crate) fn as_object(&self) -> Option<&BTreeMap<String, JsonValue>> {
        match self {
            Self::Object(value) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn as_array(&self) -> Option<&[JsonValue]> {
        match self {
            Self::Array(value) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    pub(crate) fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub(crate) fn integer(&self) -> Option<i64> {
        match self {
            Self::Number(value) if !value.contains(['.', 'e', 'E']) && value != "-0" => {
                value.parse().ok()
            }
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct JsonError;

pub(crate) fn parse(bytes: &[u8], maximum: usize) -> Result<JsonValue, JsonError> {
    if bytes.len() > maximum {
        return Err(JsonError);
    }
    let source = std::str::from_utf8(bytes).map_err(|_| JsonError)?;
    let mut parser = Parser { source, cursor: 0 };
    parser.skip_whitespace();
    let value = parser.value(0)?;
    parser.skip_whitespace();
    if parser.cursor != source.len() {
        return Err(JsonError);
    }
    Ok(value)
}

struct Parser<'a> {
    source: &'a str,
    cursor: usize,
}

impl Parser<'_> {
    fn value(&mut self, depth: usize) -> Result<JsonValue, JsonError> {
        if depth > JSON_DEPTH_MAX {
            return Err(JsonError);
        }
        match self.peek() {
            Some(b'n') => {
                self.literal("null")?;
                Ok(JsonValue::Null)
            }
            Some(b't') => {
                self.literal("true")?;
                Ok(JsonValue::Bool(true))
            }
            Some(b'f') => {
                self.literal("false")?;
                Ok(JsonValue::Bool(false))
            }
            Some(b'"') => Ok(JsonValue::String(self.string()?)),
            Some(b'[') => self.array(depth + 1),
            Some(b'{') => self.object(depth + 1),
            Some(b'-' | b'0'..=b'9') => Ok(JsonValue::Number(self.number()?)),
            _ => Err(JsonError),
        }
    }

    fn array(&mut self, depth: usize) -> Result<JsonValue, JsonError> {
        self.consume(b'[')?;
        self.skip_whitespace();
        let mut values = Vec::new();
        if self.take_if(b']') {
            return Ok(JsonValue::Array(values));
        }
        loop {
            values.push(self.value(depth)?);
            self.skip_whitespace();
            if self.take_if(b']') {
                break;
            }
            self.consume(b',')?;
            self.skip_whitespace();
        }
        Ok(JsonValue::Array(values))
    }

    fn object(&mut self, depth: usize) -> Result<JsonValue, JsonError> {
        self.consume(b'{')?;
        self.skip_whitespace();
        let mut values = BTreeMap::new();
        if self.take_if(b'}') {
            return Ok(JsonValue::Object(values));
        }
        loop {
            if self.peek() != Some(b'"') {
                return Err(JsonError);
            }
            let key = self.string()?;
            self.skip_whitespace();
            self.consume(b':')?;
            self.skip_whitespace();
            if values.insert(key, self.value(depth)?).is_some() {
                return Err(JsonError);
            }
            self.skip_whitespace();
            if self.take_if(b'}') {
                break;
            }
            self.consume(b',')?;
            self.skip_whitespace();
        }
        Ok(JsonValue::Object(values))
    }

    fn string(&mut self) -> Result<String, JsonError> {
        self.consume(b'"')?;
        let mut output = String::new();
        loop {
            let rest = self.source.get(self.cursor..).ok_or(JsonError)?;
            let character = rest.chars().next().ok_or(JsonError)?;
            self.cursor += character.len_utf8();
            match character {
                '"' => return Ok(output),
                '\\' => self.escape(&mut output)?,
                '\u{0}'..='\u{1f}' => return Err(JsonError),
                character => output.push(character),
            }
        }
    }

    fn escape(&mut self, output: &mut String) -> Result<(), JsonError> {
        match self.next_byte().ok_or(JsonError)? {
            b'"' => output.push('"'),
            b'\\' => output.push('\\'),
            b'/' => output.push('/'),
            b'b' => output.push('\u{8}'),
            b'f' => output.push('\u{c}'),
            b'n' => output.push('\n'),
            b'r' => output.push('\r'),
            b't' => output.push('\t'),
            b'u' => {
                let first = self.hex_quad()?;
                let scalar = if (0xd800..=0xdbff).contains(&first) {
                    if self.next_byte() != Some(b'\\') || self.next_byte() != Some(b'u') {
                        return Err(JsonError);
                    }
                    let second = self.hex_quad()?;
                    if !(0xdc00..=0xdfff).contains(&second) {
                        return Err(JsonError);
                    }
                    0x1_0000 + ((u32::from(first) - 0xd800) << 10) + (u32::from(second) - 0xdc00)
                } else if (0xdc00..=0xdfff).contains(&first) {
                    return Err(JsonError);
                } else {
                    u32::from(first)
                };
                output.push(char::from_u32(scalar).ok_or(JsonError)?);
            }
            _ => return Err(JsonError),
        }
        Ok(())
    }

    fn hex_quad(&mut self) -> Result<u16, JsonError> {
        let bytes = self
            .source
            .as_bytes()
            .get(self.cursor..self.cursor.saturating_add(4))
            .ok_or(JsonError)?;
        if bytes.len() != 4 {
            return Err(JsonError);
        }
        let mut value = 0_u16;
        for byte in bytes {
            let digit = match byte {
                b'0'..=b'9' => u16::from(byte - b'0'),
                b'a'..=b'f' => u16::from(byte - b'a' + 10),
                b'A'..=b'F' => u16::from(byte - b'A' + 10),
                _ => return Err(JsonError),
            };
            value = (value << 4) | digit;
        }
        self.cursor += 4;
        Ok(value)
    }

    fn number(&mut self) -> Result<String, JsonError> {
        let start = self.cursor;
        self.take_if(b'-');
        match self.peek() {
            Some(b'0') => {
                self.cursor += 1;
                if matches!(self.peek(), Some(b'0'..=b'9')) {
                    return Err(JsonError);
                }
            }
            Some(b'1'..=b'9') => {
                self.cursor += 1;
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.cursor += 1;
                }
            }
            _ => return Err(JsonError),
        }
        if self.take_if(b'.') {
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(JsonError);
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.cursor += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.cursor += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.cursor += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(JsonError);
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.cursor += 1;
            }
        }
        Ok(self.source[start..self.cursor].to_owned())
    }

    fn literal(&mut self, value: &str) -> Result<(), JsonError> {
        if self.source[self.cursor..].starts_with(value) {
            self.cursor += value.len();
            Ok(())
        } else {
            Err(JsonError)
        }
    }

    fn consume(&mut self, expected: u8) -> Result<(), JsonError> {
        if self.take_if(expected) {
            Ok(())
        } else {
            Err(JsonError)
        }
    }

    fn take_if(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.cursor += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.source.as_bytes().get(self.cursor).copied()
    }

    fn next_byte(&mut self) -> Option<u8> {
        let value = self.peek()?;
        self.cursor += 1;
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_json_handles_unicode_and_all_value_shapes() {
        let value = parse(
            br#"{"array":[null,true,false,-12,3.5e2,"\uD83D\uDE80"]}"#,
            1_024,
        )
        .unwrap();
        let object = value.as_object().unwrap();
        let array = object["array"].as_array().unwrap();
        assert_eq!(array[3].integer(), Some(-12));
        assert_eq!(array[5].as_str(), Some("🚀"));
    }

    #[test]
    fn malformed_duplicate_deep_and_oversized_json_reject() {
        for value in [
            br#"{"a":1,"a":2}"#.as_slice(),
            br#"{"a":01}"#.as_slice(),
            br#""\uD800""#.as_slice(),
            br#"[1,]"#.as_slice(),
        ] {
            assert_eq!(parse(value, 1_024), Err(JsonError));
        }
        let deep = format!("{}0{}", "[".repeat(66), "]".repeat(66));
        assert_eq!(parse(deep.as_bytes(), 1_024), Err(JsonError));
        assert_eq!(parse(b"null", 3), Err(JsonError));
    }
}
