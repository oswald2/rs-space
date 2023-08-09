#[allow(unused)]
use std::collections::BTreeSet;

use rasn::prelude::*;
use rasn::{AsnType, Decode, Encode};

use crate::types::sle::{convert_ccsds_time, Credentials, Diagnostics, Time};

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
            x => Err(format!("Requested frame quality has unexpected value: {x}")),
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

#[derive(AsnType, Debug, Clone, PartialEq, Encode, Decode)]
#[rasn(choice)]
pub enum AntennaId {
    #[rasn(tag(0))]
    GlobalForm(ObjectIdentifier),
    #[rasn(tag(1))]
    LocalForm(OctetString),
}

#[derive(Debug, Clone, PartialEq)]
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

pub type FrameSyncLockStatus = LockStatus;
pub type CarrierLockStatus = LockStatus;
pub type SymbolLockStatus = LockStatus;

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
    LossFrameSync(Integer),
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

pub fn convert_frame(frame: &RafTransferDataInvocation) -> Result<SleTMFrame, String> {
    let t = convert_ccsds_time(&frame.earth_receive_time)?;
    let fq = FrameQuality::try_from(frame.delivered_frame_quality)?;

    Ok(SleTMFrame {
        earth_receive_time: t,
        antenna_id: frame.antenna_id.clone(),
        data_link_continuity: frame.data_link_continuity,
        delivered_frame_quality: fq,
        private_annotation: frame.private_annotation.clone(),
        data: frame.data.clone(),
    })
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
