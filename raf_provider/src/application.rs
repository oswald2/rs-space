#[allow(unused)]
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use log::{info, error};
use rs_space_sle::raf::provider::RAFProvider;
use rs_space_sle::sle::config::CommonConfig;
use rs_space_sle::{
    provider::{app_interface::ProviderNotifier, config::ProviderConfig},
    raf::config::RAFProviderConfig,
    types::sle::{SleVersion, PeerAbortDiagnostic},
};
use rs_space_sle::asn1::UnbindReason;

//use rs_space_sle::{asn1::UnbindReason, raf::client::RAFClient};
use tokio::io::Error;

//use log::{error, info};

// fn frame_callback(frame: &SleTMFrame) {
//     info!("Got Frame: {:?}", frame);
// }

struct Notifier {}

impl Notifier {
    pub fn new() -> Notifier {
        Notifier {}
    }
}

impl ProviderNotifier for Notifier {
    fn bind_succeeded(&self, peer: &str, sii: &str, version: SleVersion) {
        info!("BIND SUCCEEDED from {peer} for {sii} for version {version}");
    }

    fn unbind_succeeded(&self, sii: &str, reason: UnbindReason) {
        info!("UNBIND SUCCEEDED for {sii} with reason {reason:?}");
    }

    fn peer_abort(&self, sii: &str, diagnostic: &PeerAbortDiagnostic) {
        error!("PEER ABORT: for {sii} diagnostic: {diagnostic:?}");
    }

}

pub async fn run_app(config: &ProviderConfig) -> Result<(), Error> {
    for raf_config in &config.rafs {
        run_service_instance(&config.common, raf_config.clone()).await;
    }
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

async fn run_service_instance(common_config: &CommonConfig, config: RAFProviderConfig) {
    let config2 = common_config.clone();
    
    info!("Starting SLE instance {} on TCP port {} (SLE Port {})", config.sii, config.port, config.responder_port);

    let hdl = tokio::spawn(async move {
        let notifier = Arc::new(Mutex::new(Notifier::new()));

        let mut provider = RAFProvider::new(&config2, &config, notifier);

        if let Err(err) = provider.run().await {
            error!("Provider run returned error: {err}");
        }
    });
    let _ = hdl.await;
}
