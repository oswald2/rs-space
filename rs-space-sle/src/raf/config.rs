use serde::{Deserialize, Serialize};

use crate::raf::asn1::{AntennaId, AntennaIdExt, RafDeliveryMode};
use crate::types::sle::SleVersion;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAFConfig {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAFProviderConfigExt {
    pub hostname: String,
    pub port: u16,
    pub server_init_time: u16,
    pub sii: String,
    pub mode: RafDeliveryMode,
    pub provider: String,
    pub responder_port: String,
    pub sle_operation_timeout: u16,
    pub buffer_size: u16,
    pub latency: u32,
    pub antenna_id: AntennaIdExt,
}

impl Default for RAFProviderConfigExt {
    fn default() -> Self {
        RAFProviderConfigExt {
            hostname: "127.0.0.1".to_string(),
            port: 5100,
            server_init_time: 30,
            sii: "sagr=3.spack=facility-PASS1.rsl-fg=1.raf=onlc1".to_string(),
            mode: RafDeliveryMode::RtnCompleteOnline,
            provider: "PARAGONTT".to_string(),
            responder_port: "TMPORT".to_string(),
            sle_operation_timeout: 30,
            buffer_size: 100,
            latency: 500,
            antenna_id: AntennaIdExt::LocalForm("ANTENNA_1".to_string()),
        }
    }
}


#[derive(Debug, Clone)]
pub struct RAFProviderConfig {
    pub hostname: String,
    pub port: u16,
    pub server_init_time: u16,
    pub sii: String,
    pub mode: RafDeliveryMode,
    pub provider: String,
    pub responder_port: String,
    pub sle_operation_timeout: u16,
    pub buffer_size: u16,
    pub latency: u32,
    pub antenna_id: AntennaId,
}


impl TryFrom<&RAFProviderConfigExt> for RAFProviderConfig {
    type Error = String;

    fn try_from(value: &RAFProviderConfigExt) -> Result<Self, Self::Error> {
        let ant = (&value.antenna_id).try_into()?;

        Ok(RAFProviderConfig{
            hostname: value.hostname.clone(),
            port: value.port, 
            server_init_time: value.server_init_time,
            sii: value.sii.clone(), 
            mode: value.mode,
            provider: value.provider.clone(),
            responder_port: value.responder_port.clone(),
            sle_operation_timeout: value.sle_operation_timeout,
            buffer_size: value.buffer_size, 
            latency: value.latency, 
            antenna_id: ant
        })
    }
}


