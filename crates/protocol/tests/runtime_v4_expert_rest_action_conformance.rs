// SPDX-License-Identifier: MIT

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use sha2::{Digest, Sha256};
use sts2_protocol::{
    RUNTIME_V4_EXPERT_ACTION_SCHEMA_DIGEST, RUNTIME_V4_EXPERT_REST_ACTION_ARTIFACT,
    RUNTIME_V4_EXPERT_REST_ACTION_EFFECT_WITNESS_VERSION, RUNTIME_V4_EXPERT_REST_ACTION_GENERATOR,
    RUNTIME_V4_EXPERT_REST_ACTION_PROFILE, RUNTIME_V4_EXPERT_REST_ACTION_PROTOCOL_VERSION,
    RUNTIME_V4_EXPERT_REST_ACTION_SCHEMA_DIGEST, RUNTIME_V4_EXPERT_REST_ACTION_SCHEMA_SOURCE,
    RUNTIME_V4_EXPERT_SCHEMA_DIGEST, decode_json,
};

const SOURCE_SCHEMA: &str =
    include_str!("../../../schemas/runtime-v4-expert-rest-action-v1.schema.json");
const ARTIFACT_SCHEMA: &str =
    include_str!("../../../artifacts/runtime-v4-expert-rest-action/schema.json");
const EXPERT_SCHEMA: &str = include_str!("../../../schemas/runtime-v4-expert.schema.json");
const CASE: &str = include_str!("../../../conformance/cases/runtime-v4-expert-rest-action-v1.json");
const MANIFEST: &str =
    include_str!("../../../artifacts/runtime-v4-expert-rest-action/manifest.json");
const CHECKSUMS: &str = include_str!("../../../artifacts/runtime-v4-expert-rest-action/SHA256SUMS");

fn fixture(relative: impl AsRef<Path>) -> Value {
    let path = artifact_root().join(relative);
    let text =
        fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    decode_json(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn golden(name: &str) -> Value {
    fixture(Path::new("golden").join(name))
}

fn mutation(name: &str) -> Value {
    fixture(Path::new("../../conformance/mutations/runtime-v4-expert-rest-action-v1").join(name))
}

fn fixture_names(relative: &str) -> Vec<String> {
    let directory = artifact_root().join(relative);
    let mut names: Vec<_> = fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}

fn request_actions() -> BTreeMap<String, Value> {
    fixture_names("golden")
        .into_iter()
        .map(|name| (name.clone(), golden(&name)))
        .filter(|(_, value)| value["kind"] == "action_request")
        .map(|(_, value)| {
            (
                value["operation_id"].as_str().unwrap().to_owned(),
                value["action"].clone(),
            )
        })
        .collect()
}

fn object<'a>(value: &'a Value, field: &str) -> Option<&'a Value> {
    value.get(field).filter(|candidate| candidate.is_object())
}

fn string<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value.get(field)?.as_str()
}

fn u64_value(value: &Value, field: &str) -> Option<u64> {
    value.get(field)?.as_u64()
}

fn schema_validator(text: &str) -> jsonschema::Validator {
    let schema: Value = decode_json(text).expect("schema is valid JSON");
    jsonschema::draft202012::options()
        .build(&schema)
        .expect("schema compiles as Draft 2020-12")
}

fn nested_observation_is_valid(value: &Value, expert: &jsonschema::Validator) -> bool {
    value
        .get("observation")
        .is_some_and(|observation| observation.is_null() || expert.is_valid(observation))
}

fn action_matches_transition(value: &Value, transition: &Value) -> Option<bool> {
    let action_payload = object(value.get("action")?, "action")?;
    let action_kind = string(action_payload, "kind")?;
    let transition_kind = string(transition, "kind")?;
    let transition_option = string(transition, "rest_option_id")?;
    if action_payload.get("rest_option_id").and_then(Value::as_str) != Some(transition_option) {
        return Some(false);
    }
    Some(match transition_kind {
        "rest_option_selection_requested" => action_kind == "rest_option",
        "rest_option_selection_progressed" => {
            action_kind
                == match string(transition, "selection_kind") {
                    Some("card") => "select_card",
                    Some("player") => "select_player",
                    _ => return Some(false),
                }
                && action_payload.get("selection_id").and_then(Value::as_str)
                    == string(transition, "selection_id")
        }
        "rest_option_selection_completed" => {
            action_kind == "confirm_selection"
                && action_payload.get("selection_id").and_then(Value::as_str)
                    == string(transition, "selection_id")
        }
        "rest_option_completed" => action_kind == "rest_option",
        _ => false,
    })
}

