// SPDX-License-Identifier: MIT

use serde_json::{Value, json};
use sts2_protocol::{
    RUNTIME_V2_ACTION_ID, RUNTIME_V2_ARTIFACT, RUNTIME_V2_EFFECT_KIND, RUNTIME_V2_MAX_GENERATION,
    RUNTIME_V2_MAX_TURN_INDEX, RUNTIME_V2_PROTOCOL_VERSION, RUNTIME_V2_SCHEMA_DIGEST,
    RUNTIME_V2_SCHEMA_SOURCE, RuntimeV2Action, RuntimeV2ActionResult, RuntimeV2CombatPhase,
    RuntimeV2Context, RuntimeV2Message, RuntimeV2MessageKind, RuntimeV2Metadata,
    RuntimeV2Observation, RuntimeV2Status, canonical_json, decode_json,
};

const CASE: &str = include_str!("../../../conformance/cases/runtime-v2.json");
const MANIFEST: &str = include_str!("../../../artifacts/runtime-v2/manifest.json");
const CHECKSUMS: &str = include_str!("../../../artifacts/runtime-v2/SHA256SUMS");
const SOURCE_SCHEMA: &str = include_str!("../../../schemas/runtime-v2.schema.json");
const ARTIFACT_SCHEMA: &str = include_str!("../../../artifacts/runtime-v2/schema.json");

const GOLDENS: &[(&str, &str)] = &[
    (
        "state-request",
        include_str!("../../../artifacts/runtime-v2/golden/state-request.json"),
    ),
    (
        "state-response",
        include_str!("../../../artifacts/runtime-v2/golden/state-response.json"),
    ),
    (
        "legal-action-request",
        include_str!("../../../artifacts/runtime-v2/golden/legal-action-request.json"),
    ),
    (
        "legal-action-accepted",
        include_str!("../../../artifacts/runtime-v2/golden/legal-action-accepted.json"),
    ),
    (
        "legal-action-settled",
        include_str!("../../../artifacts/runtime-v2/golden/legal-action-settled.json"),
    ),
    (
        "stale-generation-request",
        include_str!("../../../artifacts/runtime-v2/golden/stale-generation-request.json"),
    ),
    (
        "stale-generation-response",
        include_str!("../../../artifacts/runtime-v2/golden/stale-generation-response.json"),
    ),
    (
        "outside-combat-request",
        include_str!("../../../artifacts/runtime-v2/golden/outside-combat-request.json"),
    ),
    (
        "outside-combat-response",
        include_str!("../../../artifacts/runtime-v2/golden/outside-combat-response.json"),
    ),
    (
        "enemy-turn-request",
        include_str!("../../../artifacts/runtime-v2/golden/enemy-turn-request.json"),
    ),
    (
        "enemy-turn-response",
        include_str!("../../../artifacts/runtime-v2/golden/enemy-turn-response.json"),
    ),
    (
        "idempotency-conflict-request",
        include_str!("../../../artifacts/runtime-v2/golden/idempotency-conflict-request.json"),
    ),
    (
        "idempotency-conflict-response",
        include_str!("../../../artifacts/runtime-v2/golden/idempotency-conflict-response.json"),
    ),
    (
        "cancelled-before-dispatch",
        include_str!("../../../artifacts/runtime-v2/golden/cancelled-before-dispatch.json"),
    ),
    (
        "timeout-action-request",
        include_str!("../../../artifacts/runtime-v2/golden/timeout-action-request.json"),
    ),
    (
        "timeout-unknown-response",
        include_str!("../../../artifacts/runtime-v2/golden/timeout-unknown-response.json"),
    ),
    (
        "reconcile-request",
        include_str!("../../../artifacts/runtime-v2/golden/reconcile-request.json"),
    ),
    (
        "reconcile-settled-response",
        include_str!("../../../artifacts/runtime-v2/golden/reconcile-settled-response.json"),
    ),
    (
        "duplicate-replay",
        include_str!("../../../artifacts/runtime-v2/golden/duplicate-replay.json"),
    ),
];

fn payload(text: &str) -> &str {
    text.strip_suffix('\n').unwrap_or(text)
}

fn golden(name: &str) -> &str {
    GOLDENS
        .iter()
        .find_map(|(golden_name, text)| (*golden_name == name).then_some(*text))
        .expect("named Runtime-v2 golden exists")
}

fn message(name: &str) -> RuntimeV2Message {
    decode_json(golden(name)).expect("Runtime-v2 golden decodes")
}

fn checksum_for<'a>(inventory: &'a str, path: &str) -> &'a str {
    inventory
        .lines()
        .find_map(|line| {
            let (digest, listed_path) = line.split_once("  ")?;
            (listed_path == path).then_some(digest)
        })
        .expect("checksum inventory contains the requested path")
}

#[test]
fn runtime_v2_goldens_round_trip_with_stable_bytes() {
    for (name, text) in GOLDENS {
        let message: RuntimeV2Message = decode_json(text).expect("golden JSON is valid");
        message
            .validate()
            .unwrap_or_else(|error| panic!("{name} must validate: {error}"));
        assert_eq!(
            canonical_json(&message).expect("encoding succeeds"),
            payload(text)
        );
    }
}

