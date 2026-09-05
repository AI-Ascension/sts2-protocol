// SPDX-License-Identifier: MIT

use serde_json::Value;
use sts2_protocol::{
    ContractManifest, ErrorEnvelope, NeutralMetadata, PocMessage, RuntimeMessage, RuntimeV2Message,
};

const V1_SCHEMA: &str = include_str!("../../../schemas/runtime-v1.schema.json");
const V1_GOLDENS: &[&str] = &[
    include_str!("../../../artifacts/runtime-v1/golden/state-request.json"),
    include_str!("../../../artifacts/runtime-v1/golden/state-response.json"),
    include_str!("../../../artifacts/runtime-v1/golden/action-request.json"),
    include_str!("../../../artifacts/runtime-v1/golden/action-accepted.json"),
    include_str!("../../../artifacts/runtime-v1/golden/action-rejected.json"),
];
const NEUTRAL_SCHEMA: &str =
    include_str!("../../../schemas/common/neutral-contract-seam.v1.schema.json");
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
fn runtime_v1_requires_each_explicit_null_field_in_every_golden() {
    let validator = schema(V1_SCHEMA);
    for golden in V1_GOLDENS {
        for key in [
            "observation",
            "action",
            "status",
            "error_code",
            "effect_witness",
        ] {
            let mut value: Value = serde_json::from_str(golden).unwrap();
            value.as_object_mut().unwrap().remove(key);
            assert!(!validator.is_valid(&value));
            assert!(
                serde_json::from_value::<RuntimeMessage>(value).is_err(),
                "missing {key}"
            );
        }
    }
}

#[test]
fn runtime_v1_rejects_duplicate_members_at_the_envelope_and_inside_nullable_objects() {
    let accepted = V1_GOLDENS[3];
    let duplicate_status = accepted.replace(
        "\"status\":\"accepted\"",
        "\"status\":\"rejected\",\"status\":\"accepted\"",
    );
    assert_ne!(accepted, duplicate_status);
    assert!(serde_json::from_str::<RuntimeMessage>(&duplicate_status).is_err());
    let duplicate_count = accepted.replace(
        "\"action_count\":1",
        "\"action_count\":8,\"action_count\":1",
    );
    assert_ne!(accepted, duplicate_count);
    assert!(serde_json::from_str::<RuntimeMessage>(&duplicate_count).is_err());
    let duplicate_witness_kind = accepted.replace(
        "\"kind\":\"status_overlay_visible\"",
        "\"kind\":\"status_overlay_visible\",\"kind\":\"status_overlay_visible\"",
    );
    assert_ne!(accepted, duplicate_witness_kind);
    assert!(serde_json::from_str::<RuntimeMessage>(&duplicate_witness_kind).is_err());
    let untouched: RuntimeMessage = serde_json::from_str(accepted).unwrap();
    assert!(untouched.validate().is_ok());
}

#[test]
fn runtime_v1_closes_every_struct_boundary() {
    reject_unknown::<RuntimeMessage>(
        V1_SCHEMA,
        V1_GOLDENS[3],
        &[
            "",
            "/provenance",
            "/observation",
            "/action",
            "/effect_witness",
        ],
    );
    for golden in V1_GOLDENS {
        reject_unknown::<RuntimeMessage>(V1_SCHEMA, golden, &[""]);
    }
}

#[test]
fn neutral_requires_each_nullable_field() {
    let validator = schema(NEUTRAL_SCHEMA);
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

fn reject_unknown<T: serde::de::DeserializeOwned>(
    schema_source: &str,
    fixture: &str,
    pointers: &[&str],
) {
    let validator = schema(schema_source);
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
        NEUTRAL_SCHEMA,
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
    reject_unknown::<ErrorEnvelope>(NEUTRAL_SCHEMA, ERROR, &["", "/error"]);
    reject_unknown::<ContractManifest>(
        NEUTRAL_SCHEMA,
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

#[test]
fn neutral_license_uses_its_schema_alphabet_not_the_identity_alphabet() {
    let validator = schema(NEUTRAL_SCHEMA);
    for license in ["MIT/Custom", "license:MIT"] {
        let mut value: Value = serde_json::from_str(MANIFEST).unwrap();
        value["provenance"]["license"] = Value::String(license.into());
        assert!(!validator.is_valid(&value));
        let manifest: ContractManifest = serde_json::from_value(value).unwrap();
        assert!(manifest.validate().is_err());
    }
}
