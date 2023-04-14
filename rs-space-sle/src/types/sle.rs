
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum SleVersion {
    V3 = 3,
    V4 = 4,
    V5 = 5
}

impl TryFrom<u8> for SleVersion {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            3 => Ok(SleVersion::V3), 
            4 => Ok(SleVersion::V4),
            5 => Ok(SleVersion::V5),
            x => Err(format!("Illegal number for SLE Version: {}", x)),
        }
    }
}