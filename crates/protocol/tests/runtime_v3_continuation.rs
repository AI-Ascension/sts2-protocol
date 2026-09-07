// SPDX-License-Identifier: MIT

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "fixture tests fail fast on invalid test data"
)]

use serde_json::{Value, json};
use sts2_protocol::{RuntimeV3GameplayMessage, canonical_json};

const CASES: &[&str] = &[
    include_str!("../../../artifacts/runtime-v3-gameplay/golden/dispatch-proceed-request.json"),
    include_str!(
        "../../../artifacts/runtime-v3-gameplay/golden/dispatch-confirm-selection-request.json"
    ),
    include_str!(
        "../../../artifacts/runtime-v3-gameplay/golden/dispatch-cancel-selection-request.json"
    ),
];

#[test]
fn continuation_vectors_round_trip_and_reject_ambiguous_payloads() {
    let schema: Value = serde_json::from_str(include_str!(
        "../../../schemas/runtime-v3-gameplay.schema.json"
    ))
    .unwrap();
    let validator = jsonschema::draft202012::new(&schema).unwrap();
    for case in CASES {
        let message: RuntimeV3GameplayMessage = serde_json::from_str(case).unwrap();
        message.validate().unwrap();
        assert_eq!(canonical_json(&message).unwrap(), case.trim_end());
        let original: Value = serde_json::from_str(case).unwrap();
        assert!(validator.is_valid(&original));
        for field in ["card_id", "choice_id", "target_id", "coordinates"] {
            let mut invalid = original.clone();
            invalid["action"]["action"][field] = json!("invented");
            assert!(!validator.is_valid(&invalid));
            assert!(serde_json::from_value::<RuntimeV3GameplayMessage>(invalid).is_err());
        }
        let mut unknown = original.clone();
        unknown["action"]["action"]["kind"] = json!("invented_continue");
        assert!(!validator.is_valid(&unknown));
        assert!(serde_json::from_value::<RuntimeV3GameplayMessage>(unknown).is_err());
        let mut stale = original;
        stale["schema_digest"] =
            json!("b37c80f583aeaf4f81ede2083bcfb4129196baf5eb092470e8738173c4b7226c");
        let message: RuntimeV3GameplayMessage = serde_json::from_value(stale).unwrap();
        assert!(message.validate().is_err());
    }
}
