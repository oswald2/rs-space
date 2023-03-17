use rasn::types::{ConstOid, Integer, ObjectIdentifier, OctetString, VisibleString};
use rasn::{AsnType, Decode, Decoder, Encode, Encoder, Tag};

#[derive(AsnType, Decode, Encode)]
#[rasn(choice)]
pub enum ConditionalTime {
    #[rasn(tag(0))]
    Undefined,
    #[rasn(tag(1))]
    Known(Time),
}

#[derive(Debug, Clone, AsnType, Decode, Encode)]
#[rasn(choice)]
pub enum Credentials {
    #[rasn(tag(0))]
    Unused,
    #[rasn(tag(1))]
    Used(OctetString),
}

#[derive(Debug, Clone, Copy, AsnType, Decode, Encode)]
#[rasn(enumerated)]
pub enum DeliveryMode {
    RtnTimelyOnline = 0,
    RtnCompleteOnline = 1,
    RtnOffline = 2,
    FwdOnline = 3,
    FwdOffline = 4,
}

#[derive(Debug, Clone, Copy, AsnType, Decode, Encode)]
#[rasn(enumerated)]
pub enum Diagnostics {
    DuplicateInvokeId = 100,
    OtherReason = 127,
}

#[derive(AsnType)]
pub struct Duration(Integer);

#[derive(Debug, Clone, Copy, AsnType, Decode, Encode)]
#[rasn(enumerated)]
pub enum ForwardDuStatus {
    Radiated = 0,
    Expired = 1,
    Interrupted = 2,
    Acknowledged = 3,
    ProductionStarted = 4,
    ProductionNotStarted = 5,
    UnsupportedTransmissionMode = 6,
}

#[derive(AsnType, Decode, Encode)]
// #[rasn(value = "INTEGER 1..4294967295")]
pub struct IntPosLong(Integer);

#[derive(AsnType, Decode, Encode)]
pub struct IntPosShort(Integer);

#[derive(AsnType)]
pub struct IntUnsignedLong(Integer);

#[derive(AsnType)]
pub struct IntUnsignedShort(Integer);

#[derive(AsnType)]
pub struct InvokeId(Integer);

#[derive(Debug, Clone, Copy, AsnType, Decode, Encode)]
#[rasn(enumerated)]
pub enum ParameterName {
    AcquisitionSequenceLength = 201,
    ApidList = 2,
    BitLockRequired = 3,
    BlockingTimeoutPeriod = 0,
    BlockingUsage = 1,
    BufferSize = 4,
    ClcwGlobalVcid = 202,
    ClcwPhysicalChannel = 203,
    DeliveryMode = 6,
    DirectiveInvocation = 7,
    DirectiveInvocationOnline = 108,
    ExpectedDirectiveIdentification = 8,
    ExpectedEventInvocationIdentification = 9,
    ExpectedSlduIdentification = 10,
    FopSlidingWindow = 11,
    FopState = 12,
    LatencyLimit = 15,
    MapList = 16,
    MapMuxControl = 17,
    MapMuxScheme = 18,
    MaximumFrameLength = 19,
    MaximumPacketLength = 20,
    MaximumSlduLength = 21,
    MinimumDelayTime = 204,
    ModulationFrequency = 22,
    ModulationIndex = 23,
    NotificationMode = 205,
    PermittedControlWordTypeSet = 101,
    PermittedGvcidSet = 24,
    PermittedTcVcidSet = 102,
    PermittedTransmissionMode = 107,
    PermittedUpdateModeSet = 103,
    Plop1IdleSequenceLength = 206,
    PlopInEffect = 25,
    ProtocolAbortMode = 207,
    ReportingCycle = 26,
    RequestedControlWordType = 104,
    RequestedFrameQuality = 27,
    RequestedGvcid = 28,
    RequestedTcVcid = 105,
    RequestedUpdateMode = 106,
    ReturnTimeoutPeriod = 29,
    RfAvailable = 30,
    RfAvailableRequired = 31,
    SegmentHeader = 32,
    SubcarrierToBitRateRatio = 34,
    TimeoutType = 35,
    TimerInitial = 36,
    TransmissionLimit = 37,
    TransmitterFrameSequenceNumber = 38,
    VcMuxControl = 39,
    VcMuxScheme = 40,
    VirtualChannel = 41,
}

#[derive(Debug, Clone, Copy, AsnType, Decode, Encode)]
#[rasn(enumerated)]
pub enum SlduStatusNotification {
    ProduceNotification = 0,
    DoNotProduceNotification = 1,
}

#[derive(AsnType, Decode, Encode)]
pub struct SpaceLinkDataUnit(OctetString);

#[derive(AsnType, Decode, Encode)]
#[rasn(choice)]
pub enum Time {
    #[rasn(tag(0))]
    CcsdsFormat(OctetString),
    #[rasn(tag(1))]
    CcsdsPicoFormat(OctetString),
}

impl Time {
    pub fn get_octet(&self) -> &OctetString {
        match self {
            Self::CcsdsFormat(val) => val,
            Self::CcsdsPicoFormat(val) => val,
        }
    }
}

