// SPDX-License-Identifier: MIT

use std::fmt;
use std::io::{self, Write};

use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

pub const GAME_INFORMATION_CONTENT_MANIFEST_V1_PROTOCOL_VERSION: &str =
    "game-information-content-manifest-v1";
pub const GAME_INFORMATION_CONTENT_MANIFEST_V1_ARTIFACT: &str =
    "sts2-protocol/game-information-content-manifest-v1";
pub const GAME_INFORMATION_CONTENT_MANIFEST_V1_SCHEMA_SOURCE: &str =
    "schemas/game-information-content-manifest-v1.schema.json";
pub const GAME_INFORMATION_CONTENT_MANIFEST_V1_SCHEMA_DIGEST: &str =
    "416a39769445e6e462c5d5b5504f29010c255e2116a73094e55c7268e47f2ba6";
pub const GAME_INFORMATION_CONTENT_MANIFEST_V1_MAX_MESSAGE_BYTES: usize = 16 * 1024 * 1024;

const SCHEMA: &str =
    include_str!("../../../schemas/game-information-content-manifest-v1.schema.json");

/// Errors from validating or encoding one complete content-manifest envelope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameInformationContentManifestV1CodecError {
    MessageTooLarge,
    InvalidUtf8,
    MalformedJson,
    DuplicateMember,
    InvalidSchema,
    UnsupportedSchemaDigest,
    InvalidMessage,
    SerializationFailed,
}

impl fmt::Display for GameInformationContentManifestV1CodecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MessageTooLarge => "content manifest message exceeds 16 MiB",
            Self::InvalidUtf8 => "content manifest message is not valid UTF-8",
            Self::MalformedJson => "content manifest message has malformed JSON",
            Self::DuplicateMember => "content manifest message contains a duplicate member",
            Self::InvalidSchema => "content manifest schema is invalid",
            Self::UnsupportedSchemaDigest => "content manifest schema digest is unsupported",
            Self::InvalidMessage => "content manifest message does not match its schema",
            Self::SerializationFailed => "content manifest message could not be serialized",
        })
    }
}

impl std::error::Error for GameInformationContentManifestV1CodecError {}

/// Schema- and byte-bounded JSON codec for the whole-manifest wire envelope.
///
/// This validates the transport envelope only. Mapping an owner `ContentManifest` and its
/// failures into this wire representation remains the producer adapter's responsibility.
pub struct GameInformationContentManifestV1Codec {
    schema: jsonschema::Validator,
}

impl GameInformationContentManifestV1Codec {
    pub fn new() -> Result<Self, GameInformationContentManifestV1CodecError> {
        if sha256_hex(SCHEMA.as_bytes()) != GAME_INFORMATION_CONTENT_MANIFEST_V1_SCHEMA_DIGEST {
            return Err(GameInformationContentManifestV1CodecError::InvalidSchema);
        }
        let schema: Value = parse_unique(SCHEMA.as_bytes())?;
        let schema = jsonschema::draft202012::options()
            .build(&schema)
            .map_err(|_| GameInformationContentManifestV1CodecError::InvalidSchema)?;
        Ok(Self { schema })
    }

    /// Rejects oversized bytes before parsing, duplicate JSON members, unpinned schemas, and
    /// envelopes that do not match the closed v1 schema.
    pub fn decode(
        &self,
        bytes: &[u8],
    ) -> Result<Value, GameInformationContentManifestV1CodecError> {
        if bytes.len() > GAME_INFORMATION_CONTENT_MANIFEST_V1_MAX_MESSAGE_BYTES {
            return Err(GameInformationContentManifestV1CodecError::MessageTooLarge);
        }
        std::str::from_utf8(bytes)
            .map_err(|_| GameInformationContentManifestV1CodecError::InvalidUtf8)?;
        let value = parse_unique(bytes)?;
        self.validate_value(&value)?;
        Ok(value)
    }

    /// Serializes one complete envelope, refusing the write once its UTF-8 byte size would exceed
    /// 16 MiB. The returned JSON is compact and its object keys follow serde_json's sorted map
    /// order. The protocol does not prescribe array reordering.
    pub fn encode(
        &self,
        value: &Value,
    ) -> Result<Vec<u8>, GameInformationContentManifestV1CodecError> {
        let mut writer = BoundedWriter::new(GAME_INFORMATION_CONTENT_MANIFEST_V1_MAX_MESSAGE_BYTES);
        if serde_json::to_writer(&mut writer, value).is_err() {
            return if writer.exceeded {
                Err(GameInformationContentManifestV1CodecError::MessageTooLarge)
            } else {
                Err(GameInformationContentManifestV1CodecError::SerializationFailed)
            };
        }
        self.validate_value(value)?;
        Ok(writer.bytes)
    }

    fn validate_value(
        &self,
        value: &Value,
    ) -> Result<(), GameInformationContentManifestV1CodecError> {
        if value["schema_digest"] != GAME_INFORMATION_CONTENT_MANIFEST_V1_SCHEMA_DIGEST {
            return Err(GameInformationContentManifestV1CodecError::UnsupportedSchemaDigest);
        }
        if !self.schema.is_valid(value) {
            return Err(GameInformationContentManifestV1CodecError::InvalidMessage);
        }
        Ok(())
    }
}

struct BoundedWriter {
    bytes: Vec<u8>,
    maximum: usize,
    exceeded: bool,
}

impl BoundedWriter {
    fn new(maximum: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(maximum.min(64 * 1024)),
            maximum,
            exceeded: false,
        }
    }
}

impl Write for BoundedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.bytes.len().saturating_add(bytes.len()) > self.maximum {
            self.exceeded = true;
            return Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "content manifest message exceeds byte limit",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct UniqueValue;

impl<'de> DeserializeSeed<'de> for UniqueValue {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for UniqueValue {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("JSON with unique object members")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Value, E>
    where
        E: serde::de::Error,
    {
        serde_json::Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Value, E> {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Value, E> {
        Ok(Value::String(value))
    }

    fn visit_unit<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element_seed(UniqueValue)? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(serde::de::Error::custom("duplicate object member"));
            }
            values.insert(key, map.next_value_seed(UniqueValue)?);
        }
        Ok(Value::Object(values))
    }
}

fn parse_unique(bytes: &[u8]) -> Result<Value, GameInformationContentManifestV1CodecError> {
    let mut decoder = serde_json::Deserializer::from_slice(bytes);
    let value = UniqueValue.deserialize(&mut decoder).map_err(|error| {
        if error.to_string().contains("duplicate object member") {
            GameInformationContentManifestV1CodecError::DuplicateMember
        } else {
            GameInformationContentManifestV1CodecError::MalformedJson
        }
    })?;
    decoder
        .end()
        .map_err(|_| GameInformationContentManifestV1CodecError::MalformedJson)?;
    Ok(value)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use fmt::Write;
        let _ = write!(output, "{byte:02x}");
    }
    output
}
