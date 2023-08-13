#[allow(unused)]
use std::collections::BTreeSet;

use num_traits::ToPrimitive;
use rasn::{types::*, AsnType, Decode, Encode};

use crate::types::sle::{
    ConditionalTime, Credentials, Diagnostics, PeerAbortDiagnostic, ServiceInstanceIdentifier,
};

use crate::raf::asn1::{RafStartReturnResult, RafTransferBuffer};
use serde::{Deserialize, Serialize};


pub type DeliveryMode = i64;
pub type Duration = IntUnsignedLong;
pub type ForwardDuStatus = i64;
pub type IntPosLong = u32;
pub type IntPosShort = u16;
pub type IntUnsignedLong = u32;
pub type IntUnsignedShort = u16;
pub type InvokeId = IntUnsignedShort;
pub type ParameterName = i64;
pub type SlduStatusNotification = i64;

#[derive(AsnType, Debug, PartialEq, Encode, Decode)]
pub struct SpaceLinkDataUnit(Vec<u8>);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DeliveryModeEnum {
    RtnTimelyOnline = 0,
    RtnCompleteOnline = 1,
    RtnOffline = 2,
    FwdOnline = 3,
    FwdOffline = 4,
}

impl TryFrom<i64> for DeliveryModeEnum {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(DeliveryModeEnum::RtnTimelyOnline),
            1 => Ok(DeliveryModeEnum::RtnCompleteOnline),
            2 => Ok(DeliveryModeEnum::RtnOffline),
            3 => Ok(DeliveryModeEnum::FwdOnline),
            4 => Ok(DeliveryModeEnum::FwdOffline),
            x => Err(format!("Illegal value for delivery mode {x}")),
        }
    }
}



// ASN1 bind types
#[derive(AsnType, Debug, Clone, Copy, PartialEq, Encode, Decode)]
#[rasn(enumerated)]
pub enum ApplicationIdentifier {
    RtnAllFrames = 0,
    RtnInsert = 1,
    RtnChFrames = 2,
    RtnChFsh = 3,
    RtnChOcf = 4,
    RtnBitstr = 5,
    RtnSpacePkt = 6,
    FwdAosSpacePkt = 7,
    FwdAosVca = 8,
    FwdBitstr = 9,
    FwdProtoVcdu = 10,
    FwdInsert = 11,
    FwdCVcdu = 12,
    FwdTcSpacePkt = 13,
    FwdTcVca = 14,
    FwdTcFrame = 15,
    FwdCltu = 16,
}

impl TryFrom<&rasn::types::Integer> for ApplicationIdentifier {
    type Error = String;

    fn try_from(value: &Integer) -> Result<Self, Self::Error> {
        match value.to_i64() {
            Some(0) => Ok(ApplicationIdentifier::RtnAllFrames),
            Some(1) => Ok(ApplicationIdentifier::RtnInsert),
            Some(2) => Ok(ApplicationIdentifier::RtnChFrames),
            Some(3) => Ok(ApplicationIdentifier::RtnChFsh),
            Some(4) => Ok(ApplicationIdentifier::RtnChOcf),
            Some(5) => Ok(ApplicationIdentifier::RtnBitstr),
            Some(6) => Ok(ApplicationIdentifier::RtnSpacePkt),
            Some(7) => Ok(ApplicationIdentifier::FwdAosSpacePkt),
            Some(8) => Ok(ApplicationIdentifier::FwdAosVca),
            Some(9) => Ok(ApplicationIdentifier::FwdBitstr),
            Some(10) => Ok(ApplicationIdentifier::FwdProtoVcdu),
            Some(11) => Ok(ApplicationIdentifier::FwdInsert),
            Some(12) => Ok(ApplicationIdentifier::FwdCVcdu),
            Some(13) => Ok(ApplicationIdentifier::FwdTcSpacePkt),
            Some(14) => Ok(ApplicationIdentifier::FwdTcVca),
            Some(15) => Ok(ApplicationIdentifier::FwdTcFrame),
            Some(16) => Ok(ApplicationIdentifier::FwdCltu),
            Some(x) => Err(format!("Illegal value {} for ApplicationIdentifier", x)),
            None => Err("Illegal value for ApplicationIdentifier".to_string()),
        }
    }
}

pub type AuthorityIdentifier = VisibleString;

#[derive(Debug, PartialEq, Eq, Clone, Copy, AsnType, Decode, Encode)]
#[rasn(enumerated)]
pub enum BindDiagnostic {
    AccessDenied = 0,
    ServiceTypeNotSupported = 1,
    VersionNotSupported = 2,
    NoSuchServiceInstance = 3,
    AlreadyBound = 4,
    SiNotAccessibleToThisInitiator = 5,
    InconsistentServiceType = 6,
    InvalidTime = 7,
    OutOfService = 8,
    OtherReason = 127,
}

pub type IdentifierString = VisibleString;
pub type LogicalPortName = VisibleString;
pub type PortId = LogicalPortName;

#[derive(Debug, PartialEq, Eq, Clone, Copy, AsnType, Decode, Encode)]
#[rasn(enumerated)]
pub enum UnbindReason {
    End = 0,
    Suspend = 1,
    VersionNotSupported = 2,
    Other = 127,
}

impl TryFrom<&rasn::types::Integer> for UnbindReason {
    type Error = String;

