use crate::raf::config::RAFConfig;
use crate::sle::config::CommonConfig;

use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs::{read_to_string, write};
use tokio::io::{Error, ErrorKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub common: CommonConfig,
    pub rafs: Vec<RAFConfig>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        ProviderConfig {
            common: CommonConfig::default(),
            rafs: vec![RAFConfig::default()],
        }
    }
}

impl ProviderConfig {
    pub async fn read_from_file(filename: &Path) -> Result<ProviderConfig, Error> {
        let content = read_to_string(filename).await?;

        match serde_yaml::from_str(&content) {
            Ok(cfg) => Ok(cfg),
            Err(err) => return Err(Error::new(ErrorKind::Other, format!("{}", err))),
        }
    }

    pub async fn write_to_file(filename: &Path, cfg: &ProviderConfig) -> Result<(), Error> {
        match serde_yaml::to_string(cfg) {
            Err(err) => return Err(Error::new(ErrorKind::Other, format!("{}", err))),
            Ok(yaml) => {
                write(filename, yaml).await?;
                Ok(())
            }
        }
    }
}
