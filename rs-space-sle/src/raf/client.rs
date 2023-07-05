use std::sync::{Arc, Mutex};
#[allow(unused)]
use std::time::Duration;

use rand::rngs::ThreadRng;
use rand::{thread_rng, RngCore};
use rasn::types::{Utf8String, VisibleString};
use rs_space_core::time::{Time, TimeEncoding};
use tokio::io::Error;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc::{channel, Sender};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::asn1::{ApplicationIdentifier, SlePdu, UnbindReason};
use crate::raf::config::RAFConfig;
use crate::raf::state::{InternalRAFState, RAFState};
use crate::sle::config::{CommonConfig, SleAuthType};
// use crate::pdu::PDU;
use crate::tml::config::TMLConfig;
use crate::tml::message::TMLMessage;
use crate::types::aul::{check_credentials, ISP1Credentials};
use crate::types::sle::{string_to_service_instance_id, Credentials};
use log::{debug, error};

use function_name::named;

const QUEUE_SIZE: usize = 500;

pub enum SleMsg {
    Stop,
    PDU(SlePdu),
}

type InternalTask = Option<JoinHandle<()>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum HandleType {
    Bind,
    Unbind,
}

type HandleVec = Arc<Mutex<Vec<(HandleType, JoinHandle<()>)>>>;

pub struct RAFClient {
    chan: Sender<SleMsg>,
    cancellation_token: CancellationToken,
    state: Arc<Mutex<InternalRAFState>>,
    // we use an Option here, so that we can move out the JoinHandle from the struct for
    // awaiting on it
    read_task: InternalTask,
    write_task: InternalTask,
    op_timeout: Duration,
    rand: ThreadRng,
    handles: HandleVec,
}

type InternalState = Arc<Mutex<InternalRAFState>>;

impl RAFClient {
    /// Connect to the SLE RAF instance given in the RAFConfig.
    pub async fn sle_connect_raf(
        config: &CommonConfig,
        raf_config: &RAFConfig,
    ) -> Result<RAFClient, Error> {
        let sock = TcpStream::connect((raf_config.hostname.as_ref(), raf_config.port)).await?;

        let (mut rx, mut tx) = sock.into_split();

        let (sender, mut receiver) = channel::<SleMsg>(QUEUE_SIZE);
        let sender2 = sender.clone();

        let cancellation = CancellationToken::new();
        let cancel1 = cancellation.clone();
        let cancel2 = cancellation.clone();

        let cfg: &TMLConfig = &config.tml;
        let timeout = cfg.heartbeat;
        let recv_timeout = cfg.heartbeat * cfg.dead_factor;
        let sii = raf_config.sii.clone();
        let sii2 = raf_config.sii.clone();

        let raf_state = Arc::new(Mutex::new(InternalRAFState::new()));
        let raf_state2 = raf_state.clone();
        let raf_state3 = raf_state.clone();

        let common_config = config.clone();
        let raf_config2 = raf_config.clone();

        let handle_vec = Arc::new(Mutex::new(Vec::new()));
        let handle_vec2 = handle_vec.clone();

        let hdl1 = tokio::spawn(async move {
            loop {
                select!(
                    res = TMLMessage::async_read(&mut rx) => {
                        match res {
                            Err(err) => {
                                error!("Error reading SLE TML message from socket: {}", err);
                                break;
                            }
                            Ok(msg) => {
                                if msg.is_heartbeat() {
                                    debug!("SLE TML heartbeat received");
                                }
                                else
                                {
                                    parse_sle_message(&common_config, &raf_config2, &msg, raf_state.clone(), handle_vec2.clone());
                                }
                            }
                        }
                    },
                    _ = tokio::time::sleep(Duration::from_secs(recv_timeout as u64)) => {
                        // we have a receive heartbeat timeout, so report the error and disconnect
                        error!("Heartbeat timeout on service instance {}, terminating connection", sii);
                        let _ = sender2.send(SleMsg::Stop).await;
                        return;
                    }
                    _ = cancel1.cancelled() => {
                        debug!("RAF client for {} has been cancelled (read task)", sii);
                        return;
                    }
                );
            }
        });

        // we initiated the connection, so send a context message
        let ctxt = TMLMessage::context_message(cfg.heartbeat, cfg.dead_factor);
        ctxt.write_to_async(&mut tx).await?;

        let hdl2 = tokio::spawn(async move {
            loop {
                select! {
                    res = receiver.recv() => {
                                match res {
                                    Some(SleMsg::Stop) => {
                                        debug!("Stop requested");
                                        return;
                                    }
                                    Some(SleMsg::PDU(pdu)) => {
                                        debug!("Received command: {:?}", pdu);
                                        match process_sle_msg(pdu, raf_state2.clone()) {
                                            Err(err) => {
                                                error!("Error encoding SLE message: {}", err);
                                                break;
                                            }
                                            Ok(tml_message) => {
                                                // we got a valid TML message encoded, so send it
                                                if let Err(err) = tml_message.write_to_async(&mut tx).await {
                                                    error!("Error sending SLE message to socket: {}", err);
                                                    break;
                                                }
                                            }
                                        }
                                    },
                                    None => {
                                        debug!("Channel has been closed, returning...");
                                        break;
                                    }
                                }
                        },

                        _ = tokio::time::sleep(Duration::from_secs(timeout as u64)) => {
                            // we have a timeout, so send a heartbeat message
                            if let Err(err) = TMLMessage::heartbeat_message().write_to_async(&mut tx).await {
                                error!("Error sending SLE TML hearbeat message: {}", err);
                                break;
                            }
                        },
                        _ = cancel2.cancelled() => {
                            debug!("RAF client for {} has been cancelled (read task)", sii2);
                            return;
                        }

                }
            }
        });

        let ret = RAFClient {
            chan: sender,
            cancellation_token: cancellation,
            state: raf_state3,
            read_task: Some(hdl1),
            write_task: Some(hdl2),
            op_timeout: Duration::from_secs(raf_config.sle_operation_timeout as u64),
            rand: thread_rng(),
            handles: handle_vec,
        };

        Ok(ret)
    }