    fn try_from(value: &Integer) -> Result<Self, Self::Error> {
        match value.to_i64() {
            Some(0) => Ok(UnbindReason::End),
            Some(1) => Ok(UnbindReason::Suspend),
            Some(2) => Ok(UnbindReason::VersionNotSupported),
            Some(127) => Ok(UnbindReason::Other),
            x => Err(format!("Illegal UNBIND reason {x:?}")),
        }
    }
}

pub type VersionNumber = IntPosShort;

#[derive(AsnType, Debug, PartialEq, Encode, Decode)]
#[rasn(choice)]
pub enum SlePdu {
    #[rasn(tag(context, 100))]
    SleBindInvocation {
        invoker_credentials: Credentials,
        initiator_identifier: AuthorityIdentifier,
        responder_port_identifier: PortId,
        service_type: Integer,
        version_number: VersionNumber,
        service_instance_identifier: ServiceInstanceIdentifier,
    },
    #[rasn(tag(context, 101))]
    SleBindReturn {
        performer_credentials: Credentials,
        responder_identifier: AuthorityIdentifier,
        result: BindResult,
    },
    #[rasn(tag(context, 102))]
    SleUnbindInvocation {
        invoker_credentials: Credentials,
        unbind_reason: Integer,
    },
    #[rasn(tag(context, 103))]
    SleUnbindReturn {
        responder_credentials: Credentials,
        #[rasn(tag(context, 0))]
        result: (),
    },
    #[rasn(tag(context, 104))]
    SlePeerAbort { diagnostic: PeerAbortDiagnostic },
    #[rasn(tag(context, 0))]
    SleRafStartInvocation {
        invoker_credentials: Credentials,
        invoke_id: InvokeId,
        start_time: ConditionalTime,
        stop_time: ConditionalTime,
        requested_frame_quality: Integer,
    },
    #[rasn(tag(context, 1))]
    SleRafStartReturn {
        performer_credentials: Credentials,
        invoke_id: InvokeId,
        result: RafStartReturnResult,
    },
    #[rasn(tag(context, 2))]
    SleRafStopInvocation {
        invoker_credentials: Credentials,
        invoke_id: InvokeId,
    },
    #[rasn(tag(context, 3))]
    SleAcknowledgement {
        credentials: Credentials,
        invoke_id: InvokeId,
        result: SleResult,
    },
    #[rasn(tag(context, 8))]
    SleRafTransferBuffer(RafTransferBuffer),
}

impl SlePdu {
    pub fn get_credentials(&self) -> Option<&Credentials> {
        match self {
            SlePdu::SleBindInvocation {
                invoker_credentials,
                ..
            } => Some(&invoker_credentials),
            SlePdu::SleBindReturn {
                performer_credentials,
                ..
            } => Some(&performer_credentials),
            SlePdu::SleUnbindInvocation {
                invoker_credentials,
                ..
            } => Some(&invoker_credentials),
            SlePdu::SleUnbindReturn {
                responder_credentials,
                ..
            } => Some(&responder_credentials),
            SlePdu::SlePeerAbort { .. } => None,
            SlePdu::SleRafStartInvocation {
                invoker_credentials,
                ..
            } => Some(&invoker_credentials),
            SlePdu::SleRafStartReturn {
                performer_credentials,
                ..
            } => Some(&performer_credentials),
            SlePdu::SleRafStopInvocation {
                invoker_credentials,
                ..
            } => Some(&invoker_credentials),
            SlePdu::SleAcknowledgement { credentials, .. } => Some(&credentials),
            SlePdu::SleRafTransferBuffer { .. } => None,
        }
    }

    pub fn operation_name(&self) -> &str {
        match self {
            SlePdu::SleBindInvocation { .. } => "BIND",
            SlePdu::SleBindReturn { .. } => "BIND RETURN",
            SlePdu::SleUnbindInvocation { .. } => "UNBIND",
            SlePdu::SleUnbindReturn { .. } => "UNBIND RETURN",
            SlePdu::SlePeerAbort { .. } => "PEER ABORT",
            SlePdu::SleRafStartInvocation { .. } => "RAF START",
            SlePdu::SleRafStartReturn { .. } => "RAF START RETURN",
            SlePdu::SleRafStopInvocation { .. } => "RAF STOP",
            SlePdu::SleAcknowledgement { .. } => "RAF STOP RETURN",
            SlePdu::SleRafTransferBuffer { .. } => "RAF TRANSFER BUFFER",
        }
    }

    pub fn is_peer_abort(&self) -> bool {
        match self {
            SlePdu::SlePeerAbort { .. } => true,
            _ => false,
        }
    }
}

// #[derive(AsnType, Debug, Clone, PartialEq)]
// pub struct RafGetParameterInvocation {
//     pub invoker_credentials: Credentials,
//     pub invoke_id: InvokeId,
//     pub raf_parameter: RafParameterName,
// }

#[derive(AsnType, Debug, Copy, Clone, PartialEq, Encode, Decode)]
#[rasn(choice)]
pub enum BindResult {
    #[rasn(tag(0))]
    BindOK(VersionNumber),
    #[rasn(tag(1))]
    BindDiag(BindDiagnostic),
}

#[derive(AsnType, Debug, Copy, Clone, PartialEq, Encode, Decode)]
#[rasn(choice)]
pub enum SleResult {
    #[rasn(tag(0))]
    PositiveResult,
    #[rasn(tag(1))]
    NegativeResult(Diagnostics),
}
