// SPDX-License-Identifier: MIT

use std::collections::BTreeMap;

use serde_json::Value;
use sts2_protocol::{
    BlobDigest, CanonicalError, CanonicalValue, ExactCheckpointId, ExactStateDigest, MAX_DEPTH,
    blob_digest,
};

const VECTORS: &str =
    include_str!("../../../conformance/fixtures/exact-state-v1/canonical-vectors.json");
const EXPECTED: &str =
    include_str!("../../../conformance/fixtures/exact-state-v1/canonical-vectors.expected.json");
const IDENTITY: &str =
    include_str!("../../../conformance/fixtures/exact-state-v1/identity-vectors.expected.json");
const CASE: &str = include_str!("../../../conformance/cases/exact-state-v1.json");

fn parsed(text: &str) -> Value {
    serde_json::from_str(text).expect("fixture JSON is valid")
}

fn value_text(value: &Value) -> String {
    serde_json::to_string(value).expect("fixture value re-serializes")
}

fn expected_by_name(fixture: &str) -> BTreeMap<String, Value> {
    parsed(fixture)["vectors"]
        .as_array()
        .expect("vectors array")
        .iter()
        .map(|entry| {
            (
                entry["name"].as_str().expect("name").to_owned(),
                entry.clone(),
            )
        })
        .collect()
}

fn canonical(value: &Value) -> CanonicalValue {
    CanonicalValue::parse_str(&value_text(value)).expect("fixture value is inside the profile")
}

#[test]
fn case_file_declares_no_live_dependency() {
    let case = parsed(CASE);
    assert_eq!(case["profile"], "asc-jcs-state-v1");
    assert_eq!(case["setup"]["live_runtime"], false);
    assert_eq!(case["setup"]["network"], false);
}

#[test]
fn positive_vectors_match_recorded_bytes_and_state_digests() {
    let vectors = parsed(VECTORS);
    let expected = expected_by_name(EXPECTED);
    let positives = vectors["positive"].as_array().expect("positive array");
    assert_eq!(positives.len(), 26);
    for entry in positives {
        let name = entry["name"].as_str().expect("name");
        let parsed_value = canonical(&entry["value"]);
        let recorded = expected.get(name).expect("expected entry exists");
        assert_eq!(
            parsed_value.to_canonical_hex().expect("canonical hex"),
            recorded["canonical_hex"].as_str().expect("canonical_hex"),
            "canonical bytes differ for {name}"
        );
        assert_eq!(
            parsed_value.exact_state_digest().expect("digest").as_str(),
            recorded["state_id"].as_str().expect("state_id"),
            "exact-state digest differs for {name}"
        );
    }
}

#[test]
fn checkpoint_and_blob_identities_match_the_independent_witness() {
    let vectors = parsed(VECTORS);
    let identity = expected_by_name(IDENTITY);
    for entry in vectors["positive"].as_array().expect("positive array") {
        let name = entry["name"].as_str().expect("name");
        let parsed_value = canonical(&entry["value"]);
        let recorded = identity.get(name).expect("identity entry exists");
        assert_eq!(
            parsed_value
                .exact_checkpoint_id()
                .expect("checkpoint id")
                .as_str(),
            recorded["checkpoint_id"].as_str().expect("checkpoint_id"),
            "checkpoint identifier differs for {name}"
        );
        assert_eq!(
            blob_digest(&parsed_value.to_canonical_bytes().expect("bytes")).as_str(),
            recorded["blob_digest"].as_str().expect("blob_digest"),
            "blob digest differs for {name}"
        );
    }
}

#[test]
fn equivalence_pairs_share_bytes_and_identity() {
    let vectors = parsed(VECTORS);
    let expected = expected_by_name(EXPECTED);
    let by_name: BTreeMap<String, Value> = vectors["positive"]
        .as_array()
        .expect("positive array")
        .iter()
        .map(|entry| {
            (
                entry["name"].as_str().expect("name").to_owned(),
                entry["value"].clone(),
            )
        })
        .collect();
    for pair in vectors["equivalence_pairs"].as_array().expect("pairs") {
        let left = canonical(&by_name[pair["left"].as_str().expect("left")]);
        let right = canonical(&by_name[pair["right"].as_str().expect("right")]);
        let left_hex = left.to_canonical_hex().expect("hex");
        assert_eq!(left_hex, right.to_canonical_hex().expect("hex"));
        assert_eq!(
            left_hex,
            expected[pair["left"].as_str().unwrap()]["canonical_hex"]
                .as_str()
                .unwrap()
        );
    }
}

#[test]
fn distinction_pairs_differ_in_identity() {
    let vectors = parsed(VECTORS);
    let by_name: BTreeMap<String, Value> = vectors["positive"]
        .as_array()
        .expect("positive array")
        .iter()
        .map(|entry| {
            (
                entry["name"].as_str().expect("name").to_owned(),
                entry["value"].clone(),
            )
        })
        .collect();
    for pair in vectors["distinct_pairs"].as_array().expect("pairs") {
        let left = canonical(&by_name[pair["left"].as_str().expect("left")]);
        let right = canonical(&by_name[pair["right"].as_str().expect("right")]);
        assert_ne!(
            left.exact_state_digest().expect("digest").as_str(),
            right.exact_state_digest().expect("digest").as_str()
        );
    }
}

#[test]
fn raw_rejection_vectors_are_refused() {
    let vectors = parsed(VECTORS);
    let rejects = vectors["reject_raw"].as_array().expect("reject array");
    assert_eq!(rejects.len(), 14);
    for entry in rejects {
        let name = entry["name"].as_str().expect("name");
        let text = entry["text"].as_str().expect("text");
        assert!(
            CanonicalValue::parse_str(text).is_err(),
            "raw rejection {name} was accepted"
        );
    }
}

