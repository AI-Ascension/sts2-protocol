// SPDX-License-Identifier: MIT

use std::fs;

use serde_json::{Map, Value, json};

mod game_information_lookup_binding_v1_semantics;

use game_information_lookup_binding_v1_semantics::{
    binding_id_of, digest, golden, parse_unique_json, payload, read_json, rejection_after,
    repo_root, schema_validator, semantic_rejection,
};

const PROFILE: &str = "game-information-lookup-binding-v1";
const SCHEMA_DIGEST: &str = "f10f9af01d6be1de104069ba842e7971971e88f27553e782e81174ee7aa1cd58";
const ARTIFACT: &str = "sts2-protocol/game-information-lookup-binding-v1";
const MALFORMED: &str = "malformed";
const UNSUPPORTED_VERSION: &str = "unsupported_version";
const INVALID_IDENTITY: &str = "invalid_identity";
const DENIED_SCOPE: &str = "denied_scope";
const MISSING_CAPABILITY: &str = "missing_capability";
const STALE_SNAPSHOT: &str = "stale_snapshot";
const MIXED_BINDING: &str = "mixed_binding";
const REOBSERVE_UNAVAILABLE: &str = "reobserve_unavailable";

const SOURCE_SCHEMA: &str =
    include_str!("../../../schemas/game-information-lookup-binding-v1.schema.json");
const ARTIFACT_SCHEMA: &str =
    include_str!("../../../artifacts/game-information-lookup-binding-v1/schema.json");
const CASE: &str =
    include_str!("../../../conformance/cases/game-information-lookup-binding-v1.json");
const MANIFEST: &str =
    include_str!("../../../artifacts/game-information-lookup-binding-v1/manifest.json");
const CHECKSUMS: &str =
    include_str!("../../../artifacts/game-information-lookup-binding-v1/SHA256SUMS");

const GOLDENS: &[(&str, &str)] = &[
    (
        "discovery-response",
        include_str!(
            "../../../artifacts/game-information-lookup-binding-v1/golden/discovery-response.json"
        ),
    ),
    (
        "observation-response",
        include_str!(
            "../../../artifacts/game-information-lookup-binding-v1/golden/observation-response.json"
        ),
    ),
    (
        "reobserve-required-response",
        include_str!(
            "../../../artifacts/game-information-lookup-binding-v1/golden/reobserve-required-response.json"
        ),
    ),
    (
        "reobserve-exhausted-response",
        include_str!(
            "../../../artifacts/game-information-lookup-binding-v1/golden/reobserve-exhausted-response.json"
        ),
    ),
    (
        "reobserved-response",
        include_str!(
            "../../../artifacts/game-information-lookup-binding-v1/golden/reobserved-response.json"
        ),
    ),
];

fn case_context() -> Value {
    serde_json::from_str::<Value>(CASE).expect("case JSON")["context"].clone()
}

#[test]
fn source_artifact_and_every_golden_are_schema_valid_and_canonical() {
    assert_eq!(SOURCE_SCHEMA, ARTIFACT_SCHEMA);
    let case: Value = serde_json::from_str(CASE).expect("case JSON");
    let context = &case["context"];
    let validator = schema_validator();
    for (name, text) in GOLDENS {
        let value: Value = serde_json::from_str(text).expect("golden JSON");
        if !validator.is_valid(&value) {
            eprintln!("{name}: {:?}", validator.validate(&value));
        }
        assert!(validator.is_valid(&value), "{name} violates the schema");
        assert_eq!(
            serde_json::to_string(&value).expect("canonical JSON"),
            payload(text),
            "{name} is not compact sorted JSON"
        );
        assert_eq!(value["protocol_version"], PROFILE);
        assert_eq!(value["schema_digest"], SCHEMA_DIGEST);
        assert_eq!(value["provenance"]["artifact"], ARTIFACT);
        assert_eq!(
            semantic_rejection(&value, context),
            None,
            "{name} semantic rejection"
        );
    }
}