#[test]
fn runtime_v2_schema_and_artifact_are_identical_and_reject_unknowns() {
    let source: Value = decode_json(SOURCE_SCHEMA).expect("source schema is JSON");
    let artifact: Value = decode_json(ARTIFACT_SCHEMA).expect("artifact schema is JSON");
    assert_eq!(source, artifact);
    assert_eq!(SOURCE_SCHEMA.as_bytes(), ARTIFACT_SCHEMA.as_bytes());
    assert_eq!(source["$id"], "sts2-runtime-v2");
    assert_eq!(
        source["$defs"]["observation"]["properties"]["turn_index"]["maximum"],
        json!(RUNTIME_V2_MAX_TURN_INDEX)
    );
    let validator = jsonschema::draft202012::options()
        .build(&source)
        .expect("Runtime-v2 schema compiles as Draft 2020-12");
    for (_, text) in GOLDENS {
        let value: Value = decode_json(text).expect("golden is JSON");
        assert!(validator.is_valid(&value), "golden must satisfy the schema");
    }

    let mut unknown: Value = decode_json(golden("state-response")).expect("state is JSON");
    unknown["unexpected"] = json!(true);
    assert!(!validator.is_valid(&unknown));
    let mut nested_unknown: Value = decode_json(golden("legal-action-settled")).expect("receipt");
    nested_unknown["observation"]["unexpected"] = json!(true);
    assert!(!validator.is_valid(&nested_unknown));
}

#[test]
fn runtime_v2_status_rules_require_authoritative_settlement_fields() {
    let validator = jsonschema::draft202012::options()
        .build(&decode_json::<Value>(SOURCE_SCHEMA).expect("schema is JSON"))
        .expect("schema compiles");
    for name in [
        "legal-action-accepted",
        "legal-action-settled",
        "stale-generation-response",
        "cancelled-before-dispatch",
        "timeout-unknown-response",
    ] {
        let value: Value = decode_json(golden(name)).expect("golden is JSON");
        assert!(validator.is_valid(&value), "{name} is valid");
    }

    let mut settled_without_witness: Value =
        decode_json(golden("legal-action-settled")).expect("settled is JSON");
    settled_without_witness["effect_witness"] = Value::Null;
    assert!(!validator.is_valid(&settled_without_witness));
    let mut accepted_with_witness: Value =
        decode_json(golden("legal-action-accepted")).expect("accepted is JSON");
    accepted_with_witness["effect_witness"] = json!({
        "kind": RUNTIME_V2_EFFECT_KIND,
        "generation": 4
    });
    assert!(!validator.is_valid(&accepted_with_witness));
    let mut unknown_with_observation: Value =
        decode_json(golden("timeout-unknown-response")).expect("unknown is JSON");
    unknown_with_observation["observation"] = json!({
        "combat_phase": "combat/player_turn",
        "turn_index": 2,
        "host_ready": true,
        "generation": 4
    });
    assert!(!validator.is_valid(&unknown_with_observation));
}

#[test]
fn runtime_v2_case_and_manifest_bind_every_fixture_and_digest() {
    let case: Value = decode_json(CASE).expect("conformance case is JSON");
    let manifest: Value = decode_json(MANIFEST).expect("manifest is JSON");
    assert_eq!(case["case_id"], "CT-RUNTIME-V2-001");
    assert_eq!(case["profile"], RUNTIME_V2_PROTOCOL_VERSION);
    assert_eq!(case["schema"], RUNTIME_V2_SCHEMA_SOURCE);
    assert_eq!(case["checksums"], "artifacts/runtime-v2/SHA256SUMS");
    assert_eq!(case["consumers"].as_array().map(Vec::len), Some(4));
    assert_eq!(case["fixtures"].as_array().map(Vec::len), Some(9));
    assert_eq!(
        case["contract_assertions"]["action_id"],
        RUNTIME_V2_ACTION_ID
    );
    assert_eq!(
        case["contract_assertions"]["settlement_witness"],
        RUNTIME_V2_EFFECT_KIND
    );
    assert_eq!(
        case["contract_assertions"]["outcomes"],
        json!(["accepted", "settled", "rejected", "unknown", "cancelled"])
    );
    assert_eq!(
        case["contract_assertions"]["transition"]["before"]["generation"],
        json!(4)
    );
    assert_eq!(
        case["contract_assertions"]["transition"]["after"]["generation"],
        json!(5)
    );
    assert_eq!(manifest["artifact"], RUNTIME_V2_ARTIFACT);
    assert_eq!(manifest["protocol_version"], RUNTIME_V2_PROTOCOL_VERSION);
    assert_eq!(manifest["schema"], "schema.json");
    assert_eq!(manifest["schema_digest"], RUNTIME_V2_SCHEMA_DIGEST);
    assert_eq!(manifest["provenance"]["source"], RUNTIME_V2_SCHEMA_SOURCE);
    assert_eq!(manifest["provenance"]["generator"], "hand-authored");
    assert_eq!(manifest["provenance"]["license"], "MIT");
    assert_eq!(
        manifest["goldens"].as_array().map(Vec::len),
        Some(GOLDENS.len())
    );
    assert_eq!(manifest["checksums"], "SHA256SUMS");
    assert_eq!(
        checksum_for(CHECKSUMS, "schema.json"),
        RUNTIME_V2_SCHEMA_DIGEST
    );
    assert_eq!(
        checksum_for(CHECKSUMS, "../../schemas/runtime-v2.schema.json"),
        RUNTIME_V2_SCHEMA_DIGEST
    );
}

