// SPDX-License-Identifier: MIT

use serde_json::{Value, json};
use sts2_protocol::{
    RUNTIME_V3_GAMEPLAY_ACTION_ID, RUNTIME_V3_GAMEPLAY_ARTIFACT, RUNTIME_V3_GAMEPLAY_EFFECT_KIND,
    RUNTIME_V3_GAMEPLAY_PROTOCOL_VERSION, RUNTIME_V3_GAMEPLAY_SCHEMA_DIGEST,
    RuntimeV3GameplayAction, RuntimeV3GameplayContext, RuntimeV3GameplayMessage,
    RuntimeV3GameplayObservation, RuntimeV3GameplayStatus, canonical_json, decode_json,
};

const CASE: &str = include_str!("../../../conformance/cases/runtime-v3-gameplay.json");
const MANIFEST: &str = include_str!("../../../artifacts/runtime-v3-gameplay/manifest.json");
const CHECKSUMS: &str = include_str!("../../../artifacts/runtime-v3-gameplay/SHA256SUMS");
const SOURCE_SCHEMA: &str = include_str!("../../../schemas/runtime-v3-gameplay.schema.json");
const ARTIFACT_SCHEMA: &str = include_str!("../../../artifacts/runtime-v3-gameplay/schema.json");

const GOLDENS: &[(&str, &str)] = &[
    (
        "state-request",
        include_str!("../../../artifacts/runtime-v3-gameplay/golden/state-request.json"),
    ),
    (
        "state-response",
        include_str!("../../../artifacts/runtime-v3-gameplay/golden/state-response.json"),
    ),
    (
        "action-request",
        include_str!("../../../artifacts/runtime-v3-gameplay/golden/action-request.json"),
    ),
    (
        "action-accepted",
        include_str!("../../../artifacts/runtime-v3-gameplay/golden/action-accepted.json"),
    ),
    (
        "action-settled",
        include_str!("../../../artifacts/runtime-v3-gameplay/golden/action-settled.json"),
    ),
    (
        "stale-response",
        include_str!("../../../artifacts/runtime-v3-gameplay/golden/stale-response.json"),
    ),
    (
        "reconcile-request",
        include_str!("../../../artifacts/runtime-v3-gameplay/golden/reconcile-request.json"),
    ),
    (
        "reconcile-settled",
        include_str!("../../../artifacts/runtime-v3-gameplay/golden/reconcile-settled.json"),
    ),
];

