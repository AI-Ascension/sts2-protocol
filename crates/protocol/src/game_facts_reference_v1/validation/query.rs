// SPDX-License-Identifier: MIT

use super::super::types::*;
use super::{
    MAX_COMBINATIONS, MAX_INPUTS, MAX_RULES, MAX_SAFE, bounded_text, identity, instance_ref,
    inventory, locale, observation, require, safe, snapshot_ref,
};
use crate::game_facts_reference_v1::GameFactsReferenceV1Rejection as Reject;
use std::collections::BTreeSet;

pub(super) fn query_validate_response(query: &Query, result: &QueryResult) -> Result<(), Reject> {
    query_validate(query)?;
    require(
        result.rules_reference_version == query.rules_reference_version,
        Reject::UnsupportedVersion,
    )?;
    inventory(&result.inventory_binding)?;
    require(
        result.producer_version == "game-facts-reference-producer-v1",
        Reject::UnsupportedVersion,
    )?;
    require(
        result.results.len() == query.rule_ids.len(),
        Reject::InvalidBounds,
    )?;
    for (entry, requested_id) in result.results.iter().zip(&query.rule_ids) {
        require(entry.rule_id() == requested_id, Reject::InvalidRequest)?;
        match entry {
            QueryResultEntry::Found {
                evidence_status,
                inputs,
                ..
            } => {
                require(
                    !inputs.is_empty() && inputs.len() <= MAX_INPUTS,
                    Reject::InvalidBounds,
                )?;
                let mut names = BTreeSet::new();
                for input in inputs {
                    identity(&input.name)?;
                    require(names.insert(&input.name), Reject::Malformed)?;
                    observation(input, query.binding.mode)?;
                }
                let source_backed = inputs.iter().all(|input| {
                    input.applicability == Applicability::Required
                        && input
                            .observation
                            .as_ref()
                            .is_some_and(|item| item.availability == Availability::Available)
                        && input.source_ref.r#ref.is_some()
                        && matches!(
                            input.source_ref.kind,
                            SourceKind::GameMod | SourceKind::ContentManifest
                        )
                });
                if matches!(
                    evidence_status,
                    EvidenceStatus::Confirmed | EvidenceStatus::SourceDerived
                ) {
                    require(
                        inputs.iter().all(|input| {
                            input.source_ref.r#ref.is_some()
                                && matches!(
                                    input.source_ref.kind,
                                    SourceKind::GameMod | SourceKind::ContentManifest
                                )
                        }),
                        Reject::InvalidRequest,
                    )?;
                }
                if matches!(evidence_status, EvidenceStatus::Confirmed) {
                    require(
                        source_backed
                            && !result
                                .unsupported_combinations
                                .iter()
                                .any(|combo| combo.rule_ids.iter().any(|id| id == requested_id)),
                        Reject::InvalidRequest,
                    )?;
                }
            }
            QueryResultEntry::Unsupported { .. } => {}
        }
    }
    require(
        result.unsupported_combinations.len() <= MAX_COMBINATIONS,
        Reject::InvalidBounds,
    )?;
    let mut combinations = BTreeSet::new();
    for combination in &result.unsupported_combinations {
        require(
            (2..=MAX_RULES).contains(&combination.rule_ids.len()),
            Reject::InvalidBounds,
        )?;
        let mut seen = BTreeSet::new();
        for id in &combination.rule_ids {
            identity(id)?;
            require(
                query.rule_ids.contains(id) && seen.insert(id),
                Reject::Malformed,
            )?;
        }
        bounded_text(&combination.reason, 256)?;
        let mut combination_ids = combination.rule_ids.clone();
        combination_ids.sort_unstable();
        require(combinations.insert(combination_ids), Reject::Malformed)?;
    }
    Ok(())
}

impl QueryResultEntry {
    fn rule_id(&self) -> &str {
        match self {
            Self::Found { rule_id, .. } | Self::Unsupported { rule_id, .. } => rule_id,
        }
    }
}

pub(super) fn query(query: &Query) -> Result<(), Reject> {
    query_validate(query)
}

pub(super) fn query_validate(query: &Query) -> Result<(), Reject> {
    require(
        query.rules_reference_version == 2,
        Reject::UnsupportedVersion,
    )?;
    require(
        !query.rule_ids.is_empty() && query.rule_ids.len() <= MAX_RULES,
        Reject::InvalidBounds,
    )?;
    let mut ids = BTreeSet::new();
    for id in &query.rule_ids {
        identity(id)?;
        require(ids.insert(id), Reject::Malformed)?;
    }
    identity(&query.binding.content_manifest_id)?;
    locale(&query.binding.locale)?;
    match query.binding.mode {
        BindingMode::Static => require(
            query.scope.is_none()
                && query.parent_observation.is_none()
                && query.binding.visibility_scope.is_none()
                && query.binding.instance_ref.is_none()
                && query.binding.snapshot_ref.is_none(),
            Reject::InvalidBinding,
        ),
        BindingMode::Live => {
            let scope = query.scope.as_ref().ok_or(Reject::InvalidBinding)?;
            let parent = query
                .parent_observation
                .as_ref()
                .ok_or(Reject::InvalidBinding)?;
            let instance = query
                .binding
                .instance_ref
                .as_ref()
                .ok_or(Reject::InvalidBinding)?;
            let snapshot = query
                .binding
                .snapshot_ref
                .as_ref()
                .ok_or(Reject::InvalidBinding)?;
            let visibility = query
                .binding
                .visibility_scope
                .as_ref()
                .ok_or(Reject::InvalidBinding)?;
            identity(visibility)?;
            instance_ref(instance)?;
            snapshot_ref(snapshot)?;
            instance_ref(&parent.instance_ref)?;
            snapshot_ref(&parent.snapshot_ref)?;
            safe(parent.state_generation)?;
            require(
                snapshot.instance_ref == *instance
                    && parent.instance_ref == *instance
                    && parent.snapshot_ref == *snapshot
                    && parent.state_generation == snapshot.state_generation,
                Reject::InvalidBinding,
            )?;
            identity(&scope.instance_id)?;
            identity(&scope.run_id)?;
            identity(&scope.content_manifest_id)?;
            locale(&scope.locale)?;
            require(
                scope.authority_epoch > 0
                    && scope.authority_epoch <= MAX_SAFE
                    && scope.instance_id == instance.instance_id
                    && scope.run_id == instance.run_id
                    && scope.content_manifest_id == query.binding.content_manifest_id
                    && scope.locale == query.binding.locale,
                Reject::InvalidBinding,
            )?;
            Ok(())
        }
    }
}
