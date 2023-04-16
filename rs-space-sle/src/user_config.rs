use crate::raf_config::RAFConfig;
use crate::tml_config::TMLConfig;

use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs::{read_to_string, write};
use tokio::io::{Error, ErrorKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub tml_config: TMLConfig,
    pub rafs: Vec<RAFConfig>,
}

impl Default for UserConfig {
    fn default() -> Self {
        UserConfig {
            tml_config: TMLConfig::default(),
            rafs: Vec::new(),
        }
    }
}

impl UserConfig {
    pub async fn read_from_file(filename: &Path) -> Result<UserConfig, Error> {
        let content = read_to_string(filename).await?;

        match serde_yaml::from_str(&content) {
            Ok(cfg) => Ok(cfg),
            Err(err) => return Err(Error::new(ErrorKind::Other, format!("{}", err))),
        }
    }

    pub async fn write_to_file(filename: &Path, cfg: &UserConfig) -> Result<(), Error> {
        match serde_yaml::to_string(cfg) {
            Err(err) => return Err(Error::new(ErrorKind::Other, format!("{}", err))),
            Ok(yaml) => {
                write(filename, yaml).await?;
                Ok(())
            }
        }
    }
}