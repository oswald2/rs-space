use std::collections::BTreeSet;

use log::info;
use rasn::{types::*, AsnType, Decode, Encode};

// ASN1 common types
pub type ConditionalTime = Option<Time>;

#[derive(AsnType, Debug, PartialEq, Encode, Decode)]
#[rasn(choice)]
pub enum Credentials {
    #[rasn(tag(context, 0))]
    Unused,
    #[rasn(tag(context, 1))]
    Used(Vec<u8>),
}
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

#[derive(AsnType, Debug, PartialEq, Encode, Decode)]
#[rasn(choice)]
pub enum Time {
    #[rasn(tag(0))]
    CcsdsFormat(TimeCCSDS),
    #[rasn(tag(1))]
    CcsdsPicoFormat(TimeCCSDSpico),
}

#[derive(AsnType, Debug, PartialEq, Encode, Decode)]
pub struct TimeCCSDS(Vec<u8>);

#[derive(AsnType, Debug, PartialEq, Encode, Decode)]
pub struct TimeCCSDSpico(Vec<u8>);

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
#[rasn(tag(context, 100))]
pub struct SleBindInvocation {
    pub invoker_credentials: Credentials,
    pub initiator_identifier: AuthorityIdentifier,
    pub responder_port_identifier: PortId,
    pub service_type: Integer,
    pub version_number: VersionNumber,
    pub service_instance_identifier: ServiceInstanceIdentifier,
}

#[derive(AsnType, Debug, PartialEq, Encode, Decode)]
#[rasn(choice)]
pub enum BindResult {
    #[rasn(tag(0))]
    BindOK(VersionNumber),
    #[rasn(tag(1))]
    BindDiag(BindDiagnostic),
}

#[derive(AsnType, Debug, PartialEq, Encode, Decode)]
#[rasn(tag(context, 101))]
pub struct SleBindReturn {
    pub performer_credentials: Credentials,
    pub responder_identifier: AuthorityIdentifier,
    pub result: BindResult,
}

pub type SlePeerAbort = PeerAbortDiagnostic;

#[derive(AsnType, Debug, PartialEq, Encode, Decode)]
#[rasn(tag(context, 102))]
pub struct SleUnbindInvocation {
    pub invoker_credentials: Credentials,
    pub unbind_reason: Integer,
}

#[derive(AsnType, Debug, PartialEq, Encode, Decode)]
#[rasn(tag(context, 103))]
pub struct SleUnbindReturn {
    pub responder_credentials: Credentials,
    #[rasn(tag(context, 0))]
    pub result: (),
}

// #[derive(AsnType, Debug, PartialEq, Encode, Decode)]
pub type ServiceInstanceIdentifier = Vec<ServiceInstanceAttribute>;

#[derive(AsnType, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode)]
pub struct ServiceInstanceAttributeInner {
    pub identifier: ObjectIdentifier,
    pub si_attribute_value: VisibleString,
}

pub type ServiceInstanceAttribute = SetOf<ServiceInstanceAttributeInner>;

pub fn new_service_instannce_attribute(
    id: &ConstOid,
    value: &str,
) -> ServiceInstanceAttribute {
    let mut tree = BTreeSet::new();

    info!("OID: {:?}", id);
    info!("OID: {:?}", id.0);

    tree.insert(ServiceInstanceAttributeInner {
        identifier: ObjectIdentifier::new_unchecked(std::borrow::Cow::Borrowed(id.0)),
        si_attribute_value: Implicit::new(String::from(value)),
    });
    tree
}

pub const SAGR: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 52]);
pub const SPACK: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 53]);
pub const FSL_FG: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 14]);
pub const RSL_FG: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 38]);
pub const CLTU: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 7]);
pub const FSP: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 10]);
pub const RAF: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 22]);
pub const RCF: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 46]);
pub const RCFSH: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 44]);
pub const ROCF: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 49]);
pub const RSP: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 40]);
pub const TCF: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 12]);
pub const TCVA: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 16]);
