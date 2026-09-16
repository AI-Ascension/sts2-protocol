// SPDX-License-Identifier: MIT

use serde_json::{Value, json};

use super::{FRAME_LIMIT, RAW_CHUNK_LIMIT, canonical_json, frames, validator};

fn base64_value(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

/// Checks RFC 4648 base64 syntax and raw size without decoding or allocating the chunk.
fn checked_base64_decoded_len(encoded: &str) -> Option<usize> {
    let bytes = encoded.as_bytes();
    if bytes.len() < 4 || bytes.len() > 10_924 || !bytes.len().is_multiple_of(4) {
        return None;
    }
    let padding = if bytes.ends_with(b"==") {
        2
    } else if bytes.ends_with(b"=") {
        1
    } else {
        0
    };
    let content_len = bytes.len().checked_sub(padding)?;
    if bytes[..content_len]
        .iter()
        .any(|byte| base64_value(*byte).is_none())
        || bytes[content_len..].iter().any(|byte| *byte != b'=')
    {
        return None;
    }
    if padding == 2 && base64_value(*bytes.get(content_len.checked_sub(1)?)?)? & 0b1111 != 0 {
        return None;
    }
    if padding == 1 && base64_value(*bytes.get(content_len.checked_sub(1)?)?)? & 0b11 != 0 {
        return None;
    }
    bytes
        .len()
        .checked_div(4)?
        .checked_mul(3)?
        .checked_sub(padding)
}

/// Applies schema, whole-frame, canonical base64, and decoded-chunk bounds before allocation.
fn chunk_frame_admissible(frame: &Value, validator: &jsonschema::Validator) -> bool {
    if validator.validate(frame).is_err() || canonical_json(frame).len() > FRAME_LIMIT {
        return false;
    }
    let Some(encoded) = frame["payload"]["data_base64"].as_str() else {
        return false;
    };
    let Some(raw_bytes) = checked_base64_decoded_len(encoded) else {
        return false;
    };
    let Some(offset) = frame["payload"]["offset"].as_u64() else {
        return false;
    };
    let Some(total_bytes) = frame["payload"]["total_bytes"].as_u64() else {
        return false;
    };
    raw_bytes > 0
        && raw_bytes <= RAW_CHUNK_LIMIT
        && offset
            .checked_add(raw_bytes as u64)
            .is_some_and(|end| end <= total_bytes)
}

#[test]
fn full_frame_and_decoded_chunk_limits_apply_before_transfer() {
    let validator = validator();
    let mut chunk = frames()[2].clone();
    let mut encoded = "AAAA".repeat(2730);
    encoded.push_str("AAA=");
    assert_eq!(encoded.len(), 10_924);
    assert_eq!(checked_base64_decoded_len(&encoded), Some(RAW_CHUNK_LIMIT));
    chunk["payload"]["data_base64"] = json!(encoded);
    chunk["payload"]["total_bytes"] = json!(RAW_CHUNK_LIMIT);
    let bytes = canonical_json(&chunk);
    assert!(validator.validate(&chunk).is_ok());
    assert!(bytes.len() <= FRAME_LIMIT);
    assert!(chunk_frame_admissible(&chunk, &validator));

    let one_over = "AAAA".repeat(2731);
    assert_eq!(one_over.len(), 10_924);
    assert_eq!(
        checked_base64_decoded_len(&one_over),
        Some(RAW_CHUNK_LIMIT + 1)
    );
    let mut oversized_raw_chunk = chunk.clone();
    oversized_raw_chunk["payload"]["data_base64"] = json!(one_over);
    oversized_raw_chunk["payload"]["total_bytes"] = json!(RAW_CHUNK_LIMIT + 1);
    assert!(validator.validate(&oversized_raw_chunk).is_ok());
    assert!(!chunk_frame_admissible(&oversized_raw_chunk, &validator));

    let mut empty_chunk = chunk.clone();
    empty_chunk["payload"]["data_base64"] = json!("");
    empty_chunk["payload"]["total_bytes"] = json!(0);
    assert!(validator.validate(&empty_chunk).is_err());
    assert!(!chunk_frame_admissible(&empty_chunk, &validator));

    let mut noncanonical_padding = chunk.clone();
    noncanonical_padding["payload"]["data_base64"] = json!("AB==");
    assert!(validator.validate(&noncanonical_padding).is_ok());
    assert!(!chunk_frame_admissible(&noncanonical_padding, &validator));

    let mut receipt = frames()[21].clone();
    receipt["payload"]["expected_owner"]["session_id"] = json!("s".repeat(512));
    receipt["payload"]["receipt"]["destination_owner"]["session_id"] = json!("s".repeat(512));
    for key in [
        "experiment_id",
        "branch_id",
        "run_id",
        "episode_id",
        "trajectory_id",
    ] {
        receipt["payload"]["receipt"]["branch"][key] = json!("b".repeat(512));
    }
    let response_bytes = canonical_json(&receipt);
    assert!(validator.validate(&receipt).is_ok());
    assert!(response_bytes.len() <= FRAME_LIMIT);

    let mut oversized = frames()[0].clone();
    oversized["payload"]["artifacts"] = json!(
        (0..64)
            .map(|_| {
                json!({
                    "role": "r".repeat(512),
                    "digest": format!("sha256:{}", "a".repeat(64)),
                    "size_bytes": 1,
                    "codec": "c".repeat(512)
                })
            })
            .collect::<Vec<_>>()
    );
    oversized["payload"]["artifact_reference_count"] = json!(64);
    oversized["payload"]["distinct_blob_count"] = json!(1);
    oversized["payload"]["aggregate_closure_bytes"] = json!(12);
    assert!(validator.validate(&oversized).is_ok());
    assert!(canonical_json(&oversized).len() > FRAME_LIMIT);
}
