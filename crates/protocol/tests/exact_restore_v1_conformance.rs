// SPDX-License-Identifier: MIT

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::PathBuf;

use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

#[path = "exact_restore_v1_conformance/limits.rs"]
mod limits;

const CONTRACT: &str = "sts2-exact-restore-v1";
const FRAME_LIMIT: usize = 16_384;
const RAW_CHUNK_LIMIT: usize = 8192;

struct NoDuplicateJson;

impl<'de> Deserialize<'de> for NoDuplicateJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(NoDuplicateVisitor)
    }
}

struct NoDuplicateVisitor;

impl<'de> Visitor<'de> for NoDuplicateVisitor {
    type Value = NoDuplicateJson;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("JSON without duplicate object keys")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(NoDuplicateJson)
    }

    fn visit_bool<E>(self, _value: bool) -> Result<Self::Value, E> {
        Ok(NoDuplicateJson)
    }

    fn visit_i64<E>(self, _value: i64) -> Result<Self::Value, E> {
        Ok(NoDuplicateJson)
    }

    fn visit_u64<E>(self, _value: u64) -> Result<Self::Value, E> {
        Ok(NoDuplicateJson)
    }

    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E> {
        Ok(NoDuplicateJson)
    }

    fn visit_str<E>(self, _value: &str) -> Result<Self::Value, E> {
        Ok(NoDuplicateJson)
    }

    fn visit_string<E>(self, _value: String) -> Result<Self::Value, E> {
        Ok(NoDuplicateJson)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while sequence.next_element::<NoDuplicateJson>()?.is_some() {}
        Ok(NoDuplicateJson)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut keys = HashSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !keys.insert(key.clone()) {
                return Err(serde::de::Error::custom(format!(
                    "duplicate JSON object key: {key}"
                )));
            }
            map.next_value::<NoDuplicateJson>()?;
        }
        Ok(NoDuplicateJson)
    }
}

fn artifact() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../artifacts/exact-restore-v1")
}

fn read_artifact(path: &str) -> Vec<u8> {
    std::fs::read(artifact().join(path)).expect("artifact file is present")
}

fn json_artifact(path: &str) -> Value {
    serde_json::from_slice(&read_artifact(path)).expect("artifact JSON is valid")
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn schema_digest() -> String {
    sha256(
        &std::fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../schemas/exact-restore-v1.schema.json"),
        )
        .expect("source schema is present"),
    )
}

fn validator() -> jsonschema::Validator {
    let schema = json_artifact("schema.json");
    jsonschema::draft202012::options()
        .build(&schema)
        .expect("exact-restore schema compiles")
}

fn frames() -> Vec<Value> {
    json_artifact("golden/frames.json")["frames"]
        .as_array()
        .expect("frame golden array")
        .clone()
}

fn canonical_json(value: &Value) -> Vec<u8> {
    // Contract values use ASCII keys and safe integers, so serde_json's sorted compact object
    // encoding is the RFC 8785 form for these deterministic vectors.
    serde_json::to_vec(value).expect("contract value serializes")
}

#[test]
fn manifest_schema_copy_consumers_and_checksum_inventory_match() {
    let source_schema = std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../schemas/exact-restore-v1.schema.json"),
    )
    .expect("source schema is present");
    serde_json::from_slice::<NoDuplicateJson>(&source_schema)
        .expect("schema parser rejects duplicate object keys");
    assert!(
        serde_json::from_str::<NoDuplicateJson>(r#"{"schema":{"type":"object","type":"array"}}"#)
            .is_err()
    );
    let artifact_schema = read_artifact("schema.json");
    let manifest = json_artifact("manifest.json");
    assert_eq!(source_schema, artifact_schema);
    assert_eq!(manifest["artifact"], "sts2-protocol/exact-restore-v1");
    assert_eq!(manifest["protocol_version"], "exact-restore-v1");
    assert_eq!(manifest["schema_digest"], schema_digest());
    assert_eq!(
        manifest["consumers"]
            .as_array()
            .expect("consumer list")
            .len(),
        4
    );

    let checksums =
        String::from_utf8(read_artifact("SHA256SUMS")).expect("checksum inventory is UTF-8");
    let mut count = 0;
    for line in checksums.lines() {
        let (expected, path) = line.split_once("  ").expect("checksum line has two fields");
        assert_eq!(sha256(&read_artifact(path)), expected, "checksum {path}");
        count += 1;
    }
    assert_eq!(count, 6);
}

