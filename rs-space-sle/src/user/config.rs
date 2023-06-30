use crate::raf::config::RAFConfig;
use crate::sle::config::{CommonConfig, CommonConfigExt};

use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs::{read_to_string, write};
use tokio::io::{Error, ErrorKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfigExt {
    pub common: CommonConfigExt,
    pub rafs: Vec<RAFConfig>,
}

#[derive(Debug, Clone)]
pub struct UserConfig {
    pub common: CommonConfig,
    pub rafs: Vec<RAFConfig>,
}

impl UserConfig {
    pub fn from(conf: UserConfigExt) -> Self {
        UserConfig {
            common: CommonConfig::from(conf.common),
            rafs: conf.rafs,
        }
    }
}

impl Default for UserConfig {
    fn default() -> Self {
        UserConfig::from(UserConfigExt::default())
    }
}

impl Default for UserConfigExt {
    fn default() -> Self {
        UserConfigExt {
            common: CommonConfigExt::default(),
            rafs: vec![RAFConfig::default()],
        }
    }
}

impl UserConfigExt {
    pub async fn read_from_file(filename: &Path) -> Result<UserConfig, Error> {
        let content = read_to_string(filename).await?;

        match serde_yaml::from_str(&content) {
            Ok(cfg) => Ok(UserConfig::from(cfg)),
            Err(err) => return Err(Error::new(ErrorKind::Other, format!("{}", err))),
        }
    }

    pub async fn write_to_file(filename: &Path, cfg: &UserConfigExt) -> Result<(), Error> {
        match serde_yaml::to_string(cfg) {
            Err(err) => return Err(Error::new(ErrorKind::Other, format!("{}", err))),
            Ok(yaml) => {
                write(filename, yaml).await?;
                Ok(())
            }
        }
    }
}
