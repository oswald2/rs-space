use crate::raf::config::RAFProviderConfig;
use crate::sle::config::{CommonConfig, CommonConfigExt};

use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs::{read_to_string, write};
use tokio::io::{Error, ErrorKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfigExt {
    pub common: CommonConfigExt,
    pub rafs: Vec<RAFProviderConfig>,
}

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub common: CommonConfig,
    pub rafs: Vec<RAFProviderConfig>,
}

impl ProviderConfig {
    pub fn from(conf: ProviderConfigExt) -> Self {
        ProviderConfig {
            common: CommonConfig::from(conf.common),
            rafs: conf.rafs,
        }
    }
}

impl Default for ProviderConfigExt {
    fn default() -> Self {
        ProviderConfigExt {
            common: CommonConfigExt::default(),
            rafs: vec![RAFProviderConfig::default()],
        }
    }
}

impl Default for ProviderConfig {
    fn default() -> Self {
        ProviderConfig::from(ProviderConfigExt::default())
    }
}

impl ProviderConfigExt {
    pub async fn read_from_file(filename: &Path) -> Result<ProviderConfig, Error> {
        let content = read_to_string(filename).await?;

        match serde_yaml::from_str(&content) {
            Ok(cfg) => Ok(ProviderConfig::from(cfg)),
            Err(err) => return Err(Error::new(ErrorKind::Other, format!("{}", err))),
        }
    }

    pub async fn write_to_file(filename: &Path, cfg: &ProviderConfigExt) -> Result<(), Error> {
        match serde_yaml::to_string(cfg) {
            Err(err) => return Err(Error::new(ErrorKind::Other, format!("{}", err))),
            Ok(yaml) => {
                write(filename, yaml).await?;
                Ok(())
            }
        }
    }
}
