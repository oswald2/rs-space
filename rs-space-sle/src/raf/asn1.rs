#[allow(unused)]
use std::collections::BTreeSet;

use rasn::{AsnType, Decode, Encode};

use crate::types::sle::{Diagnostics};


#[derive(AsnType, Debug, Clone, Copy, PartialEq, Encode, Decode)]
#[rasn(enumerated)]
pub enum RequestedFrameQuality {
    GoodFramesOnly = 0,
    ErredFramesOnly = 1,
    AllFrames = 2,
}

impl TryFrom<u32> for RequestedFrameQuality {
    type Error = String;

    fn try_from(val: u32) -> Result<RequestedFrameQuality, String> {
        match val {
            0 => Ok(RequestedFrameQuality::GoodFramesOnly),
            1 => Ok(RequestedFrameQuality::ErredFramesOnly),
            2 => Ok(RequestedFrameQuality::AllFrames),
            x => Err(format!("Requested frame quality has unexpected value: {x}"))
        }
    }
}

#[derive(AsnType, Debug, Clone, PartialEq, Encode, Decode)]
#[rasn(choice)]
pub enum DiagnosticRafStart {
    #[rasn(tag(0))]
    Common(Diagnostics),
    #[rasn(tag(1))]
    Specific(SpecificDiagnosticRafStart),
}

#[derive(AsnType, Debug, Clone, PartialEq, Encode, Decode)]
#[rasn(choice)]
pub enum SpecificDiagnosticRafStart {
    #[rasn(tag(0))]
    OutOfService = 0,
    #[rasn(tag(1))]
    UnableToComply = 1,
    #[rasn(tag(2))]
    InvalidStartTime = 2,
    #[rasn(tag(3))]
    InvalidStopTime = 3,
    #[rasn(tag(4))]
    MissingTimeValue = 4,
}

#[derive(AsnType, Debug, Clone, PartialEq, Encode, Decode)]
#[rasn(choice)]
pub enum RafStartReturnResult {
    #[rasn(tag(0))]
    PositiveResult,
    #[rasn(tag(1))]
    NegativeResult(DiagnosticRafStart),
}

