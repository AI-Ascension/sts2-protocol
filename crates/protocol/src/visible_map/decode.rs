// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;

use super::model::{RuntimeMapV1Message, RuntimeMapV1Snapshot};
use super::validation::RuntimeMapV1ValidationError;
use super::{RUNTIME_MAP_V1_MAX_JSON_DEPTH, RUNTIME_MAP_V1_MAX_MESSAGE_BYTES};

/// Errors from the bounded byte decoder, including semantic validation failures.
#[derive(Debug)]
pub enum RuntimeMapV1DecodeError {
    TooLarge { actual: usize, maximum: usize },
    InvalidUtf8,
    DuplicateKey,
    MalformedJson,
    Json(serde_json::Error),
    Validation(RuntimeMapV1ValidationError),
}

impl std::fmt::Display for RuntimeMapV1DecodeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooLarge { actual, maximum } => {
                write!(
                    formatter,
                    "map message is {actual} bytes; maximum is {maximum}"
                )
            }
            Self::InvalidUtf8 => formatter.write_str("map message is not valid UTF-8"),
            Self::DuplicateKey => {
                formatter.write_str("map message contains a duplicate object key")
            }
            Self::MalformedJson => formatter.write_str("map message has malformed JSON"),
            Self::Json(error) => error.fmt(formatter),
            Self::Validation(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for RuntimeMapV1DecodeError {}

impl From<serde_json::Error> for RuntimeMapV1DecodeError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<RuntimeMapV1ValidationError> for RuntimeMapV1DecodeError {
    fn from(error: RuntimeMapV1ValidationError) -> Self {
        Self::Validation(error)
    }
}

/// Decodes, duplicate-checks, bounds-checks, and validates one complete map envelope.
pub fn decode_runtime_map_message(
    bytes: &[u8],
) -> Result<RuntimeMapV1Message, RuntimeMapV1DecodeError> {
    if bytes.len() > RUNTIME_MAP_V1_MAX_MESSAGE_BYTES {
        return Err(RuntimeMapV1DecodeError::TooLarge {
            actual: bytes.len(),
            maximum: RUNTIME_MAP_V1_MAX_MESSAGE_BYTES,
        });
    }
    std::str::from_utf8(bytes).map_err(|_| RuntimeMapV1DecodeError::InvalidUtf8)?;
    Scanner::new(bytes).scan()?;
    let message: RuntimeMapV1Message = serde_json::from_slice(bytes)?;
    message.validate()?;
    Ok(message)
}

/// Decodes one standalone `visible-map.json` snapshot with the same byte and duplicate-key
/// protections as the transport envelope.
pub fn decode_map_snapshot(bytes: &[u8]) -> Result<RuntimeMapV1Snapshot, RuntimeMapV1DecodeError> {
    if bytes.len() > RUNTIME_MAP_V1_MAX_MESSAGE_BYTES {
        return Err(RuntimeMapV1DecodeError::TooLarge {
            actual: bytes.len(),
            maximum: RUNTIME_MAP_V1_MAX_MESSAGE_BYTES,
        });
    }
    std::str::from_utf8(bytes).map_err(|_| RuntimeMapV1DecodeError::InvalidUtf8)?;
    Scanner::new(bytes).scan()?;
    let snapshot: RuntimeMapV1Snapshot = serde_json::from_slice(bytes)?;
    snapshot.validate()?;
    Ok(snapshot)
}

/// Short alias for callers that use the profile's map name.
pub fn decode_map_message(bytes: &[u8]) -> Result<RuntimeMapV1Message, RuntimeMapV1DecodeError> {
    decode_runtime_map_message(bytes)
}

struct Scanner<'a> {
    bytes: &'a [u8],
    position: usize,
    depth: usize,
}

