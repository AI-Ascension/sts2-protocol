// SPDX-License-Identifier: MIT

//! Bounded strict parser for the restricted `asc-jcs-state-v1` profile.

use std::collections::BTreeMap;

use super::{
    CanonicalError, CanonicalValue, MAX_CANONICAL_BYTES, MAX_DEPTH, MAX_INPUT_BYTES,
    MAX_SAFE_INTEGER,
};

/// Parses raw UTF-8 bytes without requiring canonical input.
pub(super) fn parse_raw(input: &[u8]) -> Result<CanonicalValue, CanonicalError> {
    if input.len() > MAX_INPUT_BYTES {
        return Err(CanonicalError::InputBytes);
    }
    let text = std::str::from_utf8(input).map_err(|_| CanonicalError::Utf8)?;
    if text.starts_with('\u{feff}') {
        return Err(CanonicalError::Bom);
    }
    precheck_nesting(text.as_bytes())?;
    let mut parser = Parser { text, pos: 0 };
    let value = parser.parse_value(0)?;
    parser.skip_whitespace();
    if parser.pos != text.len() {
        return Err(CanonicalError::TrailingData);
    }
    Ok(value)
}

/// Parses input that must already equal its own canonical encoding.
pub(super) fn parse_canonical(input: &[u8]) -> Result<CanonicalValue, CanonicalError> {
    let value = parse_raw(input)?;
    if value.to_canonical_bytes()? != input {
        return Err(CanonicalError::NonCanonical);
    }
    Ok(value)
}

/// Rejects excessive nesting before recursion, without parsing string contents.
fn precheck_nesting(bytes: &[u8]) -> Result<(), CanonicalError> {
    let mut depth = 0_usize;
    let mut quoted = false;
    let mut escaped = false;
    for byte in bytes {
        if quoted {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                quoted = false;
            }
            continue;
        }
        match byte {
            b'"' => quoted = true,
            b'{' | b'[' => {
                depth += 1;
                if depth > MAX_DEPTH {
                    return Err(CanonicalError::Depth);
                }
            }
            b'}' | b']' => {
                depth = depth.checked_sub(1).ok_or(CanonicalError::Syntax)?;
            }
            _ => {}
        }
    }
    Ok(())
}

struct Parser<'a> {
    text: &'a str,
    pos: usize,
}

