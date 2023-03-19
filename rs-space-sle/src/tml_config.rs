use std::path::Path;

use serde::{Deserialize, Serialize};

use tokio::fs::{read_to_string, write};
use tokio::io::{Error, ErrorKind};

#[derive(Debug, Serialize, Deserialize)]
pub struct TMLConfig {
    pub heartbeat: u16,
    pub dead_factor: u16,
    pub server_init_time: u16,
    pub min_heartbeat: u16,
    pub max_heartbeat: u16,
    pub min_dead_factor: u16,
    pub max_dead_factor: u16,
}

impl TMLConfig {
    pub async fn read_from_file(filename: &Path) -> Result<TMLConfig, Error> {
        let content = read_to_string(filename).await?;

        match serde_yaml::from_str(&content) {
            Ok(cfg) => Ok(cfg),
            Err(err) => return Err(Error::new(ErrorKind::Other, format!("{}", err))),
        }
    }

    pub async fn write_to_file(filename: &Path, cfg: &TMLConfig) -> Result<(), Error> {
        match serde_yaml::to_string(cfg) {
            Err(err)=> return Err(Error::new(ErrorKind::Other, format!("{}", err))),
            Ok(yaml) => {
                write(filename, yaml).await?;
                Ok(())
            }
        }
    }
}
