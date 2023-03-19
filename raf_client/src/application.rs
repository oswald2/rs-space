use std::collections::BTreeSet;

use rasn::types::{Implicit, Integer, VisibleString};
use rs_space_sle::asn1_2::{
    new_service_instannce_attribute, ApplicationIdentifier, Credentials, ServiceInstanceAttribute,
    ServiceInstanceAttributeInner, SleBindInvocation,
};
use rs_space_sle::pdu::PDU;
use tokio::io::{BufStream, Error, ErrorKind};

use log::{error, info};

use rs_space_sle::sle_client::sle_connect;
use rs_space_sle::tml_config::TMLConfig;

const DEFAULT_CONFIG: TMLConfig = TMLConfig {
    heartbeat: 30,
    dead_factor: 2,
    server_init_time: 30,
    min_heartbeat: 3,
    max_heartbeat: 3600,
    min_dead_factor: 2,
    max_dead_factor: 60,
};

pub async fn run_app(address: String) -> Result<(), Error> {
    info!("Connecting to {}...", address);

    let mut handle = sle_connect(&address, &DEFAULT_CONFIG).await?;

    let bind = SleBindInvocation {
        invoker_credentials: Credentials::Unused,
        initiator_identifier: VisibleString::new(rasn::types::Utf8String::from("SLETT")),
        responder_port_identifier: VisibleString::new(rasn::types::Utf8String::from("TMPORT")),
        service_type: Integer::from(ApplicationIdentifier::RtnAllFrames as u8),
        version_number: 4,
        service_instance_identifier: vec![
            new_service_instannce_attribute(&rs_space_sle::asn1_2::SAGR, "3"),
            new_service_instannce_attribute(&rs_space_sle::asn1_2::SPACK, "facility-PASS1"),
            new_service_instannce_attribute(&rs_space_sle::asn1_2::RSL_FG, "1"),
            new_service_instannce_attribute(&rs_space_sle::asn1_2::RAF, "onlc1"),
        ],
    };
    // sagr=3.spack=facility-PASS1.rsl-fg=1.raf=onlc1"

    info!("Sending BIND invocation: {:?}", bind);

    if let Err(err) = handle.send_pdu(PDU::SlePduBind(bind)).await {
        error!("Error sending PDU: {}", err);
    }

    std::thread::sleep(std::time::Duration::from_secs(5));
    Ok(())
}
