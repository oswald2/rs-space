#[allow(unused)]
use std::collections::BTreeSet;

use rasn::{types::*, AsnType, Decode, Encode};

use crate::types::sle::{ServiceInstanceIdentifier, Credentials};


pub type DeliveryMode = i64;
pub type Diagnostics = i64;
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
pub type PeerAbortDiagnostic = i64;
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
}

#[derive(AsnType, Debug, PartialEq, Encode, Decode)]
#[rasn(choice)]
pub enum BindResult {
    #[rasn(tag(0))]
    BindOK(VersionNumber),
    #[rasn(tag(1))]
    BindDiag(BindDiagnostic),
}

pub type SlePeerAbort = PeerAbortDiagnostic;

