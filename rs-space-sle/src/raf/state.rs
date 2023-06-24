use log::{error, info};

use crate::asn1_raf::BindResult;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RAFState {
    #[default]
    Unbound,
    Bound,
    Active,
}

#[derive(Debug, Default, Clone)]
pub struct InternalRAFState {
    state: RAFState,
    responder: String,
}

impl InternalRAFState {
    pub fn new() -> Self {
        InternalRAFState::default()
    }

    pub fn process_bind_return(&mut self, responder: &str, result: BindResult) {
        match result {
            BindResult::BindOK(_) => {
                info!("BIND operation successful from responder {responder}");
                self.state = RAFState::Bound;
                self.responder = responder.to_string();
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

    pub fn get_state(&self) -> RAFState {
        self.state
    }
}