#[test]
fn identity_namespaces_are_not_interchangeable() {
    let value = canonical(&serde_json::json!({"a": 1}));
    let state = value.exact_state_digest().expect("state digest");
    let checkpoint = value.exact_checkpoint_id().expect("checkpoint id");
    assert_ne!(state.as_str(), checkpoint.as_str());
    assert!(ExactStateDigest::parse(state.as_str()).is_ok());
    assert!(ExactCheckpointId::parse(checkpoint.as_str()).is_ok());
    assert!(ExactStateDigest::parse(checkpoint.as_str()).is_err());
    assert!(ExactCheckpointId::parse(state.as_str()).is_err());
    assert!(BlobDigest::parse(state.as_str()).is_err());
    assert!(BlobDigest::parse("sha256:00").is_err());
    assert!(BlobDigest::parse(&blob_digest(b"x").as_str().replace('a', "A")).is_err());
}

#[test]
fn null_absence_and_empty_are_distinct() {
    let null = CanonicalValue::Null;
    let empty_object = CanonicalValue::Object(BTreeMap::new());
    let empty_array = CanonicalValue::Array(Vec::new());
    assert_eq!(null.to_canonical_hex().unwrap(), "6e756c6c");
    assert_eq!(empty_object.to_canonical_hex().unwrap(), "7b7d");
    assert_eq!(empty_array.to_canonical_hex().unwrap(), "5b5d");
}

#[test]
fn integer_and_scalar_boundaries_are_enforced() {
    assert!(CanonicalValue::parse_str("{\"k\":9007199254740991}").is_ok());
    assert!(CanonicalValue::parse_str("{\"k\":-9007199254740991}").is_ok());
    assert_eq!(
        CanonicalValue::parse_str("{\"k\":9007199254740992}").unwrap_err(),
        CanonicalError::IntegerRange
    );
    assert_eq!(
        CanonicalValue::parse_str("{\"k\":10000000000000000}").unwrap_err(),
        CanonicalError::IntegerRange
    );
    for token in ["-0", "1.0", "1e3", "0.5", "01"] {
        let text = format!("{{\"k\":{token}}}");
        assert_eq!(
            CanonicalValue::parse_str(&text).unwrap_err(),
            CanonicalError::NumberToken,
            "token {token} was accepted"
        );
    }
}

#[test]
fn string_escapes_and_unicode_are_preserved_exactly() {
    let value =
        CanonicalValue::parse_str("{\"k\":\"a\\u2028\\u00e9\\ud83d\\ude00\"}").expect("parses");
    let text = String::from_utf8(value.to_canonical_bytes().expect("bytes")).expect("utf8");
    assert_eq!(text, "{\"k\":\"a\u{2028}\u{e9}\u{1f600}\"}");
    let controls =
        CanonicalValue::parse_str("{\"k\":\"\\u0001\\b\\t\\n\\f\\r\\\"\\\\\"}").expect("parses");
    assert_eq!(
        controls.to_canonical_hex().expect("hex"),
        "7b226b223a225c75303030315c625c745c6e5c665c725c225c5c227d"
    );
}

#[test]
fn malformed_input_is_rejected_before_comparison() {
    for text in [
        "{\"a\":1,\"\\u0061\":2}",
        "{\"A\":1}",
        "{\"\":1}",
        "{\"k\":\"\\ud800\"}",
        "{\"k\":\"\u{1}\"}",
        "{} trailing",
        "[1,2",
    ] {
        assert!(
            CanonicalValue::parse_str(text).is_err(),
            "malformed input {text:?} was accepted"
        );
    }
    assert_eq!(
        CanonicalValue::parse(b"\xef\xbb\xbf{}").unwrap_err(),
        CanonicalError::Bom
    );
    assert_eq!(
        CanonicalValue::parse(&[0xff, 0xfe]).unwrap_err(),
        CanonicalError::Utf8
    );
}

#[test]
fn depth_is_bounded_on_raw_and_canonical_paths() {
    let within = format!("{}0{}", "[".repeat(MAX_DEPTH), "]".repeat(MAX_DEPTH));
    assert!(CanonicalValue::parse_str(&within).is_ok());
    let beyond = format!(
        "{}0{}",
        "[".repeat(MAX_DEPTH + 1),
        "]".repeat(MAX_DEPTH + 1)
    );
    assert_eq!(
        CanonicalValue::parse_str(&beyond).unwrap_err(),
        CanonicalError::Depth
    );
}

#[test]
fn canonical_input_path_rejects_noncanonical_order_and_whitespace() {
    assert_eq!(
        CanonicalValue::parse_canonical(br#"{"b":1,"a":2}"#).unwrap_err(),
        CanonicalError::NonCanonical
    );
    assert_eq!(
        CanonicalValue::parse_canonical(b"{ \"a\" : 1 }").unwrap_err(),
        CanonicalError::NonCanonical
    );
    let value = CanonicalValue::parse_canonical(br#"{"a":1}"#).expect("canonical input parses");
    assert_eq!(value.to_canonical_hex().expect("hex"), "7b2261223a317d");
    let raw = CanonicalValue::parse_str("{ \"b\" : 1 , \"a\" : 2 }").expect("raw parses");
    assert_eq!(
        raw.to_canonical_hex().expect("hex"),
        "7b2261223a322c2262223a317d"
    );
}
