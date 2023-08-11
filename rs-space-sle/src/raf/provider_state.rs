use rasn::types::{Utf8String, VisibleString};

use crate::{asn1::*, sle::config::*, types::sle::*};

use super::state::RAFState;

#[derive(Debug, Clone)]
pub struct InternalRAFProviderState {
    state: RAFState,
    interval: u16,
    dead_factor: u16,
    user: VisibleString,
    version: SleVersion,
}

impl InternalRAFProviderState {
    pub fn new(config: &CommonConfig) -> InternalRAFProviderState {
        InternalRAFProviderState {
            state: RAFState::Unbound,
            interval: config.tml.heartbeat,
            dead_factor: config.tml.dead_factor,
            user: VisibleString::new(Utf8String::from("")),
            version: SleVersion::V5,
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

    pub fn process_bind(&mut self, initiator: &AuthorityIdentifier, version: SleVersion) {
        self.user = initiator.clone();
        self.version = version;
        self.state = RAFState::Bound;
    }

    pub fn process_unbind(&mut self, _reason: UnbindReason) {
        self.reset();
    }

    pub fn peer_abort(&mut self, _diagnostic: &PeerAbortDiagnostic)
    {
        self.reset();
    }
}
