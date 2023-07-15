#[allow(unused)]
use std::collections::BTreeSet;

use rs_space_sle::provider::config::ProviderConfig;
//use rs_space_sle::{asn1::UnbindReason, raf::client::RAFClient};
use tokio::io::Error;

//use log::{error, info};

// fn frame_callback(frame: &SleTMFrame) {
//     info!("Got Frame: {:?}", frame);
// }


pub async fn run_app(_config: &ProviderConfig) -> Result<(), Error> {
    // for raf_config in &config.rafs {
    //     let config = (*config).clone();
    //     let raf_config = (*raf_config).clone();

    //     let address = format!("{}:{}", raf_config.hostname, raf_config.port);
    //     info!("Listening on {}...", address);

    //     let mut raf = RAFClient::sle_connect_raf(&config.common, &raf_config, frame_callback).await?;

    //     //std::thread::sleep(std::time::Duration::from_secs(2));

    //     info!("Sending SLE BIND...");
    //     match raf.bind(&config.common, &config.rafs[0]).await {
    //         Ok(_) => {}
    //         Err(err) => {
    //             error!("Bind returned error: {err}");
    //             return Err(Error::new(std::io::ErrorKind::ConnectionRefused, err));
    //         }
    //     }

    //     std::thread::sleep(std::time::Duration::from_secs(2));

    //     info!("Sending SLE UNBIND...");
    //     match raf.unbind(&config.common, UnbindReason::End).await {
    //         Ok(_) => {}
    //         Err(err) => {
    //             error!("UNBIND returned error: {err}");
    //             return Err(Error::new(std::io::ErrorKind::ConnectionRefused, err));
    //         }
    //     }

    //     tokio::signal::ctrl_c()
    //         .await
    //         .expect("failed to listen to CTRL-C event");

    //     raf.stop_processing().await;
    // }
    Ok(())
}
