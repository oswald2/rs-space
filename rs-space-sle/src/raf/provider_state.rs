use rasn::types::{Utf8String, VisibleString};

use crate::sle::config::CommonConfig;

use super::state::RAFState;

#[derive(Debug, Clone)]
pub struct InternalRAFProviderState {
    state: RAFState,
    interval: u16,
    dead_factor: u16,
    user: VisibleString
}

impl InternalRAFProviderState {
    pub fn new(config: &CommonConfig) -> InternalRAFProviderState {
        InternalRAFProviderState {
            state: RAFState::Unbound,
            interval: config.tml.heartbeat,
            dead_factor: config.tml.dead_factor,
            user: VisibleString::new(Utf8String::from("")),
        }
    }

    pub fn reset(&mut self) {
        self.state = RAFState::Unbound
    }

    pub fn set_heartbeat_values(&mut self, interval: u16, dead_factor: u16) {
        self.interval = interval;
        self.dead_factor = dead_factor;
    }

    pub fn user(&self) -> &VisibleString {
        &self.user
    }
}