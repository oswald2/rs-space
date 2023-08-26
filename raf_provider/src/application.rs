#[allow(unused)]
use std::collections::BTreeSet;

use log::{error, info};
use rs_space_core::time::TimeEncoding;
use rs_space_sle::asn1::UnbindReason;
use rs_space_sle::raf::asn1::{FrameQuality, SleFrame};
use rs_space_sle::raf::provider::RAFProvider;
use rs_space_sle::sle::config::CommonConfig;
use rs_space_sle::{
    provider::{config::ProviderConfig, raf_interface::ProviderNotifier},
    raf::config::RAFProviderConfig,
    types::sle::{PeerAbortDiagnostic, SleVersion},
};

use bytes::Bytes;

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

    fn start_succeeded(&self, sii: &str) {
        info!("RAF START SUCCEEDED for {sii}");
    }

    fn stop_succeeded(&self, sii: &str) {
        info!("RAF STOP SUCCEEDED for {sii}");
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

    info!(
        "Starting SLE instance {} on TCP port {} (SLE Port {})",
        config.sii, config.port, config.responder_port
    );

    info!("Creating new provider...");
    let mut provider = RAFProvider::new(&config2, &config);
    //let provider2 = provider.clone();

    let notifier = Box::new(Notifier::new());


    info!("Running provider...");
    if let Err(err) = provider.run(notifier).await {
        error!("Provider run returned error: {err}");
    }

    info!("Waiting until provider is ACTIVE...");
    let _ = provider.wait_active().await;
    info!("Provider is ACTIVE!");

    let frame = SleFrame {
        earth_receive_time: rs_space_core::time::Time::now(TimeEncoding::CDS8),
        delivered_frame_quality: FrameQuality::Good,
        data: Bytes::copy_from_slice(&[0x01, 0x02, 0x03, 0x04]),
    };

    if let Err(err) = provider.send_frame(frame).await {
        error!("Error sending frame: {}", err);
        return;
    }

    provider.wait_for_termination().await;
}
