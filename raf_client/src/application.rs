#[allow(unused)]
use std::collections::BTreeSet;

use rs_space_sle::{raf::client::SleClientHandle, asn1_raf::UnbindReason};
use rs_space_sle::user_config::UserConfig;
use tokio::io::Error;

use log::{error, info};

pub async fn run_app(config: &UserConfig) -> Result<(), Error> {
    for raf_config in &config.rafs {
        let config = (*config).clone();
        let raf_config = (*raf_config).clone();

        let address = format!("{}:{}", raf_config.hostname, raf_config.port);
        info!("Connecting to {}...", address);

        let mut raf = SleClientHandle::sle_connect_raf(&config.tml_config, &raf_config).await?;

        //std::thread::sleep(std::time::Duration::from_secs(2));

        info!("Sending SLE BIND...");
        match raf.bind(&config.rafs[0]).await {
            Ok(_) => {}
            Err(err) => {
                error!("Bind returned error: {err}");
                return Err(Error::new(std::io::ErrorKind::ConnectionRefused, err));
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(2));

        info!("Sending SLE UNBIND...");
        match raf.unbind(UnbindReason::End).await {
            Ok(_) => {}
            Err(err) => {
                error!("UNBIND returned error: {err}");
                return Err(Error::new(std::io::ErrorKind::ConnectionRefused, err));
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(5));
    }
    Ok(())
}
