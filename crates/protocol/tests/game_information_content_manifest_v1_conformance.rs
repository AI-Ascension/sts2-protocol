// SPDX-License-Identifier: MIT

use std::collections::BTreeMap;

use serde_json::Value;
use sha2::{Digest, Sha256};
use sts2_protocol::{
    GAME_INFORMATION_CONTENT_MANIFEST_V1_MAX_MESSAGE_BYTES,
    GAME_INFORMATION_CONTENT_MANIFEST_V1_PROTOCOL_VERSION,
    GAME_INFORMATION_CONTENT_MANIFEST_V1_SCHEMA_DIGEST, GameInformationContentManifestV1Codec,
    GameInformationContentManifestV1CodecError,
};

const SOURCE_SCHEMA: &str =
    include_str!("../../../schemas/game-information-content-manifest-v1.schema.json");
const ARTIFACT_SCHEMA: &str =
    include_str!("../../../artifacts/game-information-content-manifest-v1/schema.json");
const MANIFEST: &str =
    include_str!("../../../artifacts/game-information-content-manifest-v1/manifest.json");
const CHECKSUMS: &str =
    include_str!("../../../artifacts/game-information-content-manifest-v1/SHA256SUMS");
const CASE: &str =
    include_str!("../../../conformance/cases/game-information-content-manifest-v1.json");
const POSITIVE: &str = include_str!(
    "../../../conformance/fixtures/game-information-content-manifest-v1/valid/canonical-manifest-response.json"
);
const ACCESS_DENIED: &str = include_str!(
    "../../../conformance/fixtures/game-information-content-manifest-v1/valid/access-denied-error-response.json"
);
const INVALID_ENVELOPES: &[&str] = &[
    include_str!(
        "../../../conformance/fixtures/game-information-content-manifest-v1/invalid/success-with-null-manifest.json"
    ),
    include_str!(
        "../../../conformance/fixtures/game-information-content-manifest-v1/invalid/error-with-null-payload.json"
    ),
    include_str!(
        "../../../conformance/fixtures/game-information-content-manifest-v1/invalid/raw-error-reason.json"
    ),
    include_str!(
        "../../../conformance/fixtures/game-information-content-manifest-v1/invalid/mismatched-error-code-reason.json"
    ),
    include_str!(
        "../../../conformance/fixtures/game-information-content-manifest-v1/invalid/unknown-member.json"
    ),
];
const UNSUPPORTED_DIGEST: &str = include_str!(
    "../../../conformance/fixtures/game-information-content-manifest-v1/invalid/unsupported-schema-digest.json"
);
const GOLDENS: &[(&str, &str)] = &[
    (
        "golden/canonical-manifest-response.json",
        include_str!(
            "../../../artifacts/game-information-content-manifest-v1/golden/canonical-manifest-response.json"
        ),
    ),
    (
        "golden/access-denied-error-response.json",
        include_str!(
            "../../../artifacts/game-information-content-manifest-v1/golden/access-denied-error-response.json"
        ),
    ),
];
const ARTIFACT_MEMBERS: &[(&str, &str)] = &[
    ("schema.json", ARTIFACT_SCHEMA),
    ("manifest.json", MANIFEST),
    (
        "README.md",
        include_str!("../../../artifacts/game-information-content-manifest-v1/README.md"),
    ),
    GOLDENS[0],
    GOLDENS[1],
];

