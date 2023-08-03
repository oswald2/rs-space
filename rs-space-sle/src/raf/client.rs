use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, Mutex};
#[allow(unused)]
use std::time::Duration;

use rand::rngs::ThreadRng;
use rand::{thread_rng, RngCore};
use rasn::types::{Utf8String, VisibleString};
use rs_space_core::time::{Time, TimeEncoding};
use tokio::io::Error;
use tokio::net::{tcp::OwnedReadHalf, TcpStream};
use tokio::select;
use tokio::sync::mpsc::{channel, Sender};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::asn1::*;
use crate::raf::asn1::{
    convert_frame, FrameOrNotification, RafTransferBuffer, RequestedFrameQuality,
};
use crate::raf::config::RAFConfig;
use crate::raf::state::{FrameCallback, InternalRAFState, RAFState};
use crate::sle::config::{CommonConfig, SleAuthType};
// use crate::pdu::PDU;
use crate::tml::config::TMLConfig;
use crate::tml::message::TMLMessage;
use crate::types::aul::{check_credentials, ISP1Credentials};
use crate::types::sle::{
    string_to_service_instance_id, to_conditional_ccsds_time, Credentials, PeerAbortDiagnostic,
};
use log::{debug, error, warn};

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
    Start,
    Stop,
}

type HandleVec = Arc<Mutex<Vec<(HandleType, JoinHandle<()>)>>>;

pub struct RAFClient {
    rx: Option<OwnedReadHalf>,
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
    invoke_id: AtomicU16,
}

type InternalState = Arc<Mutex<InternalRAFState>>;