fn selector_is_consistent(value: &Value, transition: &Value, selector: &Value) -> Option<bool> {
    if string(transition, "kind") == Some("rest_option_selection_progressed") {
        for field in [
            "selection_id",
            "selection_kind",
            "required_count",
            "selected_choice_ids",
            "remaining_count",
        ] {
            if transition.get(field) != selector.get(field) {
                return Some(false);
            }
        }
    }
    let required = u64_value(selector, "required_count").unwrap_or(0);
    let selected = selector
        .get("selected_choice_ids")
        .and_then(Value::as_array)
        .map_or(0, Vec::len) as u64;
    let remaining = u64_value(selector, "remaining_count").unwrap_or(u64::MAX);
    if selected > required || remaining != required.saturating_sub(selected) {
        return Some(false);
    }
    let selection_kind = string(selector, "selection_kind");
    let selection_id = string(selector, "selection_id");
    let option_id = string(transition, "rest_option_id");
    let mut action_ids = BTreeSet::new();
    let legal_actions = selector.get("legal_actions")?.as_array()?;
    for legal in legal_actions {
        let legal_id = string(legal, "action_id")?;
        if !action_ids.insert(legal_id) {
            return Some(false);
        }
        let payload = object(legal, "action")?;
        let kind = string(payload, "kind")?;
        if !matches!(
            kind,
            "confirm_selection" | "cancel_selection" | "select_card" | "select_player"
        ) {
            return Some(false);
        }
        if string(payload, "selection_id") != selection_id
            || string(payload, "rest_option_id") != option_id
        {
            return Some(false);
        }
        if matches!(kind, "select_card" | "select_player")
            && Some(kind.strip_prefix("select_")?) != selection_kind
        {
            return Some(false);
        }
    }
    let has_confirm = legal_actions.iter().any(|item| {
        item.get("action")
            .and_then(|action| action.get("kind"))
            .and_then(Value::as_str)
            == Some("confirm_selection")
    });
    if (remaining == 0) != has_confirm {
        return Some(false);
    }
    let action_payload = object(value.get("action")?, "action")?;
    Some(
        string(action_payload, "selection_id") == selection_id
            || string(action_payload, "kind") == Some("rest_option"),
    )
}

fn selected_cards_are_visible(value: &Value, transition: &Value) -> Option<bool> {
    if string(transition, "selection_kind") != Some("card") {
        return Some(
            string(transition, "selection_kind") == Some("player")
                && transition
                    .get("selected_choice_ids")
                    .and_then(Value::as_array)
                    .is_some_and(|ids| {
                        ids.iter()
                            .all(|id| id.as_str().is_some_and(|id| id.starts_with("player:")))
                    }),
        );
    }
    let visible_cards = value
        .get("observation")
        .and_then(|observation| observation.get("player"))
        .and_then(|player| player.get("deck"))
        .and_then(Value::as_array)?;
    Some(
        transition
            .get("selected_choice_ids")
            .and_then(Value::as_array)?
            .iter()
            .all(|selected| {
                visible_cards
                    .iter()
                    .any(|card| card.get("card_id") == Some(selected))
            }),
    )
}

fn effect_matches_option(value: &Value, transition: &Value) -> Option<bool> {
    let option = string(transition, "rest_option_id");
    let expected_kind = match option {
        Some("heal") => "heal_applied",
        Some("smith") => "smith_applied",
        Some("mend") => "mend_applied",
        _ => return Some(true),
    };
    let root = value.get("effect_witness")?;
    let nested = transition.get("effect_witness")?;
    if root != nested {
        return Some(false);
    }
    if string(root, "version") != Some(RUNTIME_V4_EXPERT_REST_ACTION_EFFECT_WITNESS_VERSION)
        || string(root, "kind") != Some(expected_kind)
        || string(root, "operation_id") != string(value, "operation_id")
        || string(root, "rest_option_id") != option
        || u64_value(root, "generation") != u64_value(value, "generation")
    {
        return Some(false);
    }
    if expected_kind == "smith_applied" {
        return Some(
            root.get("evidence")
                .and_then(|evidence| evidence.get("upgraded_card_ids"))
                == transition.get("selected_choice_ids"),
        );
    }
    if expected_kind == "mend_applied" {
        return Some(
            root.get("target_player_id")
                == transition
                    .get("selected_choice_ids")
                    .and_then(Value::as_array)
                    .and_then(|ids| ids.first()),
        );
    }
    Some(
        root.get("evidence")
            .and_then(|evidence| evidence.get("kind"))
            .and_then(Value::as_str)
            == Some("hp_change"),
    )
}