fn schema_validator() -> jsonschema::Validator {
    jsonschema::draft202012::options()
        .build(&serde_json::from_str(SOURCE_SCHEMA).expect("schema JSON"))
        .expect("schema compiles")
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn source_artifact_manifest_case_and_checksum_inventory_are_bound() {
    assert_eq!(SOURCE_SCHEMA, ARTIFACT_SCHEMA);
    assert_eq!(
        digest(SOURCE_SCHEMA.as_bytes()),
        GAME_INFORMATION_CONTENT_MANIFEST_V1_SCHEMA_DIGEST
    );

    let manifest: Value = serde_json::from_str(MANIFEST).expect("artifact manifest JSON");
    assert_eq!(
        manifest["protocol_version"],
        GAME_INFORMATION_CONTENT_MANIFEST_V1_PROTOCOL_VERSION
    );
    assert_eq!(
        manifest["schema_digest"],
        GAME_INFORMATION_CONTENT_MANIFEST_V1_SCHEMA_DIGEST
    );
    assert_eq!(
        manifest["max_message_bytes"],
        GAME_INFORMATION_CONTENT_MANIFEST_V1_MAX_MESSAGE_BYTES
    );
    assert_eq!(
        manifest["goldens"].as_array().map(Vec::len),
        Some(GOLDENS.len())
    );

    let case: Value = serde_json::from_str(CASE).expect("conformance case JSON");
    assert_eq!(
        case["schema_digest"],
        GAME_INFORMATION_CONTENT_MANIFEST_V1_SCHEMA_DIGEST
    );
    assert_eq!(
        case["max_message_bytes"],
        GAME_INFORMATION_CONTENT_MANIFEST_V1_MAX_MESSAGE_BYTES
    );
    assert_eq!(
        manifest["conformance"],
        "../../conformance/cases/game-information-content-manifest-v1.json"
    );

    let checksums = CHECKSUMS
        .lines()
        .filter_map(|line| line.split_once("  "))
        .map(|(hash, path)| (path, hash))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(checksums.len(), ARTIFACT_MEMBERS.len());
    for (path, bytes) in ARTIFACT_MEMBERS {
        assert_eq!(
            checksums.get(path).copied(),
            Some(digest(bytes.as_bytes()).as_str()),
            "checksum for {path}"
        );
    }
    for (path, golden) in GOLDENS {
        assert_eq!(
            *golden,
            match *path {
                "golden/canonical-manifest-response.json" => POSITIVE,
                "golden/access-denied-error-response.json" => ACCESS_DENIED,
                _ => unreachable!("all artifact goldens are listed above"),
            }
        );
    }
}

#[test]
fn positive_manifest_and_mapped_access_denial_match_the_real_schema() {
    let validator = schema_validator();
    let manifest: Value = serde_json::from_str(POSITIVE).expect("manifest fixture JSON");
    let error: Value = serde_json::from_str(ACCESS_DENIED).expect("error fixture JSON");
    assert!(validator.is_valid(&manifest));
    assert!(validator.is_valid(&error));

    assert_eq!(
        manifest["protocol_version"],
        GAME_INFORMATION_CONTENT_MANIFEST_V1_PROTOCOL_VERSION
    );
    assert_eq!(manifest["manifest"]["locale"], "en_US");
    assert_eq!(
        manifest["manifest"]["packages"][0]["package_id"],
        "custom_pack"
    );
    assert_eq!(manifest["manifest"]["packages"][0]["order"], 0);
    assert_eq!(manifest["manifest"]["families"][0]["definition_count"], 1);
    assert_eq!(
        manifest["manifest"]["definitions"][0]["namespaced_id"],
        "card_1"
    );
    assert_eq!(
        manifest["manifest"]["definitions"][0]["override_chain"][0],
        "base_card"
    );
    assert_eq!(error["error"]["code"], "access_denied");
    assert_eq!(error["error"]["reason"], "source_access_denied");

    for invalid in INVALID_ENVELOPES {
        let value: Value = serde_json::from_str(invalid).expect("negative fixture JSON");
        assert!(
            !validator.is_valid(&value),
            "invalid envelope accepted: {value}"
        );
    }

    let unsupported_digest: Value =
        serde_json::from_str(UNSUPPORTED_DIGEST).expect("digest fixture JSON");
    assert!(
        validator.is_valid(&unsupported_digest),
        "the schema checks digest syntax; the codec pins its value"
    );
}

#[test]
fn codec_pins_digest_rejects_duplicate_members_and_round_trips_closed_envelopes() {
    let codec = GameInformationContentManifestV1Codec::new().expect("pinned schema");
    for fixture in [POSITIVE, ACCESS_DENIED] {
        let decoded = codec.decode(fixture.as_bytes()).expect("valid envelope");
        let encoded = codec.encode(&decoded).expect("valid encoding");
        assert_eq!(codec.decode(&encoded).expect("round trip"), decoded);
        assert!(encoded.len() <= GAME_INFORMATION_CONTENT_MANIFEST_V1_MAX_MESSAGE_BYTES);
    }

    assert_eq!(
        codec.decode(br#"{"x":1,"x":2}"#),
        Err(GameInformationContentManifestV1CodecError::DuplicateMember)
    );
    assert_eq!(
        codec.decode(UNSUPPORTED_DIGEST.as_bytes()),
        Err(GameInformationContentManifestV1CodecError::UnsupportedSchemaDigest)
    );
}

#[test]
fn codec_refuses_messages_over_the_utf8_byte_limit_without_truncating() {
    let codec = GameInformationContentManifestV1Codec::new().expect("pinned schema");
    let oversized_input = vec![b'x'; GAME_INFORMATION_CONTENT_MANIFEST_V1_MAX_MESSAGE_BYTES + 1];
    assert_eq!(
        codec.decode(&oversized_input),
        Err(GameInformationContentManifestV1CodecError::MessageTooLarge)
    );

    // Each BMP character occupies one UTF-16 code unit but two UTF-8 bytes. The serialized string
    // adds two quotes, so this crosses the byte limit despite fitting under it by code-unit count.
    let unicode =
        Value::String("é".repeat(GAME_INFORMATION_CONTENT_MANIFEST_V1_MAX_MESSAGE_BYTES / 2));
    assert_eq!(
        codec.encode(&unicode),
        Err(GameInformationContentManifestV1CodecError::MessageTooLarge)
    );
}
