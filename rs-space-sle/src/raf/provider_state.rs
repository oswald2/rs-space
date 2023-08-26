use std::sync::{atomic::Ordering, Arc};

use rasn::types::{Utf8String, VisibleString};

use super::state::{AtomicRAFState, RAFState};
use crate::types::sle::ParameterName;
use crate::{asn1::*, raf::asn1::*, sle::config::*, types::sle::*};
use crate::raf::config::RAFProviderConfig;


#[derive(Clone)]
pub struct InternalRAFProviderState {
    state: Arc<AtomicRAFState>,
    interval: u16,
    dead_factor: u16,
    user: VisibleString,
    version: SleVersion,
    start_time: Option<rs_space_core::time::Time>,
    stop_time: Option<rs_space_core::time::Time>,
    requested_quality: RequestedFrameQuality,
    buffer_size: u16,
    latency_limit: u16,
    delivery_mode: RafDeliveryMode,
}

impl InternalRAFProviderState {
    pub fn new(config: &CommonConfig, raf_config: &RAFProviderConfig, raf_state: Arc<AtomicRAFState>) -> InternalRAFProviderState {
        InternalRAFProviderState {
            state: raf_state,
            interval: config.tml.heartbeat,
            dead_factor: config.tml.dead_factor,
            user: VisibleString::new(Utf8String::from("")),
            version: SleVersion::V5,
            start_time: None,
            stop_time: None,
            requested_quality: RequestedFrameQuality::AllFrames,
            buffer_size: raf_config.buffer_size,
            latency_limit: 600,
            delivery_mode: raf_config.mode,
        }
    }

    pub fn reset(&mut self) {
        self.user = VisibleString::new(Utf8String::from(""));
        self.version = SleVersion::V5;
        self.state.store(RAFState::Unbound, Ordering::Relaxed);
    }

    pub fn set_heartbeat_values(&mut self, interval: u16, dead_factor: u16) {
        self.interval = interval;
        self.dead_factor = dead_factor;
    }

    pub fn user(&self) -> &VisibleString {
        &self.user
    }

    pub fn state(&self) -> RAFState {
        self.state.load(Ordering::Relaxed)
    }

    pub fn process_bind(
        &mut self,
        initiator: &AuthorityIdentifier,
        version: SleVersion,
    ) -> Result<(), String> {
        if self.state.load(Ordering::Acquire) != RAFState::Unbound {
            Err(format!("RAF BIND while in state {:?}", self.state))
        } else {
            self.user = initiator.clone();
            self.version = version;
            self.state.store(RAFState::Bound, Ordering::Relaxed);
            Ok(())
        }
    }

    pub fn process_unbind(&mut self, _reason: UnbindReason) -> Result<(), String> {
        if self.state.load(Ordering::Acquire) != RAFState::Bound {
            Err(format!("RAF UNBIND while in state {:?}", self.state))
        } else {
            self.reset();
            Ok(())
        }
    }

    pub fn peer_abort(&mut self, _diagnostic: &PeerAbortDiagnostic) {
        self.reset();
    }

    pub fn process_start(
        &mut self,
        start_time: Option<rs_space_core::time::Time>,
        stop_time: Option<rs_space_core::time::Time>,
        quality: RequestedFrameQuality,
    ) -> Result<(), String> {
        if self.state.load(Ordering::Acquire) == RAFState::Bound {
            self.requested_quality = quality;
            self.start_time = start_time;
            self.stop_time = stop_time;
            self.state.store(RAFState::Active, Ordering::Relaxed);
            Ok(())
        } else {
            Err(format!("RAF START while in state {:?}", self.state))
        }
    }

    pub fn process_stop(&mut self) -> Result<(), String> {
        if self.state.load(Ordering::Acquire) == RAFState::Active {
            self.state.store(RAFState::Bound, Ordering::Relaxed);
            self.start_time = None;
            self.stop_time = None;
            Ok(())
        } else {
            Err(format!("RAF STOP while in state {:?}", self.state))
        }
    }

    pub fn process_get_param(&self, param_name: ParameterName) -> RafGetReturnResult {
        match param_name {
            ParameterName::BufferSize => {
                RafGetReturnResult::PositiveResult(RafGetParameter::ParBufferSize {
                    parameter_name: ParameterName::BufferSize as i64,
                    parameter_value: self.buffer_size,
                })
            }
            ParameterName::LatencyLimit => {
                RafGetReturnResult::PositiveResult(RafGetParameter::ParLatencyLimit {
                    parameter_name: ParameterName::LatencyLimit as i64,
                    parameter_value: LatencyLimitValue::Online(self.latency_limit),
                })
            }
            ParameterName::DeliveryMode => {
                RafGetReturnResult::PositiveResult(RafGetParameter::ParDeliveryMode {
                    parameter_name: ParameterName::DeliveryMode as i64,
                    parameter_value: self.delivery_mode as i64,
                })
            }
            _ => RafGetReturnResult::NegativeResult(DiagnosticRafGet::Specific(
                SpecificDiagnosticRafGet::UnknownParameter,
            )),
        }
    }
}