fn payload(text: &str) -> &str {
    text.strip_suffix('\n').unwrap_or(text)
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
fn v3_goldens_round_trip_and_preserve_the_new_action_shape() {
    for (_, text) in GOLDENS {
        let message: sts2_protocol::RuntimeV3GameplayMessage =
            decode_json(text).expect("Runtime-v3 gameplay golden is JSON");
        message
            .validate()
            .expect("Runtime-v3 gameplay golden validates");
        assert_eq!(
            canonical_json(&message).expect("encoding succeeds"),
            payload(text)
        );
    }
    let action: RuntimeV3GameplayAction = decode_json::<RuntimeV3GameplayMessage>(
        GOLDENS
            .iter()
            .find(|(name, _)| *name == "action-request")
            .map(|(_, text)| *text)
            .expect("action golden exists"),
    )
    .expect("action envelope decodes")
    .action
    .expect("action exists");
    assert_eq!(action.action_id, RUNTIME_V3_GAMEPLAY_ACTION_ID);
    assert_eq!(action.card_index, 0);
    assert_eq!(action.target_id.as_deref(), Some("enemy-1"));
}

#[test]
fn v3_schema_manifest_and_checksums_bind_the_profile() {
    let source: Value = decode_json(SOURCE_SCHEMA).expect("source schema is JSON");
    let artifact: Value = decode_json(ARTIFACT_SCHEMA).expect("artifact schema is JSON");
    assert_eq!(source, artifact);
    assert_eq!(SOURCE_SCHEMA.as_bytes(), ARTIFACT_SCHEMA.as_bytes());
    assert_eq!(source["$id"], "sts2-runtime-v3-gameplay");
    assert_eq!(
        source["$defs"]["action"]["properties"]["action_id"]["const"],
        RUNTIME_V3_GAMEPLAY_ACTION_ID
    );
    let validator = jsonschema::draft202012::options()
        .build(&source)
        .expect("Runtime-v3 gameplay schema compiles");
    for (_, text) in GOLDENS {
        let value: Value = decode_json(text).expect("golden is JSON");
        assert!(validator.is_valid(&value), "golden must satisfy the schema");
    }
    let mut unknown: Value = decode_json(GOLDENS[1].1).expect("state is JSON");
    unknown["unexpected"] = json!(true);
    assert!(!validator.is_valid(&unknown));

    let case: Value = decode_json(CASE).expect("conformance case is JSON");
    let manifest: Value = decode_json(MANIFEST).expect("manifest is JSON");
    assert_eq!(case["profile"], RUNTIME_V3_GAMEPLAY_PROTOCOL_VERSION);
    assert_eq!(case["consumers"].as_array().map(Vec::len), Some(5));
    assert_eq!(case["fixtures"].as_array().map(Vec::len), Some(4));
    assert_eq!(
        case["contract_assertions"]["settlement_witness"],
        RUNTIME_V3_GAMEPLAY_EFFECT_KIND
    );
    assert_eq!(manifest["artifact"], RUNTIME_V3_GAMEPLAY_ARTIFACT);
    assert_eq!(manifest["schema_digest"], RUNTIME_V3_GAMEPLAY_SCHEMA_DIGEST);
    assert_eq!(
        manifest["goldens"].as_array().map(Vec::len),
        Some(GOLDENS.len())
    );
    assert_eq!(
        checksum_for(CHECKSUMS, "schema.json"),
        RUNTIME_V3_GAMEPLAY_SCHEMA_DIGEST
    );
    assert_eq!(
        checksum_for(CHECKSUMS, "../../schemas/runtime-v3-gameplay.schema.json"),
        RUNTIME_V3_GAMEPLAY_SCHEMA_DIGEST
    );
}

#[test]
fn v3_constructors_preserve_fresh_settlement_and_reject_invalid_target_lists() {
    let context = RuntimeV3GameplayContext::new("corr", "instance", "session", "lease", 1, 4);
    let action = RuntimeV3GameplayAction::play_card(0, Some(String::from("enemy-1")));
    let request = RuntimeV3GameplayMessage::action_request(
        Default::default(),
        context.clone(),
        "operation",
        action.clone(),
    );
    request.validate().expect("action request is valid");
    let observation = RuntimeV3GameplayObservation {
        combat_phase: sts2_protocol::RuntimeV3GameplayCombatPhase::PlayerTurn,
        turn_index: 2,
        host_ready: true,
        generation: 5,
        hand_count: 4,
        energy: 1,
        draw_pile_count: 10,
        discard_pile_count: 1,
        exhaust_pile_count: 0,
        enemies: vec![sts2_protocol::RuntimeV3GameplayEnemy {
            target_id: String::from("enemy-1"),
            alive: true,
            hittable: true,
        }],
    };
    let settled = RuntimeV3GameplayMessage::result(
        Default::default(),
        RuntimeV3GameplayContext::new("corr", "instance", "session", "lease", 1, 5),
        "operation",
        action,
        sts2_protocol::RuntimeV3GameplayActionResult {
            status: RuntimeV3GameplayStatus::Settled,
            observation: Some(observation),
            error_code: None,
            effect_witness: Some(sts2_protocol::RuntimeV3GameplayEffectWitness {
                kind: String::from(RUNTIME_V3_GAMEPLAY_EFFECT_KIND),
                generation: 5,
                card_index: 0,
                target_id: Some(String::from("enemy-1")),
            }),
        },
        sts2_protocol::RuntimeV3GameplayMessageKind::ActionResponse,
    );
    settled.validate().expect("settled result is valid");
    assert_eq!(settled.status, Some(RuntimeV3GameplayStatus::Settled));

    let mut invalid = settled;
    invalid
        .observation
        .as_mut()
        .expect("observation exists")
        .enemies
        .push(sts2_protocol::RuntimeV3GameplayEnemy {
            target_id: String::from("enemy-1"),
            alive: true,
            hittable: true,
        });
    assert!(invalid.validate().is_err());
}