    /// Send a SleMsg as a command to control the machinery
    #[named]
    pub async fn command(&mut self, msg: SleMsg) -> Result<(), String> {
        debug!(function_name!());
        if let Err(err) = self.chan.send(msg).await {
            return Err(format!("Could not process msg: {}", err));
        }
        Ok(())
    }

    /// Send a PDU to the connected instance
    #[named]
    pub async fn send_pdu(&mut self, pdu: SlePdu) -> Result<(), String> {
        debug!(function_name!());
        self.command(SleMsg::PDU(pdu)).await
    }

    fn new_credentials(&mut self, common_config: &CommonConfig) -> Credentials {
        let credentials = if common_config.auth_type == SleAuthType::AuthNone {
            Credentials::Unused
        } else {
            let isp1 = ISP1Credentials::new(
                common_config.hash_to_use,
                &Time::now(TimeEncoding::CDS8),
                self.rand.next_u32() as i32,
                &common_config.authority_identifier,
                &common_config.password,
            );
            Credentials::Used(isp1)
        };
        credentials
    }

    /// Bind the service given in the config to the end point, establish a connection and execute
    /// the SLE BIND operation
    #[named]
    pub async fn bind(
        &mut self,
        common_config: &CommonConfig,
        config: &RAFConfig,
    ) -> Result<(), String> {
        debug!(function_name!());

        // first check if we are in a correct state
        let state;
        {
            let st = self
                .state
                .lock()
                .expect("Error locking RAF internal state mutex");
            state = st.get_state();
        }
        if state == RAFState::Bound || state == RAFState::Active {
            return Err("BIND error: not in UNBOUND state".to_string());
        }

        // first, convert the SII string from the config into a ASN1 structure
        let sii = string_to_service_instance_id(&config.sii)?;

        // generate the credentials
        let credentials = self.new_credentials(common_config);

        // Create the BIND SLE PDU
        let pdu = SlePdu::SleBindInvocation {
            invoker_credentials: credentials,
            initiator_identifier: common_config.authority_identifier.clone(),
            responder_port_identifier: VisibleString::new(Utf8String::from(&config.responder_port)),
            service_type: (ApplicationIdentifier::RtnAllFrames as i32).into(),
            version_number: config.version as u16,
            service_instance_identifier: sii,
        };

        // And finally, send the PDU
        self.send_pdu(pdu).await?;

        let timeout = Duration::from_secs(config.sle_operation_timeout as u64);
        let cancel = self.cancellation_token.clone();

        self.handles.lock().unwrap().push((
            HandleType::Bind,
            tokio::spawn(async move {
                tokio::time::sleep(timeout).await;
                error!("Timeout waiting for BIND RESPONSE, terminating connection");
                cancel.cancel();
            }),
        ));

        Ok(())
    }

