// SPDX-License-Identifier: MIT

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use super::*;

pub(crate) fn fixture(relative: impl AsRef<Path>) -> Value {
    let path = artifact_root().join(relative);
    let text =
        fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    decode_json(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

pub(crate) fn golden(name: &str) -> Value {
    fixture(Path::new("golden").join(name))
}

pub(crate) fn mutation(name: &str) -> Value {
    fixture(Path::new("../../conformance/mutations/runtime-v4-expert-rest-action-v1").join(name))
}

pub(crate) fn fixture_names(relative: &str) -> Vec<String> {
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

fn object<'a>(value: &'a Value, field: &str) -> Option<&'a Value> {
    value.get(field).filter(|candidate| candidate.is_object())
}

fn string<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value.get(field)?.as_str()
}

fn u64_value(value: &Value, field: &str) -> Option<u64> {
    value.get(field)?.as_u64()
}

pub(crate) fn schema_validator(text: &str) -> jsonschema::Validator {
    let schema: Value = decode_json(text).expect("schema is valid JSON");
    jsonschema::draft202012::options()
        .build(&schema)
        .expect("schema compiles as Draft 2020-12")
}

pub(crate) fn nested_observation_is_valid(value: &Value, expert: &jsonschema::Validator) -> bool {
    value
        .get("observation")
        .is_some_and(|observation| observation.is_null() || expert.is_valid(observation))
}

pub(crate) const PRODUCER_FIXTURES: [&str; 2] = [
    "smith-selection-lifecycle.json",
    "mend-selection-lifecycle.json",
];

#[derive(Debug)]
pub(crate) struct SelectorAdmission {
    option_id: String,
    selection_kind: String,
    required_count: u64,
    choice_ids: BTreeSet<String>,
}

pub(crate) fn producer_fixture(name: &str) -> Value {
    fixture(Path::new("producer").join(name))
}

pub(crate) fn request_actions() -> BTreeMap<String, Value> {
    let mut actions = BTreeMap::new();
    for name in fixture_names("golden") {
        let value = golden(&name);
        if value["kind"] == "action_request" {
            actions.insert(
                value["operation_id"].as_str().unwrap().to_owned(),
                value["action"].clone(),
            );
        }
    }
    for name in PRODUCER_FIXTURES {
        for step in producer_fixture(name)["messages"].as_array().unwrap() {
            let request = &step["request"];
            actions.insert(
                request["operation_id"].as_str().unwrap().to_owned(),
                request["action"].clone(),
            );
        }
    }
    actions
}

pub(crate) fn ids(value: &Value, field: &str) -> Option<Vec<String>> {
    value
        .get(field)?
        .as_array()?
        .iter()
        .map(|item| item.as_str().map(str::to_owned))
        .collect()
}

fn option_witness_kind(option: &str) -> Option<&'static str> {
    Some(match option {
        "clone" => "clone_applied",
        "cook" => "cook_applied",
        "dig" => "dig_applied",
        "hatch" => "hatch_applied",
        "heal" => "heal_applied",
        "kindle" => "kindle_applied",
        "lift" => "lift_applied",
        "smith" => "smith_applied",
        "mend" => "mend_applied",
        _ => return None,
    })
}

fn selector_kind_for_option(option: &str) -> Option<&'static str> {
    match option {
        "smith" => Some("card"),
        "mend" => Some("player"),
        _ => None,
    }
}

fn visible_choice_ids(value: &Value) -> Option<BTreeSet<String>> {
    let choices = value
        .get("observation")?
        .get("state")?
        .get("choices")?
        .as_array()?;
    if choices.is_empty() {
        return None;
    }
    let mut ids = BTreeSet::new();
    for choice in choices {
        if !ids.insert(string(choice, "choice_id")?.to_owned()) {
            return None;
        }
    }
    Some(ids)
}

pub(crate) fn selector_admissions() -> BTreeMap<String, SelectorAdmission> {
    let mut admissions = BTreeMap::new();
    for name in PRODUCER_FIXTURES {
        for step in producer_fixture(name)["messages"].as_array().unwrap() {
            let response = &step["response"];
            let transition = &response["transition"];
            if !matches!(
                string(transition, "kind"),
                Some("rest_option_selection_requested" | "rest_option_selection_progressed")
            ) {
                continue;
            }
            let selector = transition.get("selector").unwrap();
            let selection_id = string(selector, "selection_id").unwrap();
            let option_id = string(transition, "rest_option_id").unwrap();
            let selection_kind = string(selector, "selection_kind").unwrap();
            let required_count = u64_value(selector, "required_count").unwrap();
            let entry = admissions
                .entry(selection_id.to_owned())
                .or_insert_with(|| SelectorAdmission {
                    option_id: option_id.to_owned(),
                    selection_kind: selection_kind.to_owned(),
                    required_count,
                    choice_ids: BTreeSet::new(),
                });
            assert_eq!(entry.option_id, option_id);
            assert_eq!(entry.selection_kind, selection_kind);
            assert_eq!(entry.required_count, required_count);
            for selected in ids(selector, "selected_choice_ids").unwrap() {
                entry.choice_ids.insert(selected);
            }
            for legal in selector["legal_actions"].as_array().unwrap() {
                let payload = object(legal, "action").unwrap();
                let choice = match string(payload, "kind") {
                    Some("select_card") => string(payload, "card_id"),
                    Some("select_player") => string(payload, "player_id"),
                    _ => None,
                };
                if let Some(choice) = choice {
                    entry.choice_ids.insert(choice.to_owned());
                }
            }
        }
    }
    admissions
}

#[path = "semantics.rs"]
mod semantics;

pub(crate) use semantics::strict_semantics;

pub(crate) fn artifact_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("artifacts/runtime-v4-expert-rest-action")
}
