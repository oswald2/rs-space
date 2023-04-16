#[allow(unused)]
use std::collections::BTreeSet;

use rasn::types::{Integer, VisibleString};
use rs_space_sle::asn1_raf::{
    new_service_instance_attribute, ApplicationIdentifier, Credentials, SlePdu,
};
use rs_space_sle::sle_client::sle_connect_raf;
use rs_space_sle::user_config::UserConfig;
use tokio::io::{Error};

use log::{error, info};
use tokio::task::JoinHandle;

pub async fn run_app(config: &UserConfig) -> Result<(), Error> {
    for raf_config in &config.rafs {
        let config = (*config).clone();
        let raf_config = (*raf_config).clone();

        let _hdl: JoinHandle<Result<(), Error>> = tokio::spawn(async move {
            let address = format!("{}:{}", raf_config.hostname, raf_config.port);
            info!("Connecting to {}...", address);

            let mut handle = sle_connect_raf(&config.tml_config, &raf_config).await?;

            let bind = SlePdu::SleBindInvocation {
                invoker_credentials: Credentials::Unused,
                initiator_identifier: VisibleString::new(rasn::types::Utf8String::from(
                    raf_config.initiator,
                )),
                responder_port_identifier: VisibleString::new(rasn::types::Utf8String::from(
                    raf_config.responder_port,
                )),
                service_type: Integer::from(ApplicationIdentifier::RtnAllFrames as u8),
                version_number: raf_config.version as u16,
                service_instance_identifier: vec![
                    new_service_instance_attribute(&rs_space_sle::asn1_raf::SAGR, "3"),
                    new_service_instance_attribute(
                        &rs_space_sle::asn1_raf::SPACK,
                        "facility-PASS1",
                    ),
                    new_service_instance_attribute(&rs_space_sle::asn1_raf::RSL_FG, "1"),
                    new_service_instance_attribute(&rs_space_sle::asn1_raf::RAF, "onlc1"),
                ],
            };
            // sagr=3.spack=facility-PASS1.rsl-fg=1.raf=onlc1"

            info!("Sending BIND invocation: {:?}", bind);

            if let Err(err) = handle.send_pdu(bind).await {
                error!("Error sending PDU: {}", err);
            }

            std::thread::sleep(std::time::Duration::from_secs(5));
            Ok(())
        });
    }
    Ok(())
}
