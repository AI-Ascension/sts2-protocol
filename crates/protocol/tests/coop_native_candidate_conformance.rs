// SPDX-License-Identifier: MIT

use serde::de::{DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fmt;

const SOURCE_SCHEMA: &str = include_str!("../../../schemas/coop-native-v1.schema.json");
const ARTIFACT_SCHEMA: &str = include_str!("../../../artifacts/coop-native-v1/schema.json");
const MANIFEST: &str = include_str!("../../../artifacts/coop-native-v1/manifest.json");
const CASES: &str = include_str!("../../../artifacts/coop-native-v1/conformance.json");
const ROOT_CASES: &str = include_str!("../../../conformance/cases/coop-native-v1.json");
const PRODUCER_CAPTURE: &str =
    include_str!("../../../artifacts/coop-native-v1/producer-capture.json");

macro_rules! golden {
    ($name:literal) => {
        (
            concat!("golden/", $name),
            include_str!(concat!("../../../artifacts/coop-native-v1/golden/", $name)),
        )
    };
}

const GOLDENS: &[(&str, &str)] = &[
    golden!("observation-response.json"),
    golden!("legal-catalog-request.json"),
    golden!("legal-catalog-response.json"),
    golden!("local-action-settled-request.json"),
    golden!("local-action-settled-response.json"),
    golden!("local-action-rejected-request.json"),
    golden!("local-action-rejected-response.json"),
    golden!("local-action-unknown-request.json"),
    golden!("local-action-unknown-response.json"),
    golden!("local-action-recovered-request.json"),
    golden!("local-action-recovered-response.json"),
    golden!("shared-vote-settled-request.json"),
    golden!("shared-vote-settled-response.json"),
    golden!("rejoin-pending-request.json"),
    golden!("rejoin-pending-response.json"),
    golden!("rejoin-recovered-request.json"),
    golden!("rejoin-recovered-response.json"),
];

struct StrictValue;

impl<'de> DeserializeSeed<'de> for StrictValue {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for StrictValue {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value with unique object members")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Value, E>
    where
        E: serde::de::Error,
    {
        serde_json::Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Value, E> {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Value, E> {
        Ok(Value::String(value))
    }

    fn visit_unit<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_none<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        StrictValue.deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element_seed(StrictValue)? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(<A::Error as serde::de::Error>::custom(format!(
                    "duplicate object member {key}"
                )));
            }
            values.insert(key, map.next_value_seed(StrictValue)?);
        }
        Ok(Value::Object(values))
    }
}

fn strict_json(text: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let mut deserializer = serde_json::Deserializer::from_str(text);
    let value = deserializer.deserialize_any(StrictValue)?;
    deserializer.end()?;
    Ok(value)
}

fn validator() -> jsonschema::Validator {
    let schema: Value = serde_json::from_str(SOURCE_SCHEMA).expect("candidate schema is JSON");
    jsonschema::draft202012::new(&schema).expect("candidate schema compiles")
}

fn fixture(path: &str) -> &'static str {
    GOLDENS
        .iter()
        .find_map(|(listed, text)| (*listed == path).then_some(*text))
        .unwrap_or_else(|| panic!("unknown fixture {path}"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn source_artifact_manifest_and_cases_are_bound() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(SOURCE_SCHEMA, ARTIFACT_SCHEMA);
    assert_eq!(CASES, ROOT_CASES);
    let manifest: Value = strict_json(MANIFEST)?;
    assert_eq!(manifest["artifact"], "sts2-protocol/coop-native-v1");
    assert_eq!(manifest["status"], "accepted_component");
    assert_eq!(manifest["admission"], "component");
    assert_eq!(
        manifest["consumers"],
        json!(["sts2-gateway", "sts2-mcp-server", "sts2-harness"])
    );
    assert_eq!(
        manifest["schema_digest"],
        "2f3bc99e53080fa11b39592b64fb0ab964a16f568719a2622d0b2caf766ab629"
    );
    assert_eq!(
        manifest["producer_declared_schema_digest"],
        "2f3bc99e53080fa11b39592b64fb0ab964a16f568719a2622d0b2caf766ab629"
    );
    assert_eq!(manifest["producer_digest_matches_candidate"], true);
    assert_eq!(manifest["producer_capture"], "producer-capture.json");
    assert_eq!(
        manifest["producer_source_commit"],
        "ee2d834482c6a410cf223e6e1d25887d2c6e4b5c"
    );
    assert_eq!(
        manifest["producer_source_tree"],
        "8db34601298af4abfe9f97ca831ccc0ca62b50ca"
    );
    let capture: Value = strict_json(PRODUCER_CAPTURE)?;
    assert_eq!(
        capture["source"]["commit"],
        manifest["producer_source_commit"]
    );
    assert_eq!(
        capture["producer"]["declared_schema_digest"],
        manifest["producer_declared_schema_digest"]
    );
    let captures = capture["captures"].as_array().ok_or("missing captures")?;
    let mut mapped = HashSet::new();
    for capture in captures {
        assert_eq!(capture["wrapper_sha256"].as_str().map(str::len), Some(64));
        let members = capture["members"]
            .as_array()
            .ok_or("missing capture members")?;
        for member in members {
            let source_pointer = member["source_pointer"]
                .as_str()
                .ok_or("missing source pointer")?;
            assert!(matches!(
                source_pointer,
                "/capture/request" | "/capture/response"
            ));
            let path = member["golden"].as_str().ok_or("missing mapped golden")?;
            assert!(
                mapped.insert(path.to_owned()),
                "duplicate mapped golden {path}"
            );
            let expected = member["golden_sha256"]
                .as_str()
                .ok_or("missing golden hash")?;
            assert_eq!(sha256_hex(fixture(path).as_bytes()), expected, "{path}");
        }
    }
    assert_eq!(mapped.len(), GOLDENS.len());
    let manifest_goldens: HashSet<&str> = manifest["goldens"]
        .as_array()
        .ok_or("missing manifest goldens")?
        .iter()
        .map(|path| path.as_str().ok_or("non-string manifest golden"))
        .collect::<Result<_, _>>()?;
    assert_eq!(
        manifest_goldens,
        mapped.iter().map(String::as_str).collect()
    );
    let cases: Value = strict_json(CASES)?;
    let cases = cases["cases"].as_array().ok_or("missing cases")?;
    assert_eq!(cases.len(), GOLDENS.len());
    let check = validator();
    for case in cases {
        let path = case["fixture"].as_str().ok_or("missing fixture")?;
        let value = strict_json(fixture(path))?;
        assert_eq!(
            check.is_valid(&value),
            case["schema_valid"] == true,
            "{path}"
        );
        assert!(
            check.is_valid(&value),
            "candidate fixture must validate: {path}"
        );
    }
    Ok(())
}

#[test]
fn all_captured_envelopes_are_strict_json_and_schema_valid() {
    let check = validator();
    for (path, text) in GOLDENS {
        let value = strict_json(text).unwrap_or_else(|error| panic!("{path}: {error}"));
        assert!(check.is_valid(&value), "schema rejected {path}");
    }
}

#[test]
fn schema_rejects_unknown_missing_and_conflicting_shapes() {
    let check = validator();
    let base = strict_json(fixture("golden/local-action-settled-request.json")).unwrap();
    let mut unknown = base.clone();
    unknown["unexpected"] = json!(true);
    assert!(!check.is_valid(&unknown));

    let mut missing = base.clone();
    missing.as_object_mut().unwrap().remove("status");
    assert!(!check.is_valid(&missing));

    let mut nested = base.clone();
    nested["action"]["private_rng"] = json!(1);
    assert!(!check.is_valid(&nested));

    let mut conflict = base;
    conflict["vote"] =
        json!({"proposal_id":"event:campfire","voter_peer":"peer:host1","choice":"index:1"});
    assert!(!check.is_valid(&conflict));

    let mut rejected = strict_json(fixture("golden/local-action-rejected-response.json")).unwrap();
    rejected["effect"] = json!({"effect_id":"effect:invalid"});
    assert!(!check.is_valid(&rejected));

    let mut non_peer = strict_json(fixture("golden/local-action-settled-request.json")).unwrap();
    non_peer["actor_peer"] = json!("actor:host1");
    assert!(!check.is_valid(&non_peer));

    let mut bad_recovery =
        strict_json(fixture("golden/local-action-recovered-response.json")).unwrap();
    bad_recovery["recovery"]["kind"] = json!("rejoin");
    assert!(!check.is_valid(&bad_recovery));

    let mut unsupported_action =
        strict_json(fixture("golden/local-action-settled-request.json")).unwrap();
    unsupported_action["action"]["kind"] = json!("legacy_action");
    assert!(!check.is_valid(&unsupported_action));

    let mut unsupported_effect =
        strict_json(fixture("golden/local-action-settled-response.json")).unwrap();
    unsupported_effect["effect"]["kind"] = json!("native_legacy_settled");
    assert!(!check.is_valid(&unsupported_effect));
}

#[test]
fn source_action_effect_and_checksum_vocabularies_are_schema_bound() {
    let check = validator();
    let mut action = strict_json(fixture("golden/local-action-settled-request.json")).unwrap();
    for kind in [
        "play_card",
        "end_turn",
        "select_card",
        "choose_reward",
        "confirm_selection",
    ] {
        action["action"]["kind"] = json!(kind);
        assert!(check.is_valid(&action), "action kind {kind}");
    }

    let mut effect = strict_json(fixture("golden/local-action-settled-response.json")).unwrap();
    for kind in [
        "turn_ended",
        "shared_event_vote",
        "native_end_turn_settled",
        "native_play_card_settled",
        "native_shared_event_vote_settled",
        "native_treasure_relic_vote_settled",
    ] {
        effect["effect"]["kind"] = json!(kind);
        assert!(check.is_valid(&effect), "effect kind {kind}");
    }

    let mut observation = strict_json(fixture("golden/observation-response.json")).unwrap();
    for status in [
        "available",
        "unavailable",
        "unknown",
        "disabled",
        "enabled_unread",
        "enabled",
        "divergent",
        "matched",
    ] {
        observation["observation"]["checksum_status"] = json!(status);
        observation["observation"]["peers"][0]["checksum_status"] = json!(status);
        assert!(check.is_valid(&observation), "checksum status {status}");
    }
    observation["observation"]["peers"][1]["checkpoint_id"] = Value::Null;
    assert!(check.is_valid(&observation), "unread remote checkpoint");
}

#[test]
fn outcome_and_recovery_vectors_preserve_operation_identity_and_relations() {
    {
        let path = "golden/local-action-unknown-response.json";
        let value = strict_json(fixture(path)).unwrap();
        assert_eq!(value["operation_id"], "op:native:action:unknown");
        assert_eq!(value["kind"], "effect_response");
        assert_eq!(value["status"], "unknown");
        assert!(value["observation"].is_object());
        assert!(value["recovery"].is_null());
    }

    for path in [
        "golden/local-action-recovered-response.json",
        "golden/rejoin-pending-response.json",
        "golden/rejoin-recovered-response.json",
    ] {
        let value = strict_json(fixture(path)).unwrap();
        let expected_operation = if path.contains("rejoin") {
            "op:native:rejoin"
        } else {
            "op:native:action:unknown"
        };
        assert_eq!(value["operation_id"], expected_operation);
        assert_eq!(value["kind"], "recovery_response");
        assert!(matches!(
            value["status"].as_str(),
            Some("settled" | "unknown")
        ));
        assert!(value["observation"].is_object());
        assert!(matches!(
            value["recovery"]["kind"].as_str(),
            Some("rejoin" | "reconcile")
        ));
    }

    for path in [
        "golden/local-action-settled-response.json",
        "golden/shared-vote-settled-response.json",
    ] {
        let value = strict_json(fixture(path)).unwrap();
        let observation = &value["observation"];
        let effect = &value["effect"];
        assert_eq!(value["kind"], "effect_response");
        assert_eq!(value["status"], "settled");
        assert_eq!(effect["operation_id"], value["operation_id"]);
        assert_eq!(effect["to_generation"], observation["host_generation"]);
        assert_eq!(effect["state_digest"], observation["state_digest"]);
        assert_eq!(effect["authority_id"], observation["authority_id"]);
        assert_eq!(
            effect["authority_epoch"],
            observation["host_authority_epoch"]
        );
        assert_eq!(effect["checkpoint_id"], observation["checkpoint_id"]);
        assert!(
            effect["from_generation"].as_u64().unwrap() < effect["to_generation"].as_u64().unwrap()
        );
        let peers = observation["peers"].as_array().unwrap();
        assert_eq!(
            peers.iter().filter(|peer| peer["role"] == "local").count(),
            1
        );
        for peer in peers {
            assert_eq!(peer["authority_id"], observation["authority_id"]);
            assert_eq!(peer["checkpoint_id"], observation["checkpoint_id"]);
            assert_eq!(peer["state_digest"], observation["state_digest"]);
        }
    }
}
