use num_traits::ToPrimitive;
#[allow(unused)]
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use rasn::prelude::*;
use rasn::{AsnType, Decode, Encode};

use crate::types::sle::{convert_ccsds_time, Credentials, Diagnostics, Time};
use crate::asn1::IntPosShort;

use bytes::Bytes;

#[derive(AsnType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Encode, Decode)]
#[rasn(enumerated)]
pub enum RequestedFrameQuality {
    GoodFramesOnly = 0,
    ErredFramesOnly = 1,
    AllFrames = 2,
}

impl TryFrom<&Integer> for RequestedFrameQuality {
    type Error = String;

    fn try_from(val: &Integer) -> Result<RequestedFrameQuality, String> {
        match val.to_i64() {
            Some(0) => Ok(RequestedFrameQuality::GoodFramesOnly),
            Some(1) => Ok(RequestedFrameQuality::ErredFramesOnly),
            Some(2) => Ok(RequestedFrameQuality::AllFrames),
            Some(x) => Err(format!("Requested frame quality has unexpected value: {x}")),
            None => Err(format!("Requested frame quality has unexpected value")),
        }
    }
}

#[derive(AsnType, Debug, Clone, Copy, PartialEq, Encode, Decode)]
#[rasn(choice)]
pub enum DiagnosticRafStart {
    #[rasn(tag(0))]
    Common(Diagnostics),
    #[rasn(tag(1))]
    Specific(SpecificDiagnosticRafStart),
}

#[derive(AsnType, Debug, Clone, Copy, PartialEq, Encode, Decode)]
#[rasn(choice)]
pub enum DiagnosticRafGet {
    #[rasn(tag(0))]
    Common(Diagnostics),
    #[rasn(tag(1))]
    Specific(SpecificDiagnosticRafGet),
}

#[derive(AsnType, Debug, Clone, Copy, PartialEq, Encode, Decode)]
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

#[derive(AsnType, Debug, Clone, Copy, PartialEq, Encode, Decode)]
#[rasn(choice)]
pub enum SpecificDiagnosticRafGet {
    #[rasn(tag(0))]
    UnknownParameter = 0,
}

#[derive(AsnType, Debug, Clone, Copy, PartialEq, Encode, Decode)]
#[rasn(choice)]
pub enum RafStartReturnResult {
    #[rasn(tag(0))]
    PositiveResult,
    #[rasn(tag(1))]
    NegativeResult(DiagnosticRafStart),
}

#[derive(AsnType, Debug, Clone, Copy, PartialEq, Encode, Decode)]
#[rasn(choice)]
pub enum RafGetReturnResult {
    #[rasn(tag(0))]
    PositiveResult,
    #[rasn(tag(1))]
    NegativeResult(DiagnosticRafGet),
}


#[derive(AsnType, Debug, Clone, PartialEq, Encode, Decode)]
#[rasn(choice)]
pub enum AntennaId {
    #[rasn(tag(0))]
    GlobalForm(ObjectIdentifier),
    #[rasn(tag(1))]
    LocalForm(OctetString),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AntennaIdExt {
    GlobalForm(Vec<u32>),
    LocalForm(String),
}

impl TryFrom<&AntennaIdExt> for AntennaId {
    type Error = String;

