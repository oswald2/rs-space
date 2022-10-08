use serde::{Deserialize, Serialize};

use crate::pus_packet::PUSPacket;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    SendPkt(PUSPacket),
    RepeatN(u32, Box<EDSL>),
    Log(String)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EDSL {
    pub actions: Vec<Action>
}