    #[named]
    pub async fn unbind(
        &mut self,
        common_config: &CommonConfig,
        reason: UnbindReason,
    ) -> Result<(), String> {
        debug!(function_name!());

        // first check if we are in a correct state
        let state;
        {
            let st = self
                .state
                .lock()
                .expect("Error locking RAF internal state mutex");
            state = st.get_state();
        }
        if state == RAFState::Unbound {
            // Unbind is always accepted, though in UNBOUND state, we don't need
            // to send
            return Ok(());
        }

        // generate the credentials
        let credentials = self.new_credentials(common_config);

        let pdu = SlePdu::SleUnbindInvocation {
            invoker_credentials: credentials,
            unbind_reason: (reason as i32).into(),
        };

        self.send_pdu(pdu).await?;

        let dur = self.op_timeout;
        let cancel = self.cancellation_token.clone();

        self.handles.lock().unwrap().push((
            HandleType::Unbind,
            tokio::spawn(async move {
                tokio::time::sleep(dur).await;
                error!("Timeout waiting for UNBIND RESPONSE, terminating connection");
                cancel.cancel();
            }),
        ));

        Ok(())
    }

    pub async fn stop(&mut self) {
        let _ = self.chan.send(SleMsg::Stop).await;
        if let Some(handle) = self.write_task.take() {
            let _ = handle.await;
        }
        if let Some(handle) = self.read_task.take() {
            drop(handle);
        }
    }

    /// Stop the machinery
    pub async fn cancel(&self) {
        self.cancellation_token.cancel();
    }

}


fn cancel_timer(op_type: HandleType, handle_vec: HandleVec) {
    for (t, handle) in handle_vec.lock().unwrap().iter() {
        if *t == op_type {
            handle.abort();
            return;
        }
    }
}



fn process_sle_msg(pdu: SlePdu, _state: InternalState) -> Result<TMLMessage, String> {
    match rasn::der::encode(&pdu) {
        Err(err) => Err(format!("Error encoding PDU to ASN1: {}", err)),
        Ok(val) => Ok(TMLMessage::new_with_data(val)),
    }
}

fn parse_sle_message(
    config: &CommonConfig,
    raf_cfg: &RAFConfig,
    msg: &TMLMessage,
    state: InternalState,
    handle_vec: HandleVec,
) {
    let res: Result<SlePdu, _> = rasn::der::decode(&msg.data[..]);

    debug!("Decoded SLE PDU: {:?}", res);

    match res {
        Ok(pdu) => {
            // check authentication
            if !check_authentication(config, raf_cfg, &pdu) {
                error!("SLE PDU failed authentication");
                // TODO: terminate the connection
            }

            // then continue processing
            process_sle_pdu(&pdu, state, handle_vec)
        }
        Err(err) => {
            error!("Error on decoding SLE PDU: {err}");
        }
    }
}

fn process_sle_pdu(pdu: &SlePdu, state: InternalState, handle_vec: HandleVec) {
    match pdu {
        SlePdu::SleBindReturn {
            performer_credentials: _,
            responder_identifier,
            result,
        } => {
            // We have a valid return, so cancel the timer waiting for the return 
            cancel_timer(HandleType::Bind, handle_vec);

            // Lock our state and process the BIND RETURN
            let mut lock = state.lock().expect("Mutex lock failed");
            lock.process_bind_return(&responder_identifier, result);
        }
        SlePdu::SleUnbindReturn {
            responder_credentials: _,
            result: _,
        } => {
            // We have a valid return, so cancel the timer waiting for the return 
            cancel_timer(HandleType::Unbind, handle_vec);

            // Lock our state and process the UNBIND RETURN
            let mut lock = state.lock().expect("Mutex lock failed");
            lock.process_unbind();
        }
        pdu => {
            debug!("Received: {:?}", pdu);
        }
    }
}

fn check_authentication(config: &CommonConfig, _raf_cfg: &RAFConfig, pdu: &SlePdu) -> bool {
    match config.auth_type {
        SleAuthType::AuthNone =>
        // in case we have not authentication configured, all is good
        {
            true
        }
        SleAuthType::AuthBind =>
        // for AUTH_BIND, we need to only check the BIND and BIND RETURN invocations
        {
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
                    Credentials::Used(creds) => check_peer(config, creds, initiator_identifier),
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
                    Credentials::Used(isp1) => check_peer(config, isp1, responder_identifier),
                },
                _ => {
                    // All other SLE PDUs are fine without authentication
                    return true;
                }
            }
        }
        SleAuthType::AuthAll => check_all(config, pdu),
    }
}

fn check_all(_config: &CommonConfig, _pdu: &SlePdu) -> bool {
    return true;
}

fn check_peer(config: &CommonConfig, isp1: &ISP1Credentials, identifier: &VisibleString) -> bool {
    match config.get_peer(identifier) {
        Some(peer) => {
            return check_credentials(&isp1, identifier, &peer.password);
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