    fn try_from(value: &AntennaIdExt) -> Result<Self, Self::Error> {
        match value {
            AntennaIdExt::GlobalForm(vec) => {
                let vec2 = vec.clone();
                match ObjectIdentifier::new(vec.clone()) {
                    None => Err(format!("Illegal AntennaID value (Global Form): {:?}", vec2)),
                    Some(val) => Ok(AntennaId::GlobalForm(val)),
                }
            }
            AntennaIdExt::LocalForm(str) => {
                Ok(AntennaId::LocalForm(Bytes::copy_from_slice(str.as_ref())))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameQuality {
    Good = 0,
    Erred = 1,
    Undetermined = 2,
}

impl TryFrom<i32> for FrameQuality {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FrameQuality::Good),
            1 => Ok(FrameQuality::Erred),
            2 => Ok(FrameQuality::Undetermined),
            x => Err(format!("Invalid value for Frame Quality {x}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LockStatus {
    InLock = 0,
    OutOfLock = 1,
    NotInUse = 2,
    Unknown = 3,
}

pub type FrameSyncLockStatus = Integer;
pub type CarrierLockStatus = Integer;
pub type SymbolLockStatus = Integer;

#[derive(AsnType, Debug, Clone, PartialEq)]
pub struct LockStatusReport {
    pub time: Time,
    pub carrier_lock_status: CarrierLockStatus,
    pub subcarrier_lock_status: LockStatus,
    pub symbol_sync_lock_status: SymbolLockStatus,
}

#[derive(AsnType, Debug, Clone, PartialEq, Encode, Decode)]
#[rasn(choice)]
pub enum Notification {
    #[rasn(tag(0))]
    LossFrameSync {
        time: Time,
        carrier_lock_status: CarrierLockStatus,
        subcarrier_lock_status: Integer,
        symbol_sync_lock_status: SymbolLockStatus,
    },
    #[rasn(tag(1))]
    ProductionStatusChange(Integer),
    #[rasn(tag(2))]
    ExcessiveDataBacklog,
    #[rasn(tag(3))]
    EndOfData,
}

#[derive(AsnType, Debug, Clone, PartialEq, Decode, Encode)]
#[rasn(choice)]
pub enum FrameOrNotification {
    #[rasn(tag(0))]
    AnnotatedFrame(RafTransferDataInvocation),
    #[rasn(tag(1))]
    SyncNotification(RafSyncNotifyInvocation),
}

#[derive(AsnType, Debug, Clone, PartialEq, Encode, Decode)]
pub struct RafTransferDataInvocation {
    pub invoker_credentials: Credentials,
    pub earth_receive_time: Time,
    pub antenna_id: AntennaId,
    pub data_link_continuity: i32,
    pub delivered_frame_quality: i32,
    pub private_annotation: PrivateAnnotation,
    pub data: SpaceLinkDataUnit,
}

#[derive(Debug, Clone)]
pub struct SleTMFrame {
    pub earth_receive_time: rs_space_core::time::Time,
    pub antenna_id: AntennaId,
    pub data_link_continuity: i32,
    pub delivered_frame_quality: FrameQuality,
    pub private_annotation: PrivateAnnotation,
    pub data: SpaceLinkDataUnit,
}

impl TryFrom<&RafTransferDataInvocation> for SleTMFrame {
    type Error = String;

    fn try_from(value: &RafTransferDataInvocation) -> Result<Self, Self::Error> {
        let t = convert_ccsds_time(&value.earth_receive_time)?;
        let fq = FrameQuality::try_from(value.delivered_frame_quality)?;

        Ok(SleTMFrame {
            earth_receive_time: t,
            antenna_id: value.antenna_id.clone(),
            data_link_continuity: value.data_link_continuity,
            delivered_frame_quality: fq,
            private_annotation: value.private_annotation.clone(),
            data: value.data.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct SleFrame {
    pub earth_receive_time: rs_space_core::time::Time,
    pub delivered_frame_quality: FrameQuality,
    pub data: SpaceLinkDataUnit,
}

#[derive(AsnType, Debug, Clone, PartialEq, Encode, Decode)]
#[rasn(choice)]
pub enum PrivateAnnotation {
    #[rasn(tag(0))]
    Null,
    #[rasn(tag(1))]
    NotNull(OctetString),
}

#[derive(AsnType, Debug, Clone, PartialEq, Encode, Decode)]
pub struct RafSyncNotifyInvocation {
    pub invoker_credentials: Credentials,
    pub notification: Notification,
}

pub type RafTransferBuffer = Vec<FrameOrNotification>;

pub type SpaceLinkDataUnit = OctetString;



#[derive(AsnType, Debug, Copy, Clone, PartialEq, PartialOrd, Encode, Decode)]
#[rasn(enumerated)]
pub enum RafDeliveryMode {
    RtnTimelyOnline,
    RtnCompleteOnline,
    RtnOffline,
}

#[derive(Debug, PartialEq, Clone, AsnType, Decode, Encode)]
#[rasn(choice)]
pub enum LatencyLimitValue {
    Online(IntPosShort),
    Offline,
}

#[derive(Debug, PartialEq, Clone, rasn::AsnType, Decode, Encode)]
pub struct PermittedFrameQualitySet(SetOf<RequestedFrameQuality>);

type ReportingCycle = Integer;


// CurrentReportingCycle definition
#[derive(Debug, PartialEq, Clone, rasn::AsnType, rasn::Decode, rasn::Encode)]
#[rasn(choice)]
pub enum CurrentReportingCycle {
    PeriodicReportingOff,        // Corresponds to the NULL value
    PeriodicReportingOn(ReportingCycle),
}


type TimeoutPeriod = Integer; 

#[derive(Debug, PartialEq, Clone, AsnType, Decode, Encode)]
#[rasn(choice)]
pub enum RafGetParameter {
    #[rasn(tag(Context, 0))]
    ParBufferSize {
        parameter_name: Integer,
        parameter_value: IntPosShort,
    },
    #[rasn(tag(Context, 1))]
    ParDeliveryMode {
        parameter_name: Integer,
        parameter_value: RafDeliveryMode,
    },
    #[rasn(tag(Context, 2))]
    ParLatencyLimit {
        parameter_name: Integer,
        parameter_value: LatencyLimitValue,
    },
    #[rasn(tag(Context, 7))]
    ParMinReportingCycle {
        parameter_name: Integer,
        parameter_value: IntPosShort,
    },
    #[rasn(tag(Context, 6))]
    ParPermittedFrameQuality {
        parameter_name: Integer,
        parameter_value: PermittedFrameQualitySet,
    },
    #[rasn(tag(Context, 3))]
    ParReportingCycle {
        parameter_name: Integer,
        parameter_value: CurrentReportingCycle,
    },
    #[rasn(tag(Context, 4))]
    ParReqFrameQuality {
        parameter_name: Integer,
        parameter_value: RequestedFrameQuality,
    },
    #[rasn(tag(Context, 5))]
    ParReturnTimeout {
        parameter_name: Integer,
        parameter_value: TimeoutPeriod,
    },
}