impl Parser<'_> {
    fn byte(&self) -> Option<u8> {
        self.text.as_bytes().get(self.pos).copied()
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.byte(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    fn expect(&mut self, expected: u8) -> Result<(), CanonicalError> {
        if self.byte() == Some(expected) {
            self.pos += 1;
            Ok(())
        } else {
            Err(CanonicalError::Syntax)
        }
    }

    fn literal(&mut self, word: &[u8]) -> Result<(), CanonicalError> {
        if self.text.as_bytes()[self.pos..].starts_with(word) {
            self.pos += word.len();
            Ok(())
        } else {
            Err(CanonicalError::Syntax)
        }
    }

    fn parse_value(&mut self, depth: usize) -> Result<CanonicalValue, CanonicalError> {
        self.skip_whitespace();
        match self.byte().ok_or(CanonicalError::Syntax)? {
            b'{' => self.parse_object(depth),
            b'[' => self.parse_array(depth),
            b'"' => Ok(CanonicalValue::String(self.parse_string()?)),
            b't' => {
                self.literal(b"true")?;
                Ok(CanonicalValue::Bool(true))
            }
            b'f' => {
                self.literal(b"false")?;
                Ok(CanonicalValue::Bool(false))
            }
            b'n' => {
                self.literal(b"null")?;
                Ok(CanonicalValue::Null)
            }
            b'-' | b'0'..=b'9' => self.parse_integer(),
            _ => Err(CanonicalError::Syntax),
        }
    }

    fn parse_object(&mut self, depth: usize) -> Result<CanonicalValue, CanonicalError> {
        if depth >= MAX_DEPTH {
            return Err(CanonicalError::Depth);
        }
        self.expect(b'{')?;
        let mut entries = BTreeMap::new();
        self.skip_whitespace();
        if self.byte() == Some(b'}') {
            self.pos += 1;
            return Ok(CanonicalValue::Object(entries));
        }
        loop {
            self.skip_whitespace();
            if self.byte() != Some(b'"') {
                return Err(CanonicalError::Syntax);
            }
            let key = self.parse_string()?;
            validate_key(&key)?;
            self.skip_whitespace();
            self.expect(b':')?;
            let value = self.parse_value(depth + 1)?;
            if entries.insert(key, value).is_some() {
                return Err(CanonicalError::DuplicateKey);
            }
            self.skip_whitespace();
            match self.byte() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(CanonicalValue::Object(entries));
                }
                _ => return Err(CanonicalError::Syntax),
            }
        }
    }

    fn parse_array(&mut self, depth: usize) -> Result<CanonicalValue, CanonicalError> {
        if depth >= MAX_DEPTH {
            return Err(CanonicalError::Depth);
        }
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.skip_whitespace();
        if self.byte() == Some(b']') {
            self.pos += 1;
            return Ok(CanonicalValue::Array(items));
        }
        loop {
            items.push(self.parse_value(depth + 1)?);
            self.skip_whitespace();
            match self.byte() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    return Ok(CanonicalValue::Array(items));
                }
                _ => return Err(CanonicalError::Syntax),
            }
        }
    }

    fn parse_string(&mut self) -> Result<String, CanonicalError> {
        self.expect(b'"')?;
        let mut out = String::new();
        loop {
            match self.byte() {
                None => return Err(CanonicalError::Syntax),
                Some(b'"') => {
                    self.pos += 1;
                    return Ok(out);
                }
                Some(b'\\') => {
                    self.pos += 1;
                    self.parse_escape(&mut out)?;
                }
                Some(control) if control < 0x20 => return Err(CanonicalError::Syntax),
                Some(_) => {
                    let character = self.text[self.pos..]
                        .chars()
                        .next()
                        .ok_or(CanonicalError::Syntax)?;
                    out.push(character);
                    self.pos += character.len_utf8();
                }
            }
            if out.len() > MAX_CANONICAL_BYTES {
                return Err(CanonicalError::CanonicalBytes);
            }
        }
    }

    fn parse_escape(&mut self, out: &mut String) -> Result<(), CanonicalError> {
        let escape = self.byte().ok_or(CanonicalError::Syntax)?;
        self.pos += 1;
        match escape {
            b'"' => out.push('"'),
            b'\\' => out.push('\\'),
            b'/' => out.push('/'),
            b'b' => out.push('\u{8}'),
            b'f' => out.push('\u{c}'),
            b'n' => out.push('\n'),
            b'r' => out.push('\r'),
            b't' => out.push('\t'),
            b'u' => {
                let unit = self.parse_hex4()?;
                let scalar = if (0xd800..0xdc00).contains(&unit) {
                    let low = self.parse_low_surrogate()?;
                    let combined =
                        0x10000 + ((u32::from(unit) - 0xd800) << 10) + (u32::from(low) - 0xdc00);
                    char::from_u32(combined).ok_or(CanonicalError::Syntax)?
                } else if (0xdc00..0xe000).contains(&unit) {
                    return Err(CanonicalError::Syntax);
                } else {
                    char::from_u32(u32::from(unit)).ok_or(CanonicalError::Syntax)?
                };
                out.push(scalar);
            }
            _ => return Err(CanonicalError::Syntax),
        }
        Ok(())
    }

    fn parse_low_surrogate(&mut self) -> Result<u16, CanonicalError> {
        self.expect(b'\\')?;
        self.expect(b'u')?;
        let unit = self.parse_hex4()?;
        if (0xdc00..0xe000).contains(&unit) {
            Ok(unit)
        } else {
            Err(CanonicalError::Syntax)
        }
    }

    fn parse_hex4(&mut self) -> Result<u16, CanonicalError> {
        let end = self.pos.checked_add(4).ok_or(CanonicalError::Syntax)?;
        let digits = self.text.get(self.pos..end).ok_or(CanonicalError::Syntax)?;
        let unit = u16::from_str_radix(digits, 16).map_err(|_| CanonicalError::Syntax)?;
        self.pos = end;
        Ok(unit)
    }

    fn parse_integer(&mut self) -> Result<CanonicalValue, CanonicalError> {
        let start = self.pos;
        if self.byte() == Some(b'-') {
            self.pos += 1;
        }
        let digits_start = self.pos;
        while matches!(self.byte(), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
        if matches!(self.byte(), Some(b'.' | b'e' | b'E')) {
            return Err(CanonicalError::NumberToken);
        }
        let digits_len = self.pos - digits_start;
        if digits_len == 0 {
            return Err(CanonicalError::Syntax);
        }
        let token = &self.text[start..self.pos];
        let digits = &self.text[digits_start..self.pos];
        if digits.len() > 1 && digits.starts_with('0') {
            return Err(CanonicalError::NumberToken);
        }
        if token == "-0" {
            return Err(CanonicalError::NumberToken);
        }
        if digits_len > 16 {
            return Err(CanonicalError::IntegerRange);
        }
        let number: i64 = token.parse().map_err(|_| CanonicalError::IntegerRange)?;
        if !(-MAX_SAFE_INTEGER..=MAX_SAFE_INTEGER).contains(&number) {
            return Err(CanonicalError::IntegerRange);
        }
        Ok(CanonicalValue::Int(number))
    }
}

/// Requires an ASCII schema field name: `[a-z][a-z0-9_]*`.
fn validate_key(key: &str) -> Result<(), CanonicalError> {
    let mut characters = key.chars();
    if !matches!(characters.next(), Some(first) if first.is_ascii_lowercase()) {
        return Err(CanonicalError::Key);
    }
    for character in characters {
        if !(character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_') {
            return Err(CanonicalError::Key);
        }
    }
    Ok(())
}
