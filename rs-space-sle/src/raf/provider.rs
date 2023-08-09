use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use log::{debug, error, info, warn};
use rand::{rngs::StdRng, Rng, SeedableRng};
use rasn::types::*;

use tokio::{
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener,
    },
    select,
    sync::mpsc::{channel, Receiver, Sender},
};
use tokio_util::sync::CancellationToken;

use crate::{
    asn1::{
        ApplicationIdentifier, AuthorityIdentifier, BindDiagnostic, BindResult, PortId, SlePdu,
        VersionNumber,
    },
    provider::app_interface::ProviderNotifier,
    sle::config::{CommonConfig, SleAuthType},
    tml::message::TMLMessage,
    types::{
        aul::{check_credentials, ISP1Credentials},
        sle::{
            service_instance_identifier_to_string, Credentials, ServiceInstanceIdentifier,
            SleVersion,
        },
    },
};
use rs_space_core::time::{Time, TimeEncoding};

use super::{
    asn1::FrameOrNotification, config::RAFProviderConfig, provider_state::InternalRAFProviderState,
};

const QUEUE_SIZE: usize = 500;

pub enum SleMsg {
    Stop,
    BindReturn(SlePdu, u16, u16),
    PDU(SlePdu),
}

type InternalState = Arc<Mutex<InternalRAFProviderState>>;

type Notifier = Arc<Mutex<dyn ProviderNotifier + Send + Sync>>;

pub struct RAFProvider {
    common_config: CommonConfig,
    raf_config: RAFProviderConfig,
    state: InternalState,
    cancel_token: CancellationToken,
    app_notifier: Notifier,
    chan: Option<Sender<SleMsg>>,
}

