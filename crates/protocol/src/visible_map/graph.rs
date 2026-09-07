// SPDX-License-Identifier: MIT

use std::collections::{BTreeMap, BTreeSet};

use super::model::{
    RuntimeMapV1ActionBinding, RuntimeMapV1NavigationAction, RuntimeMapV1Position,
    RuntimeMapV1Snapshot,
};
use super::validation::RuntimeMapV1ValidationError;
use super::{
    RUNTIME_MAP_V1_MAX_ACTION_OPTION_ID_BYTES, RUNTIME_MAP_V1_MAX_BINDINGS,
    RUNTIME_MAP_V1_MAX_COORDINATE, RUNTIME_MAP_V1_MAX_EDGES, RUNTIME_MAP_V1_MAX_HISTORY,
    RUNTIME_MAP_V1_MAX_HOST_ACTION_ID_BYTES, RUNTIME_MAP_V1_MAX_ID_BYTES, RUNTIME_MAP_V1_MAX_NODES,
    RUNTIME_MAP_V1_MIN_COORDINATE, valid_identity,
};

pub(super) fn validate_graph(
    snapshot: &RuntimeMapV1Snapshot,
) -> Result<(), RuntimeMapV1ValidationError> {
    if snapshot.nodes.len() > RUNTIME_MAP_V1_MAX_NODES
        || snapshot.edges.len() > RUNTIME_MAP_V1_MAX_EDGES
        || snapshot.bindings.len() > RUNTIME_MAP_V1_MAX_BINDINGS
        || snapshot.history.len() > RUNTIME_MAP_V1_MAX_HISTORY
        || snapshot.terminal_node_ids.len() > RUNTIME_MAP_V1_MAX_HISTORY
    {
        return Err(RuntimeMapV1ValidationError::CollectionBounds);
    }
    let mut nodes = BTreeMap::new();
    for node in &snapshot.nodes {
        if !valid_identity(&node.id, RUNTIME_MAP_V1_MAX_ID_BYTES) {
            return Err(RuntimeMapV1ValidationError::InvalidNode);
        }
        if !(RUNTIME_MAP_V1_MIN_COORDINATE..=RUNTIME_MAP_V1_MAX_COORDINATE).contains(&node.row)
            || !(RUNTIME_MAP_V1_MIN_COORDINATE..=RUNTIME_MAP_V1_MAX_COORDINATE)
                .contains(&node.column)
        {
            return Err(RuntimeMapV1ValidationError::CoordinateBounds);
        }
        if nodes.insert(node.id.as_str(), nodes.len()).is_some() {
            return Err(RuntimeMapV1ValidationError::DuplicateNode);
        }
    }
    let mut adjacency = vec![Vec::new(); snapshot.nodes.len()];
    let mut edge_keys = BTreeSet::new();
    for edge in &snapshot.edges {
        let Some(&from) = nodes.get(edge.from.as_str()) else {
            return Err(RuntimeMapV1ValidationError::UnknownEdgeEndpoint);
        };
        let Some(&to) = nodes.get(edge.to.as_str()) else {
            return Err(RuntimeMapV1ValidationError::UnknownEdgeEndpoint);
        };
        if from == to {
            return Err(RuntimeMapV1ValidationError::InvalidEdge);
        }
        if !edge_keys.insert((edge.from.as_str(), edge.to.as_str())) {
            return Err(RuntimeMapV1ValidationError::DuplicateEdge);
        }
        adjacency[from].push(to);
    }
    let mut colors = vec![0_u8; snapshot.nodes.len()];
    for index in 0..snapshot.nodes.len() {
        if colors[index] == 0 && has_cycle(index, &adjacency, &mut colors) {
            return Err(RuntimeMapV1ValidationError::CyclicGraph);
        }
    }
    validate_position(snapshot, &nodes)?;
    validate_ids(snapshot, &nodes)?;
    validate_bindings(snapshot, &nodes)
}

