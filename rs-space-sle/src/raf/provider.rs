use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use log::{debug, error, info, warn};
use rasn::types::*;

use tokio::{
    net::{tcp::OwnedReadHalf, TcpListener},
    select,
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use crate::{
    asn1::{ApplicationIdentifier, AuthorityIdentifier, PortId, SlePdu, VersionNumber},
    sle::config::{CommonConfig, SleAuthType},
    tml::message::TMLMessage,
    types::{
        aul::{check_credentials, ISP1Credentials},
        sle::{Credentials, ServiceInstanceIdentifier, SleVersion, service_instance_identifier_to_string},
    },
};

use super::{
    asn1::FrameOrNotification, config::RAFProviderConfig, provider_state::InternalRAFProviderState,
};

type InternalState = Arc<Mutex<InternalRAFProviderState>>;

pub struct RAFProvider {
    common_config: CommonConfig,
    raf_config: RAFProviderConfig,
    state: InternalState,
    cancel_token: CancellationToken,
    read_task: Option<JoinHandle<()>>,
    write_task: Option<JoinHandle<()>>,
}

impl RAFProvider {
    pub fn new(common_config: &CommonConfig, raf_config: &RAFProviderConfig) -> RAFProvider {
        RAFProvider {
            common_config: common_config.clone(),
            raf_config: raf_config.clone(),
            state: Arc::new(Mutex::new(InternalRAFProviderState::new(common_config))),
            cancel_token: CancellationToken::new(),
            read_task: None,
            write_task: None,
        }
    }

    pub async fn run(&mut self) -> tokio::io::Result<()> {
        let listener =
            TcpListener::bind((self.raf_config.hostname.as_ref(), self.raf_config.port)).await?;

        let (socket, peer) = listener.accept().await?;

        info!(
            "Connection on RAF instance {} from {}",
            self.raf_config.sii, peer
        );

        let (mut rx, tx) = socket.into_split();
        let cancel2 = self.cancel_token.clone();
        let config2 = self.common_config.clone();
        let raf_config2 = self.raf_config.clone();
        let server_timeout = Duration::from_secs(self.raf_config.server_init_time as u64);
        let state2 = self.state.clone();

        let read_handle = tokio::spawn(async move {
            // First, we expect a TML context message. If not, we bail out
            match read_context_message(&config2, &mut rx, server_timeout).await {
                Err(err) => {
                    error!("Error reading SLE TML Context Message: {err}");
                    cancel2.cancel();
                }
                Ok((interval, dead_factor)) => {
                    match read_pdus(
                        &config2,
                        &raf_config2,
                        cancel2.clone(),
                        state2,
                        &mut rx,
                        interval,
                        dead_factor,
                    )
                    .await
                    {
                        Err(err) => {
                            error!("{err}");
                            cancel2.cancel();
                        }
                        Ok(_) => {
                            cancel2.cancel();
                        }
                    }
                }
            }
        });

        let write_handle = tokio::spawn(async move {});

        self.read_task = Some(read_handle);
        self.write_task = Some(write_handle);

        Ok(())
    }

    pub async fn send_frame(&mut self) -> tokio::io::Result<()> {
        Ok(())
    }
}

async fn read_context_message(
    config: &CommonConfig,
    rx: &mut OwnedReadHalf,
    server_startup_interval: Duration,
) -> Result<(u16, u16), String> {
    select!(
        res = TMLMessage::async_read(rx) => {
            match res {
                Err(err) => { return Err(format!("Error reading TML Context Message: {err}")); }
                Ok(msg) => {
                    let (interval, dead_factor) = msg.check_context()?;

                    let tml = &config.tml;

                    if interval < tml.min_heartbeat || interval > tml.max_heartbeat {
                        return Err(format!("Error: TML Context message interval ({interval}) is out of allowed range ([{}, {}])", tml.min_heartbeat, tml.max_heartbeat));
                    }

                    if dead_factor < tml.min_dead_factor || dead_factor > tml.max_dead_factor {
                        return Err(format!("Error: TML Context message dead factor ({dead_factor}) is out of allowed range ([{}, {}])", tml.min_dead_factor, tml.max_dead_factor));
                    }

                    return Ok((interval, dead_factor));
                }
            }
        }
        _ = tokio::time::sleep(server_startup_interval) => {
            return Err("Timeout waiting for TML Context Message".to_string());
        }
    );
}

async fn read_pdus(
    config: &CommonConfig,
    raf_config: &RAFProviderConfig,
    cancel_token: CancellationToken,
    state: InternalState,
    rx: &mut OwnedReadHalf,
    interval: u16,
    dead_factor: u16,
) -> Result<(), String> {
    let timeout = Duration::from_secs(interval as u64 * dead_factor as u64);

    loop {
        select!(
            res = TMLMessage::async_read(rx) => {
                match res {
                    Err(err) => {
                        return Err(format!("Error reading SLE TML Message: {err}"));
                    }
                    Ok(msg) => {
                        if msg.is_heartbeat() {
                            debug!("SLE TML heartbeat received");
                        }
                        else
                        {
                            parse_sle_message(config, raf_config, cancel_token.clone(), state.clone(), &msg).await;
                        }
                    }
                }
            }
            _ = tokio::time::sleep(timeout) => {
                return Err(format!("Timeout waiting for heartbeat message"));
            }
        );
    }
}

async fn parse_sle_message(
    config: &CommonConfig,
    raf_config: &RAFProviderConfig,
    cancel_token: CancellationToken,
    state: InternalState,
    msg: &TMLMessage,
) {
    let res: Result<SlePdu, _> = rasn::der::decode(&msg.data[..]);

    match res {
        Ok(SlePdu::SlePeerAbort { diagnostic: diag }) => {
            warn!("Received Peer Abort with diagnostic: {:?}", diag);
            cancel_token.cancel();
        }
        Ok(pdu) => {
            // check authentication
            if !check_authentication(config, state.clone(), &pdu) {
                error!("SLE PDU failed authentication");
                cancel_token.cancel();
            }

            // then continue processing
            process_sle_pdu(raf_config, &pdu, state).await;
        }
        Err(err) => {
            error!("Error on decoding SLE PDU: {err}");
        }
    }
}

fn check_authentication(config: &CommonConfig, state: InternalState, pdu: &SlePdu) -> bool {
    match config.auth_type {
        SleAuthType::AuthNone =>
        // in case we have not authentication configured, all is good
        {
            true
        }
        // for AUTH_BIND, we need to only check the BIND and BIND RETURN invocations
        SleAuthType::AuthBind => check_bind(config, pdu),
        SleAuthType::AuthAll => check_all(config, state, pdu),
    }
}

fn check_bind(config: &CommonConfig, pdu: &SlePdu) -> bool {
    match pdu {
        SlePdu::SleBindInvocation {
            invoker_credentials,
            initiator_identifier,
            ..
        } => match invoker_credentials {
            Credentials::Unused => {
                error!("BIND Authentication failed: AUTH_BIND requested, but BIND invocation does not contain credentials");
                return false;
            }
            Credentials::Used(creds) => check_peer(config, creds, initiator_identifier, "BIND"),
        },
        SlePdu::SleBindReturn {
            performer_credentials,
            responder_identifier,
            ..
        } => match performer_credentials {
            Credentials::Unused => {
                error!("BIND Authentication failed: AUTH_BIND requested, but BIND RETURN does not contain credentials");
                return false;
            }
            Credentials::Used(isp1) => {
                check_peer(config, isp1, responder_identifier, "BIND RETURN")
            }
        },
        _ => {
            // All other SLE PDUs are fine without authentication
            return true;
        }
    }
}

fn check_all(config: &CommonConfig, state: InternalState, pdu: &SlePdu) -> bool {
    let credentials = pdu.get_credentials();

    match pdu {
        SlePdu::SleRafTransferBuffer(buffer) => {
            for frame in buffer {
                let res = check_buffer_credential(config, state.clone(), frame);
                if !res {
                    return false;
                }
            }
            return true;
        }
        SlePdu::SleBindInvocation {
            invoker_credentials,
            initiator_identifier,
            ..
        } => match invoker_credentials {
            Credentials::Unused => {
                error!("BIND Authentication failed: AUTH_BIND requested, but BIND invocation does not contain credentials");
                return false;
            }
            Credentials::Used(creds) => check_peer(config, creds, initiator_identifier, "BIND"),
        },
        SlePdu::SleBindReturn {
            performer_credentials,
            responder_identifier,
            ..
        } => match performer_credentials {
            Credentials::Unused => {
                error!("BIND Authentication failed: AUTH_BIND requested, but BIND RETURN does not contain credentials");
                return false;
            }
            Credentials::Used(isp1) => {
                check_peer(config, isp1, responder_identifier, "BIND RETURN")
            }
        },
        _ => match credentials {
            Some(creds) => match creds {
                Credentials::Unused => {
                    error!("Authentication failed, no credentials provided");
                    return false;
                }
                Credentials::Used(isp1) => {
                    let name;
                    {
                        let lock = state.lock().expect("Mutex lock failed");
                        name = lock.user().clone();
                    }
                    let op_name = pdu.operation_name();
                    return check_peer(config, isp1, &name, op_name);
                }
            },
            None => {
                return true;
            }
        },
    }
}

fn check_buffer_credential(
    config: &CommonConfig,
    state: InternalState,
    buf_part: &FrameOrNotification,
) -> bool {
    let creds = match buf_part {
        FrameOrNotification::AnnotatedFrame(trans) => &trans.invoker_credentials,
        FrameOrNotification::SyncNotification(notif) => &notif.invoker_credentials,
    };

    match creds {
        Credentials::Unused => {
            error!("Authentication of TM frame failed, no credentials provided");
            return false;
        }
        Credentials::Used(isp1) => {
            let name;
            {
                let lock = state.lock().expect("Mutex lock failed");
                name = lock.user().clone();
            }
            let op_name = "RAF TRANSFER BUFFER";
            return check_peer(config, &isp1, &name, op_name);
        }
    }
}

fn check_peer(
    config: &CommonConfig,
    isp1: &ISP1Credentials,
    identifier: &VisibleString,
    op_name: &str,
) -> bool {
    match config.get_peer(identifier) {
        Some(peer) => {
            if !check_credentials(&isp1, identifier, &peer.password) {
                error!("{}: authentication failed", op_name);
                return false;
            }
            return true;
        }
        None => {
            error!(
                "Peer '{}' is not configured, authentication failed!",
                identifier.value.as_str()
            );
            return false;
        }
    }
}

async fn process_sle_pdu(raf_config: &RAFProviderConfig, pdu: &SlePdu, state: InternalState) {
    match pdu {
        SlePdu::SleBindInvocation {
            invoker_credentials: _,
            initiator_identifier,
            responder_port_identifier,
            service_type,
            version_number,
            service_instance_identifier,
        } => {
            // First: perform checks
            if let Err(err) = process_bind(
                raf_config,
                initiator_identifier,
                responder_port_identifier,
                service_type,
                version_number,
                service_instance_identifier,
            )
            .await
            {
                error!("Error processing BIND: {err}");
            }
        }
        pdu => {
            info!("Not yet implemented: processing for PDU: {:?}", pdu);
        }
    }
}

async fn process_bind(
    raf_config: &RAFProviderConfig,
    initiator_identifier: &AuthorityIdentifier,
    responder_port_identifier: &PortId,
    service_type: &Integer,
    version_number: &VersionNumber,
    service_instance_identifier: &ServiceInstanceIdentifier,
) -> Result<(), String> {

    // First perform all checks to see if the BIND request is legal
    let appl = match ApplicationIdentifier::try_from(service_type) {
        Ok(ApplicationIdentifier::RtnAllFrames) => ApplicationIdentifier::RtnAllFrames,
        Ok(val) => {
            return Err(format!("BIND with illegal application identifier for RAF: {val:?}"));
        }
        Err(err) => {
            return Err(err);
        }
    };

    let version = SleVersion::try_from(*version_number as u8)?;

    let sii = service_instance_identifier_to_string(service_instance_identifier)?;
    if sii != raf_config.sii {
        return Err(format!("BIND with illegal service instance ID: {}, allowed: {}", sii, raf_config.sii));
    }

    if responder_port_identifier.value != raf_config.responder_port {
        return Err(format!(
            "BIND requested illegal port {}, allowed is {}",
            responder_port_identifier.value, raf_config.responder_port
        ));
    }
    
    // Ok, BIND is ok, so process the request now

    Ok(())
}