impl RAFClient {
    /// Connect to the SLE RAF instance given in the RAFConfig.
    pub async fn sle_connect_raf(
        config: &CommonConfig,
        raf_config: &RAFConfig,
        frame_callback: FrameCallback,
    ) -> Result<RAFClient, Error> {
        let sock = TcpStream::connect((raf_config.hostname.as_ref(), raf_config.port)).await?;

        let (rx, mut tx) = sock.into_split();

        let (sender, mut receiver) = channel::<SleMsg>(QUEUE_SIZE);
        //let sender2 = sender.clone();

        let cancellation = CancellationToken::new();
        //let cancel1 = cancellation.clone();
        let cancel2 = cancellation.clone();

        let cfg: &TMLConfig = &config.tml;
        let timeout = cfg.heartbeat;
        let timeout2 = timeout.clone();
        let dead_factor2 = cfg.dead_factor.clone();
        //let recv_timeout = cfg.heartbeat * cfg.dead_factor;
        //let sii = raf_config.sii.clone();
        let sii2 = raf_config.sii.clone();

        let raf_state = Arc::new(Mutex::new(InternalRAFState::new(frame_callback)));
        let raf_state2 = raf_state.clone();
        let raf_state3 = raf_state.clone();

        //let common_config = config.clone();
        //let raf_config2 = raf_config.clone();

        let handle_vec = Arc::new(Mutex::new(Vec::new()));
        //let handle_vec2 = handle_vec.clone();

        // let hdl1 = tokio::spawn(async move {
        //     loop {
        //         select!(
        //             res = TMLMessage::async_read(&mut rx) => {
        //                 match res {
        //                     Err(err) => {
        //                         error!("Error reading SLE TML message from socket: {}", err);
        //                         break;
        //                     }
        //                     Ok(msg) => {
        //                         if msg.is_heartbeat() {
        //                             debug!("SLE TML heartbeat received");
        //                         }
        //                         else
        //                         {
        //                             parse_sle_message(&common_config, &raf_config2, &msg, raf_state.clone(), handle_vec2.clone(), cancel1.clone());
        //                         }
        //                     }
        //                 }
        //             },
        //             _ = tokio::time::sleep(Duration::from_secs(recv_timeout as u64)) => {
        //                 // we have a receive heartbeat timeout, so report the error and disconnect
        //                 error!("Heartbeat timeout on service instance {}, terminating connection", sii);
        //                 let _ = sender2.send(SleMsg::Stop).await;
        //                 return;
        //             }
        //             _ = cancel1.cancelled() => {
        //                 debug!("RAF client for {} has been cancelled (read task)", sii);
        //                 return;
        //             }
        //         );
        //     }
        // });

        let hdl2 = tokio::spawn(async move {
            // we initiated the connection, so send a context message
            let ctxt = TMLMessage::context_message(timeout2, dead_factor2);
            if let Err(err) = ctxt.write_to_async(&mut tx).await {
                error!("Error sending SLE TML context message to provider: {err}");
                return;
            }

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
                            if let Ok(tml) = process_sle_msg(SlePdu::SlePeerAbort{ diagnostic: PeerAbortDiagnostic::OtherReason}, raf_state2.clone()) {
                                let _ = tml.write_to_async(&mut tx).await;
                            }
                            return;
                        }

                }
            }
        });

        let ret = RAFClient {
            rx: Some(rx),
            chan: sender,
            cancellation_token: cancellation,
            state: raf_state3,
            read_task: None,
            write_task: Some(hdl2),
            op_timeout: Duration::from_secs(raf_config.sle_operation_timeout as u64),
            rand: thread_rng(),
            handles: handle_vec,
            invoke_id: AtomicU16::new(0),
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

    pub async fn bind(
        &mut self,
        common_config: &CommonConfig,
        config: &RAFConfig,
    ) -> Result<(), String> {
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

        let rx = self.rx.take().unwrap();
        let new_rx = self
            .check_return(rx, common_config, config, check_bind_return)
            .await?;

        self.rx = Some(new_rx);

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

        self.cancellation_token.cancel();
        Ok(())
    }

    pub async fn start(
        &mut self,
        common_config: &CommonConfig,
        config: &RAFConfig,
        start: Option<Time>,
        stop: Option<Time>,
        frame_quality: RequestedFrameQuality,
    ) -> Result<(), String> {
        // first check if we are in a correct state
        let state;
        {
            let st = self
                .state
                .lock()
                .expect("Error locking RAF internal state mutex");
            state = st.get_state();
        }
        if state == RAFState::Unbound || state == RAFState::Active {
            return Err("RAF START error: not in BOUND state".to_string());
        };

        // generate the credentials
        let credentials = self.new_credentials(common_config);

        let start_time = to_conditional_ccsds_time(start)?;
        let stop_time = to_conditional_ccsds_time(stop)?;

        // Create the RAF START SLE PDU
        let pdu = SlePdu::SleRafStartInvocation {
            invoker_credentials: credentials,
            invoke_id: self.invoke_id.fetch_add(1, Ordering::AcqRel),
            start_time: start_time,
            stop_time: stop_time,
            requested_frame_quality: (frame_quality as u32).into(),
        };

        // And finally, send the PDU
        self.send_pdu(pdu).await?;
        let rx = self.rx.take().unwrap();
        let mut rx2 = self
            .check_return(rx, common_config, config, check_start_return)
            .await?;

        let common_config2 = common_config.clone();
        let raf_config2 = config.clone();
        let raf_state = self.state.clone();
        let handle_vec2 = self.handles.clone();
        let cancel1 = self.cancellation_token.clone();
        let recv_timeout = common_config.tml.heartbeat * common_config.tml.dead_factor;
        let sii = config.sii.clone();
        let sender2 = self.chan.clone();

        let hdl = tokio::spawn(async move {
            loop {
                select!(
                    res = TMLMessage::async_read(&mut rx2) => {
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
                                    parse_sle_message(&common_config2, &raf_config2, &msg, raf_state.clone(), handle_vec2.clone(), cancel1.clone());
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

        self.read_task = Some(hdl);

        Ok(())
    }

    pub async fn stop(
        &mut self,
        common_config: &CommonConfig,
        config: &RAFConfig,
    ) -> Result<(), String> {
        // first check if we are in a correct state
        let state;
        {
            let st = self
                .state
                .lock()
                .expect("Error locking RAF internal state mutex");
            state = st.get_state();
        }
        if state != RAFState::Active {
            return Err("RAF STOP error: not in ACTIVE state".to_string());
        };

        // generate the credentials
        let credentials = self.new_credentials(common_config);

        // Create the RAF START SLE PDU
        let pdu = SlePdu::SleRafStopInvocation {
            invoker_credentials: credentials,
            invoke_id: self.invoke_id.fetch_add(1, Ordering::AcqRel),
        };

        // And finally, send the PDU
        self.send_pdu(pdu).await?;

        let timeout = Duration::from_secs(config.sle_operation_timeout as u64);
        let cancel = self.cancellation_token.clone();

        self.handles.lock().unwrap().push((
            HandleType::Stop,
            tokio::spawn(async move {
                tokio::time::sleep(timeout).await;
                error!("Timeout waiting for RAF START RESPONSE, terminating connection");
                cancel.cancel();
            }),
        ));

        Ok(())
    }

    pub async fn peer_abort(&mut self, diagnostic: PeerAbortDiagnostic) {
        // Create the RAF START SLE PDU
        let pdu = SlePdu::SlePeerAbort { diagnostic };

        // And finally, send the PDU
        let _ = self.send_pdu(pdu).await;
        self.cancellation_token.cancel();
    }

    pub async fn stop_processing(&mut self) {
        let _ = self.chan.send(SleMsg::Stop).await;
        if let Some(handle) = self.write_task.take() {
            let _ = handle.await;
        }
        // if let Some(handle) = self.read_task.take() {
        //     drop(handle);
        // }
    }

    /// Stop the machinery
    pub async fn cancel(&self) {
        self.cancellation_token.cancel();
    }

    async fn check_return<F>(
        &mut self,
        mut rx: OwnedReadHalf,
        common_config: &CommonConfig,
        config: &RAFConfig,
        action: F,
    ) -> Result<OwnedReadHalf, String>
    where
        F: Fn(
            &CommonConfig,
            &RAFConfig,
            InternalState,
            CancellationToken,
            &TMLMessage,
        ) -> Result<(), String>,
    {
        select!(
            res = TMLMessage::async_read(&mut rx) => {
                match res {
                    Err(err) => {
                        self.cancellation_token.cancel();
                        return Err(format!("Error reading SLE TML message from socket: {}", err));
                    }
                    Ok(msg) => {
                        if msg.is_heartbeat() {
                            debug!("SLE TML heartbeat received");
                            return Ok(rx);
                        }
                        else
                        {
                            action(&common_config, &config, self.state.clone(), self.cancellation_token.clone(), &msg)?;
                            return Ok(rx);
                        }
                    }
                }
            },
            _ = tokio::time::sleep(self.op_timeout) => {
                // we have a receive heartbeat timeout, so report the error and disconnect
                self.cancellation_token.cancel();
                return Err(format!("Timeout waiting for BIND RETURN on {}, terminating connection", config.sii));
            }
        );
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
    cancel_token: CancellationToken,
) {
    let res: Result<SlePdu, _> = rasn::der::decode(&msg.data[..]);

    debug!("Decoded SLE PDU: {:?}", res);

    match res {
        Ok(SlePdu::SlePeerAbort { diagnostic: diag }) => {
            warn!("Received Peer Abort with diagnostic: {:?}", diag);
            cancel_token.cancel();
        }
        Ok(pdu) => {
            // check authentication
            if !check_authentication(config, raf_cfg, state.clone(), &pdu) {
                error!("SLE PDU failed authentication");
                cancel_token.cancel();
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
        SlePdu::SleRafStartReturn {
            performer_credentials: _,
            invoke_id: _,
            result,
        } => {
            cancel_timer(HandleType::Start, handle_vec);

            let mut lock = state.lock().expect("Mutex lock failed");
            lock.process_start(result);
        }
        SlePdu::SleAcknowledgement {
            credentials: _,
            invoke_id: _,
            result,
        } => {
            cancel_timer(HandleType::Stop, handle_vec);

            let mut lock = state.lock().expect("Mutex lock failed");
            lock.process_stop(result);
        }
        SlePdu::SleRafTransferBuffer(buffer) => {
            process_transfer_frame_buffer(state, buffer);
        }
        pdu => {
            debug!("Received: {:?}", pdu);
        }
    }
}

fn check_authentication(
    config: &CommonConfig,
    _raf_cfg: &RAFConfig,
    state: InternalState,
    pdu: &SlePdu,
) -> bool {
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
                    Credentials::Used(creds) => {
                        check_peer(config, creds, initiator_identifier, "BIND")
                    }
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
        SleAuthType::AuthAll => check_all(config, state, pdu),
    }
}

fn check_all(config: &CommonConfig, state: InternalState, pdu: &SlePdu) -> bool {
    let credentials = pdu.get_credentials();

    if let SlePdu::SleRafTransferBuffer(buffer) = pdu {
        for frame in buffer {
            let res = check_buffer_credential(config, state.clone(), frame);
            if !res {
                return false;
            }
        }
        return true;
    } else {
        match credentials {
            Some(creds) => match creds {
                Credentials::Unused => {
                    error!("Authentication failed, no credentials provided");
                    return false;
                }
                Credentials::Used(isp1) => {
                    let name;
                    {
                        let lock = state.lock().expect("Mutex lock failed");
                        name = lock.provider().clone();
                    }
                    let op_name = pdu.operation_name();
                    return check_peer(config, isp1, &name, op_name);
                }
            },
            None => {
                return true;
            }
        }
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
                name = lock.provider().clone();
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

fn process_transfer_frame_buffer(state: InternalState, buffer: &RafTransferBuffer) {
    let lock = state.lock().expect("Mutex lock failed");

    for elem in buffer {
        match elem {
            FrameOrNotification::AnnotatedFrame(frame) => match convert_frame(frame) {
                Ok(frame) => {
                    lock.process_tm_frame(&frame);
                }
                Err(err) => {
                    error!("Error decoding TM frame: {}", err);
                }
            },
            FrameOrNotification::SyncNotification(notif) => {
                debug!("Got SYNC Notification: {:?}", notif);
            }
        }
    }
}

fn authenticate_pdu_and_forward<F>(
    config: &CommonConfig,
    raf_cfg: &RAFConfig,
    state: InternalState,
    cancel_token: CancellationToken,
    msg: &TMLMessage,
    action: F,
) -> Result<(), String>
where
    F: Fn(
        &CommonConfig,
        &RAFConfig,
        InternalState,
        CancellationToken,
        &SlePdu,
    ) -> Result<(), String>,
{
    let res: Result<SlePdu, _> = rasn::der::decode(&msg.data[..]);

    debug!("Decoded SLE PDU: {:?}", res);

    match res {
        Ok(SlePdu::SlePeerAbort { diagnostic: diag }) => {
            warn!("Received Peer Abort with diagnostic: {:?}", diag);
            cancel_token.cancel();
            Ok(())
        }
        Ok(pdu) => {
            // First, check the authentication

            if !check_authentication(config, raf_cfg, state.clone(), &pdu) {
                cancel_token.cancel();
                return Err("SLE PDU failed authentication".to_string());
            }

            // we are ok, so now check, if this is a BIND RETURN. If not, we error out
            return action(config, raf_cfg, state, cancel_token, &pdu);
        }
        Err(err) => Err(format!("Error on decoding SLE PDU: {err}")),
    }
}

fn check_bind_return(
    config: &CommonConfig,
    raf_cfg: &RAFConfig,
    state: InternalState,
    cancel_token: CancellationToken,
    msg: &TMLMessage,
) -> Result<(), String> {
    authenticate_pdu_and_forward(
        config,
        raf_cfg,
        state,
        cancel_token,
        msg,
        |_config, _raf_cfg, state, _cancel_token, pdu| {
            match pdu {
                SlePdu::SleBindReturn {
                    responder_identifier,
                    result,
                    ..
                } => {
                    // Lock our state and process the BIND RETURN
                    {
                        let mut lock = state.lock().expect("Mutex lock failed");
                        lock.process_bind_return(&responder_identifier, &result);
                    }
                    return Ok(());
                }
                pdu => {
                    return Err(format!(
                        "Expected BIND RETURN, received unexpected PDU: {:?}",
                        pdu
                    ));
                }
            }
        },
    )
}

fn check_start_return(
    config: &CommonConfig,
    raf_cfg: &RAFConfig,
    state: InternalState,
    cancel_token: CancellationToken,
    msg: &TMLMessage,
) -> Result<(), String> {
    authenticate_pdu_and_forward(
        config,
        raf_cfg,
        state,
        cancel_token,
        msg,
        |_config, _raf_cfg, state, _cancel_token, pdu| {
            match pdu {
                SlePdu::SleRafStartReturn {
                    performer_credentials: _,
                    invoke_id: _,
                    result,
                } => {
                    // Lock our state and process the BIND RETURN
                    {
                        let mut lock = state.lock().expect("Mutex lock failed");
                        lock.process_start(&result);
                    }
                    return Ok(());
                }
                pdu => {
                    return Err(format!(
                        "Expected START RETURN, received unexpected PDU: {:?}",
                        pdu
                    ));
                }
            }
        },
    )
}