fn validate_position(
    snapshot: &RuntimeMapV1Snapshot,
    nodes: &BTreeMap<&str, usize>,
) -> Result<(), RuntimeMapV1ValidationError> {
    match &snapshot.position {
        RuntimeMapV1Position::Current { node_id } => {
            let Some(index) = nodes.get(node_id.as_str()) else {
                return Err(RuntimeMapV1ValidationError::InvalidPosition);
            };
            if !snapshot.nodes[*index].visited {
                return Err(RuntimeMapV1ValidationError::InvalidPosition);
            }
        }
        RuntimeMapV1Position::PreStart {} => {
            if !snapshot.history.is_empty() {
                return Err(RuntimeMapV1ValidationError::InvalidPosition);
            }
        }
        RuntimeMapV1Position::Unavailable {} => {}
    }
    Ok(())
}

fn validate_ids(
    snapshot: &RuntimeMapV1Snapshot,
    nodes: &BTreeMap<&str, usize>,
) -> Result<(), RuntimeMapV1ValidationError> {
    let mut history = BTreeSet::new();
    for id in &snapshot.history {
        let Some(index) = nodes.get(id.as_str()) else {
            return Err(RuntimeMapV1ValidationError::InvalidHistory);
        };
        if !snapshot.nodes[*index].visited || !history.insert(id.as_str()) {
            return Err(RuntimeMapV1ValidationError::InvalidHistory);
        }
    }
    let mut terminals = BTreeSet::new();
    for id in &snapshot.terminal_node_ids {
        if !nodes.contains_key(id.as_str()) {
            return Err(RuntimeMapV1ValidationError::InvalidTerminal);
        }
        if !terminals.insert(id.as_str()) {
            return Err(RuntimeMapV1ValidationError::DuplicateTerminal);
        }
    }
    Ok(())
}

fn validate_bindings(
    snapshot: &RuntimeMapV1Snapshot,
    nodes: &BTreeMap<&str, usize>,
) -> Result<(), RuntimeMapV1ValidationError> {
    let mut actions = BTreeSet::new();
    let mut host_ids = BTreeSet::new();
    let mut option_ids = BTreeSet::new();
    for binding in &snapshot.bindings {
        validate_binding(binding, nodes)?;
        if matches!(&snapshot.position, RuntimeMapV1Position::Current { node_id } if node_id == &binding.graph_node_id)
        {
            return Err(RuntimeMapV1ValidationError::InvalidBinding);
        }
        let option_id = match &binding.action {
            RuntimeMapV1NavigationAction::SelectMapNode { node_id } => node_id,
        };
        if !actions.insert(binding.graph_node_id.as_str())
            || !host_ids.insert(binding.host_action_id.as_str())
        {
            return Err(RuntimeMapV1ValidationError::DuplicateBinding);
        }
        if !option_ids.insert(option_id.as_str()) {
            return Err(RuntimeMapV1ValidationError::DuplicateActionOption);
        }
    }
    Ok(())
}

fn validate_binding(
    binding: &RuntimeMapV1ActionBinding,
    nodes: &BTreeMap<&str, usize>,
) -> Result<(), RuntimeMapV1ValidationError> {
    if !valid_identity(&binding.graph_node_id, RUNTIME_MAP_V1_MAX_ID_BYTES)
        || !valid_identity(
            &binding.host_action_id,
            RUNTIME_MAP_V1_MAX_HOST_ACTION_ID_BYTES,
        )
        || binding.host_action_id == binding.graph_node_id
    {
        return Err(RuntimeMapV1ValidationError::InvalidBinding);
    }
    if !nodes.contains_key(binding.graph_node_id.as_str()) {
        return Err(RuntimeMapV1ValidationError::UnknownBindingNode);
    }
    match &binding.action {
        RuntimeMapV1NavigationAction::SelectMapNode { node_id } => {
            if !valid_identity(node_id, RUNTIME_MAP_V1_MAX_ACTION_OPTION_ID_BYTES) {
                return Err(RuntimeMapV1ValidationError::InvalidActionOption);
            }
        }
    }
    Ok(())
}

fn has_cycle(index: usize, adjacency: &[Vec<usize>], colors: &mut [u8]) -> bool {
    colors[index] = 1;
    for &next in &adjacency[index] {
        if colors[next] == 1 || (colors[next] == 0 && has_cycle(next, adjacency, colors)) {
            return true;
        }
    }
    colors[index] = 2;
    false
}
