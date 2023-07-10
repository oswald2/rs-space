#[allow(unused)]
use std::collections::BTreeSet;

use rasn::{types::*, AsnType, Decode, Encode};

use crate::types::sle::{ServiceInstanceIdentifier, Credentials, ConditionalTime, PeerAbortDiagnostic};


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

#[derive(AsnType, Debug, Clone, Copy, PartialEq, Encode, Decode)]
#[rasn(enumerated)]
pub enum RequestedFrameQuality {
    GoodFramesOnly = 0,
    ErredFramesOnly = 1, 
    AllFrames = 2,
}


#[derive(AsnType, Debug, PartialEq, Encode, Decode)]
pub struct SpaceLinkDataUnit(Vec<u8>);

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
    #[rasn(tag(context,104))]
    SlePeerAbort {
        diagnostic: PeerAbortDiagnostic
    },
    #[rasn(tag(context, 0))]
    SleRafStartInvocation {
        invoker_credentials: Credentials,
        invoke_id: InvokeId,
        start_time: ConditionalTime,
        stop_time: ConditionalTime,
        requested_frame_quality: RequestedFrameQuality,
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
    }
}



// #[derive(AsnType, Debug, Clone, PartialEq)]
// pub struct RafGetParameterInvocation {
//     pub invoker_credentials: Credentials,
//     pub invoke_id: InvokeId,
//     pub raf_parameter: RafParameterName,
// }




#[derive(AsnType, Debug, Clone, PartialEq, Encode, Decode)]
#[rasn(choice)]
pub enum BindResult {
    #[rasn(tag(0))]
    BindOK(VersionNumber),
    #[rasn(tag(1))]
    BindDiag(BindDiagnostic),
}


#[derive(AsnType, Debug, Clone, PartialEq, Encode, Decode)]
#[rasn(choice)]
pub enum Diagnostics {
    #[rasn(tag(100))]
    DuplicateInvokeId = 100,
    #[rasn(tag(127))]
    OtherReason = 127,
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

#[derive(AsnType, Debug, Clone, PartialEq, Encode, Decode)]
#[rasn(choice)]
pub enum SleResult {
    #[rasn(tag(0))]
    PositiveResult,
    #[rasn(tag(1))]
    NegativeResult(Diagnostics)
}



