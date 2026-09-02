// SPDX-License-Identifier: MIT

use super::message::RuntimeV2Message;
use super::{
    RuntimeV2Action, RuntimeV2Context, RuntimeV2EffectWitness, RuntimeV2MessageKind,
    RuntimeV2Metadata, RuntimeV2Observation, RuntimeV2Status,
};

/// Fields owned by a settled, rejected, uncertain, or cancelled receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeV2ActionResult {
    pub status: RuntimeV2Status,
    pub observation: Option<RuntimeV2Observation>,
    pub error_code: Option<String>,
    pub effect_witness: Option<RuntimeV2EffectWitness>,
}

impl RuntimeV2Message {
    /// Creates a state request with no operation or result fields.
    #[must_use]
    pub fn state_request(metadata: RuntimeV2Metadata, context: RuntimeV2Context) -> Self {
        Self::base(metadata, context, RuntimeV2MessageKind::StateRequest)
    }

    /// Creates a state response carrying a bounded observation.
    #[must_use]
    pub fn state_response(
        metadata: RuntimeV2Metadata,
        context: RuntimeV2Context,
        observation: RuntimeV2Observation,
    ) -> Self {
        Self {
            observation: Some(observation),
            ..Self::base(metadata, context, RuntimeV2MessageKind::StateResponse)
        }
    }

    /// Creates an action request with a stable operation identity.
    #[must_use]
    pub fn action_request(
        metadata: RuntimeV2Metadata,
        context: RuntimeV2Context,
        operation_id: impl Into<String>,
        action: RuntimeV2Action,
    ) -> Self {
        Self {
            operation_id: Some(operation_id.into()),
            action: Some(action),
            ..Self::base(metadata, context, RuntimeV2MessageKind::ActionRequest)
        }
    }

    /// Creates a reconciliation request without another mutation action.
    #[must_use]
    pub fn reconcile_request(
        metadata: RuntimeV2Metadata,
        context: RuntimeV2Context,
        operation_id: impl Into<String>,
    ) -> Self {
        Self {
            operation_id: Some(operation_id.into()),
            ..Self::base(metadata, context, RuntimeV2MessageKind::ReconcileRequest)
        }
    }

    /// Creates an action or reconciliation result with explicit lifecycle fields.
    #[must_use]
    pub fn result(
        metadata: RuntimeV2Metadata,
        context: RuntimeV2Context,
        operation_id: impl Into<String>,
        action: RuntimeV2Action,
        result: RuntimeV2ActionResult,
        kind: RuntimeV2MessageKind,
    ) -> Self {
        Self {
            operation_id: Some(operation_id.into()),
            observation: result.observation,
            action: Some(action),
            status: Some(result.status),
            error_code: result.error_code,
            effect_witness: result.effect_witness,
            ..Self::base(metadata, context, kind)
        }
    }

    fn base(
        metadata: RuntimeV2Metadata,
        context: RuntimeV2Context,
        kind: RuntimeV2MessageKind,
    ) -> Self {
        Self {
            protocol_version: metadata.protocol_version,
            schema_digest: metadata.schema_digest,
            provenance: metadata.provenance,
            correlation_id: context.correlation_id,
            instance_id: context.instance_id,
            session_id: context.session_id,
            lease_id: context.lease_id,
            lease_epoch: context.lease_epoch,
            generation: context.generation,
            kind,
            operation_id: None,
            observation: None,
            action: None,
            status: None,
            error_code: None,
            effect_witness: None,
        }
    }
}