fn strict_semantics(value: &Value, expected_actions: &BTreeMap<String, Value>) -> Option<bool> {
    if string(value, "protocol_version") != Some(RUNTIME_V4_EXPERT_REST_ACTION_PROTOCOL_VERSION)
        || string(value, "profile") != Some(RUNTIME_V4_EXPERT_REST_ACTION_PROFILE)
        || string(value, "schema_digest") != Some(RUNTIME_V4_EXPERT_REST_ACTION_SCHEMA_DIGEST)
        || value.get("provenance")
            != Some(&serde_json::json!({
                "artifact": RUNTIME_V4_EXPERT_REST_ACTION_ARTIFACT,
                "source": RUNTIME_V4_EXPERT_REST_ACTION_SCHEMA_SOURCE,
                "generator": RUNTIME_V4_EXPERT_REST_ACTION_GENERATOR
            }))
    {
        return Some(false);
    }
    if let Some(expected) = expected_actions.get(string(value, "operation_id").unwrap())
        && value.get("action") != Some(expected)
    {
        return Some(false);
    }
    if string(value, "kind") != Some("action_response")
        || string(value, "status") != Some("settled")
    {
        return Some(true);
    }
    let observation = value.get("observation")?;
    if observation.get("state_id") != value.get("state_id")
        || observation.get("generation") != value.get("generation")
        || string(observation, "schema_digest") != Some(RUNTIME_V4_EXPERT_SCHEMA_DIGEST)
    {
        return Some(false);
    }
    let transition = value.get("transition")?;
    let before = u64_value(transition, "before_generation")?;
    let after = u64_value(transition, "after_generation")?;
    if before >= after
        || Some(after) != u64_value(value, "generation")
        || !action_matches_transition(value, transition).is_some_and(|matches| matches)
    {
        return Some(false);
    }
    Some(match string(transition, "kind")? {
        "rest_option_selection_requested" | "rest_option_selection_progressed" => {
            if value.get("effect_witness") != Some(&Value::Null)
                || transition.get("effect_witness") != Some(&Value::Null)
            {
                return Some(false);
            }
            selector_is_consistent(value, transition, transition.get("selector")?)?
        }
        "rest_option_selection_completed" => {
            let selected = transition.get("selected_choice_ids")?.as_array()?;
            let required = u64_value(transition, "required_count")? as usize;
            u64_value(transition, "remaining_count")? == 0
                && selected.len() == required
                && selected_cards_are_visible(value, transition)?
                && effect_matches_option(value, transition)?
        }
        "rest_option_completed" => effect_matches_option(value, transition)?,
        _ => false,
    })
}

fn artifact_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("artifacts/runtime-v4-expert-rest-action")
}

#[test]
fn metadata_case_and_manifest_pin_candidate_identity_without_claiming_consumers() {
    let case: Value = decode_json(CASE).expect("conformance case is JSON");
    let manifest: Value = decode_json(MANIFEST).expect("manifest is JSON");
    assert_eq!(case["status"], "candidate");
    assert_eq!(case["consumers"], serde_json::json!([]));
    assert_eq!(manifest["status"], "candidate");
    assert_eq!(manifest["consumers"], serde_json::json!([]));
    assert_eq!(
        manifest["schema_digest"],
        RUNTIME_V4_EXPERT_REST_ACTION_SCHEMA_DIGEST
    );
    assert_eq!(
        case["contract_assertions"]["old_potion_profile_unchanged"],
        true
    );
    assert_eq!(
        case["contract_assertions"]["old_potion_schema_digest"],
        RUNTIME_V4_EXPERT_ACTION_SCHEMA_DIGEST
    );
    assert_eq!(case["http_assignment"]["request"]["method"], "POST");
    assert_eq!(case["http_assignment"]["reconcile"]["method"], "GET");
    assert_eq!(case["http_assignment"]["reconcile"]["body"], Value::Null);
    assert_eq!(case["http_assignment"]["status_mapping"]["202"], "accepted");
    assert_eq!(case["http_assignment"]["status_mapping"]["200"], "settled");
}

#[test]
fn settled_goldens_enforce_cross_field_identity_counts_and_witnesses() {
    assert_eq!(SOURCE_SCHEMA.as_bytes(), ARTIFACT_SCHEMA.as_bytes());
    let schema = schema_validator(SOURCE_SCHEMA);
    let expert = schema_validator(EXPERT_SCHEMA);
    let requests = request_actions();
    let names = fixture_names("golden");
    for name in names {
        let value = golden(&name);
        assert!(schema.is_valid(&value), "{name} schema");
        assert!(
            nested_observation_is_valid(&value, &expert),
            "{name} observation"
        );
        assert!(
            strict_semantics(&value, &requests).is_some_and(|valid| valid),
            "{name} strict semantics"
        );
    }
}

#[test]
fn independent_mutations_are_rejected_by_schema_or_semantic_conformance() {
    let schema = schema_validator(SOURCE_SCHEMA);
    let requests = request_actions();
    let names = fixture_names("../../conformance/mutations/runtime-v4-expert-rest-action-v1");
    for name in names {
        let value = mutation(&name);
        if schema.is_valid(&value) {
            assert!(
                !strict_semantics(&value, &requests).is_some_and(|valid| valid),
                "{name} bypassed strict semantics"
            );
        }
    }
}

#[test]
fn checksum_inventory_covers_schema_case_manifest_goldens_and_mutations() {
    let root = artifact_root();
    for line in CHECKSUMS.lines().filter(|line| !line.trim().is_empty()) {
        let (expected, relative) = line.split_once("  ").expect("checksum has two columns");
        assert_eq!(expected.len(), 64, "{relative}");
        let bytes =
            fs::read(root.join(relative)).unwrap_or_else(|error| panic!("{relative}: {error}"));
        let actual = format!("{:x}", Sha256::digest(bytes));
        assert_eq!(actual, expected, "{relative}");
    }
    assert_eq!(
        CHECKSUMS
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count(),
        36
    );
    assert!(CHECKSUMS.contains("../../schemas/runtime-v4-expert-rest-action-v1.schema.json"));
    assert!(CHECKSUMS.contains("../../conformance/cases/runtime-v4-expert-rest-action-v1.json"));
}
