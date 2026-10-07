// SPDX-License-Identifier: MIT

use super::support::{self, TestResult, VECTORS};
use sts2_protocol::game_facts_reference_v1::GameFactsReferenceV1Codec;

#[test]
fn every_canonical_vector_decodes_validates_and_round_trips() -> TestResult {
    for name in VECTORS {
        let bytes = support::golden(name)?;
        let message =
            GameFactsReferenceV1Codec::decode(&bytes).map_err(|error| error.to_string())?;
        GameFactsReferenceV1Codec::validate(&message).map_err(|error| error.to_string())?;
        let encoded =
            GameFactsReferenceV1Codec::encode(&message).map_err(|error| error.to_string())?;
        let decoded =
            GameFactsReferenceV1Codec::decode(&encoded).map_err(|error| error.to_string())?;
        assert_eq!(decoded, message, "round trip {name}");
        assert_eq!(
            GameFactsReferenceV1Codec::encode(&decoded).map_err(|error| error.to_string())?,
            encoded
        );
    }
    Ok(())
}
