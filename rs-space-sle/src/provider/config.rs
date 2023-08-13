use crate::raf::config::{RAFProviderConfig, RAFProviderConfigExt};
use crate::sle::config::{CommonConfig, CommonConfigExt};

use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs::{read_to_string, write};
use tokio::io::{Error, ErrorKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfigExt {
    pub common: CommonConfigExt,
    pub rafs: Vec<RAFProviderConfigExt>,
}

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub common: CommonConfig,
    pub rafs: Vec<RAFProviderConfig>,
}

impl ProviderConfig {
    pub fn from(conf: ProviderConfigExt) -> Result<Self, String> {
        let mut new_provs = Vec::new();
        
        for prov in &conf.rafs {
            let new_prov = prov.try_into()?;
            new_provs.push(new_prov);
        }
      
        Ok(ProviderConfig {
            common: CommonConfig::from(conf.common),
            rafs: new_provs,
        })
    }
}

impl Default for ProviderConfigExt {
    fn default() -> Self {
        ProviderConfigExt {
            common: CommonConfigExt::default(),
            rafs: vec![RAFProviderConfigExt::default()],
        }
    }
}

impl Default for ProviderConfig {
    fn default() -> Self {
        ProviderConfig::from(ProviderConfigExt::default()).unwrap()
    }
}

impl ProviderConfigExt {
    pub async fn read_from_file(filename: &Path) -> Result<ProviderConfig, Error> {
        let content = read_to_string(filename).await?;

        match serde_yaml::from_str::<ProviderConfigExt>(&content) {
            Ok(cfg) => {
                match ProviderConfig::from(cfg) {
                    Err(err) => Err(Error::new(ErrorKind::InvalidData, format!("{err}"))),
                    Ok(cfg) => Ok(cfg)
                }
            }
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