#[test]
fn all_phase_goldens_are_closed_and_correlated_request_digests_match() {
    let validator = validator();
    let frames = frames();
    assert_eq!(frames.len(), 25);
    assert_eq!(frames[0]["kind"], "exact_restore_begin_request");
    assert_eq!(frames[1]["kind"], "exact_restore_begin_request");
    assert_eq!(frames[0]["payload"], frames[1]["payload"]);
    assert_ne!(frames[0]["message_id"], frames[1]["message_id"]);
    assert_ne!(frames[0]["correlation_id"], frames[1]["correlation_id"]);
    let created_begin = frames
        .iter()
        .find(|frame| {
            frame["kind"] == "exact_restore_begin_response"
                && frame["correlation_id"] == frames[0]["message_id"]
        })
        .expect("first begin response");
    assert_eq!(created_begin["payload"]["result"], "CREATED");
    assert_eq!(created_begin["payload"]["state"], "STAGING");
    let repeated_begin = frames
        .iter()
        .find(|frame| {
            frame["kind"] == "exact_restore_begin_response"
                && frame["correlation_id"] == frames[1]["message_id"]
        })
        .expect("repeated begin response");
    assert_eq!(repeated_begin["payload"]["result"], "EXISTING");
    assert_eq!(repeated_begin["payload"]["state"], "CLOSURE_VERIFIED");
    let mut requests = HashMap::new();
    for (index, frame) in frames.iter().enumerate() {
        assert!(
            validator.is_valid(frame),
            "invalid golden frame {index}: {}",
            frame["kind"]
        );
        assert_eq!(frame["contract"], CONTRACT);
        assert_eq!(frame["schema_digest"], schema_digest());
        let bytes = canonical_json(frame);
        assert!(bytes.len() <= FRAME_LIMIT);
        assert!(!bytes.ends_with(b"\n"));
        if frame["kind"]
            .as_str()
            .is_some_and(|kind| kind.ends_with("_request"))
        {
            let message_id = frame["message_id"].as_str().expect("request message id");
            assert_eq!(frame["correlation_id"], message_id);
            assert!(requests.insert(message_id.to_owned(), bytes).is_none());
        }
    }

    for frame in &frames {
        if frame["kind"]
            .as_str()
            .is_some_and(|kind| kind.ends_with("_response"))
        {
            let correlation = frame["correlation_id"]
                .as_str()
                .expect("response correlation");
            let request = requests
                .get(correlation)
                .expect("response identifies its request");
            let expected = format!("sha256:{}", sha256(request));
            assert_eq!(frame["payload"]["request_digest"], expected);
            assert!(canonical_json(frame).len() <= FRAME_LIMIT);
        }
    }
}

#[test]
fn receipt_digest_and_closure_digest_match_their_synthetic_vectors() {
    let verified = frames()
        .into_iter()
        .find(|frame| {
            frame["kind"] == "exact_restore_lookup_response"
                && frame["payload"]["state"] == "RESTORE_VERIFIED"
        })
        .expect("verified lookup vector");
    let mut receipt = verified["payload"]["receipt"].clone();
    let claimed_digest = receipt["receipt_digest"].clone();
    receipt
        .as_object_mut()
        .expect("receipt object")
        .remove("receipt_digest");
    assert_eq!(
        claimed_digest,
        format!("sha256:{}", sha256(&canonical_json(&receipt)))
    );

    let closure = json_artifact("golden/closure-digest.json");
    let manifest = closure["manifest_utf8"].as_str().expect("manifest bytes");
    let payload = closure["canonical_payload_utf8"]
        .as_str()
        .expect("payload bytes");
    let artifacts = closure["restore_artifacts_utf8"]
        .as_array()
        .expect("artifact byte strings");
    let mut hasher = Sha256::new();
    hasher.update(b"STS2/EXACT-RESTORE-CLOSURE/v1\0");
    hasher.update((manifest.len() as u64).to_be_bytes());
    hasher.update(manifest.as_bytes());
    let mut seen = HashSet::new();
    for bytes in std::iter::once(payload.as_bytes()).chain(
        artifacts
            .iter()
            .map(|entry| entry.as_str().expect("artifact bytes").as_bytes()),
    ) {
        let digest = Sha256::digest(bytes);
        if seen.insert(digest.to_vec()) {
            hasher.update((bytes.len() as u64).to_be_bytes());
            hasher.update(bytes);
        }
    }
    let closure_digest = format!(
        "sha256:{}",
        hasher
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
    assert_eq!(closure["closure_digest"], closure_digest);
    assert_eq!(closure["artifact_reference_count"], 5);
    assert_eq!(closure["distinct_blob_count"], 3);
    assert_eq!(closure["aggregate_closure_bytes"], 9);
}

#[test]
fn rejection_vectors_cover_closure_state_receipt_and_unknown_members() {
    let validator = validator();
    let good = frames();
    let rejections = json_artifact("golden/rejections.json");
    for rejection in rejections.as_array().expect("rejection vector array") {
        let mut mutated = good[rejection["frame_index"].as_u64().unwrap() as usize].clone();
        let path = rejection["path"].as_array().expect("mutation path");
        let mut target = &mut mutated;
        for segment in &path[..path.len() - 1] {
            target = target
                .get_mut(segment.as_str().expect("path key"))
                .expect("mutation parent exists");
        }
        let key = path.last().unwrap().as_str().unwrap();
        match rejection["operation"].as_str().unwrap() {
            "add" | "set" => {
                target[key] = rejection["value"].clone();
            }
            "remove" => {
                target
                    .as_object_mut()
                    .expect("mutation target object")
                    .remove(key)
                    .expect("mutation member exists");
            }
            operation => panic!("unexpected rejection operation: {operation}"),
        }
        assert!(
            validator.validate(&mutated).is_err(),
            "negative vector {} should fail",
            rejection["case"]
        );
    }

    let mut wrong_owner_effect = frames()[22].clone();
    wrong_owner_effect["payload"]["outcome"] = json!("STALE_OWNER");
    wrong_owner_effect["payload"]["host_effect"] = json!("may_have_started");
    assert!(validator.validate(&wrong_owner_effect).is_err());
}
