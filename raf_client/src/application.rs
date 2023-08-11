#[allow(unused)]
use std::collections::BTreeSet;

use rs_space_sle::asn1::UnbindReason;
use rs_space_sle::raf::asn1::{RequestedFrameQuality, SleTMFrame};
use rs_space_sle::raf::client::RAFUser;
use rs_space_sle::user::config::UserConfig;
use tokio::io::Error;

use log::{error, info};

fn frame_callback(frame: &SleTMFrame) {
    info!("Got Frame: {:?}", frame);
}

pub async fn run_app(config: &UserConfig) -> Result<(), Error> {
    for raf_config in &config.rafs {
        let config = (*config).clone();
        let raf_config = (*raf_config).clone();

        let address = format!("{}:{}", raf_config.hostname, raf_config.port);
        info!("Connecting to {}...", address);

        let mut raf = RAFUser::new(&config.common, &raf_config, frame_callback);

        //std::thread::sleep(std::time::Duration::from_secs(2));

        info!("Sending SLE BIND...");
        if let Err(err) = raf.bind().await {
            error!("Bind returned error: {err}");
            return Err(Error::new(std::io::ErrorKind::ConnectionRefused, err));
        }

        std::thread::sleep(std::time::Duration::from_secs(1));

        info!("Starting SLE RAF service...");
        if let Err(err) = raf
            .start(None, None, RequestedFrameQuality::AllFrames)
            .await
        {
            error!("RAF Start returned error: {err}");
            return Err(Error::new(std::io::ErrorKind::ConnectionRefused, err));
        }

        // std::thread::sleep(std::time::Duration::from_secs(5));
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen to CTRL-C event");

        info!("Stopping SLE RAF service...");
        if let Err(err) = raf.stop().await {
            error!("RAF Start returned error: {err}");
            return Err(Error::new(std::io::ErrorKind::ConnectionRefused, err));
        }

        info!("Sending SLE UNBIND...");
        match raf.unbind(UnbindReason::End).await {
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
