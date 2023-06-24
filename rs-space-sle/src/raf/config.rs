use serde::{Deserialize, Serialize};

use crate::types::sle::SleVersion;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAFConfig{
    pub hostname: String, 
    pub port: u16, 
    pub sii: String,
    pub initiator: String,
    pub responder_port: String, 
    pub version: SleVersion, 
    pub sle_operation_timeout: u16,
}


impl Default for RAFConfig {
    fn default() -> Self {
        RAFConfig {
            hostname: "localhost".to_string(),
            port: 5100,
            sii: "sagr=3.spack=facility-PASS1.rsl-fg=1.raf=onlc1".to_string(),
            initiator: "SLETT".to_string(),
            responder_port: "TMPORT".to_string(),
            version: SleVersion::V4,
            sle_operation_timeout: 30,
        }
    }
}