#[test]
fn the_artifact_indexes_agree_with_the_pinned_digest() {
    let case: Value = serde_json::from_str(CASE).expect("case JSON");
    let manifest: Value = serde_json::from_str(MANIFEST).expect("manifest JSON");
    assert_eq!(manifest["artifact"], ARTIFACT);
    assert_eq!(manifest["protocol_version"], PROFILE);
    assert_eq!(manifest["schema_digest"], SCHEMA_DIGEST);
    assert_eq!(
        case["contract"],
        "sts2.protocol/game-information-lookup-binding-v1"
    );
    assert_eq!(case["profile"], PROFILE);
    assert_eq!(case["case_id"], "CT-GAME-INFORMATION-LOOKUP-BINDING-V1-001");
    assert_eq!(manifest["consumers"], case["consumers"]);
    assert_eq!(
        case["valid_vectors"].as_array().map(Vec::len),
        Some(7),
        "valid vector count"
    );
    assert_eq!(
        case["invalid_vectors"].as_array().map(Vec::len),
        Some(12),
        "invalid vector count"
    );
}

#[test]
fn checksums_cover_every_artifact_member() {
    let directory = repo_root().join("artifacts/game-information-lookup-binding-v1");
    let mut checked = 0;
    for line in CHECKSUMS.lines() {
        let Some((expected, name)) = line.split_once("  ") else {
            continue;
        };
        let path = directory.join(name);
        let bytes =
            fs::read(&path).unwrap_or_else(|error| panic!("checksums cover {name}: {error}"));
        assert_eq!(digest(&bytes), expected, "checksum for {name}");
        checked += 1;
    }
    assert!(checked >= 20, "checksum inventory lists {checked} members");
}

#[test]
fn the_binding_identifier_recomputes_from_the_declared_members() {
    let fixtures =
        repo_root().join("conformance/fixtures/game-information-lookup-binding-v1/valid");
    let base = read_json(&fixtures.join("binding-identity-input.json"));
    let epoch = read_json(&fixtures.join("binding-identity-epoch.json"));
    for fixture in [&base, &epoch] {
        let declared = fixture["identity_input"]
            .as_object()
            .expect("identity members");
        let mut members = Map::new();
        for (key, value) in declared {
            members.insert(key.clone(), value.clone());
        }
        let canonical =
            serde_json::to_string(&Value::Object(members)).expect("canonical identity JSON");
        assert_eq!(
            canonical, fixture["canonical_json"],
            "canonical identity bytes"
        );
        assert_eq!(
            digest(canonical.as_bytes()),
            fixture["binding_id"],
            "declared binding identifier"
        );
    }
    assert_ne!(
        base["binding_id"], epoch["binding_id"],
        "epoch changes identity"
    );
    for (name, text) in GOLDENS {
        let value: Value = serde_json::from_str(text).expect("golden JSON");
        assert_eq!(
            binding_id_of(&value["binding"]),
            value["binding"]["binding_id"],
            "{name} binding identifier recomputes"
        );
    }
    assert_eq!(
        golden("discovery-response")["binding"]["binding_id"],
        base["binding_id"]
    );
}

#[test]
fn observations_are_bound_and_re_observation_preserves_identity() {
    let observed = golden("observation-response");
    let reobserved = golden("reobserved-response");
    assert_eq!(
        observed["observation"]["binding_id"],
        observed["binding"]["binding_id"]
    );
    assert_eq!(
        reobserved["observation"]["binding_id"], observed["observation"]["binding_id"],
        "re-observation preserves the binding identifier"
    );
    assert_ne!(
        reobserved["observation"]["observation_id"], observed["observation"]["observation_id"],
        "re-observation issues a new observation identifier"
    );
    assert!(
        reobserved["observation"]["state_generation"].as_u64()
            > observed["observation"]["state_generation"].as_u64(),
        "re-observation advances the state generation"
    );
}

#[test]
fn the_terminal_fail_closed_state_is_consistent() {
    let required = golden("reobserve-required-response");
    assert_eq!(
        required["discovery"]["observation_state"],
        "reobserve_required"
    );
    assert!(required["observation"].is_null());
    assert_eq!(required["discovery"]["reobserve"]["attempts"], 0);
    assert_eq!(
        required["discovery"]["reobserve"]["supersedes_observation_id"],
        "observation-1"
    );

    let exhausted = golden("reobserve-exhausted-response");
    assert_eq!(
        exhausted["discovery"]["observation_state"],
        "reobserve_exhausted"
    );
    assert!(
        exhausted["observation"].is_null(),
        "exhausted carries no observation"
    );
    assert!(
        exhausted["discovery"]["reobserve"]["attempts"]
            .as_u64()
            .unwrap()
            >= 1
    );

    let context = case_context();
    assert_eq!(
        semantic_rejection(&exhausted, &context),
        None,
        "the terminal state is not itself a rejection"
    );
    let mut fabricated = exhausted.clone();
    fabricated["observation"] = golden("observation-response")["observation"].clone();
    assert_eq!(
        semantic_rejection(&fabricated, &context),
        Some(MALFORMED),
        "an exhausted binding must not fall back to a fabricated observation"
    );
}