#[test]
fn runtime_v2_fixtures_bind_transition_lifecycle_and_idempotency() {
    let initial = message("state-response");
    assert_eq!(initial.observation.unwrap().generation, 4);
    assert_eq!(
        initial.observation.unwrap().combat_phase,
        RuntimeV2CombatPhase::PlayerTurn
    );
    let accepted = message("legal-action-accepted");
    assert_eq!(accepted.status, Some(RuntimeV2Status::Accepted));
    assert_eq!(accepted.observation.unwrap().turn_index, 2);
    assert!(accepted.effect_witness.is_none());

    let settled = message("legal-action-settled");
    let observation = settled.observation.unwrap();
    assert_eq!(settled.status, Some(RuntimeV2Status::Settled));
    assert_eq!(settled.operation_id.as_deref(), Some("op-1"));
    assert_eq!(observation.combat_phase, RuntimeV2CombatPhase::PlayerTurn);
    assert_eq!(observation.turn_index, 3);
    assert_eq!(observation.generation, 5);
    assert_eq!(settled.effect_witness.unwrap().kind, RUNTIME_V2_EFFECT_KIND);
    assert_eq!(
        serde_json::from_str::<Value>(golden("duplicate-replay")).unwrap(),
        serde_json::from_str::<Value>(golden("legal-action-settled")).unwrap()
    );

    let stale_request = message("stale-generation-request");
    assert_eq!(stale_request.generation, 3);
    assert_eq!(
        message("stale-generation-response").error_code.as_deref(),
        Some("sts2.game-core/stale_generation")
    );
    assert_eq!(
        message("outside-combat-response")
            .observation
            .unwrap()
            .combat_phase,
        RuntimeV2CombatPhase::OutsideCombat
    );
    assert_eq!(
        message("enemy-turn-response").error_code.as_deref(),
        Some("sts2.game-core/not_player_turn")
    );
    let conflict = message("idempotency-conflict-response");
    assert_eq!(conflict.operation_id, settled.operation_id);
    assert_eq!(conflict.error_code.as_deref(), Some("idempotency_conflict"));
    assert_eq!(
        message("cancelled-before-dispatch").status,
        Some(RuntimeV2Status::Cancelled)
    );
    let unknown = message("timeout-unknown-response");
    assert_eq!(unknown.status, Some(RuntimeV2Status::Unknown));
    assert!(unknown.observation.is_none());
    assert_eq!(unknown.operation_id.as_deref(), Some("op-timeout"));
    let reconcile_request = message("reconcile-request");
    let reconcile = message("reconcile-settled-response");
    assert_eq!(
        reconcile_request.kind,
        RuntimeV2MessageKind::ReconcileRequest
    );
    assert_eq!(reconcile.operation_id, unknown.operation_id);
    assert_eq!(reconcile.status, Some(RuntimeV2Status::Settled));
    assert_eq!(reconcile.observation.unwrap().generation, 5);
}

#[test]
fn runtime_v2_constructors_preserve_metadata_and_result_shape() {
    let metadata = RuntimeV2Metadata::new();
    let action = RuntimeV2Action {
        action_id: RUNTIME_V2_ACTION_ID.to_owned(),
    };
    let request = RuntimeV2Message::action_request(
        metadata.clone(),
        RuntimeV2Context::new(
            "corr-test",
            "instance-test",
            "session-test",
            "lease-test",
            1,
            4,
        ),
        "op-test",
        action.clone(),
    );
    request.validate().expect("action request is valid");
    let settled = RuntimeV2Message::result(
        metadata,
        RuntimeV2Context::new(
            "corr-test",
            "instance-test",
            "session-test",
            "lease-test",
            1,
            5,
        ),
        "op-test",
        action,
        RuntimeV2ActionResult {
            status: RuntimeV2Status::Settled,
            observation: Some(RuntimeV2Observation {
                combat_phase: RuntimeV2CombatPhase::PlayerTurn,
                turn_index: 3,
                host_ready: true,
                generation: 5,
            }),
            error_code: None,
            effect_witness: Some(sts2_protocol::RuntimeV2EffectWitness {
                kind: RUNTIME_V2_EFFECT_KIND.to_owned(),
                generation: 5,
            }),
        },
        RuntimeV2MessageKind::ReconcileResponse,
    );
    settled.validate().expect("settled receipt is valid");
    assert_eq!(settled.generation, 5);
    assert_eq!(settled.schema_digest, RUNTIME_V2_SCHEMA_DIGEST);
    assert!(RUNTIME_V2_MAX_GENERATION > settled.generation);
}
