use rasn::types::{Utf8String, VisibleString};

use crate::{asn1::*, raf::asn1::*, sle::config::*, types::sle::*};

use super::state::RAFState;

#[derive(Debug, Clone)]
pub struct InternalRAFProviderState {
    state: RAFState,
    interval: u16,
    dead_factor: u16,
    user: VisibleString,
    version: SleVersion,
    start_time: Option<rs_space_core::time::Time>,
    stop_time: Option<rs_space_core::time::Time>,
    requested_quality: RequestedFrameQuality,
}

impl InternalRAFProviderState {
    pub fn new(config: &CommonConfig) -> InternalRAFProviderState {
        InternalRAFProviderState {
            state: RAFState::Unbound,
            interval: config.tml.heartbeat,
            dead_factor: config.tml.dead_factor,
            user: VisibleString::new(Utf8String::from("")),
            version: SleVersion::V5,
            start_time: None,
            stop_time: None,
            requested_quality: RequestedFrameQuality::AllFrames,
        }
    }

    pub fn reset(&mut self) {
        self.user = VisibleString::new(Utf8String::from(""));
        self.version = SleVersion::V5;
        self.state = RAFState::Unbound;
    }

    pub fn set_heartbeat_values(&mut self, interval: u16, dead_factor: u16) {
        self.interval = interval;
        self.dead_factor = dead_factor;
    }

    pub fn user(&self) -> &VisibleString {
        &self.user
    }

    pub fn process_bind(&mut self, initiator: &AuthorityIdentifier, version: SleVersion) -> Result<(), String> {
        if self.state != RAFState::Unbound {
            Err(format!("RAF BIND while in state {:?}", self.state))
        } else {
            self.user = initiator.clone();
            self.version = version;
            self.state = RAFState::Bound;
            Ok(())
        }
    }

    pub fn process_unbind(&mut self, _reason: UnbindReason) -> Result<(), String> {
        if self.state != RAFState::Bound {
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
        if self.state == RAFState::Bound {
            self.requested_quality = quality;
            self.start_time = start_time;
            self.stop_time = stop_time;
            self.state = RAFState::Active;
            Ok(())
        } else {
            Err(format!("RAF START while in state {:?}", self.state))
        }
    }

    pub fn process_stop(&mut self) -> Result<(), String> {
        if self.state == RAFState::Active {
            self.state = RAFState::Bound;
            self.start_time = None;
            self.stop_time = None;
            Ok(())
        }
        else {
            Err(format!("RAF STOP while in state {:?}", self.state))
        }

    }
}