impl RAFProvider {
    pub fn new(
        common_config: &CommonConfig,
        raf_config: &RAFProviderConfig,
        notifier: Notifier,
    ) -> RAFProvider {
        RAFProvider {
            common_config: common_config.clone(),
            raf_config: raf_config.clone(),
            state: Arc::new(Mutex::new(InternalRAFProviderState::new(common_config))),
            cancel_token: CancellationToken::new(),
            app_notifier: notifier,
            chan: None,
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

        let (sender, mut receiver) = channel::<SleMsg>(QUEUE_SIZE);
        let sender2 = sender.clone();
        self.chan = Some(sender);

        let (mut rx, tx) = socket.into_split();

        let cancel2 = self.cancel_token.clone();
        let cancel3 = self.cancel_token.clone();

        let config2 = self.common_config.clone();
        let config3 = self.common_config.clone();
        let raf_config2 = self.raf_config.clone();
        let raf_config3 = self.raf_config.clone();

        let server_timeout = Duration::from_secs(self.raf_config.server_init_time as u64);
        let state2 = self.state.clone();

        let notifier = self.app_notifier.clone();

        let mut rand: StdRng = SeedableRng::from_entropy();

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
                        &mut rand,
                        &sender2,
                        notifier,
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

        let write_handle = tokio::spawn(async move {
            match write_task(&config3, &raf_config3, tx, &mut receiver, cancel3.clone()).await {
                Err(err) => {
                    error!("Error in write task: {err}");
                    cancel3.cancel();
                }
                Ok(_) => cancel3.cancel(),
            }
        });

        // we keep running, so we await on the handles of the tasks
        let _ = read_handle.await;
        let _ = write_handle.await;

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
    select! {
        res = TMLMessage::async_read(rx) => {
            match res {
                Err(err) => { return Err(format!("Error reading TML Context Message: {err}")); }
                Ok(msg) => {
                    debug!("Read TML message {msg:?}");

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
    };
}

async fn read_pdus(
    config: &CommonConfig,
    raf_config: &RAFProviderConfig,
    rand: &mut StdRng,
    sender: &Sender<SleMsg>,
    notifier: Notifier,
    cancel_token: CancellationToken,
    state: InternalState,
    rx: &mut OwnedReadHalf,
    interval: u16,
    dead_factor: u16,
) -> Result<(), String> {
    let timeout = Duration::from_secs(interval as u64 * dead_factor as u64);

    loop {
        select! {
            biased;

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
                            parse_sle_message(config, raf_config, rand, interval, dead_factor, sender, notifier.clone(), cancel_token.clone(), state.clone(), &msg).await;
                        }
                    }
                }
            }
            _ = tokio::time::sleep(timeout) => {
                return Err(format!("Timeout waiting for heartbeat message"));
            }
            _ = cancel_token.cancelled() => {
                debug!("RAF provider for {} read loop has been cancelled", raf_config.sii);
                return Ok(());
            }
        };
    }
}

async fn parse_sle_message(
    config: &CommonConfig,
    raf_config: &RAFProviderConfig,
    rand: &mut StdRng,
    interval: u16, 
    dead_factor: u16,
    sender: &Sender<SleMsg>,
    notifier: Notifier,
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
            // then continue processing
            process_sle_pdu(config, raf_config, rand, &pdu, interval, dead_factor, state, sender, notifier).await;
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

async fn process_sle_pdu(
    common_config: &CommonConfig,
    raf_config: &RAFProviderConfig,
    rand: &mut StdRng,
    pdu: &SlePdu,
    interval: u16,
    dead_factor: u16,
    state: InternalState,
    sender: &Sender<SleMsg>,
    notifier: Notifier,
) {
    match pdu {
        SlePdu::SleBindInvocation {
            invoker_credentials: _,
            initiator_identifier,
            responder_port_identifier,
            service_type,
            version_number,
            service_instance_identifier,
        } => {
            // check authentication
            if !check_authentication(common_config, state.clone(), &pdu) {
                error!("SLE PDU failed authentication");

                let credentials = new_credentials(common_config, rand);

                // send back a negative acknowledge
                let ret = SlePdu::SleBindReturn {
                    performer_credentials: credentials,
                    responder_identifier: VisibleString::new(raf_config.provider.clone()),
                    result: BindResult::BindDiag(BindDiagnostic::AccessDenied),
                };

                let _ = sender.send(SleMsg::PDU(ret)).await;
                return;
            }

            // First: perform checks
            if let Err(err) = process_bind(
                common_config,
                raf_config,
                rand,
                sender,
                state.clone(),
                notifier,
                initiator_identifier,
                responder_port_identifier,
                service_type,
                version_number,
                service_instance_identifier,
                interval, 
                dead_factor
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

async fn write_task(
    common_config: &CommonConfig,
    raf_config: &RAFProviderConfig,
    mut tx: OwnedWriteHalf,
    receiver: &mut Receiver<SleMsg>,
    cancel: CancellationToken,
) -> Result<(), String> {
    let mut timeout = common_config.tml.heartbeat;

    loop {
        select! {
                res = receiver.recv() => {
                    match res {
                        Some(SleMsg::Stop) => {
                            debug!("Stop requested");
                            return Ok(());
                        }
                        Some(SleMsg::BindReturn(pdu, hb, _dead_factor)) => {
                            timeout = hb;
                            send_sle_msg(pdu, &mut tx).await?;
                        }
                        Some(SleMsg::PDU(pdu)) => {
                            send_sle_msg(pdu, &mut tx).await?;
                        }
                        None => {
                            debug!("Send channel has been closed, returning...");
                            return Ok(());
                        }
                    }
                },
                _ = tokio::time::sleep(Duration::from_secs(timeout as u64)) => {
                    // we have a timeout, so send a heartbeat message
                    if let Err(err) = TMLMessage::heartbeat_message().write_to_async(&mut tx).await {
                        return Err(format!("Error sending SLE TML hearbeat message: {}", err));
                    }
                },
                _ = cancel.cancelled() => {
                    debug!("RAF provider for {} has been cancelled (write task)", raf_config.sii);
                    // if let Ok(tml) = process_sle_msg(SlePdu::SlePeerAbort{ diagnostic: PeerAbortDiagnostic::OtherReason}, raf_state3.clone()) {
                    //     let _ = tml.write_to_async(&mut tx).await;
                    // }
                    return Ok(());
                }
        }
    }
}

async fn send_sle_msg(pdu: SlePdu, tx: &mut OwnedWriteHalf) -> Result<(), String> {
    match rasn::der::encode(&pdu) {
        Err(err) => {
            return Err(format!("Error encoding PDU to ASN1: {err}"));
        }
        Ok(val) => {
            let tml = TMLMessage::new_with_data(val);
            if let Err(err) = tml.write_to_async(tx).await {
                return Err(format!("Could not send SLE PDU: {err}"));
            }
        }
    }
    Ok(())
}

async fn process_bind(
    common_config: &CommonConfig,
    raf_config: &RAFProviderConfig,
    rand: &mut StdRng,
    sender: &Sender<SleMsg>,
    state: InternalState,
    notifier: Notifier,
    initiator_identifier: &AuthorityIdentifier,
    responder_port_identifier: &PortId,
    service_type: &Integer,
    version_number: &VersionNumber,
    service_instance_identifier: &ServiceInstanceIdentifier,
    interval: u16,
    dead_factor: u16
) -> Result<(), String> {
    // First perform all checks to see if the BIND request is legal
    let (bind_result, version) = 
        if match ApplicationIdentifier::try_from(service_type) {
            Ok(ApplicationIdentifier::RtnAllFrames) => true,
            Ok(_) => false,
            Err(_) => false
        } { (BindResult::BindDiag(BindDiagnostic::ServiceTypeNotSupported), SleVersion::V5) }
        else {
            match SleVersion::try_from(*version_number as u8) {
                Err(_) => (BindResult::BindDiag(BindDiagnostic::VersionNotSupported), SleVersion::V5), 
                Ok(version) => {
                    match service_instance_identifier_to_string(service_instance_identifier) {
                        Err(_) => (BindResult::BindDiag(BindDiagnostic::NoSuchServiceInstance), SleVersion::V5),
                        Ok(sii) => {
                            if sii != raf_config.sii {
                                (BindResult::BindDiag(BindDiagnostic::NoSuchServiceInstance), SleVersion::V5)
                            } else {
                                if responder_port_identifier.value != raf_config.responder_port {
                                    (BindResult::BindDiag(BindDiagnostic::SiNotAccessibleToThisInitiator), SleVersion::V5)
                                } else {
                                    // Ok, BIND is ok, so process the request now
                                    let mut lock = state.lock().unwrap();
                                    lock.process_bind(initiator_identifier, version);
                                    (BindResult::BindOK(version as u16), version)
                                }
                            }
                        }
                    }
                }
            }
        };

    match bind_result {
        BindResult::BindOK(_) => {
            info!("BIND on {} successful", raf_config.sii);
        }
        BindResult::BindDiag(diag) => {
            return Err(format!("BIND on {} failed: {diag:?}", raf_config.sii));
        }
    };
    
    // send a response
    let credentials = new_credentials(common_config, rand);
    let pdu = SlePdu::SleBindReturn { performer_credentials: credentials, 
            responder_identifier: VisibleString::new(raf_config.provider.clone()), 
            result: bind_result };

    let _ = sender.send(SleMsg::BindReturn(pdu, interval, dead_factor)).await;

    {
        let lock = notifier.lock().unwrap();
        lock.bind_succeeded(&initiator_identifier.value, &raf_config.sii, version);
    }

    Ok(())
}

fn new_credentials(config: &CommonConfig, rand: &mut StdRng) -> Credentials {
    match config.auth_type {
        SleAuthType::AuthNone => Credentials::Unused,
        SleAuthType::AuthAll | SleAuthType::AuthBind => {
            let isp1 = ISP1Credentials::new(
                config.hash_to_use,
                &Time::now(TimeEncoding::CDS8),
                rand.gen(),
                &config.authority_identifier,
                &config.password,
            );
            Credentials::Used(isp1)
        }
    }
}