impl<'a> Scanner<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            position: 0,
            depth: 0,
        }
    }

    fn scan(mut self) -> Result<(), RuntimeMapV1DecodeError> {
        self.value()?;
        self.whitespace();
        (self.position == self.bytes.len())
            .then_some(())
            .ok_or(RuntimeMapV1DecodeError::MalformedJson)
    }

    fn value(&mut self) -> Result<(), RuntimeMapV1DecodeError> {
        self.whitespace();
        match self.bytes.get(self.position).copied() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => self.string().map(|_| ()),
            Some(b't') => self.literal(b"true"),
            Some(b'f') => self.literal(b"false"),
            Some(b'n') => self.literal(b"null"),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err(RuntimeMapV1DecodeError::MalformedJson),
        }
    }

    fn object(&mut self) -> Result<(), RuntimeMapV1DecodeError> {
        self.enter()?;
        self.position += 1;
        let mut keys = BTreeSet::new();
        self.whitespace();
        if self.consume(b'}') {
            self.leave();
            return Ok(());
        }
        loop {
            self.whitespace();
            let key_bytes = self.string()?;
            let key: String = serde_json::from_slice(key_bytes)
                .map_err(|_| RuntimeMapV1DecodeError::MalformedJson)?;
            if !keys.insert(key) {
                return Err(RuntimeMapV1DecodeError::DuplicateKey);
            }
            self.whitespace();
            if !self.consume(b':') {
                return Err(RuntimeMapV1DecodeError::MalformedJson);
            }
            self.value()?;
            self.whitespace();
            if self.consume(b'}') {
                self.leave();
                return Ok(());
            }
            if !self.consume(b',') {
                return Err(RuntimeMapV1DecodeError::MalformedJson);
            }
        }
    }

    fn array(&mut self) -> Result<(), RuntimeMapV1DecodeError> {
        self.enter()?;
        self.position += 1;
        self.whitespace();
        if self.consume(b']') {
            self.leave();
            return Ok(());
        }
        loop {
            self.value()?;
            self.whitespace();
            if self.consume(b']') {
                self.leave();
                return Ok(());
            }
            if !self.consume(b',') {
                return Err(RuntimeMapV1DecodeError::MalformedJson);
            }
        }
    }

    fn string(&mut self) -> Result<&'a [u8], RuntimeMapV1DecodeError> {
        let start = self.position;
        if !self.consume(b'"') {
            return Err(RuntimeMapV1DecodeError::MalformedJson);
        }
        while let Some(byte) = self.bytes.get(self.position).copied() {
            match byte {
                b'"' => {
                    self.position += 1;
                    return Ok(&self.bytes[start..self.position]);
                }
                b'\\' => {
                    self.position += 1;
                    if self.position >= self.bytes.len() {
                        return Err(RuntimeMapV1DecodeError::MalformedJson);
                    }
                    self.position += 1;
                }
                0..=0x1f => return Err(RuntimeMapV1DecodeError::MalformedJson),
                _ => self.position += 1,
            }
        }
        Err(RuntimeMapV1DecodeError::MalformedJson)
    }

    fn literal(&mut self, literal: &[u8]) -> Result<(), RuntimeMapV1DecodeError> {
        let end = self.position.saturating_add(literal.len());
        if self.bytes.get(self.position..end) == Some(literal) {
            self.position = end;
            Ok(())
        } else {
            Err(RuntimeMapV1DecodeError::MalformedJson)
        }
    }

    fn number(&mut self) -> Result<(), RuntimeMapV1DecodeError> {
        let start = self.position;
        while let Some(byte) = self.bytes.get(self.position).copied() {
            if byte.is_ascii_whitespace() || matches!(byte, b',' | b']' | b'}') {
                break;
            }
            self.position += 1;
        }
        serde_json::from_slice::<serde_json::Value>(&self.bytes[start..self.position])
            .map(|_| ())
            .map_err(|_| RuntimeMapV1DecodeError::MalformedJson)
    }

    fn whitespace(&mut self) {
        while self
            .bytes
            .get(self.position)
            .is_some_and(|byte| byte.is_ascii_whitespace())
        {
            self.position += 1;
        }
    }

    fn consume(&mut self, byte: u8) -> bool {
        if self.bytes.get(self.position) == Some(&byte) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn enter(&mut self) -> Result<(), RuntimeMapV1DecodeError> {
        self.depth += 1;
        if self.depth > RUNTIME_MAP_V1_MAX_JSON_DEPTH {
            Err(RuntimeMapV1DecodeError::MalformedJson)
        } else {
            Ok(())
        }
    }

    fn leave(&mut self) {
        self.depth -= 1;
    }
}