#[derive(Debug, Clone, Copy, AsnType, Decode, Encode)]
#[rasn(enumerated)]
pub enum ApplicationIdentifier {
    #[rasn(tag(0))]
    RtnAllFrames,
    #[rasn(tag(1))]
    RtnInsert,
    #[rasn(tag(2))]
    RtnChFrames,
    #[rasn(tag(3))]
    RtnChFsh,
    #[rasn(tag(4))]
    RtnChOcf,
    #[rasn(tag(5))]
    RtnBitstr,
    #[rasn(tag(6))]
    RtnSpacePkt,
    #[rasn(tag(7))]
    FwdAosSpacePkt,
    #[rasn(tag(8))]
    FwdAosVca,
    #[rasn(tag(9))]
    FwdBitstr,
    #[rasn(tag(10))]
    FwdProtoVcdu,
    #[rasn(tag(11))]
    FwdInsert,
    #[rasn(tag(12))]
    FwdCVcdu,
    #[rasn(tag(13))]
    FwdTcSpacePkt,
    #[rasn(tag(14))]
    FwdTcVca,
    #[rasn(tag(15))]
    FwdTcFrame,
    #[rasn(tag(16))]
    FwdCltu,
}

#[derive(AsnType, Encode, Decode, Debug, Clone)]
pub struct AuthorityIdentifier(VisibleString);


#[derive(AsnType, Encode, Decode, Debug, Clone)]
pub struct PortId {
    pub port_id: VisibleString,
}

#[derive(Debug, Clone, Copy, AsnType, Decode, Encode)]
#[rasn(enumerated)]
pub enum BindDiagnostic {
    #[rasn(tag(0))]
    AccessDenied,
    #[rasn(tag(1))]
    ServiceTypeNotSupported,
    #[rasn(tag(2))]
    VersionNotSupported,
    #[rasn(tag(3))]
    NoSuchServiceInstance,
    #[rasn(tag(4))]
    AlreadyBound,
    #[rasn(tag(5))]
    SiNotAccessibleToThisInitiator,
    #[rasn(tag(6))]
    InconsistentServiceType,
    #[rasn(tag(7))]
    InvalidTime,
    #[rasn(tag(8))]
    OutOfService,
    #[rasn(tag(127))]
    OtherReason,
}

#[derive(Debug, Clone, AsnType, Encode, Decode)]
pub struct VersionNumber(Integer);

pub const SAGR_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 52]);
pub const SPACK_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 53]);
pub const FSL_FG_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 14]);
pub const RSL_FG_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 38]);
pub const CLTU_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 7]);
pub const FSP_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 10]);
pub const RAF_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 22]);
pub const RCF_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 46]);
pub const RCFSH_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 44]);
pub const ROCF_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 49]);
pub const RSP_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 40]);
pub const TCF_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 12]);
pub const TCVA_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 16]);

#[derive(Debug, Clone, AsnType, Encode, Decode)]
pub struct Attribute {
    pub id: rasn::types::ObjectIdentifier,
}

// GeneralAttributes ATTRIBUTE ::=
// { serviceAgreement
// | servicePackage
// | forwardService
// | returnService
// }

#[derive(Debug, Clone, Copy, AsnType, Encode, Decode)]
#[rasn(enumerated)]
pub enum GeneralAttributes {
    ServiceAgreement,
    ServicePackage,
    ForwardService,
    ReturnService,
}

// ServiceInstanceAttribute ::=
//     SET SIZE(1) OF SEQUENCE {
//         identifier ATTRIBUTE.&id({ServiceInstanceAttributes}),
//         siAttributeValue VisibleString (SIZE (1..256))
//     }

#[derive(Debug, Clone, AsnType, Encode, Decode)]
pub struct ServiceInstanceAttribute {
    pub identifier: Attribute,
    pub si_attribute_value: VisibleString,
}

// ServiceInstanceIdentifier ::= SEQUENCE OF ServiceInstanceAttribute

pub type ServiceInstanceIdentifier = Vec<ServiceInstanceAttribute>;

// ServiceNames ATTRIBUTE ::=
// { rafService
// | rcfService
// | rcfshService
// | rocfService
// | rspService
// | cltuService
// | fspService
// | tcfService
// | tcvaService
// }

#[derive(Debug, Clone, Copy, AsnType, Encode, Decode)]
#[rasn(enumerated)]
enum ServiceNames {
    RafService,
    RcfService,
    RcfshService,
    RocfService,
    RspService,
    CltuService,
    FspService,
    TcfService,
    TcvaService,
}

// ServiceInstanceAttributes ATTRIBUTE ::=
// { GeneralAttributes
// | ServiceNames
// }

#[derive(Debug, Clone, AsnType, Encode, Decode)]
#[rasn(choice)]
enum ServiceInstanceAttributes {
    #[rasn(tag(0))]
    General(GeneralAttributes),
    #[rasn(tag(1))]
    Names(ServiceNames),
}

// #[test]
// fn test_asn1_der() {
//     let attr = Attribute {
//         id: rasn::types::ObjectIdentifier::from_string("1.2.3.4.5.6.7").unwrap(),
//     };

//     let si_attr = ServiceInstanceAttribute {
//         identifier: attr,
//         si_attribute_value: rasn::types::VisibleString::from("some value"),
//     };

//     let si_id = vec![si_attr];

//     let _ = GeneralAttributes::ServiceAgreement;

//     let _ = ServiceInstanceAttributes::General(GeneralAttributes::ServiceAgreement);
// }

// SleBindInvocation
#[derive(Debug, Clone, AsnType, Decode, Encode)]
#[rasn(tag(context, 100))]
pub struct SleBindInvocation {
    pub invoker_credentials: Credentials,
    pub initiator_identifier: VisibleString,
    pub responder_port_identifier: VisibleString,
    pub service_type: Integer,
    pub version_number: Integer,
    pub service_instance_identifier: ServiceInstanceIdentifier,
}
