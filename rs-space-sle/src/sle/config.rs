use rs_space_core::pus_types::HexBytes;
use serde::{Deserialize, Serialize};

use crate::{tml::config::TMLConfig, types::sle::SleVersion};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum HashToUse {
    SHA1,
    SHA256,
}

impl Default for HashToUse {
    fn default() -> Self {
        HashToUse::SHA256
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum SleAuthType {
    AuthNone,
    AuthBind,
    AuthAll,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonConfig {
    pub tml: TMLConfig,
    pub authority_identifier: String,
    pub password: HexBytes,
    pub auth_type: SleAuthType,
    pub hash_to_use: HashToUse,
    pub version: SleVersion,
}

impl Default for CommonConfig {
    fn default() -> Self {
        CommonConfig {
            tml: TMLConfig::default(),
            authority_identifier: "PARAGONTT".to_string(),
            password: HexBytes(vec![
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
                0x0e, 0x0f,
            ]),
            auth_type: SleAuthType::AuthNone,
            hash_to_use: HashToUse::SHA256,
            version: SleVersion::V3,
        }
    }
}
