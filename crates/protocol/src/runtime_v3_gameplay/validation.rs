// SPDX-License-Identifier: MIT

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeV3GameplayValidationError {
    Metadata,
    InvalidIdentity,
    DuplicateTarget,
    GenerationBounds,
    ObservationBounds,
    ActionBounds,
    EffectBounds,
    ResultShape,
}

impl std::fmt::Display for RuntimeV3GameplayValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Metadata => "runtime-v3-gameplay metadata is unsupported",
            Self::InvalidIdentity => "runtime-v3-gameplay identity is empty, unsafe, or too long",
            Self::DuplicateTarget => "runtime-v3-gameplay observation contains a duplicate target",
            Self::GenerationBounds => "runtime-v3-gameplay generation is outside the bound",
            Self::ObservationBounds => "runtime-v3-gameplay observation is outside the bound",
            Self::ActionBounds => "runtime-v3-gameplay action is outside the play_card profile",
            Self::EffectBounds => {
                "runtime-v3-gameplay effect witness is outside the play_card profile"
            }
            Self::ResultShape => {
                "runtime-v3-gameplay message fields do not match the message kind/status"
            }
        };
        formatter.write_str(text)
    }
}

impl std::error::Error for RuntimeV3GameplayValidationError {}
