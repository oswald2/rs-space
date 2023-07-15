#[allow(unused)]
use std::collections::BTreeSet;

use rs_space_sle::raf::asn1::RequestedFrameQuality;
use rs_space_sle::user::config::UserConfig;
use rs_space_sle::{asn1::UnbindReason, raf::client::RAFClient};
use tokio::io::Error;

use log::{error, info};

pub async fn run_app(config: &UserConfig) -> Result<(), Error> {
    for raf_config in &config.rafs {
        let config = (*config).clone();
        let raf_config = (*raf_config).clone();

        let address = format!("{}:{}", raf_config.hostname, raf_config.port);
        info!("Connecting to {}...", address);

        let mut raf = RAFClient::sle_connect_raf(&config.common, &raf_config).await?;

        //std::thread::sleep(std::time::Duration::from_secs(2));

        info!("Sending SLE BIND...");
        if let Err(err) = raf.bind(&config.common, &config.rafs[0]).await {
            error!("Bind returned error: {err}");
            return Err(Error::new(std::io::ErrorKind::ConnectionRefused, err));
        }

        std::thread::sleep(std::time::Duration::from_secs(1));

        info!("Starting SLE RAF service...");
        if let Err(err) = raf.start(&config.common, &config.rafs[0], None, None, RequestedFrameQuality::AllFrames).await {
            error!("RAF Start returned error: {err}");
            return Err(Error::new(std::io::ErrorKind::ConnectionRefused, err));
        }

        // std::thread::sleep(std::time::Duration::from_secs(5));
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen to CTRL-C event");

        info!("Stopping SLE RAF service...");
        if let Err(err) = raf.stop(&config.common, &config.rafs[0]).await {
            error!("RAF Start returned error: {err}");
            return Err(Error::new(std::io::ErrorKind::ConnectionRefused, err));
        }


        info!("Sending SLE UNBIND...");
        match raf.unbind(&config.common, UnbindReason::End).await {
            Ok(_) => {}
            Err(err) => {
                error!("UNBIND returned error: {err}");
                return Err(Error::new(std::io::ErrorKind::ConnectionRefused, err));
            }
        }

        raf.stop_processing().await;
    }
    Ok(())
}