#[test]
fn invalid_vectors_match_schema_expectations_and_typed_errors() {
    let case: Value = serde_json::from_str(CASE).expect("case JSON");
    let context = &case["context"];
    let validator = schema_validator();
    let root = repo_root();
    let invalid = case["invalid_vectors"].as_array().expect("invalid vectors");
    assert!(invalid.len() >= 12);
    for descriptor in invalid {
        let path = root.join(descriptor["fixture"].as_str().expect("fixture path"));
        let fixture = read_json(&path);
        assert_eq!(fixture["id"], descriptor["id"], "fixture ID");
        assert_eq!(
            fixture["expected_error"], descriptor["expected_error"],
            "vector ID {}",
            descriptor["id"]
        );
        if let Some(raw) = fixture["raw"].as_str() {
            assert!(
                parse_unique_json(raw).is_err(),
                "duplicate-key witness was accepted for {}",
                fixture["id"]
            );
            assert_eq!(fixture["expected_error"], MALFORMED);
            continue;
        }
        let document = &fixture["document"];
        assert_eq!(
            validator.is_valid(document),
            fixture["schema_valid"].as_bool().expect("schema_valid"),
            "schema result for {}",
            descriptor["id"]
        );
        let fixture_context = if fixture["context"].is_null() {
            context
        } else {
            &fixture["context"]
        };
        assert_eq!(
            semantic_rejection(document, fixture_context),
            fixture["expected_error"].as_str(),
            "semantic result for {}",
            descriptor["id"]
        );
    }
}

#[test]
fn the_witness_precedence_is_deterministic() {
    let context = case_context();

    let mut forged = golden("observation-response");
    forged["binding"]["binding_id"] = Value::String("f".repeat(64));
    forged["observation"]["binding_id"] = Value::String("f".repeat(64));
    forged["binding"]["scope"]["run_id"] = Value::String("run-99".to_owned());
    assert_eq!(
        semantic_rejection(&forged, &context),
        Some(INVALID_IDENTITY),
        "identity recomputation is evaluated before the scope fence"
    );

    let mut scoped = golden("observation-response");
    scoped["binding"]["scope"]["agent_id"] = Value::String("agent-9".to_owned());
    scoped["binding"]["binding_id"] = Value::String(binding_id_of(&scoped["binding"]));
    scoped["observation"]["state_generation"] = Value::from(1);
    assert_eq!(
        semantic_rejection(&scoped, &context),
        Some(DENIED_SCOPE),
        "the scope fence is evaluated before staleness"
    );
    assert_eq!(
        rejection_after(
            &context,
            |document| document["discovery"]["required_capabilities"]["profile"] =
                Value::String("other-profile".to_owned())
        ),
        Some(MISSING_CAPABILITY)
    );
    assert_eq!(
        rejection_after(&context, |document| {
            document["observation"]["binding_id"] = Value::String("a".repeat(64));
        }),
        Some(MIXED_BINDING)
    );
    assert_eq!(
        rejection_after(
            &context,
            |document| document["observation"]["state_generation"] = Value::from(0)
        ),
        Some(STALE_SNAPSHOT)
    );
    assert_eq!(
        rejection_after(&context, |document| document["protocol_version"] =
            Value::from("other-v2")),
        Some(UNSUPPORTED_VERSION)
    );
    assert_eq!(
        rejection_after(&context, |document| document["kind"] =
            Value::from("error_response")),
        Some(MALFORMED)
    );
}

#[test]
fn the_terminal_reobserve_code_agrees_with_its_state() {
    let context = case_context();
    let terminal = |state: &str| {
        let mut document = golden("reobserve-exhausted-response");
        document["kind"] = Value::from("error_response");
        document["discovery"]["observation_state"] = Value::from(state);
        document["error"] = json!({
            "code": REOBSERVE_UNAVAILABLE,
            "field": "observation",
            "reason": "re-observation is impossible",
        });
        semantic_rejection(&document, &context)
    };
    assert_eq!(
        terminal("reobserve_exhausted"),
        None,
        "the terminal code is legal in the terminal state"
    );
    assert_eq!(
        terminal("not_yet_observed"),
        Some(MALFORMED),
        "the terminal code contradicts any other state"
    );
}
