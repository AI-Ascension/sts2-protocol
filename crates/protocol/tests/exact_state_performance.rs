// SPDX-License-Identifier: MIT

//! Declared-budget checks for the canonical encoding and bounded diff primitives.
//!
//! Ceilings are generous by design: they catch algorithmic regressions on the synthetic profile, not
//! machine noise, and they say nothing about capture pause or a real game snapshot.

use std::time::{Duration, Instant};

use sts2_protocol::{CanonicalValue, describe_differences};

const PREFIX: &str = "{\"gameplay\":{\"entries\":[";
const SUFFIX: &str = "]},\"execution\":{},\"rng\":[],\"legal_actions\":[],\"extensions\":[]}";
const ENTRIES: usize = 20_000;
const CHANGE_STRIDE: usize = 32;

fn payload(change_stride: Option<usize>) -> String {
    let mut text = String::with_capacity(PREFIX.len() + ENTRIES * 48 + SUFFIX.len());
    text.push_str(PREFIX);
    for index in 0..ENTRIES {
        if index > 0 {
            text.push(',');
        }
        text.push_str(&format!(
            "{{\"instance_id\":\"card_{index:05}\",\"upgrades\":{},\"value\":{}}}",
            index % 3,
            if change_stride.is_some_and(|stride| index % stride == 0) {
                index + 1
            } else {
                index
            }
        ));
    }
    text.push_str(SUFFIX);
    text
}

#[test]
fn canonicalization_and_hashing_stay_within_the_declared_budget() {
    let text = payload(None);
    assert!(text.len() > 800_000, "workload is near the declared size");
    let started = Instant::now();
    let parsed = CanonicalValue::parse_str(&text).expect("payload parses");
    let hex = parsed.to_canonical_hex().expect("canonicalizes");
    let digest = parsed.exact_state_digest().expect("digests");
    let elapsed = started.elapsed();
    assert!(hex.len() > 500_000);
    assert!(digest.as_str().starts_with("asc-state:v1:sha256:"));
    println!("canonicalize+hash {} bytes: {:?}", hex.len() / 2, elapsed);
    assert!(
        elapsed < Duration::from_secs(2),
        "canonicalize+hash exceeded the declared budget: {elapsed:?}"
    );
}

#[test]
fn bounded_diff_stays_within_the_declared_budget() {
    let left = CanonicalValue::parse_str(&payload(None)).expect("left parses");
    let right = CanonicalValue::parse_str(&payload(Some(CHANGE_STRIDE))).expect("right parses");
    let started = Instant::now();
    let differences = describe_differences(&left, &right).expect("diff fits its budget");
    let elapsed = started.elapsed();
    println!("bounded diff {} entries: {:?}", differences.len(), elapsed);
    assert!(!differences.is_empty());
    assert!(
        elapsed < Duration::from_secs(2),
        "bounded diff exceeded the declared budget: {elapsed:?}"
    );
}

#[test]
fn resource_bounds_reject_oversized_inputs() {
    let mut oversized = String::with_capacity(17 * 1024 * 1024);
    oversized.push('[');
    oversized.push_str(&"0,".repeat(9 * 1024 * 1024));
    oversized.push_str("0]");
    assert!(CanonicalValue::parse_str(&oversized).is_err());
}
