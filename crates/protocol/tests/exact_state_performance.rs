// SPDX-License-Identifier: MIT

//! Reported-budget checks for the canonical encoding and bounded diff primitives.
//!
//! These measure work the primitives actually do and print it. They deliberately do **not** gate
//! on wall-clock, because cargo runs test binaries in parallel and an absolute ceiling then reports
//! the machine's load rather than the code's behaviour.
//!
//! What was here before was `assert!(elapsed < Duration::from_secs(2))`, which is not a usable
//! signal: the identical binary measured 0.79s-1.46s alone and 2.8s-4.3s under the rest of the
//! suite, so it was green or red depending on what else was running. A check that flips on
//! contention trains reviewers to ignore it, which is worse than no check at all.
//!
//! What a real regression looks like instead is *algorithmic*: work that grows superlinearly with
//! the payload, or a canonical form whose size changes. Both are asserted, and both are
//! load-independent:
//!
//! - the canonical encoding must be a fixed multiple of the input, so a change to the encoding
//!   cannot silently inflate every digest a caller stores;
//! - throughput must stay within a wide band, so a quadratic regression trips it while ordinary
//!   contention does not.
//!
//! The throughput band is deliberately loose -- it spans the observed unloaded range with large
//! margin on both sides -- because its job is to catch an algorithmic regression, not to police a
//! service-level objective. A hard wall-clock requirement belongs in an isolated job that does not
//! compete with the rest of the suite; see #78.
//!
//! Nothing here says anything about capture pause or a real game snapshot.

use std::time::Instant;

use sts2_protocol::{CanonicalValue, describe_differences};

const PREFIX: &str = "{\"gameplay\":{\"entries\":[";
const SUFFIX: &str = "]},\"execution\":{},\"rng\":[],\"legal_actions\":[],\"extensions\":[]}";
const ENTRIES: usize = 20_000;
const CHANGE_STRIDE: usize = 32;

/// The canonical encoding is exactly two characters per input byte: one hex digit per nibble.
/// A change here means every digest a caller persists changes shape, which is a contract-visible
/// change rather than a performance one.
const CANONICAL_HEX_PER_INPUT_BYTE: usize = 2;

/// Lower bound on bytes processed per second by `canonicalize + hash`.
///
/// Measured unloaded on the reference machine at 0.79s-1.46s for ~1.1 MB, and 2.8s-4.3s under
/// full-suite load. This floor sits an order of magnitude below the *loaded* figure, so contention
/// cannot trip it, and roughly two orders below the unloaded one, so only a real algorithmic
/// regression can.
const MIN_BYTES_PER_SECOND: f64 = 100_000.0;

/// Lower bound on entries diffed per second by the bounded diff.
///
/// The bounded diff does far less work than the canonicalization pass -- 625 entries against
/// 1,108,975 bytes -- so its per-entry rate is much higher and correspondingly easier to depress.
/// Measured unloaded at ~1,570/s, and ~960/s with four spinners competing, which is why the first
/// value tried here (1,000.0) failed under synthetic load while the canonicalization floor did
/// not. This floor sits a further order of magnitude below the loaded figure so contention cannot
/// reach it.
const MIN_ENTRIES_PER_SECOND: f64 = 100.0;

/// Reports a measurement and asserts it clears the floor, without ever gating on wall-clock.
fn report_within_budget(
    rate: f64,
    floor: f64,
    unit: &str,
    elapsed: std::time::Duration,
    amount: f64,
) {
    let rate_text = format!("{rate:.0}");
    println!("{unit}: {amount:.0} in {elapsed:?} ({rate_text} per second)");
    assert!(
        rate >= floor,
        "{unit} fell to {rate_text} per second, below the {floor:.0} floor: \
         the work grew superlinearly rather than the machine being slow"
    );
}

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
    let bytes = text.len() as f64;
    assert_eq!(
        hex.len(),
        text.len() * CANONICAL_HEX_PER_INPUT_BYTE,
        "the canonical encoding must stay exactly two hex characters per input byte"
    );
    assert!(hex.len() > 500_000);
    assert!(digest.as_str().starts_with("asc-state:v1:sha256:"));
    report_within_budget(
        bytes / elapsed.as_secs_f64(),
        MIN_BYTES_PER_SECOND,
        "canonicalize+hash",
        elapsed,
        bytes,
    );
}

#[test]
fn bounded_diff_stays_within_the_declared_budget() {
    let left = CanonicalValue::parse_str(&payload(None)).expect("left parses");
    let right = CanonicalValue::parse_str(&payload(Some(CHANGE_STRIDE))).expect("right parses");
    let started = Instant::now();
    let differences = describe_differences(&left, &right).expect("diff fits its budget");
    let elapsed = started.elapsed();
    let entries = differences.len() as f64;
    assert!(!differences.is_empty());
    report_within_budget(
        entries / elapsed.as_secs_f64(),
        MIN_ENTRIES_PER_SECOND,
        "bounded diff",
        elapsed,
        entries,
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
