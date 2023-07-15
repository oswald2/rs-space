use log::{error, info};

use crate::asn1::{BindResult, SleResult};
use crate::raf::asn1::{RafStartReturnResult, SleTMFrame};
use rasn::types::{Utf8String, VisibleString};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RAFState {
    #[default]
    Unbound,
    Bound,
    Active,
}

pub type FrameCallback = fn(&SleTMFrame);

#[derive(Debug, Clone)]
pub struct InternalRAFState {
    state: RAFState,
    provider: VisibleString,
    frame_callback: FrameCallback,
}

impl InternalRAFState {
    pub fn new(frame_callback: FrameCallback) -> Self {
        InternalRAFState {
            state: RAFState::Unbound,
            provider: VisibleString::new(Utf8String::from("")),
            frame_callback: frame_callback,
        }
    }

    pub fn provider(&self) -> &VisibleString {
        &self.provider
    }

    pub fn process_bind_return(&mut self, responder: &VisibleString, result: &BindResult) {
        match result {
            BindResult::BindOK(_) => {
                info!(
                    "BIND operation successful from responder {}",
                    responder.value
                );
                self.state = RAFState::Bound;
                self.provider = responder.clone();
            }
            BindResult::BindDiag(diag) => {
                error!("BIND returned error: {:?}", diag);
            }
        }
    }

    pub fn process_unbind(&mut self) {
        self.state = RAFState::Unbound;
        info!("UNBIND operation successful");
    }

    pub fn process_start(&mut self, res: &RafStartReturnResult) {
        match res {
            RafStartReturnResult::PositiveResult => {
                self.state = RAFState::Active;
                info!("RAF START operation successful");
            }
            RafStartReturnResult::NegativeResult(err) => {
                error!("RAF START failed with result: {:?}", err);
            }
        }
    }

    pub fn process_stop(&mut self, res: &SleResult) {
        match res {
            SleResult::PositiveResult => {
                self.state = RAFState::Bound;
                info!("RAF STOP operation successful");
            }
            SleResult::NegativeResult(err) => {
                error!("RAF STOP failed with result: {:?}", err);
            }
        }
    }

    pub fn get_state(&self) -> RAFState {
        self.state
    }

    pub fn process_tm_frame(&self, res: &SleTMFrame) {
        (self.frame_callback)(res);
    }
}
