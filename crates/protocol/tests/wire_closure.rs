// SPDX-License-Identifier: MIT

use serde_json::Value;
use sts2_protocol::{
    ContractManifest, ErrorEnvelope, NeutralMetadata, PocMessage, RuntimeV2Message,
};

const V2: &str = include_str!("../../../artifacts/runtime-v2/golden/state-request.json");
const NEUTRAL: &str = include_str!("../../../conformance/golden/neutral-metadata.v1.json");
const ERROR: &str = include_str!("../../../conformance/golden/error-envelope.v1.json");
const MANIFEST: &str = include_str!("../../../conformance/golden/contract-manifest.v1.json");

fn schema(source: &str) -> jsonschema::Validator {
    jsonschema::draft202012::options()
        .build(&serde_json::from_str::<Value>(source).unwrap())
        .unwrap()
}

#[test]
fn runtime_v2_requires_each_explicit_null_field() {
    let validator = schema(include_str!("../../../schemas/runtime-v2.schema.json"));
    for key in [
        "operation_id",
        "observation",
        "action",
        "status",
        "error_code",
        "effect_witness",
    ] {
        let mut value: Value = serde_json::from_str(V2).unwrap();
        value.as_object_mut().unwrap().remove(key);
        assert!(!validator.is_valid(&value));
        assert!(
            serde_json::from_value::<RuntimeV2Message>(value).is_err(),
            "missing {key}"
        );
    }
}

#[test]
fn neutral_requires_each_nullable_field() {
    let validator = schema(include_str!(
        "../../../schemas/common/neutral-contract-seam.v1.schema.json"
    ));
    for (pointer, key) in [
        ("/identity", "session"),
        ("/correlation", "trace"),
        ("/correlation", "operation"),
        ("/lineage", "parent"),
        ("/lineage", "artifact"),
        ("", "deadline"),
        ("/cancellation", "reason"),
    ] {
        let mut value: Value = serde_json::from_str(NEUTRAL).unwrap();
        value
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(key);
        assert!(!validator.is_valid(&value));
        assert!(
            serde_json::from_value::<NeutralMetadata>(value).is_err(),
            "missing {pointer}/{key}"
        );
    }
}

fn reject_unknown<T: serde::de::DeserializeOwned>(fixture: &str, pointers: &[&str]) {
    let validator = schema(include_str!(
        "../../../schemas/common/neutral-contract-seam.v1.schema.json"
    ));
    for pointer in pointers {
        let mut value: Value = serde_json::from_str(fixture).unwrap();
        value
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unrecognized".into(), Value::Bool(true));
        assert!(!validator.is_valid(&value));
        assert!(
            serde_json::from_value::<T>(value).is_err(),
            "unknown at {pointer}"
        );
    }
}

#[test]
fn neutral_closes_every_struct_boundary() {
    reject_unknown::<NeutralMetadata>(
        NEUTRAL,
        &[
            "",
            "/identity",
            "/identity/subject",
            "/correlation",
            "/lineage",
            "/sequence",
            "/lifecycle",
            "/deadline",
            "/cancellation",
        ],
    );
    reject_unknown::<ErrorEnvelope>(ERROR, &["", "/error"]);
    reject_unknown::<ContractManifest>(
        MANIFEST,
        &[
            "",
            "/manifest_version",
            "/digest",
            "/provenance",
            "/provenance/source_digest",
        ],
    );
}

#[test]
fn poc_rejects_duplicate_members_inside_required_nullable_objects() {
    let action = include_str!("../../../artifacts/poc-v1/golden/action-request.json");
    let duplicate = action.replace("\"units\":1", "\"units\":8,\"units\":1");
    assert_ne!(action, duplicate);
    assert!(serde_json::from_str::<PocMessage>(&duplicate).is_err());
    let observation = include_str!("../../../artifacts/poc-v1/golden/state-response.json");
    let duplicate = observation.replace(
        "\"available_units\":3",
        "\"available_units\":8,\"available_units\":3",
    );
    assert_ne!(observation, duplicate);
    assert!(serde_json::from_str::<PocMessage>(&duplicate).is_err());
}
